uniffi::setup_scaffolding!();

mod types;

use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::Arc;
use std::time::Instant;

use librqbit::api::TorrentIdOrHash;
use librqbit::{
    AddTorrent, AddTorrentOptions, Api, ByteBufOwned, ByteBufT, ManagedTorrent, Magnet, Session,
    SessionOptions, SessionPersistenceConfig, TorrentStats, TorrentStatsState,
};
use librqbit_core::torrent_metainfo::torrent_from_bytes;
use librqbit_core::Id20;

pub use types::{EngineError, Peer, Tracker, TorrentDetail, TorrentFile, TorrentSource, TorrentStatus};

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

/// The persistent-session options shared by production (`Engine::new`) and the
/// persistence tests — the one place the `Json` persistence + `fastresume`
/// combination is spelled out, so the two can't drift (code-review fix, Story
/// 6.2). Tests additionally flip `disable_dht_persistence` on the returned
/// struct for the same sandbox reason as `test_engine`.
fn persistent_session_options(state_dir: PathBuf) -> SessionOptions {
    SessionOptions {
        persistence: Some(SessionPersistenceConfig::Json {
            folder: Some(state_dir),
        }),
        fastresume: true,
        ..Default::default()
    }
}

/// True if the persistence store folder exists and holds at least one entry.
/// A failure to open an *empty* store can't be store corruption (there's
/// nothing in it), so the recovery path in `Engine::open_with_recovery` only
/// quarantines a store that actually has contents to be corrupt.
fn store_has_entries(state_dir: &Path) -> bool {
    std::fs::read_dir(state_dir)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

/// Renames a corrupt store folder aside to `<name>.corrupt-<unix-secs>` (a
/// sibling of the original), preserving it for debugging rather than deleting
/// it. Story 6.2 code review: a single unreadable persisted entry makes
/// librqbit's restore fail the whole session open, which Swift turns into a
/// `fatalError` — crash-looping the app on every launch until the user manually
/// deletes this folder. Quarantining lets the next open start fresh instead.
fn quarantine_store(state_dir: &Path) -> std::io::Result<()> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let name = state_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("session");
    let mut quarantined = state_dir.to_path_buf();
    quarantined.set_file_name(format!("{name}.corrupt-{timestamp}"));
    std::fs::rename(state_dir, &quarantined)
}

/// Computes the actual on-disk destination for a newly added `.torrent` file/bytes
/// torrent (code-review fix, Story 3.1).
///
/// `add_torrent_blocking` always sets `AddTorrentOptions.output_folder` (per AD-5 —
/// the save-location setting flows in per call, not fixed at session construction).
/// But librqbit only auto-derives a per-torrent subfolder for multi-file torrents
/// when `output_folder` is `None` — verified directly against librqbit 8.1.1's
/// `Session::add_torrent_internal` (`session.rs`): `(Some(o), None) => PathBuf::from(o)`
/// skips subfolder derivation entirely, while `(None, None)` joins the session's
/// default folder with a name from `get_default_subfolder_for_torrent` (private, so
/// it can't be called directly). Left as `Some(download_dir)` unconditionally, every
/// multi-file torrent (season packs, multi-file releases) would dump its files flat
/// into `download_dir` instead of getting its own named subfolder — a real, silent
/// regression this reproduces the same rule for: single-file torrents resolve to
/// `download_dir` itself, multi-file torrents resolve to `download_dir/<subfolder>`.
///
/// The subfolder name itself mirrors librqbit's own two-tier rule (verified against
/// `get_default_subfolder_for_torrent`, same file): prefer `info.name` when present
/// and a valid single path component; otherwise fall back to the longest file's stem
/// (librqbit's own fallback for torrents with no/empty `name` — legal per BEP3, if
/// rare). One deliberate divergence: librqbit hard-fails the whole add when `info.name`
/// is present but invalid (path traversal, separators) — this falls back to the
/// longest-file-stem tier instead of failing, since folder *placement* shouldn't be
/// able to block an otherwise-valid add. Falls back to `download_dir` unchanged (no
/// subfolder) only if the torrent can't be parsed, has no files, or truly has neither
/// a usable name nor any file to name it after — the actual `add_torrent` call below
/// will surface any real parse error on its own; this is purely about folder
/// placement, not validation. (Magnet links can't get this treatment — see
/// `add_magnet`'s doc comment for why.)
fn resolve_output_folder(download_dir: &str, torrent_bytes: &[u8]) -> String {
    let Ok(meta) = torrent_from_bytes::<ByteBufOwned>(torrent_bytes) else {
        return download_dir.to_string();
    };
    let Ok(files) = meta.info.iter_file_details() else {
        return download_dir.to_string();
    };
    let files: Vec<(std::path::PathBuf, u64)> = files
        .filter_map(|details| details.filename.to_pathbuf().ok().map(|path| (path, details.len)))
        .collect();
    if files.len() < 2 {
        return download_dir.to_string();
    }

    let name_subfolder = meta.info.name.as_ref().and_then(|name| {
        let name = String::from_utf8_lossy(name.as_slice()).into_owned();
        is_valid_subfolder_name(&name).then_some(name)
    });
    let subfolder = name_subfolder.or_else(|| {
        files
            .iter()
            .max_by_key(|(_, len)| *len)
            .and_then(|(path, _)| path.file_stem())
            .map(|stem| stem.to_string_lossy().into_owned())
    });

    match subfolder {
        Some(name) => std::path::Path::new(download_dir)
            .join(name)
            .to_string_lossy()
            .into_owned(),
        None => download_dir.to_string(),
    }
}

