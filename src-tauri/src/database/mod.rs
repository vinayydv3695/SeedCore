/// Database module for persistent storage using Sled
/// Stores torrent metadata, download progress, and settings
use crate::debrid::types::{DebridProviderType, DownloadSource};
use crate::error::{Error, Result};
use crate::torrent::Metainfo;
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;

/// Database keys
const KEY_TORRENTS: &[u8] = b"torrents";
const KEY_PROGRESS: &[u8] = b"progress";
const KEY_SETTINGS: &[u8] = b"settings";
const KEY_DEBRID_CREDENTIALS: &[u8] = b"debrid_credentials";
const KEY_MASTER_PASSWORD: &[u8] = b"master_password";

/// Download session data stored in database (renamed from TorrentSession)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentSession {
    /// Unique torrent ID (info hash as hex)
    pub id: String,
    /// Torrent metadata
    pub metainfo: Metainfo,
    /// Download progress (bitfield as bytes)
    pub bitfield: Vec<u8>,
    /// Number of pieces
    pub num_pieces: usize,
    /// Downloaded bytes
    pub downloaded: u64,
    /// Uploaded bytes
    pub uploaded: u64,
    /// Torrent state
    pub state: String, // "downloading", "seeding", "paused", "stopped"
    /// Download directory
    pub download_dir: String,
    /// Time added (Unix timestamp)
    pub added_at: i64,
    /// Last activity timestamp
    pub last_activity: i64,
    /// Download source type (P2P, Debrid, or Hybrid)
    pub source: DownloadSource,
}

/// Debrid provider credentials stored encrypted in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebridCredentials {
    /// Provider type (Torbox or RealDebrid)
    pub provider: DebridProviderType,
    /// Encrypted API key (AES-256-GCM ciphertext)
    pub api_key_encrypted: Vec<u8>,
    /// Nonce used for encryption
    pub nonce: Vec<u8>,
    /// Time credentials were added (Unix timestamp)
    pub created_at: i64,
    /// Last time credentials were validated (Unix timestamp)
    pub last_validated: i64,
    /// Whether credentials are valid
    pub is_valid: bool,
}

/// Master password hash stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterPasswordData {
    /// Argon2 password hash
    pub password_hash: Vec<u8>,
    /// Salt used for key derivation (for API key encryption)
    pub salt: Vec<u8>,
}

/// Application settings stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Default download directory
    pub download_dir: String,
    /// Maximum download speed (bytes/sec, 0 = unlimited)
    pub max_download_speed: u64,
    /// Maximum upload speed (bytes/sec, 0 = unlimited)
    pub max_upload_speed: u64,
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: usize,
    /// Port for incoming connections
    pub listen_port: u16,
    /// Enable DHT
    pub enable_dht: bool,
    /// Enable PEX
    pub enable_pex: bool,
    /// Enable debrid services
    pub enable_debrid: bool,
    /// Debrid provider preference order (first = most preferred)
    pub debrid_preference: Vec<DebridProviderType>,
    /// Smart mode: auto-select best source (cloud vs P2P)
    pub smart_mode_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_dir: dirs::download_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap())
                .to_string_lossy()
                .to_string(),
            max_download_speed: 0, // Unlimited
            max_upload_speed: 0,   // Unlimited
            max_concurrent_downloads: 3,
            listen_port: 6881,
            enable_dht: true,
            enable_pex: true,
            enable_debrid: false,
            debrid_preference: vec![DebridProviderType::Torbox, DebridProviderType::RealDebrid],
            smart_mode_enabled: true,
        }
    }
}

/// Database manager
pub struct Database {
    db: Db,
}

impl Database {
    /// Open or create a database at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)
            .map_err(|e| Error::IoError(format!("Failed to open database: {}", e)))?;

