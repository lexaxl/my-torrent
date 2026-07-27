uniffi::setup_scaffolding!();

mod types;

use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::Arc;

use librqbit::api::TorrentIdOrHash;
use librqbit::{
    AddTorrent, Api, ManagedTorrent, Magnet, Session, SessionOptions, TorrentStats, TorrentStatsState,
};
use librqbit_core::Id20;

pub use types::{EngineError, TorrentDetail, TorrentFile, TorrentSource, TorrentStatus};

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

/// Shared by `get_all_torrents` and `get_torrent_details` (Story 2.1) — both derive
/// the same list-level fields from a `TorrentStats` snapshot; kept in one place so
/// the two calls can't silently drift apart (see Story 2.1 Dev Notes).
fn summarize_stats(stats: &TorrentStats) -> (String, f64, u64, u64, u32) {
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
    (status.to_string(), progress_percent, down_speed_bps, up_speed_bps, peers_connected)
}

struct PendingTorrent {
    name: String,
    // Set once the background resolution in `add_magnet` fails, so `get_all_torrents`
    // can report "error" instead of the entry just disappearing with no explanation.
    failed: bool,
    // Set by `remove_torrent` when the user removes this torrent before the background
    // resolution in `add_magnet` has finished. The entry stays in `pending` (hidden from
    // `get_all_torrents`, see below) only so that background task can notice the removal
    // and undo whatever it was about to do — delete a torrent it just registered in the
    // session, or drop the entry instead of marking it "error" — rather than letting a
    // torrent the user already removed silently reappear or resurface as an error row.
    removed: bool,
    // Assigned from `Engine::next_pending_seq` at insertion — `pending` is a HashMap
    // (unordered), but the UX spec requires insertion order, no sorting. See
    // `get_all_torrents` for how this is used to sort pending entries back into order.
    seq: u64,
}

#[derive(uniffi::Object)]
pub struct Engine {
    session: Arc<Session>,
    // Magnet links whose metadata librqbit hasn't resolved yet (see add_torrent below) —
    // not visible via `session.with_torrents` until resolution finishes, so tracked here
    // so `get_all_torrents` can still surface them immediately.
    pending: Arc<Mutex<HashMap<String, PendingTorrent>>>,
    // Only public way to read a torrent's output folder (Story 1.5, reveal_path) —
    // `ManagedTorrentShared::options` is `pub(crate)` inside librqbit, unreachable
    // from here directly.
    api: Api,
    // Monotonic counter for `PendingTorrent::seq` — see there.
    next_pending_seq: AtomicU64,
}

