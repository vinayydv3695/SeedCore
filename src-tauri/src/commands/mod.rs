//! Tauri commands - Frontend to Backend communication
//!
//! Split into focused submodules for maintainability:
//! - `general`: App info, settings, greeting
//! - `torrent`: P2P torrent operations (add, remove, start, pause, load)
//! - `debrid`: Cloud debrid operations (add cloud torrent, cache, debrid torrent management)
//! - `credentials`: Master password and credential management
//! - `info`: Monitoring data (peers, trackers, pieces, files, disk space)

mod general;
mod torrent;
mod debrid;
mod credentials;
mod info;

// Re-export all commands so lib.rs can reference them as commands::command_name
pub use general::*;
pub use torrent::*;
pub use debrid::*;
pub use credentials::*;
pub use info::*;

// Shared types used across submodules
use serde::{Serialize, Deserialize};

/// Serializable torrent metadata for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentMetadata {
    pub name: String,
    pub info_hash: String,
    pub total_size: u64,
    pub files: Vec<crate::torrent::FileInfoUI>,
    pub announce: String,
    pub creation_date: Option<i64>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
}

/// Credential status for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialStatus {
    pub provider: String,
    pub is_configured: bool,
    pub is_valid: Option<bool>,
    pub last_validated: Option<i64>,
}

/// Debrid settings for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebridSettings {
    pub enable_debrid: bool,
    pub debrid_preference: Vec<String>,
    pub smart_mode_enabled: bool,
}

/// Parse a provider string from the frontend into a DebridProviderType.
/// This is the single source of truth for provider name → enum mapping.
pub(crate) fn parse_provider(provider: &str) -> Result<crate::debrid::types::DebridProviderType, String> {
    match provider {
        "torbox" => Ok(crate::debrid::types::DebridProviderType::Torbox),
        "real-debrid" => Ok(crate::debrid::types::DebridProviderType::RealDebrid),
        _ => Err(format!("Unknown provider: {}", provider)),
    }
}