        Ok(Self { db })
    }

    /// Save a torrent session
    pub fn save_torrent(&self, session: &TorrentSession) -> Result<()> {
        let tree = self
            .db
            .open_tree(KEY_TORRENTS)
            .map_err(|e| Error::IoError(format!("Failed to open torrents tree: {}", e)))?;

        let data = bincode::serialize(session)
            .map_err(|e| Error::IoError(format!("Failed to serialize torrent: {}", e)))?;

        tree.insert(session.id.as_bytes(), data)
            .map_err(|e| Error::IoError(format!("Failed to save torrent: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| Error::IoError(format!("Failed to flush database: {}", e)))?;

        tracing::debug!("Saved torrent session: {}", session.id);
        Ok(())
    }

    /// Load a torrent session by ID
    pub fn load_torrent(&self, id: &str) -> Result<Option<TorrentSession>> {
        let tree = self
            .db
            .open_tree(KEY_TORRENTS)
            .map_err(|e| Error::IoError(format!("Failed to open torrents tree: {}", e)))?;

        match tree
            .get(id.as_bytes())
            .map_err(|e| Error::IoError(format!("Failed to load torrent: {}", e)))?
        {
            Some(data) => {
                let session = bincode::deserialize(&data)
                    .map_err(|e| Error::IoError(format!("Failed to deserialize torrent: {}", e)))?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// Load all torrent sessions
    pub fn load_all_torrents(&self) -> Result<Vec<TorrentSession>> {
        let tree = self
            .db
            .open_tree(KEY_TORRENTS)
            .map_err(|e| Error::IoError(format!("Failed to open torrents tree: {}", e)))?;

        let mut sessions = Vec::new();

        for item in tree.iter() {
            let (_, data) =
                item.map_err(|e| Error::IoError(format!("Failed to iterate torrents: {}", e)))?;

            let session = bincode::deserialize(&data)
                .map_err(|e| Error::IoError(format!("Failed to deserialize torrent: {}", e)))?;

            sessions.push(session);
        }

        tracing::info!("Loaded {} torrent sessions", sessions.len());
        Ok(sessions)
    }

    /// Delete a torrent session
    pub fn delete_torrent(&self, id: &str) -> Result<()> {
        let tree = self
            .db
            .open_tree(KEY_TORRENTS)
            .map_err(|e| Error::IoError(format!("Failed to open torrents tree: {}", e)))?;

        tree.remove(id.as_bytes())
            .map_err(|e| Error::IoError(format!("Failed to delete torrent: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| Error::IoError(format!("Failed to flush database: {}", e)))?;

        tracing::debug!("Deleted torrent session: {}", id);
        Ok(())
    }

    /// Update torrent progress
    pub fn update_progress(
        &self,
        id: &str,
        bitfield: Vec<u8>,
        downloaded: u64,
        uploaded: u64,
    ) -> Result<()> {
        if let Some(mut session) = self.load_torrent(id)? {
            session.bitfield = bitfield;
            session.downloaded = downloaded;
            session.uploaded = uploaded;
            session.last_activity = chrono::Utc::now().timestamp();
            self.save_torrent(&session)?;
        }
        Ok(())
    }

    /// Update torrent state
    pub fn update_state(&self, id: &str, state: String) -> Result<()> {
        if let Some(mut session) = self.load_torrent(id)? {
            session.state = state;
            session.last_activity = chrono::Utc::now().timestamp();
            self.save_torrent(&session)?;
        }
        Ok(())
    }

    /// Save application settings
    pub fn save_settings(&self, settings: &AppSettings) -> Result<()> {
        let tree = self
            .db
            .open_tree(KEY_SETTINGS)
            .map_err(|e| Error::IoError(format!("Failed to open settings tree: {}", e)))?;

        let data = bincode::serialize(settings)
            .map_err(|e| Error::IoError(format!("Failed to serialize settings: {}", e)))?;

        tree.insert(b"app", data)
            .map_err(|e| Error::IoError(format!("Failed to save settings: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| Error::IoError(format!("Failed to flush database: {}", e)))?;

        tracing::debug!("Saved application settings");
        Ok(())
    }

    /// Load application settings
    pub fn load_settings(&self) -> Result<AppSettings> {
        let tree = self
            .db
            .open_tree(KEY_SETTINGS)
            .map_err(|e| Error::IoError(format!("Failed to open settings tree: {}", e)))?;

        match tree
            .get(b"app")
            .map_err(|e| Error::IoError(format!("Failed to load settings: {}", e)))?
        {
            Some(data) => {
                let settings = bincode::deserialize(&data).map_err(|e| {
                    Error::IoError(format!("Failed to deserialize settings: {}", e))
                })?;
                Ok(settings)
            }
            None => {
                // Return default settings if none exist
                let settings = AppSettings::default();
                self.save_settings(&settings)?;
                Ok(settings)
            }
        }
    }

    /// Clear all data (for testing)
    pub fn clear_all(&self) -> Result<()> {
        self.db
            .clear()
            .map_err(|e| Error::IoError(format!("Failed to clear database: {}", e)))?;
        self.db
            .flush()
            .map_err(|e| Error::IoError(format!("Failed to flush database: {}", e)))?;
        Ok(())
    }

    /// Get database statistics
    pub fn stats(&self) -> DatabaseStats {
        DatabaseStats {
            size_on_disk: self.db.size_on_disk().unwrap_or(0),
        }
    }

    /// Save debrid credentials for a provider
    pub fn save_debrid_credentials(&self, credentials: &DebridCredentials) -> Result<()> {
        let tree = self
            .db
            .open_tree(KEY_DEBRID_CREDENTIALS)
            .map_err(|e| Error::IoError(format!("Failed to open credentials tree: {}", e)))?;

        let data = bincode::serialize(credentials)
            .map_err(|e| Error::IoError(format!("Failed to serialize credentials: {}", e)))?;

        let key = credentials.provider.as_str().as_bytes();
        tree.insert(key, data)
            .map_err(|e| Error::IoError(format!("Failed to save credentials: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| Error::IoError(format!("Failed to flush database: {}", e)))?;

        tracing::debug!("Saved credentials for provider: {:?}", credentials.provider);
        Ok(())
    }

    /// Load debrid credentials for a provider
    pub fn load_debrid_credentials(
        &self,
        provider: DebridProviderType,
    ) -> Result<Option<DebridCredentials>> {
        let tree = self
            .db
            .open_tree(KEY_DEBRID_CREDENTIALS)
            .map_err(|e| Error::IoError(format!("Failed to open credentials tree: {}", e)))?;

        let key = provider.as_str().as_bytes();
        match tree
            .get(key)
            .map_err(|e| Error::IoError(format!("Failed to load credentials: {}", e)))?
        {
            Some(data) => {
                let credentials = bincode::deserialize(&data).map_err(|e| {
                    Error::IoError(format!("Failed to deserialize credentials: {}", e))
                })?;
                Ok(Some(credentials))
            }
            None => Ok(None),
        }
    }

    /// Load all debrid credentials
    pub fn load_all_debrid_credentials(&self) -> Result<Vec<DebridCredentials>> {
        let tree = self
            .db
            .open_tree(KEY_DEBRID_CREDENTIALS)
            .map_err(|e| Error::IoError(format!("Failed to open credentials tree: {}", e)))?;

        let mut credentials_list = Vec::new();

        for item in tree.iter() {
            let (_, data) =
                item.map_err(|e| Error::IoError(format!("Failed to iterate credentials: {}", e)))?;

            let credentials = bincode::deserialize(&data)
                .map_err(|e| Error::IoError(format!("Failed to deserialize credentials: {}", e)))?;

            credentials_list.push(credentials);
        }

        tracing::info!("Loaded {} debrid credentials", credentials_list.len());
        Ok(credentials_list)
    }

    /// Delete debrid credentials for a provider
    pub fn delete_debrid_credentials(&self, provider: DebridProviderType) -> Result<()> {
        let tree = self
            .db
            .open_tree(KEY_DEBRID_CREDENTIALS)
            .map_err(|e| Error::IoError(format!("Failed to open credentials tree: {}", e)))?;

        let key = provider.as_str().as_bytes();
        tree.remove(key)
            .map_err(|e| Error::IoError(format!("Failed to delete credentials: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| Error::IoError(format!("Failed to flush database: {}", e)))?;

        tracing::debug!("Deleted credentials for provider: {:?}", provider);
        Ok(())
    }

    /// Save master password data (hash and salt)
    pub fn save_master_password(&self, password_data: &MasterPasswordData) -> Result<()> {
        let tree = self
            .db
            .open_tree(KEY_MASTER_PASSWORD)
            .map_err(|e| Error::IoError(format!("Failed to open master password tree: {}", e)))?;

        let data = bincode::serialize(password_data)
            .map_err(|e| Error::IoError(format!("Failed to serialize master password: {}", e)))?;

        tree.insert(b"data", data)
            .map_err(|e| Error::IoError(format!("Failed to save master password: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| Error::IoError(format!("Failed to flush database: {}", e)))?;

        tracing::debug!("Saved master password data");
        Ok(())
    }

    /// Load master password data
    pub fn load_master_password(&self) -> Result<Option<MasterPasswordData>> {
        let tree = self
            .db
            .open_tree(KEY_MASTER_PASSWORD)
            .map_err(|e| Error::IoError(format!("Failed to open master password tree: {}", e)))?;

        match tree
            .get(b"data")
            .map_err(|e| Error::IoError(format!("Failed to load master password: {}", e)))?
        {
            Some(data) => {
                let password_data = bincode::deserialize(&data).map_err(|e| {
                    Error::IoError(format!("Failed to deserialize master password: {}", e))
                })?;
                Ok(Some(password_data))
            }
            None => Ok(None),
        }
    }

    /// Check if master password is set
    pub fn has_master_password(&self) -> Result<bool> {
        Ok(self.load_master_password()?.is_some())
    }

    /// Delete master password (and all debrid credentials for security)
    pub fn delete_master_password(&self) -> Result<()> {
        let tree = self
            .db
            .open_tree(KEY_MASTER_PASSWORD)
            .map_err(|e| Error::IoError(format!("Failed to open master password tree: {}", e)))?;

        tree.clear()
            .map_err(|e| Error::IoError(format!("Failed to clear master password: {}", e)))?;

        // Also clear all debrid credentials since they can't be decrypted without the password
        let creds_tree = self
            .db
            .open_tree(KEY_DEBRID_CREDENTIALS)
            .map_err(|e| Error::IoError(format!("Failed to open credentials tree: {}", e)))?;

        creds_tree
            .clear()
            .map_err(|e| Error::IoError(format!("Failed to clear credentials: {}", e)))?;

        self.db
            .flush()
            .map_err(|e| Error::IoError(format!("Failed to flush database: {}", e)))?;

        tracing::warn!("Deleted master password and all debrid credentials");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub size_on_disk: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::{FileInfo, TorrentInfo};
    use tempfile::TempDir;

    fn create_test_metainfo() -> Metainfo {
        Metainfo {
            announce: "http://tracker.example.com/announce".to_string(),
            announce_list: vec![],
            info: TorrentInfo {
                piece_length: 16384,
                pieces: vec![0u8; 40],
                piece_count: 2,
                files: vec![FileInfo {
                    path: vec!["test.txt".to_string()],
                    length: 20000,
                }],
                name: "test.txt".to_string(),
                total_size: 20000,
                is_single_file: true,
            },
            info_hash: [0u8; 20],
            creation_date: None,
            comment: None,
            created_by: None,
        }
    }

    #[test]
    fn test_database_open() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(&db_path).unwrap();
        assert!(db_path.exists());
    }

    #[test]
    fn test_save_and_load_torrent() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path().join("test.db")).unwrap();

        let session = TorrentSession {
            id: "test123".to_string(),
            metainfo: create_test_metainfo(),
            bitfield: vec![0b11000000],
            num_pieces: 2,
            downloaded: 16384,
            uploaded: 0,
            state: "downloading".to_string(),
            download_dir: "/tmp/downloads".to_string(),
            added_at: 1234567890,
            last_activity: 1234567890,
            source: DownloadSource::P2P,
        };

        db.save_torrent(&session).unwrap();

        let loaded = db.load_torrent("test123").unwrap().unwrap();
        assert_eq!(loaded.id, session.id);
        assert_eq!(loaded.downloaded, session.downloaded);
        assert_eq!(loaded.state, session.state);
    }

    #[test]
    fn test_load_all_torrents() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path().join("test.db")).unwrap();

        let session1 = TorrentSession {
            id: "torrent1".to_string(),
            metainfo: create_test_metainfo(),
            bitfield: vec![],
            num_pieces: 2,
            downloaded: 0,
            uploaded: 0,
            state: "downloading".to_string(),
            download_dir: "/tmp".to_string(),
            added_at: 1234567890,
            last_activity: 1234567890,
            source: DownloadSource::P2P,
        };

        let session2 = TorrentSession {
            id: "torrent2".to_string(),
            metainfo: create_test_metainfo(),
            bitfield: vec![],
            num_pieces: 2,
            downloaded: 0,
            uploaded: 0,
            state: "seeding".to_string(),
            download_dir: "/tmp".to_string(),
            added_at: 1234567890,
            last_activity: 1234567890,
            source: DownloadSource::P2P,
        };

        db.save_torrent(&session1).unwrap();
        db.save_torrent(&session2).unwrap();

        let all = db.load_all_torrents().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_delete_torrent() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path().join("test.db")).unwrap();

        let session = TorrentSession {
            id: "delete_me".to_string(),
            metainfo: create_test_metainfo(),
            bitfield: vec![],
            num_pieces: 2,
            downloaded: 0,
            uploaded: 0,
            state: "downloading".to_string(),
            download_dir: "/tmp".to_string(),
            added_at: 1234567890,
            last_activity: 1234567890,
            source: DownloadSource::P2P,
        };

        db.save_torrent(&session).unwrap();
        assert!(db.load_torrent("delete_me").unwrap().is_some());

        db.delete_torrent("delete_me").unwrap();
        assert!(db.load_torrent("delete_me").unwrap().is_none());
    }

    #[test]
    fn test_update_progress() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path().join("test.db")).unwrap();

        let session = TorrentSession {
            id: "progress_test".to_string(),
            metainfo: create_test_metainfo(),
            bitfield: vec![0b00000000],
            num_pieces: 2,
            downloaded: 0,
            uploaded: 0,
            state: "downloading".to_string(),
            download_dir: "/tmp".to_string(),
            added_at: 1234567890,
            last_activity: 1234567890,
            source: DownloadSource::P2P,
        };

        db.save_torrent(&session).unwrap();

        db.update_progress("progress_test", vec![0b11000000], 16384, 1024)
            .unwrap();

        let updated = db.load_torrent("progress_test").unwrap().unwrap();
        assert_eq!(updated.bitfield, vec![0b11000000]);
        assert_eq!(updated.downloaded, 16384);
        assert_eq!(updated.uploaded, 1024);
    }

    #[test]
    fn test_save_and_load_settings() {
        let temp_dir = TempDir::new().unwrap();
        let db = Database::open(temp_dir.path().join("test.db")).unwrap();

        let settings = AppSettings {
            download_dir: "/custom/downloads".to_string(),
            max_download_speed: 1024000,
            max_upload_speed: 512000,
            max_concurrent_downloads: 5,
            listen_port: 6882,
            enable_dht: false,
            enable_pex: true,
            enable_debrid: true,
            debrid_preference: vec![DebridProviderType::RealDebrid],
            smart_mode_enabled: false,
        };

        db.save_settings(&settings).unwrap();

        let loaded = db.load_settings().unwrap();
        assert_eq!(loaded.download_dir, settings.download_dir);
        assert_eq!(loaded.max_download_speed, settings.max_download_speed);
        assert_eq!(loaded.listen_port, settings.listen_port);
    }
}