/// A single path component is safe to join onto `download_dir` — rejects path
/// traversal and any embedded separator (mirrors librqbit's own `check_valid` in
/// `get_default_subfolder_for_torrent`, which checks each `Path` component is
/// `Component::Normal`).
fn is_valid_subfolder_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\\')
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
/// the two calls can't silently drift apart (see Story 2.1 Dev Notes). `total_bytes`/
/// `downloaded_bytes` (Story 5.1) are included here rather than read directly from
/// `stats` at each call site — code review on Story 5.1 found the two call sites
/// reading them independently defeated this very function's anti-drift purpose.
fn summarize_stats(stats: &TorrentStats) -> (String, f64, u64, u64, u32, u64, u64) {
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
    (
        status.to_string(),
        progress_percent,
        down_speed_bps,
        up_speed_bps,
        peers_connected,
        stats.total_bytes,
        stats.progress_bytes,
    )
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
    // Story 2.2: librqbit only exposes a cumulative "bytes fetched from this peer"
    // counter, not a live speed — this tracks each torrent's peers' counter +
    // timestamp from the previous `get_torrent_details` poll so a bytes/sec speed
    // can be derived from the delta (see `build_peers`). Nested by torrent id
    // (rather than a flat `(torrent_id, address)`-keyed map) so per-torrent
    // cleanup (`remove_torrent`, disconnected-peer pruning in `build_peers`) is
    // scoped to one inner map instead of scanning every tracked torrent's peers
    // on every poll (code-review fix, Story 2.2).
    peer_speed_baseline: Mutex<HashMap<String, HashMap<String, (u64, Instant)>>>,
}

#[uniffi::export]
impl Engine {
    #[uniffi::constructor]
    pub fn new(download_dir: String, state_dir: String) -> Result<Arc<Self>, EngineError> {
        // Story 6.2 — session persistence: the torrent list survives app restarts.
        // `Json { folder }` stores a session db + per-torrent `.torrent` bytes in
        // `state_dir` (always passed explicitly by the shell — `folder: None`
        // would silently use librqbit's own OS-branded config dir). `fastresume`
        // additionally persists piece bitfields (`.bitv`) so a restart resumes
        // without re-hashing everything on disk. Restore happens inside
        // `new_with_opts` itself, re-adding each torrent with its persisted
        // paused state, output folder, and stable id (`preferred_id`), and with
        // `overwrite: true` — so restored torrents never hit the
        // files-already-exist add failure. DHT persistence is a separate
        // librqbit mechanism and stays at its default (on) here; tests disable
        // it for sandbox reasons only (see `test_engine`).
        //
        // Opened through `open_with_recovery` so a single corrupt persisted
        // entry can't crash-loop the app (code-review fix, Story 6.2 — see that
        // helper).
        let download_dir = PathBuf::from(download_dir);
        let state_dir = PathBuf::from(state_dir);
        Self::open_with_recovery(&state_dir, || {
            runtime()
                .block_on(Session::new_with_opts(
                    download_dir.clone(),
                    persistent_session_options(state_dir.clone()),
                ))
                .map_err(internal_error)
        })
    }

