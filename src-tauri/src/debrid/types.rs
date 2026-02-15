// Debrid service types and shared structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Debrid provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DebridProviderType {
    Torbox,
    RealDebrid,
}

impl DebridProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DebridProviderType::Torbox => "torbox",
            DebridProviderType::RealDebrid => "real-debrid",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DebridProviderType::Torbox => "Torbox",
            DebridProviderType::RealDebrid => "Real-Debrid",
        }
    }
}

/// Cache status for a torrent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheStatus {
    pub is_cached: bool,
    pub files: Vec<CachedFile>,
    pub instant_download: bool,
}

impl CacheStatus {
    pub fn not_cached() -> Self {
        Self {
            is_cached: false,
            files: Vec::new(),
            instant_download: false,
        }
    }

    pub fn cached(files: Vec<CachedFile>) -> Self {
        Self {
            is_cached: true,
            instant_download: !files.is_empty(),
            files,
        }
    }
}

/// Cached file information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedFile {
    pub id: usize,
    pub name: String,
    pub size: u64,
    pub selected: bool,
}

/// Debrid file with download link
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebridFile {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub download_link: Option<String>,
    pub stream_link: Option<String>,
    pub mime_type: Option<String>,
}

/// Debrid download progress
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebridProgress {
    pub torrent_id: String,
    pub status: DebridStatus,
    pub progress: f32, // 0.0 to 100.0
    pub speed: u64,    // bytes/sec
    pub downloaded: u64,
    pub total_size: u64,
    pub seeders: Option<u32>,
    pub eta: Option<u64>, // seconds
}

/// Debrid torrent status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DebridStatus {
    /// Waiting for file selection
    WaitingFilesSelection,
    /// Queued for download
    Queued,
    /// Currently downloading to cloud
    Downloading,
    /// Downloaded and ready
    Downloaded,
    /// Compressing files
    Compressing,
    /// Uploading to storage
    Uploading,
    /// Error occurred
    Error,
    /// Dead torrent
    Dead,
    /// Magnet conversion in progress
    MagnetConversion,
}

impl DebridStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, DebridStatus::Downloaded)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, DebridStatus::Error | DebridStatus::Dead)
    }
}

/// User information from debrid service
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub username: String,
    pub email: Option<String>,
    pub is_premium: bool,
    pub premium_expires: Option<i64>, // Unix timestamp
    pub points: Option<i64>,
}

/// Connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub connected: bool,
    pub valid: bool,
    pub message: String,
    pub user_info: Option<UserInfo>,
}

/// Torrent ID from debrid service
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentId {
    pub id: String,
    pub uri: Option<String>,
}

/// File selection request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSelection {
    pub torrent_id: String,
    pub file_ids: Vec<usize>,
}

/// Download source type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DownloadSource {
    /// Pure P2P download
    P2P,
    /// Pure cloud download
    Debrid {
        provider: DebridProviderType,
        torrent_id: String,
    },
    /// Hybrid: some files from cloud, some from P2P
    Hybrid {
        debrid_provider: DebridProviderType,
        debrid_torrent_id: String,
        debrid_file_ids: Vec<usize>,
        p2p_file_ids: Vec<usize>,
    },
}

impl DownloadSource {
    pub fn is_p2p(&self) -> bool {
        matches!(self, DownloadSource::P2P)
    }

    pub fn is_debrid(&self) -> bool {
        matches!(self, DownloadSource::Debrid { .. })
    }

    pub fn is_hybrid(&self) -> bool {
        matches!(self, DownloadSource::Hybrid { .. })
    }

    pub fn get_provider(&self) -> Option<DebridProviderType> {
        match self {
            DownloadSource::Debrid { provider, .. } => Some(*provider),
            DownloadSource::Hybrid {
                debrid_provider, ..
            } => Some(*debrid_provider),
            DownloadSource::P2P => None,
        }
    }
}

/// Cache check result for all providers
pub type CacheCheckResult = HashMap<DebridProviderType, CacheStatus>;
