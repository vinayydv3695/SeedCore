//! Tracker communication module
//! 
//! Implements HTTP and UDP tracker protocols for peer discovery.

pub mod http;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Tracker announce response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceResponse {
    /// Tracker warning message (optional)
    pub warning_message: Option<String>,
    
    /// Interval in seconds between announces
    pub interval: u32,
    
    /// Minimum announce interval
    pub min_interval: Option<u32>,
    
    /// Tracker ID for future requests
    pub tracker_id: Option<String>,
    
    /// Number of seeders (complete)
    pub complete: u32,
    
    /// Number of leechers (incomplete)
    pub incomplete: u32,
    
    /// List of peers
    pub peers: Vec<Peer>,
}

/// Peer information from tracker
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Peer {
    /// Peer ID (20 bytes)
    pub peer_id: Option<Vec<u8>>,
    
    /// Peer IP address and port
    pub addr: SocketAddr,
}

/// Announce event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnounceEvent {
    /// Regular announce (default)
    None,
    
    /// First announce (torrent started)
    Started,
    
    /// Torrent completed download
    Completed,
    
    /// Torrent stopped
    Stopped,
}

impl AnnounceEvent {
    /// Convert to string for HTTP tracker
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::None => None,
            Self::Started => Some("started"),
            Self::Completed => Some("completed"),
            Self::Stopped => Some("stopped"),
        }
    }
}

/// Announce request parameters
#[derive(Debug, Clone)]
pub struct AnnounceRequest {
    /// Info hash (20 bytes)
    pub info_hash: [u8; 20],
    
    /// Our peer ID (20 bytes)
    pub peer_id: [u8; 20],
    
    /// Port we're listening on
    pub port: u16,
    
    /// Total uploaded bytes
    pub uploaded: u64,
    
    /// Total downloaded bytes
    pub downloaded: u64,
    
    /// Bytes left to download
    pub left: u64,
    
    /// Whether we want compact peer list
    pub compact: bool,
    
    /// Number of peers wanted (default 50)
    pub numwant: Option<u32>,
    
    /// Event type
    pub event: AnnounceEvent,
}

impl Default for AnnounceRequest {
    fn default() -> Self {
        Self {
            info_hash: [0u8; 20],
            peer_id: [0u8; 20],
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: 0,
            compact: true,
            numwant: Some(50),
            event: AnnounceEvent::None,
        }
    }
}

/// Tracker status for UI display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackerStatus {
    /// Tracker is working normally
    Working,
    /// Currently updating (announcing)
    Updating,
    /// Tracker returned an error
    Error,
    /// Tracker disabled by user
    Disabled,
}

/// Detailed tracker information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerInfo {
    /// Tracker URL
    pub url: String,
    /// Current status
    pub status: TrackerStatus,
    /// Status message or error
    pub message: String,
    /// Number of peers from last announce
    pub peers: u32,
    /// Number of seeds (complete peers)
    pub seeds: u32,
    /// Number of leechers (incomplete peers)
    pub leechers: u32,
    /// Number of completed downloads
    pub downloaded: u32,
    /// Last successful announce time (unix timestamp)
    pub last_announce: Option<i64>,
    /// Next scheduled announce time (unix timestamp)
    pub next_announce: Option<i64>,
}