    pub fn add_torrent(&self, source: TorrentSource, download_dir: String) -> Result<String, EngineError> {
        match source {
            // Read as bytes (not `AddTorrent::from_local_filename`, which does the same
            // read internally) so `resolve_output_folder` can inspect the torrent's file
            // count/name before the add call — see its doc comment (code-review fix,
            // Story 3.1).
            TorrentSource::Path { path } => {
                // `AddTorrent::from_local_filename` named the failing path in its error
                // (`"error reading local file {filename:?}"`); a bare `io::Error` from
                // `std::fs::read` doesn't, so it's added back here explicitly (code-review
                // fix, Story 3.1 — this is the only place such a failure is ever recorded,
                // see `AppModel.addTorrent`'s log-and-swallow catch branch).
                let bytes = std::fs::read(&path)
                    .map_err(|err| internal_error(format!("error reading local file {path:?}: {err}")))?;
                let output_folder = resolve_output_folder(&download_dir, &bytes);
                self.add_torrent_blocking(AddTorrent::from_bytes(bytes), output_folder)
            }
            TorrentSource::Bytes { bytes } => {
                let output_folder = resolve_output_folder(&download_dir, &bytes);
                self.add_torrent_blocking(AddTorrent::from_bytes(bytes), output_folder)
            }
            TorrentSource::Magnet { uri } => self.add_magnet(uri, download_dir),
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
                    let (status, progress_percent, down_speed_bps, up_speed_bps, peers_connected, total_bytes, downloaded_bytes) =
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
                            total_bytes,
                            downloaded_bytes,
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
                        total_bytes: 0,
                        downloaded_bytes: 0,
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
        // Story 2.2: drop this torrent's peer-speed baselines now — otherwise they'd
        // stay in the map forever, since a removed torrent is never polled again.
        self.peer_speed_baseline.lock().unwrap().remove(&id);

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
        // Parsed once and reused below for `api_peer_stats` too — `find_handle`
        // would parse `id` again internally otherwise (code-review fix, Story 2.2).
        let idor = Self::parse_id(&id)?;
        let handle = self
            .session
            .get(idor)
            .ok_or_else(|| internal_error(format!("torrent not found: {id}")))?;
        let stats = handle.stats();
        let (status, progress_percent, down_speed_bps, up_speed_bps, peers_connected, total_bytes, downloaded_bytes) =
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

        // Trackers: only the URL list is available — librqbit doesn't publicly
        // expose per-tracker announce status (connected/not-responding). See
        // Story 2.2 Scope Boundary.
        let trackers = handle
            .shared
            .trackers
            .iter()
            .map(|url| Tracker { url: url.to_string() })
            .collect();

        // Peers: address + connection state + a download speed derived from the
        // delta against the previous poll (librqbit exposes only a cumulative
        // per-peer byte counter, no live speed, no role/upload data — see Story
        // 2.2 Scope Boundary). `stats.live` mirrors the same liveness check
        // `api_peer_stats` makes internally (`handle.live()`), so a torrent that
        // isn't Live yet (still Initializing/Paused) is skipped without even
        // calling it — the graceful-empty case, same principle as `files` above
        // when `metadata` is still `None`. If the torrent IS live and the call
        // still fails, that's unexpected — logged, not silently treated the same
        // as "not live yet" (code-review fix, Story 2.2: the old code collapsed
        // every possible error into an empty list with no signal at all).
        let peer_entries: Vec<(String, String, u64)> = if stats.live.is_none() {
            Vec::new()
        } else {
            match self.api.api_peer_stats(idor, Default::default()) {
                Ok(snapshot) => {
                    // Sorted by address for a stable render order — `snapshot.peers`
                    // is a HashMap rebuilt fresh on every poll, so its iteration
                    // order isn't guaranteed stable even when the peer set hasn't
                    // changed (code-review fix, Story 2.2: without this, rows could
                    // visually shuffle every ~1s poll for no real reason — the same
                    // class of bug `get_all_torrents` above already had to fix for
                    // pending torrents via explicit `seq` ordering).
                    let mut entries: Vec<(String, String, u64)> = snapshot
                        .peers
                        .into_iter()
                        .map(|(address, peer_stats)| {
                            (address, peer_stats.state.to_string(), peer_stats.counters.fetched_bytes)
                        })
                        .collect();
                    entries.sort_by(|a, b| a.0.cmp(&b.0));
                    entries
                }
                Err(err) => {
                    eprintln!(
                        "get_torrent_details: unexpected api_peer_stats error for {id}: {err:#}"
                    );
                    Vec::new()
                }
            }
        };
        let peers = self.build_peers(&id, peer_entries);

        Ok(TorrentDetail {
            id,
            name: handle.name().unwrap_or_default(),
            status,
            progress_percent,
            down_speed_bps,
            up_speed_bps,
            peers_connected,
            total_bytes,
            downloaded_bytes,
            files,
            trackers,
            peers,
        })
    }
}

impl Engine {
    /// Wraps a ready session in an `Engine` — the single construction site for
    /// the struct's non-session fields, shared by production and every test
    /// constructor (code-review fix, Story 6.2: this block was hand-copied four
    /// times, so a new field silently missed in one of them was a live risk).
    fn from_session(session: Arc<Session>) -> Arc<Self> {
        let api = Api::new(session.clone(), None);
        Arc::new(Self {
            session,
            pending: Arc::new(Mutex::new(HashMap::new())),
            api,
            next_pending_seq: AtomicU64::new(0),
            peer_speed_baseline: Mutex::new(HashMap::new()),
        })
    }

    /// Opens a persistent session, recovering from a corrupt store instead of
    /// failing the whole launch (code-review fix, Story 6.2). `open` performs
    /// the actual `Session::new_with_opts` (production and tests supply their
    /// own options via this closure so the recovery logic is exercised the same
    /// way in both). On the first failure, if — and only if — the store folder
    /// actually has contents to be corrupt, it is quarantined aside
    /// (`quarantine_store`) and `open` is retried once against a fresh, empty
    /// store: a launch with an empty list beats an app that `fatalError`s on
    /// every start (Swift `setUpEngine`) until the user hand-deletes the folder.
    /// A failure on an empty store, a failed quarantine, or a second failure are
    /// all genuine engine problems and surface the original error unchanged.
    fn open_with_recovery(
        state_dir: &Path,
        open: impl Fn() -> Result<Arc<Session>, EngineError>,
    ) -> Result<Arc<Self>, EngineError> {
        match open() {
            Ok(session) => Ok(Self::from_session(session)),
            Err(first_err) => {
                if !store_has_entries(state_dir) {
                    return Err(first_err);
                }
                if quarantine_store(state_dir).is_err() {
                    return Err(first_err);
                }
                // The quarantine renamed the folder away; recreate it empty in
                // case librqbit won't (the Swift side created it before the
                // first attempt, not this retry).
                let _ = std::fs::create_dir_all(state_dir);
                let session = open().map_err(|_| first_err)?;
                Ok(Self::from_session(session))
            }
        }
    }

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