#[uniffi::export]
impl Engine {
    #[uniffi::constructor]
    pub fn new(download_dir: String) -> Result<Arc<Self>, EngineError> {
        let session = runtime()
            .block_on(Session::new(download_dir.into()))
            .map_err(internal_error)?;
        let api = Api::new(session.clone(), None);
        Ok(Arc::new(Self {
            session,
            pending: Arc::new(Mutex::new(HashMap::new())),
            api,
            next_pending_seq: AtomicU64::new(0),
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
        // `with_torrents`' `usize` is the index librqbit itself assigned when the
        // torrent was added to the session (`AddTorrentResponse::Added(usize, ..)`)
        // — a real insertion-order signal from the engine, not invented here. The
        // UX spec requires insertion order with no sorting; `with_torrents`'
        // iteration order is not guaranteed to match it, so sort by that index
        // explicitly rather than trusting iteration order.
        let mut session_torrents: Vec<(usize, TorrentStatus)> = self.session.with_torrents(|torrents| {
            torrents
                .map(|(idx, torrent)| {
                    let stats = torrent.stats();
                    let (status, progress_percent, down_speed_bps, up_speed_bps, peers_connected) =
                        summarize_stats(&stats);
                    (
                        idx,
                        TorrentStatus {
                            id: torrent.info_hash().as_string(),
                            name: torrent.name().unwrap_or_default(),
                            status,
                            progress_percent,
                            down_speed_bps,
                            up_speed_bps,
                            peers_connected,
                        },
                    )
                })
                .collect()
        });
        session_torrents.sort_by_key(|(idx, _)| *idx);
        let mut result: Vec<TorrentStatus> =
            session_torrents.into_iter().map(|(_, status)| status).collect();

        let existing_ids: HashSet<String> = result.iter().map(|t| t.id.clone()).collect();
        let mut pending_torrents: Vec<(u64, TorrentStatus)> = self
            .pending
            .lock()
            .unwrap()
            .iter()
            .filter(|(id, pending)| !pending.removed && !existing_ids.contains(*id))
            .map(|(id, pending)| {
                (
                    pending.seq,
                    TorrentStatus {
                        id: id.clone(),
                        name: pending.name.clone(),
                        status: if pending.failed { "error" } else { "resolving" }.to_string(),
                        progress_percent: 0.0,
                        down_speed_bps: 0,
                        up_speed_bps: 0,
                        peers_connected: 0,
                    },
                )
            })
            .collect();
        pending_torrents.sort_by_key(|(seq, _)| *seq);
        result.extend(pending_torrents.into_iter().map(|(_, status)| status));

        result
    }

    pub fn pause_torrent(&self, id: String) -> Result<(), EngineError> {
        let handle = self.find_handle(&id)?;
        runtime()
            .block_on(self.session.pause(&handle))
            .map_err(internal_error)
    }

    pub fn resume_torrent(&self, id: String) -> Result<(), EngineError> {
        let handle = self.find_handle(&id)?;
        runtime()
            .block_on(self.session.unpause(&handle))
            .map_err(internal_error)
    }

    /// Per Scope Boundary (Story 1.5): never deletes downloaded files
    /// (`delete_files: false`) — only stops librqbit from tracking the torrent.
    /// Handles both a torrent already known to the session, and one still only
    /// in `pending` (magnet metadata not resolved yet) — `session.delete` only
    /// covers the former; for the latter, marks the entry `removed` instead of
    /// dropping it outright so `add_magnet`'s background task can still notice
    /// and undo itself once resolution finishes (see `PendingTorrent::removed`).
    pub fn remove_torrent(&self, id: String) -> Result<(), EngineError> {
        let idor = Self::parse_id(&id)?;
        if self.session.get(idor).is_some() {
            runtime()
                .block_on(self.session.delete(idor, false))
                .map_err(internal_error)?;
            // Idempotent: clears a pending entry even here, in case resolution
            // finished and registered the torrent in the session between
            // `get_all_torrents` and this call.
            self.pending.lock().unwrap().remove(&id);
        } else {
            match self.pending.lock().unwrap().get_mut(&id) {
                Some(entry) => entry.removed = true,
                None => return Err(internal_error(format!("torrent not found: {id}"))),
            }
        }
        Ok(())
    }

    pub fn reveal_path(&self, id: String) -> Result<String, EngineError> {
        let idor = Self::parse_id(&id)?;
        Ok(self
            .api
            .api_torrent_details(idor)
            .map_err(internal_error)?
            .output_folder)
    }

    /// Separate from `get_all_torrents` on purpose (Story 2.1 Dev Notes / Consistency
    /// Conventions "Snapshot granularity") — per-file detail is only fetched for the
    /// one torrent whose detail window is open, polled on its own cadence, not folded
    /// into the always-on list poll. Not found for a torrent still only in `pending`
    /// (`find_handle` covers that — see Scope Boundary: the detail window still opens,
    /// it just stays empty until resolution finishes).
    pub fn get_torrent_details(&self, id: String) -> Result<TorrentDetail, EngineError> {
        let handle = self.find_handle(&id)?;
        let stats = handle.stats();
        let (status, progress_percent, down_speed_bps, up_speed_bps, peers_connected) =
            summarize_stats(&stats);

        // `file_progress` is aligned by index with `file_infos` — both come from the
        // same per-file iteration order librqbit assigns at metadata-parse time.
        // `metadata` is `None` until the torrent has finished initializing, same case
        // `torrent.name()` already falls back on elsewhere in this file.
        let files = handle
            .metadata
            .load()
            .as_ref()
            .map(|metadata| {
                metadata
                    .file_infos
                    .iter()
                    .enumerate()
                    .map(|(i, info)| {
                        let downloaded = stats.file_progress.get(i).copied().unwrap_or(0);
                        let file_progress_percent = if info.len == 0 {
                            0.0
                        } else {
                            downloaded as f64 / info.len as f64 * 100.0
                        };
                        TorrentFile {
                            name: info.relative_filename.to_string_lossy().into_owned(),
                            size_bytes: info.len,
                            progress_percent: file_progress_percent,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(TorrentDetail {
            id,
            name: handle.name().unwrap_or_default(),
            status,
            progress_percent,
            down_speed_bps,
            up_speed_bps,
            peers_connected,
            files,
        })
    }
}

impl Engine {
    fn parse_id(id: &str) -> Result<TorrentIdOrHash, EngineError> {
        Id20::from_str(id)
            .map(TorrentIdOrHash::Hash)
            .map_err(internal_error)
    }

    /// Shared by `pause_torrent`/`resume_torrent` — both need a live session handle
    /// and neither applies to a torrent still only in `pending` (no handle exists
    /// until librqbit has actually registered it).
    fn find_handle(&self, id: &str) -> Result<Arc<ManagedTorrent>, EngineError> {
        let idor = Self::parse_id(id)?;
        self.session
            .get(idor)
            .ok_or_else(|| internal_error(format!("torrent not found: {id}")))
    }

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

        let seq = self.next_pending_seq.fetch_add(1, Ordering::Relaxed);
        self.pending.lock().unwrap().insert(
            id.clone(),
            PendingTorrent {
                name,
                failed: false,
                removed: false,
                seq,
            },
        );

        let session = self.session.clone();
        let pending = self.pending.clone();
        let pending_id = id.clone();
        runtime().spawn(async move {
            match session.add_torrent(AddTorrent::from_url(uri), None).await {
                Ok(_) => {
                    let was_removed = pending
                        .lock()
                        .unwrap()
                        .get(&pending_id)
                        .map(|entry| entry.removed)
                        .unwrap_or(false);
                    if was_removed {
                        // The user removed this torrent while it was still resolving —
                        // undo the registration `session.add_torrent` just completed so
                        // it doesn't resurface in `get_all_torrents` (Story 1.5).
                        if let Ok(idor) = Self::parse_id(&pending_id) {
                            if let Err(err) = session.delete(idor, false).await {
                                eprintln!(
                                    "add_magnet: failed to undo removed-while-resolving torrent {pending_id}: {err:#}"
                                );
                            }
                        }
                    }
                    pending.lock().unwrap().remove(&pending_id);
                }
                Err(err) => {
                    eprintln!("add_magnet: background resolution failed for {pending_id}: {err:#}");
                    let mut guard = pending.lock().unwrap();
                    let was_removed = guard.get(&pending_id).map(|e| e.removed).unwrap_or(false);
                    if was_removed {
                        // Already removed by the user — don't resurrect it as an "error" row.
                        guard.remove(&pending_id);
                    } else if let Some(entry) = guard.get_mut(&pending_id) {
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
        let api = Api::new(session.clone(), None);
        Arc::new(Engine {
            session,
            pending: Arc::new(Mutex::new(HashMap::new())),
            api,
            next_pending_seq: AtomicU64::new(0),
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
        let api = Api::new(session.clone(), None);
        Arc::new(Engine {
            session,
            pending: Arc::new(Mutex::new(HashMap::new())),
            api,
            next_pending_seq: AtomicU64::new(0),
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
    fn pause_torrent_then_resume_torrent_round_trips() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        let id = engine
            .add_torrent(TorrentSource::Path {
                path: fixture.to_string(),
            })
            .expect("add_torrent should succeed for a valid .torrent file");

        // `pause` only succeeds once librqbit's internal state has left
        // "Initializing" (the disk-hash-check phase) for "Live" — see
        // `add_torrent_from_file_reports_progress_and_live_metric_fields` above,
        // which shows this transition isn't deterministic in timing. Bounded
        // poll instead of asserting instantly (same pattern as the magnet
        // resolution-failure test below).
        let mut checking_done = false;
        for _ in 0..100 {
            if engine
                .get_all_torrents()
                .into_iter()
                .any(|t| t.id == id && t.status != "checking")
            {
                checking_done = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(checking_done, "torrent never left the checking state");

        engine
            .pause_torrent(id.clone())
            .expect("pause_torrent should succeed for a known torrent");
        let paused = engine.get_all_torrents();
        assert_eq!(paused.len(), 1);
        assert_eq!(paused[0].status, "paused");

        engine
            .resume_torrent(id.clone())
            .expect("resume_torrent should succeed for a paused torrent");
        let resumed = engine.get_all_torrents();
        assert_eq!(resumed.len(), 1);
        assert_ne!(resumed[0].status, "paused");
    }

    #[test]
    fn remove_torrent_removes_from_get_all_torrents() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        let id = engine
            .add_torrent(TorrentSource::Path {
                path: fixture.to_string(),
            })
            .expect("add_torrent should succeed for a valid .torrent file");

        engine
            .remove_torrent(id)
            .expect("remove_torrent should succeed for a known torrent");

        assert!(engine.get_all_torrents().is_empty());
    }

    #[test]
    fn remove_torrent_removes_still_pending_magnet() {
        let engine = test_engine_no_peer_sources();
        let magnet = "magnet:?xt=urn:btih:0000000000000000000000000000000000000000";
        let id = engine
            .add_torrent(TorrentSource::Magnet {
                uri: magnet.to_string(),
            })
            .expect("add_torrent should succeed for a well-formed magnet link");

        // Whether background resolution has already failed (marking the entry
        // "error") or is still in flight ("resolving") at this point is a race —
        // remove_torrent must succeed either way, since both cases only exist in
        // `pending`, never in the session (no real peers/DHT in this test setup).
        engine
            .remove_torrent(id)
            .expect("remove_torrent should succeed for a still-pending magnet");

        assert!(engine.get_all_torrents().is_empty());
    }

    #[test]
    fn get_all_torrents_preserves_insertion_order_for_pending_magnets() {
        // `pending` is a HashMap — iteration order is not insertion order. This
        // test would be flaky/fail without the `seq`-based sort in
        // `get_all_torrents` (three distinct, well-formed-but-unreachable
        // infohashes, so nothing resolves during the test).
        let engine = test_engine();
        // 40-char hex infohashes ("1".repeat(39) + "a", etc.) — well-formed but
        // unreachable, same convention as the other magnet tests in this file.
        let ids: Vec<String> = ["1", "2", "3"]
            .iter()
            .map(|d| format!("{}{d}", d.repeat(39)))
            .collect();

        for id in &ids {
            engine
                .add_torrent(TorrentSource::Magnet {
                    uri: format!("magnet:?xt=urn:btih:{id}"),
                })
                .expect("add_torrent should succeed for a well-formed magnet link");
        }

        let torrents = engine.get_all_torrents();
        let observed: Vec<&str> = torrents.iter().map(|t| t.id.as_str()).collect();
        let expected: Vec<&str> = ids.iter().map(String::as_str).collect();
        assert_eq!(observed, expected);
    }

    #[test]
    fn pause_torrent_returns_error_for_unknown_id() {
        let engine = test_engine();
        let unknown_id = "0".repeat(40);

        assert!(engine.pause_torrent(unknown_id).is_err());
    }

    #[test]
    fn get_torrent_details_returns_files_with_sizes() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        let id = engine
            .add_torrent(TorrentSource::Path {
                path: fixture.to_string(),
            })
            .expect("add_torrent should succeed for a valid .torrent file");

        let detail = engine
            .get_torrent_details(id.clone())
            .expect("get_torrent_details should succeed for a known torrent");

        assert_eq!(detail.id, id);
        assert_eq!(detail.files.len(), 1, "fixture is a single-file torrent");
        assert!(detail.files[0].name.contains("test.txt"));
        assert!(detail.files[0].size_bytes > 0);
        assert!((0.0..=100.0).contains(&detail.files[0].progress_percent));
        assert!((0.0..=100.0).contains(&detail.progress_percent));
    }

    #[test]
    fn get_torrent_details_returns_error_for_unknown_id() {
        let engine = test_engine();
        let unknown_id = "0".repeat(40);

        assert!(engine.get_torrent_details(unknown_id).is_err());
    }

    #[test]
    fn get_torrent_details_matches_get_all_torrents_summary_fields() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        let id = engine
            .add_torrent(TorrentSource::Path {
                path: fixture.to_string(),
            })
            .expect("add_torrent should succeed for a valid .torrent file");

        let summary = engine
            .get_all_torrents()
            .into_iter()
            .find(|t| t.id == id)
            .expect("torrent should appear in get_all_torrents");
        let detail = engine
            .get_torrent_details(id)
            .expect("get_torrent_details should succeed for a known torrent");

        // Both go through `summarize_stats` — a future edit to one call site that
        // forgets the other would show up here as a mismatch.
        assert_eq!(detail.status, summary.status);
        assert_eq!(detail.progress_percent, summary.progress_percent);
        assert_eq!(detail.down_speed_bps, summary.down_speed_bps);
        assert_eq!(detail.up_speed_bps, summary.up_speed_bps);
        assert_eq!(detail.peers_connected, summary.peers_connected);
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
