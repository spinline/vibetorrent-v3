use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TorrentActionRequest {
    /// The hash of the torrent
    #[schema(example = "5D4C9065...")]
    pub hash: String,
    /// The action to perform: "start", "stop", "delete", "delete_with_data"
    #[schema(example = "start")]
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, ToSchema)]
pub enum Theme {
    Midnight,
    Light,
    Amoled,
}
