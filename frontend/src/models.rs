use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TorrentStatus {
    Downloading,
    Seeding,
    Paused,
    Error,
    Checking,
    Queued,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum AppEvent {
    FullList(Vec<Torrent>, u64),
    Update(TorrentUpdate),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TorrentUpdate {
    pub hash: String,
    pub down_rate: Option<i64>,
    pub up_rate: Option<i64>,
    pub percent_complete: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Theme {
    Midnight,
    Light,
    Amoled,
}
