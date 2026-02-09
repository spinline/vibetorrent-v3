use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[cfg(feature = "ssr")]
pub mod scgi;

#[cfg(feature = "ssr")]
pub mod xmlrpc;

pub mod server_fns;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, ToSchema)]
pub struct Torrent {
    pub hash: String,
    pub name: String,
    pub size: i64,
    pub completed: i64,
    pub down_rate: i64,
    pub up_rate: i64,
    pub eta: i64,
    pub percent_complete: f64,
    pub status: TorrentStatus,
    pub error_message: String,
    pub added_date: i64,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub enum TorrentStatus {
    Downloading,
    Seeding,
    Paused,
    Error,
    Checking,
    Queued,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    FullList {
        torrents: Vec<Torrent>,
        timestamp: u64,
    },
    Update(TorrentUpdate),
    Stats(GlobalStats),
    Notification(SystemNotification),
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq, Eq)]
pub struct SystemNotification {
    pub level: NotificationLevel,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, Default)]
pub struct GlobalStats {
    pub down_rate: i64,
    pub up_rate: i64,
    pub down_limit: Option<i64>,
    pub up_limit: Option<i64>,
    pub free_space: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TorrentUpdate {
    pub hash: String,
    pub name: Option<String>,
    pub size: Option<i64>,
    pub down_rate: Option<i64>,
    pub up_rate: Option<i64>,
    pub percent_complete: Option<f64>,
    pub completed: Option<i64>,
    pub eta: Option<i64>,
    pub status: Option<TorrentStatus>,
    pub error_message: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TorrentActionRequest {
    #[schema(example = "5D4C9065...")]
    pub hash: String,
    #[schema(example = "start")]
    pub action: String,
}

// --- NEW STRUCTS FOR ADVANCED FEATURES ---

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TorrentFile {
    pub index: u32,
    pub path: String,
    pub size: i64,
    pub completed_chunks: i64,
    pub priority: u8, // 0: Off, 1: Normal, 2: High
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TorrentPeer {
    pub ip: String,
    pub client: String,
    pub down_rate: i64,
    pub up_rate: i64,
    pub progress: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TorrentTracker {
    pub url: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SetFilePriorityRequest {
    pub hash: String,
    pub file_index: u32,
    pub priority: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GlobalLimitRequest {
    pub max_upload_rate: Option<i64>,   // in bytes/s
    pub max_download_rate: Option<i64>, // in bytes/s
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SetLabelRequest {
    pub hash: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AddTorrentRequest {
    #[schema(example = "magnet:?xt=urn:btih:...")]
    pub uri: String,
}