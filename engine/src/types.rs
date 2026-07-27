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
