uniffi::setup_scaffolding!();

mod types;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::Arc;

use librqbit::{AddTorrent, Magnet, Session, SessionOptions, TorrentStatsState};

pub use types::{EngineError, TorrentSource, TorrentStatus};

#[uniffi::export]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("failed to start Tokio runtime for engine")
    })
}

fn internal_error(message: impl std::fmt::Display) -> EngineError {
    EngineError::Internal {
        message: message.to_string(),
    }
}

/// librqbit reports speed as `Speed { mbps: f64 }` — mebibytes/sec, not bytes/sec.
/// The FFI boundary carries bytes/sec (integer) per the Architecture Spine's
/// Consistency Conventions, so this conversion is required on every call.
fn mbps_to_bytes_per_sec(mbps: f64) -> u64 {
    (mbps * 1024.0 * 1024.0).round() as u64
}

/// "checking" matches the Architecture Spine's Consistency Conventions vocabulary
/// (active = {Downloading, Checking, Seeding}), not librqbit's own state name
/// ("Initializing") — see Story 1.3 Dev Notes. A pure function so every branch —
/// including "seeding", unreachable in a unit test without a real completed
/// download — can be tested directly.
fn derive_status(state: TorrentStatsState, finished: bool) -> &'static str {
    match state {
        TorrentStatsState::Initializing => "checking",
        TorrentStatsState::Paused => "paused",
        TorrentStatsState::Error => "error",
        TorrentStatsState::Live if finished => "seeding",
        TorrentStatsState::Live => "downloading",
    }
}

struct PendingTorrent {
    name: String,
    // Set once the background resolution in `add_magnet` fails, so `get_all_torrents`
    // can report "error" instead of the entry just disappearing with no explanation.
    failed: bool,
}

#[derive(uniffi::Object)]
pub struct Engine {
    session: Arc<Session>,
    // Magnet links whose metadata librqbit hasn't resolved yet (see add_torrent below) —
    // not visible via `session.with_torrents` until resolution finishes, so tracked here
    // so `get_all_torrents` can still surface them immediately.
    pending: Arc<Mutex<HashMap<String, PendingTorrent>>>,
}

#[uniffi::export]
impl Engine {
    #[uniffi::constructor]
    pub fn new(download_dir: String) -> Result<Arc<Self>, EngineError> {
        let session = runtime()
            .block_on(Session::new(download_dir.into()))
            .map_err(internal_error)?;
        Ok(Arc::new(Self {
            session,
            pending: Arc::new(Mutex::new(HashMap::new())),
        }))
    }

    pub fn add_torrent(&self, source: TorrentSource) -> Result<String, EngineError> {
        match source {
            TorrentSource::Path { path } => {
                let add = AddTorrent::from_local_filename(&path).map_err(internal_error)?;
                self.add_torrent_blocking(add)
            }
            TorrentSource::Bytes { bytes } => {
                self.add_torrent_blocking(AddTorrent::from_bytes(bytes))
            }
            TorrentSource::Magnet { uri } => self.add_magnet(uri),
        }
    }

    pub fn get_all_torrents(&self) -> Vec<TorrentStatus> {
        let mut result: Vec<TorrentStatus> = self.session.with_torrents(|torrents| {
            torrents
                .map(|(_, torrent)| {
                    let stats = torrent.stats();
                    let status = derive_status(stats.state, stats.finished);
                    let (down_speed_bps, up_speed_bps, peers_connected) = match &stats.live {
                        Some(live) => (
                            mbps_to_bytes_per_sec(live.download_speed.mbps),
                            mbps_to_bytes_per_sec(live.upload_speed.mbps),
                            live.snapshot.peer_stats.live as u32,
                        ),
                        None => (0, 0, 0),
                    };
                    let progress_percent = if stats.total_bytes == 0 {
                        0.0
                    } else {
                        stats.progress_bytes as f64 / stats.total_bytes as f64 * 100.0
                    };
                    TorrentStatus {
                        id: torrent.info_hash().as_string(),
                        name: torrent.name().unwrap_or_default(),
                        status: status.to_string(),
                        progress_percent,
                        down_speed_bps,
                        up_speed_bps,
                        peers_connected,
                    }
                })
                .collect()
        });

        let existing_ids: HashSet<String> = result.iter().map(|t| t.id.clone()).collect();
        for (id, pending) in self.pending.lock().unwrap().iter() {
            if !existing_ids.contains(id) {
                result.push(TorrentStatus {
                    id: id.clone(),
                    name: pending.name.clone(),
                    status: if pending.failed { "error" } else { "resolving" }.to_string(),
                    progress_percent: 0.0,
                    down_speed_bps: 0,
                    up_speed_bps: 0,
                    peers_connected: 0,
                });
            }
        }

        result
    }
}

