//! Application state management

use crate::database::Database;
use crate::debrid::{types::DownloadSource, DebridManager};
use crate::engine::TorrentEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::RwLock as TokioRwLock;
use tokio::task::JoinHandle;

/// Global application state
pub struct AppState {
    /// Active torrent engines (by info_hash hex)
    pub engines: Arc<TokioRwLock<HashMap<String, Arc<TokioRwLock<TorrentEngine>>>>>,

    /// Running engine task handles to prevent double-spawning
    pub engine_tasks: Arc<TokioRwLock<HashMap<String, JoinHandle<()>>>>,

    /// Active torrents metadata for quick UI access
    pub torrents: Arc<RwLock<HashMap<String, TorrentInfo>>>,

    /// Application settings
    pub settings: Arc<RwLock<Settings>>,

    /// Database for persistence
    pub database: Arc<Database>,

    /// Debrid service manager (handles Torbox, Real-Debrid, etc.)
    pub debrid_manager: Arc<TokioRwLock<DebridManager>>,

    /// Master password cached in memory (cleared on app exit)
    /// This is used to decrypt API keys when needed
    pub master_password: Arc<TokioRwLock<Option<String>>>,

    /// Cloud download task handles (by info_hash)
    pub cloud_download_tasks: Arc<TokioRwLock<HashMap<String, JoinHandle<()>>>>,

    /// Cloud file download progress (by info_hash -> file name -> progress)
    pub cloud_file_progress: Arc<RwLock<HashMap<String, HashMap<String, CloudFileProgress>>>>,
}

/// Cloud file download progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFileProgress {
    /// File name
    pub name: String,

    /// File size in bytes
    pub size: u64,

    /// Downloaded bytes
    pub downloaded: u64,

    /// Download speed (bytes/sec)
    pub speed: u64,

    /// File state
    pub state: CloudFileState,
}

/// Cloud file download state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudFileState {
    /// Waiting to start
    Queued,

    /// Currently downloading
    Downloading,

    /// Download complete
    Complete,

    /// Download failed
    Error,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> Self {
        // Get config directory
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("seedcore");

        // Create config directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            tracing::warn!("Failed to create config directory: {}", e);
        }

        // Open database
        let db_path = config_dir.join("data.db");
        let database = Database::open(&db_path).unwrap_or_else(|e| {
            tracing::error!("Failed to open database at {:?}: {}", db_path, e);
            panic!("Cannot start without database");
        });

        tracing::info!("Database opened at: {:?}", db_path);

        // Load settings from database
        let settings = database.load_settings().unwrap_or_default();

        // Initialize debrid manager (providers will be loaded when master password is provided)
        let debrid_manager = DebridManager::new();

        Self {
            engines: Arc::new(TokioRwLock::new(HashMap::new())),
            engine_tasks: Arc::new(TokioRwLock::new(HashMap::new())),
            torrents: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(settings.into())),
            database: Arc::new(database),
            debrid_manager: Arc::new(TokioRwLock::new(debrid_manager)),
            master_password: Arc::new(TokioRwLock::new(None)),
            cloud_download_tasks: Arc::new(TokioRwLock::new(HashMap::new())),
            cloud_file_progress: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Torrent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    /// Unique torrent ID (info hash)
    pub id: String,

    /// Torrent name
    pub name: String,

    /// Total size in bytes
    pub size: u64,

    /// Downloaded bytes
    pub downloaded: u64,

    /// Uploaded bytes
    pub uploaded: u64,

    /// Current state
    pub state: TorrentState,

    /// Download speed (bytes/sec)
    pub download_speed: u64,

    /// Upload speed (bytes/sec)
    pub upload_speed: u64,

    /// Number of connected peers
    pub peers: u32,

    /// Number of seeds
    pub seeds: u32,

    /// Download source type (P2P, Cloud, or Hybrid)
    pub source: DownloadSource,
}

/// Torrent state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TorrentState {
    /// Downloading
    Downloading,

    /// Seeding
    Seeding,

    /// Paused
    Paused,

    /// Checking files
    Checking,

    /// Error
    Error,

    /// Queued
    Queued,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Global download speed limit (bytes/sec, 0 = unlimited)
    pub download_limit: u64,

    /// Global upload speed limit (bytes/sec, 0 = unlimited)
    pub upload_limit: u64,

    /// Maximum number of active downloads
    pub max_active_downloads: u32,

    /// Maximum number of active uploads
    pub max_active_uploads: u32,

    /// Port for incoming connections
    pub listen_port: u16,

    /// Enable DHT
    pub enable_dht: bool,

    /// Enable PEX (Peer Exchange)
    pub enable_pex: bool,

    /// Dark mode enabled
    pub dark_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_limit: 0,
            upload_limit: 0,
            max_active_downloads: 3,
            max_active_uploads: 3,
            listen_port: 6881,
            enable_dht: true,
            enable_pex: true,
            dark_mode: true,
        }
    }
}

// Convert from database AppSettings to Settings
impl From<crate::database::AppSettings> for Settings {
    fn from(db_settings: crate::database::AppSettings) -> Self {
        Self {
            download_limit: db_settings.max_download_speed,
            upload_limit: db_settings.max_upload_speed,
            max_active_downloads: db_settings.max_concurrent_downloads as u32,
            max_active_uploads: 3, // Not stored in DB, use default
            listen_port: db_settings.listen_port,
            enable_dht: db_settings.enable_dht,
            enable_pex: db_settings.enable_pex,
            dark_mode: true, // Not stored in DB, use default
        }
    }
}
