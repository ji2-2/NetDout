use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
    pub output: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub id: String,
    pub url: String,
    pub output: PathBuf,
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Queued,
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct ChunkPlan {
    pub index: usize,
    pub start: u64,
    pub end_inclusive: u64,
}