impl Engine {
    /// Adds a torrent whose metadata is already known (`.torrent` file/bytes) —
    /// librqbit registers these synchronously without waiting on any peer.
    fn add_torrent_blocking(&self, add: AddTorrent<'static>) -> Result<String, EngineError> {
        let response = runtime()
            .block_on(self.session.add_torrent(add, None))
            .map_err(internal_error)?;

        let handle = response
            .into_handle()
            .ok_or_else(|| internal_error("add_torrent did not return a managed torrent handle"))?;

        Ok(handle.info_hash().as_string())
    }

    /// Adds a magnet link without waiting for librqbit to resolve its metadata.
    ///
    /// `librqbit::Session::add_torrent` blocks until a peer actually sends back the
    /// torrent metadata for magnet links (BEP-9) — for a link with no immediately
    /// reachable peers that can take a long time or never happen. Blocking on that
    /// here would break AC1/AC2 ("row appears instantly"), so instead: parse the
    /// infohash directly out of the magnet URI (it's already there in `xt=urn:btih:`),
    /// register it in `pending` so `get_all_torrents` can show it right away, and let
    /// the real `session.add_torrent` resolve in the background. On success `pending`
    /// is cleared — the torrent is visible via `session.with_torrents` from then on.
    /// On failure the entry is kept but marked `failed`, so the row turns into an
    /// "error" status instead of just disappearing with no explanation.
    fn add_magnet(&self, uri: String) -> Result<String, EngineError> {
        let magnet = Magnet::parse(&uri).map_err(internal_error)?;
        let id20 = magnet
            .as_id20()
            .ok_or_else(|| internal_error("magnet link is missing a v1 infohash"))?;
        let id = id20.as_string();
        let name = magnet.name.clone().unwrap_or_else(|| id.clone());

        self.pending.lock().unwrap().insert(
            id.clone(),
            PendingTorrent {
                name,
                failed: false,
            },
        );

        let session = self.session.clone();
        let pending = self.pending.clone();
        let pending_id = id.clone();
        runtime().spawn(async move {
            match session.add_torrent(AddTorrent::from_url(uri), None).await {
                Ok(_) => {
                    pending.lock().unwrap().remove(&pending_id);
                }
                Err(err) => {
                    eprintln!("add_magnet: background resolution failed for {pending_id}: {err:#}");
                    if let Some(entry) = pending.lock().unwrap().get_mut(&pending_id) {
                        entry.failed = true;
                    }
                }
            }
        });

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_version_returns_non_empty_string() {
        assert!(!engine_version().is_empty());
    }

    fn test_engine() -> Arc<Engine> {
        // DHT persistence disabled: librqbit's default persistent DHT writes to a fixed
        // OS cache dir (not the per-test tempdir), which is unavailable in some sandboxed
        // test environments. DHT itself is left on on purpose: with it fully off and no
        // trackers, the background resolution spawned by `add_magnet` would fail (and
        // clear `pending`) synchronously, racing the test's immediate `get_all_torrents`
        // check. Leaving DHT on makes that background resolution actually async (it
        // waits on real network I/O that never completes here), so `pending` reliably
        // still holds the entry when the test asserts on it right after `add_torrent`.
        let dir = tempfile::tempdir().unwrap();
        let session = runtime()
            .block_on(Session::new_with_opts(
                dir.path().to_path_buf(),
                SessionOptions {
                    disable_dht_persistence: true,
                    ..Default::default()
                },
            ))
            .unwrap();
        Arc::new(Engine {
            session,
            pending: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn test_engine_no_peer_sources() -> Arc<Engine> {
        // DHT fully disabled and no trackers/initial_peers — unlike `test_engine`,
        // this makes the background resolution in `add_magnet` fail fast and
        // deterministically ("no known way to resolve peers"), so tests can observe
        // the failure path instead of the still-resolving path.
        let dir = tempfile::tempdir().unwrap();
        let session = runtime()
            .block_on(Session::new_with_opts(
                dir.path().to_path_buf(),
                SessionOptions {
                    disable_dht: true,
                    ..Default::default()
                },
            ))
            .unwrap();
        Arc::new(Engine {
            session,
            pending: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    #[test]
    fn add_torrent_from_file_appears_in_get_all_torrents() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");

        let id = engine
            .add_torrent(TorrentSource::Path {
                path: fixture.to_string(),
            })
            .expect("add_torrent should succeed for a valid .torrent file");

        assert!(!id.is_empty());

        let torrents = engine.get_all_torrents();
        assert_eq!(torrents.len(), 1);
        assert_eq!(torrents[0].id, id);
    }

    #[test]
    fn add_torrent_from_magnet_registers_without_network() {
        let engine = test_engine();
        // Well-formed but unreachable infohash — add_torrent must return immediately
        // (the infohash is parsed straight out of the URI), without waiting for
        // librqbit to actually resolve metadata from a peer.
        let magnet = "magnet:?xt=urn:btih:0000000000000000000000000000000000000000";

        let id = engine
            .add_torrent(TorrentSource::Magnet {
                uri: magnet.to_string(),
            })
            .expect("add_torrent should succeed for a well-formed magnet link");

        assert_eq!(id, "0000000000000000000000000000000000000000");

        let torrents = engine.get_all_torrents();
        assert_eq!(torrents.len(), 1);
        assert_eq!(torrents[0].id, id);
        assert_eq!(torrents[0].status, "resolving");
    }

    #[test]
    fn add_torrent_from_magnet_marks_error_status_on_resolution_failure() {
        let engine = test_engine_no_peer_sources();
        let magnet = "magnet:?xt=urn:btih:0000000000000000000000000000000000000000";

        let id = engine
            .add_torrent(TorrentSource::Magnet {
                uri: magnet.to_string(),
            })
            .expect("add_torrent should succeed for a well-formed magnet link");

        // The background task fails almost immediately (no DHT/trackers/peers), but
        // it still runs on another thread — poll with a bounded wait instead of
        // asserting instantly.
        let mut status = String::new();
        for _ in 0..100 {
            if let Some(torrent) = engine.get_all_torrents().into_iter().find(|t| t.id == id) {
                status = torrent.status;
                if status == "error" {
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        assert_eq!(status, "error");
    }

    #[test]
    fn add_torrent_from_file_reports_progress_and_live_metric_fields() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");

        engine
            .add_torrent(TorrentSource::Path {
                path: fixture.to_string(),
            })
            .expect("add_torrent should succeed for a valid .torrent file");

        let torrents = engine.get_all_torrents();
        assert_eq!(torrents.len(), 1);
        let torrent = &torrents[0];

        // No real peers in this test, so status is whichever early state librqbit
        // reports before it has fetched anything — not deterministic which one.
        assert!(
            matches!(torrent.status.as_str(), "checking" | "downloading"),
            "unexpected status: {}",
            torrent.status
        );
        assert!((0.0..=100.0).contains(&torrent.progress_percent));
        // No peers connected in this test — computing these fields must not panic,
        // and with nothing connected they're expected to be zero.
        assert_eq!(torrent.down_speed_bps, 0);
        assert_eq!(torrent.up_speed_bps, 0);
        assert_eq!(torrent.peers_connected, 0);
    }

    #[test]
    fn mbps_to_bytes_per_sec_converts_correctly() {
        assert_eq!(mbps_to_bytes_per_sec(0.0), 0);
        assert_eq!(mbps_to_bytes_per_sec(1.0), 1024 * 1024);
        assert_eq!(mbps_to_bytes_per_sec(2.5), (2.5f64 * 1024.0 * 1024.0).round() as u64);
        // Rounds rather than truncates — a fractional Mbps must not be
        // systematically under-reported.
        assert_eq!(mbps_to_bytes_per_sec(0.000001), 1);
    }

    #[test]
    fn derive_status_covers_every_state() {
        assert_eq!(derive_status(TorrentStatsState::Initializing, false), "checking");
        assert_eq!(derive_status(TorrentStatsState::Paused, false), "paused");
        assert_eq!(derive_status(TorrentStatsState::Error, false), "error");
        assert_eq!(derive_status(TorrentStatsState::Live, false), "downloading");
        // The one branch a real end-to-end run can't reach without a fully
        // downloaded torrent and a live swarm (unreachable in this environment —
        // see Story 1.3 Task 6 Debug Log) — covered here directly instead.
        assert_eq!(derive_status(TorrentStatsState::Live, true), "seeding");
    }
}