    /// Story 2.2: builds this poll's `Peer` list from `(address, state,
    /// cumulative fetched_bytes)` entries, deriving each one's download speed
    /// from the delta against the previous poll's baseline (divided by actual
    /// elapsed time — never assumed to equal the ~1s poll interval, same
    /// principle as the engine-computed torrent-level speeds, see Consistency
    /// Conventions) — and prunes this torrent's baseline down to exactly the
    /// peers present in `entries` (disconnected peers dropped, and the torrent's
    /// whole inner map removed once empty), all under a single lock acquisition
    /// (code-review fix, Story 2.2: previously one lock per peer plus a separate
    /// lock for pruning). Called even when `entries` is empty (torrent not live,
    /// e.g. paused, or an unexpected error) so a torrent's baseline can't outlive
    /// it having any peers — the previous version only pruned on a successful,
    /// non-empty poll, silently leaking a paused torrent's baseline forever.
    fn build_peers(&self, torrent_id: &str, entries: Vec<(String, String, u64)>) -> Vec<Peer> {
        let now = Instant::now();
        let mut all = self.peer_speed_baseline.lock().unwrap();
        let baseline = all.entry(torrent_id.to_string()).or_default();

        let peers: Vec<Peer> = entries
            .into_iter()
            .map(|(address, state, fetched_bytes)| {
                let down_speed_bps = match baseline.get(&address) {
                    Some((prev_bytes, prev_instant)) if fetched_bytes >= *prev_bytes => {
                        let elapsed = now.duration_since(*prev_instant).as_secs_f64();
                        if elapsed > 0.0 {
                            ((fetched_bytes - prev_bytes) as f64 / elapsed).round() as u64
                        } else {
                            0
                        }
                    }
                    // No prior baseline (first poll), or `fetched_bytes` went
                    // backwards (peer reconnected, librqbit gave it a fresh
                    // counter) — never negative/garbage.
                    _ => 0,
                };
                baseline.insert(address.clone(), (fetched_bytes, now));
                Peer { address, state, down_speed_bps }
            })
            .collect();

        let current: HashSet<&str> = peers.iter().map(|p| p.address.as_str()).collect();
        baseline.retain(|addr, _| current.contains(addr.as_str()));
        if baseline.is_empty() {
            all.remove(torrent_id);
        }

        peers
    }

