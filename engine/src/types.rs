#[derive(uniffi::Enum)]
pub enum TorrentSource {
    Path { path: String },
    Bytes { bytes: Vec<u8> },
    Magnet { uri: String },
}

#[derive(uniffi::Error, Debug)]
pub enum EngineError {
    Internal { message: String },
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::Internal { message } => write!(f, "{message}"),
        }
    }
}

#[derive(uniffi::Record)]
pub struct TorrentStatus {
    pub id: String,
    pub name: String,
    pub status: String,
    pub progress_percent: f64,
    pub down_speed_bps: u64,
    pub up_speed_bps: u64,
    pub peers_connected: u32,
}

#[derive(uniffi::Record)]
pub struct TorrentFile {
    pub name: String,
    pub size_bytes: u64,
    pub progress_percent: f64,
}

#[derive(uniffi::Record)]
pub struct Tracker {
    pub url: String,
}

#[derive(uniffi::Record)]
pub struct Peer {
    pub address: String,
    pub state: String,
    pub down_speed_bps: u64,
}

#[derive(uniffi::Record)]
pub struct TorrentDetail {
    pub id: String,
    pub name: String,
    pub status: String,
    pub progress_percent: f64,
    pub down_speed_bps: u64,
    pub up_speed_bps: u64,
    pub peers_connected: u32,
    pub files: Vec<TorrentFile>,
    pub trackers: Vec<Tracker>,
    pub peers: Vec<Peer>,
}