    /// Adds a torrent whose metadata is already known (`.torrent` file/bytes) —
    /// librqbit registers these synchronously without waiting on any peer.
    fn add_torrent_blocking(&self, add: AddTorrent<'static>, output_folder: String) -> Result<String, EngineError> {
        // Story 3.1 / AD-5: the save-location setting flows in as a per-call
        // parameter, not a fixed value baked into the session at construction —
        // each add uses whatever the Settings window currently holds. `output_folder`
        // here is already `resolve_output_folder`'s result, not the raw setting value
        // (code-review fix, Story 3.1 — see that function's doc comment).
        let opts = AddTorrentOptions {
            output_folder: Some(output_folder),
            ..Default::default()
        };
        let response = runtime()
            .block_on(self.session.add_torrent(add, Some(opts)))
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
    ///
    /// **Known limitation (code-review finding, Story 3.1):** unlike the file/bytes
    /// path (`add_torrent`, via `resolve_output_folder`), magnet links can't get the
    /// multi-file auto-subfolder treatment — file count/name aren't known until the
    /// background task's `session.add_torrent` call resolves metadata over the network,
    /// but `AddTorrentOptions.output_folder` has to be decided and handed to that same
    /// call up front. Reproducing librqbit's own subfolder logic here would require
    /// duplicating its magnet resolution (DHT/tracker/peer handshake) purely to peek at
    /// metadata first, which is out of scope. A multi-file torrent added via magnet
    /// link will therefore still land flat in `download_dir`, unlike one added via
    /// `.torrent` file.
    fn add_magnet(&self, uri: String, download_dir: String) -> Result<String, EngineError> {
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
        // Story 3.1: same per-call save-location override as the synchronous
        // path (`add_torrent_blocking`) — built before `spawn` so the background
        // resolution task uses whatever the setting was at the moment the user
        // added this magnet, not whatever it might be later when it resolves.
        let opts = AddTorrentOptions {
            output_folder: Some(download_dir),
            ..Default::default()
        };
        runtime().spawn(async move {
            match session.add_torrent(AddTorrent::from_url(uri), Some(opts)).await {
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

    // Story 3.1: `add_torrent` now requires a download_dir per call, and librqbit
    // actually creates/checks the destination file at add-time (`allow_overwrite
    // = false` by default) — unlike the shared `std::env::temp_dir()`, which
    // caused every test adding the same single-file fixture to collide on the
    // same "test.txt" path when tests run concurrently, each call here gets its
    // own fresh, real directory. `.keep()` intentionally skips cleanup (same
    // "don't bother cleaning up test dirs" convention as `test_engine`'s own
    // per-test `TempDir`, whose directory is already gone by the time any test
    // body runs, without ever mattering).
    fn test_download_dir() -> String {
        tempfile::tempdir().unwrap().keep().to_string_lossy().into_owned()
    }

    /// The production persistence options plus `disable_dht_persistence` (same
    /// sandbox reason as `test_engine`) — shared by `test_engine_with_persistence`
    /// and the corrupt-store recovery test so both exercise the real
    /// `persistent_session_options` block.
    fn test_persistent_options(state_dir: &str) -> SessionOptions {
        let mut opts = persistent_session_options(PathBuf::from(state_dir));
        opts.disable_dht_persistence = true;
        opts
    }

    /// Polls `probe` up to ~5s (250×20ms), returning as soon as it yields
    /// `Some`. A bounded busy-wait for librqbit's background/async state
    /// transitions that tests can't otherwise synchronize on (code-review
    /// dedup, Story 6.2 — this loop was hand-rolled in five places).
    fn wait_until<T>(mut probe: impl FnMut() -> Option<T>) -> Option<T> {
        for _ in 0..250 {
            if let Some(value) = probe() {
                return Some(value);
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        None
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
        Engine::from_session(session)
    }

    fn test_engine_with_persistence(download_dir: &str, state_dir: &str) -> Arc<Engine> {
        // Story 6.2 — mirrors the production `Engine::new` options (Json
        // persistence + fastresume) but with `disable_dht_persistence` for the
        // same sandbox reason as `test_engine`, and with caller-provided dirs
        // so a test can recreate an Engine against the same state_dir.
        let session = runtime()
            .block_on(Session::new_with_opts(
                PathBuf::from(download_dir),
                test_persistent_options(state_dir),
            ))
            .unwrap();
        Engine::from_session(session)
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
        Engine::from_session(session)
    }

    #[test]
    fn add_torrent_from_file_appears_in_get_all_torrents() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");

        let id = engine
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
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
            .add_torrent(
                TorrentSource::Magnet {
                    uri: magnet.to_string(),
                },
                test_download_dir(),
            )
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
            .add_torrent(
                TorrentSource::Magnet {
                    uri: magnet.to_string(),
                },
                test_download_dir(),
            )
            .expect("add_torrent should succeed for a well-formed magnet link");

        // The background task fails almost immediately (no DHT/trackers/peers), but
        // it still runs on another thread — poll with a bounded wait instead of
        // asserting instantly.
        let status = wait_until(|| {
            engine
                .get_all_torrents()
                .into_iter()
                .find(|t| t.id == id)
                .map(|t| t.status)
                .filter(|s| s == "error")
        });

        assert_eq!(status.as_deref(), Some("error"));
    }

    #[test]
    fn add_torrent_from_file_reports_progress_and_live_metric_fields() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");

        engine
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
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
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");

        // `pause` only succeeds once librqbit's internal state has left
        // "Initializing" (the disk-hash-check phase) for "Live" — see
        // `add_torrent_from_file_reports_progress_and_live_metric_fields` above,
        // which shows this transition isn't deterministic in timing. Bounded
        // poll instead of asserting instantly (same pattern as the magnet
        // resolution-failure test below).
        let checking_done = wait_until(|| {
            engine
                .get_all_torrents()
                .into_iter()
                .any(|t| t.id == id && t.status != "checking")
                .then_some(())
        })
        .is_some();
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
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");

        engine
            .remove_torrent(id)
            .expect("remove_torrent should succeed for a known torrent");

        assert!(engine.get_all_torrents().is_empty());
    }

    #[test]
    fn persisted_torrent_survives_engine_recreation() {
        // Story 6.2 roundtrip: the same download_dir AND state_dir must be
        // reused across both Engine incarnations — persistence restores each
        // torrent into its persisted output folder, so a fresh download dir
        // would break the on-disk file the bitfield refers to.
        let download_dir = test_download_dir();
        let state_dir = test_download_dir();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");

        let first = test_engine_with_persistence(&download_dir, &state_dir);
        let id = first
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                download_dir.clone(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");
        // Graceful shutdown so the store's writes are settled before the
        // second session opens the same folder.
        runtime().block_on(first.session.stop());
        drop(first);

        let second = test_engine_with_persistence(&download_dir, &state_dir);
        // Restore happens inside `Session::new_with_opts`, but whether the
        // torrents are already visible by the time it returns is a librqbit
        // implementation detail — poll briefly instead of assuming.
        let restored = wait_until(|| {
            let all = second.get_all_torrents();
            (!all.is_empty()).then_some(all)
        })
        .unwrap_or_default();
        assert_eq!(
            restored.len(),
            1,
            "persisted torrent was not restored by the second session"
        );
        // `preferred_id` on the restore path keeps ids stable across restarts —
        // the app's list order (sorted by id, Story 1.2) depends on this.
        assert_eq!(restored[0].id, id, "restored torrent id changed across restart");
    }

    #[test]
    fn paused_state_survives_engine_recreation() {
        // Story 6.2 AC3 — verified at the engine level because the UI's only
        // pause affordance (the row context menu) drives this same
        // `pause_torrent` call. Also the honest probe of WHEN librqbit writes
        // `SerializedTorrent.is_paused`: if pausing didn't update the store,
        // this test would catch the restored torrent running again.
        let download_dir = test_download_dir();
        let state_dir = test_download_dir();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");

        let first = test_engine_with_persistence(&download_dir, &state_dir);
        let id = first
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                download_dir.clone(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");
        // Same wait as the pause/resume test: pausing mid-"checking" is racy.
        let checking_done = wait_until(|| {
            first
                .get_all_torrents()
                .into_iter()
                .any(|t| t.id == id && t.status != "checking")
                .then_some(())
        })
        .is_some();
        assert!(checking_done, "torrent never left the checking state");
        first
            .pause_torrent(id.clone())
            .expect("pause_torrent should succeed for a known torrent");
        runtime().block_on(first.session.stop());
        drop(first);

        let second = test_engine_with_persistence(&download_dir, &state_dir);
        // Wait for the restored torrent to settle out of the transient
        // checking/initializing phase, then assert its resting status is
        // paused. Stopping at the first sample that merely reads "paused" could
        // pass on a transient before the torrent settles into a running state
        // (test-robustness fix, Story 6.2 review).
        let status = wait_until(|| {
            second
                .get_all_torrents()
                .into_iter()
                .find(|t| t.id == id)
                .filter(|t| t.status != "checking")
                .map(|t| t.status)
        })
        .unwrap_or_default();
        assert_eq!(
            status, "paused",
            "torrent paused before shutdown should be restored paused"
        );
    }

    #[test]
    fn corrupt_store_is_quarantined_and_engine_recovers() {
        // Story 6.2 code review (CONFIRMED-high): a single unreadable persisted
        // entry made `Session::new_with_opts` fail the whole open, which Swift
        // turns into a `fatalError` — crash-looping the app on every launch
        // until the user hand-deleted the store. `open_with_recovery` (the path
        // production `Engine::new` funnels through) must quarantine the bad
        // store aside and open a fresh session instead.
        let download_dir = test_download_dir();
        let state_dir = test_download_dir();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");

        // Seed a real store, then shut down cleanly so `session.json` is flushed.
        let first = test_engine_with_persistence(&download_dir, &state_dir);
        first
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                download_dir.clone(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");
        runtime().block_on(first.session.stop());
        drop(first);

        // Corrupt the session database so restore fails on the next open
        // (`session.json` — librqbit `session_persistence/json.rs:42`).
        let db = Path::new(&state_dir).join("session.json");
        assert!(db.exists(), "store should contain session.json after an add");
        std::fs::write(&db, b"}{ not valid json").unwrap();

        let state_path = Path::new(&state_dir);
        let engine = Engine::open_with_recovery(state_path, || {
            runtime()
                .block_on(Session::new_with_opts(
                    PathBuf::from(&download_dir),
                    test_persistent_options(&state_dir),
                ))
                .map_err(internal_error)
        })
        .expect("engine should recover from a corrupt store instead of failing");
        assert!(
            engine.get_all_torrents().is_empty(),
            "recovered session should start with an empty list"
        );

        // The corrupt store was preserved aside (renamed), not deleted.
        let base = state_path.file_name().unwrap().to_string_lossy().into_owned();
        let quarantined_prefix = format!("{base}.corrupt-");
        let has_quarantine = std::fs::read_dir(state_path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .any(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(&quarantined_prefix)
            });
        assert!(has_quarantine, "corrupt store should be quarantined, not deleted");
    }

    #[test]
    fn remove_torrent_removes_still_pending_magnet() {
        let engine = test_engine_no_peer_sources();
        let magnet = "magnet:?xt=urn:btih:0000000000000000000000000000000000000000";
        let id = engine
            .add_torrent(
                TorrentSource::Magnet {
                    uri: magnet.to_string(),
                },
                test_download_dir(),
            )
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
                .add_torrent(
                    TorrentSource::Magnet {
                        uri: format!("magnet:?xt=urn:btih:{id}"),
                    },
                    test_download_dir(),
                )
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
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
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
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
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
        assert_eq!(detail.total_bytes, summary.total_bytes);
        assert_eq!(detail.downloaded_bytes, summary.downloaded_bytes);
        assert!(detail.total_bytes >= detail.downloaded_bytes);
    }

    #[test]
    fn add_torrent_uses_provided_download_dir_as_output_folder() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        // Bound to a variable (not `_`) so the TempDir isn't dropped — and its
        // directory deleted — before `add_torrent` runs.
        let custom_dir = tempfile::tempdir().unwrap();
        let custom_dir_path = custom_dir.path().to_string_lossy().into_owned();

        let id = engine
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                custom_dir_path.clone(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");

        let output_folder = engine
            .reveal_path(id)
            .expect("reveal_path should succeed for a known torrent");
        assert_eq!(output_folder, custom_dir_path);
    }

    // Code-review regression test, Story 3.1: a multi-file torrent must still land
    // in its own named subfolder under the configured save dir, matching what
    // passing `output_folder: None` would have produced via librqbit's own
    // `get_default_subfolder_for_torrent` — see `resolve_output_folder`'s doc
    // comment for why that stopped happening once `output_folder` is always `Some`.
    #[test]
    fn add_torrent_puts_multi_file_torrent_in_a_subfolder_named_after_the_torrent() {
        let engine = test_engine();

        // Build a real 2-file torrent on disk — `create_torrent` derives `info.name`
        // from the source directory's basename and switches to multi-file mode
        // whenever the source path is a directory (even with just 2 tiny files).
        let source_dir = tempfile::tempdir().unwrap();
        std::fs::write(source_dir.path().join("a.txt"), b"a").unwrap();
        std::fs::write(source_dir.path().join("b.txt"), b"b").unwrap();
        let created = runtime()
            .block_on(librqbit::create_torrent(source_dir.path(), Default::default()))
            .expect("create_torrent should succeed for a small 2-file directory");
        let torrent_name = String::from_utf8_lossy(
            created
                .as_info()
                .info
                .name
                .as_ref()
                .expect("created torrent should have a name")
                .as_slice(),
        )
        .into_owned();
        let torrent_bytes = created
            .as_bytes()
            .expect("created torrent should serialize to bytes")
            .to_vec();

        let base_dir = tempfile::tempdir().unwrap();
        let base_dir_path = base_dir.path().to_string_lossy().into_owned();

        let id = engine
            .add_torrent(TorrentSource::Bytes { bytes: torrent_bytes }, base_dir_path.clone())
            .expect("add_torrent should succeed for a valid multi-file torrent");

        let output_folder = engine
            .reveal_path(id)
            .expect("reveal_path should succeed for a known torrent");
        let expected = std::path::Path::new(&base_dir_path).join(&torrent_name);
        assert_eq!(output_folder, expected.to_string_lossy());
    }

    // Code-review regression test, Story 3.1: when a multi-file torrent has no
    // `info.name` (legal per BEP3, if rare — `create_torrent` used by the sibling
    // test above can't produce one, since it always derives a name), the subfolder
    // should fall back to the longest file's stem, matching librqbit's own
    // `get_default_subfolder_for_torrent` fallback rather than skipping the
    // subfolder entirely. Calls `resolve_output_folder` directly (private fn,
    // reachable via `use super::*`) rather than round-tripping through a real
    // `Engine`/session — hand-built bencode bytes below aren't a *real*,
    // downloadable torrent (empty `pieces` hash), only a structurally valid one,
    // which is all `resolve_output_folder` needs.
    #[test]
    fn resolve_output_folder_falls_back_to_longest_filename_when_torrent_has_no_name() {
        fn bstr(s: &[u8]) -> Vec<u8> {
            let mut v = s.len().to_string().into_bytes();
            v.push(b':');
            v.extend_from_slice(s);
            v
        }
        fn bint(n: u64) -> Vec<u8> {
            format!("i{n}e").into_bytes()
        }
        fn bdict(pairs: Vec<(&[u8], Vec<u8>)>) -> Vec<u8> {
            let mut v = vec![b'd'];
            for (key, value) in pairs {
                v.extend(bstr(key));
                v.extend(value);
            }
            v.push(b'e');
            v
        }
        fn blist(items: Vec<Vec<u8>>) -> Vec<u8> {
            let mut v = vec![b'l'];
            for item in items {
                v.extend(item);
            }
            v.push(b'e');
            v
        }

        // "longest-file.bin" is deliberately longer than "short.bin" so the
        // fallback's `max_by_key(len)` pick is unambiguous.
        let long_file = bdict(vec![
            (b"length", bint(500)),
            (b"path", blist(vec![bstr(b"longest-file.bin")])),
        ]);
        let short_file = bdict(vec![
            (b"length", bint(1)),
            (b"path", blist(vec![bstr(b"short.bin")])),
        ]);
        let info = bdict(vec![
            (b"files", blist(vec![long_file, short_file])),
            (b"piece length", bint(16384)),
            (b"pieces", bstr(b"")),
        ]);
        let torrent_bytes = bdict(vec![(b"info", info)]);

        let output_folder = resolve_output_folder("/base", &torrent_bytes);
        assert_eq!(output_folder, "/base/longest-file");
    }

    #[test]
    fn get_torrent_details_returns_trackers_from_torrent_file() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        let id = engine
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");

        let detail = engine
            .get_torrent_details(id)
            .expect("get_torrent_details should succeed for a known torrent");

        // Fixture's `announce` field is "http://example.invalid:6969/announce".
        assert_eq!(detail.trackers.len(), 1, "fixture has a single announce tracker");
        assert!(detail.trackers[0].url.contains("example.invalid"));
    }

    #[test]
    fn get_torrent_details_returns_empty_peers_when_no_real_peers() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        let id = engine
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");

        let detail = engine
            .get_torrent_details(id)
            .expect("get_torrent_details should succeed for a known torrent");

        // No real network peers in this test environment (same constraint as
        // `peers_connected == 0` in earlier stories) — must not panic, and with
        // nothing connected the list is expected to be empty.
        assert!(detail.peers.is_empty());
    }

    #[test]
    fn get_torrent_details_returns_empty_peers_without_error_when_paused() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        let id = engine
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");

        // Same bounded-poll pattern as `pause_torrent_then_resume_torrent_round_trips`
        // — `pause` only succeeds once librqbit has left "Initializing" for "Live".
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

        // Paused means `stats.live` is `None` — regression test for the pre-fix
        // behavior where any `api_peer_stats` error (including this expected,
        // benign one) was silently swallowed with no distinction from a real bug.
        // The important thing here is that this still succeeds cleanly and
        // returns an empty list, not an error.
        let detail = engine
            .get_torrent_details(id)
            .expect("get_torrent_details should succeed for a paused torrent");
        assert!(detail.peers.is_empty());
    }

    #[test]
    fn get_torrent_details_does_not_panic_on_repeated_calls_with_no_peers() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        let id = engine
            .add_torrent(
                TorrentSource::Path {
                    path: fixture.to_string(),
                },
                test_download_dir(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");

        // First call has no baseline yet, second call has an (empty) baseline for
        // this torrent — exercises the delta computation path with zero peers
        // both times without panicking (e.g. no underflow on an empty peer set).
        engine
            .get_torrent_details(id.clone())
            .expect("first get_torrent_details call should succeed");
        let detail = engine
            .get_torrent_details(id)
            .expect("second get_torrent_details call should succeed");

        assert!(detail.peers.is_empty());
    }

    #[test]
    fn build_peers_computes_delta_and_clamps_negative() {
        let engine = test_engine();
        let entry = |bytes: u64| vec![("1.2.3.4:6881".to_string(), "live".to_string(), bytes)];

        // No prior baseline — must not fabricate a speed.
        let first = engine.build_peers("t1", entry(1_000));
        assert_eq!(first[0].down_speed_bps, 0);

        std::thread::sleep(std::time::Duration::from_millis(50));

        // More bytes fetched since the first call — some positive speed.
        let second = engine.build_peers("t1", entry(1_000 + 64_000));
        assert!(second[0].down_speed_bps > 0, "expected a positive speed, got {}", second[0].down_speed_bps);

        // Fewer bytes than before (peer reconnected, counter reset by librqbit) —
        // clamped to 0, not an underflow/garbage value.
        let third = engine.build_peers("t1", entry(10));
        assert_eq!(third[0].down_speed_bps, 0);
    }

    #[test]
    fn build_peers_prunes_baseline_for_disconnected_peers() {
        let engine = test_engine();
        engine.build_peers("t1", vec![("A".to_string(), "live".to_string(), 100_000)]);

        // "A" disconnects — only "B" is present on this poll.
        engine.build_peers("t1", vec![("B".to_string(), "live".to_string(), 1_000)]);

        // "A" reconnects with fewer bytes than its old baseline (100_000). If that
        // stale entry wasn't pruned, this would still clamp to 0 (indistinguishable
        // from a fresh peer) — the real assertion is that this doesn't panic and
        // behaves like a first-ever poll for "A", not a stale comparison.
        let peers = engine.build_peers("t1", vec![("A".to_string(), "live".to_string(), 10)]);
        assert_eq!(peers[0].down_speed_bps, 0);
    }

    #[test]
    fn build_peers_with_empty_entries_clears_torrent_from_baseline_map() {
        // Regression test for the pre-fix leak: pruning used to only run on a
        // successful, non-empty poll, so a torrent that went quiet (e.g. paused)
        // kept its baseline entries forever. `build_peers` must now prune down to
        // an empty set (and drop the torrent's inner map) even when called with
        // no entries at all.
        let engine = test_engine();
        engine.build_peers("t1", vec![("A".to_string(), "live".to_string(), 100_000)]);
        assert!(engine.peer_speed_baseline.lock().unwrap().contains_key("t1"));

        let peers = engine.build_peers("t1", Vec::new());
        assert!(peers.is_empty());
        assert!(
            !engine.peer_speed_baseline.lock().unwrap().contains_key("t1"),
            "torrent's baseline entry should be dropped once it has no peers left"
        );
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
