//! Error types for SeedCore

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type alias for SeedCore operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for SeedCore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    /// Bencode parsing error
    BencodeError(String),

    /// Torrent metainfo error
    MetainfoError(String),

    /// Network error
    NetworkError(String),

    /// I/O error
    IoError(String),

    /// Invalid data
    InvalidData(String),

    /// Torrent not found
    TorrentNotFound(String),

    /// Request timed out
    Timeout(String),

    /// Cryptographic operation failed
    CryptoError(String),

    /// Database operation failed
    DatabaseError(String),

    /// Debrid service error
    DebridError(String),

    /// Generic error
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BencodeError(msg) => write!(f, "Bencode error: {msg}"),
            Self::MetainfoError(msg) => write!(f, "Metainfo error: {msg}"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
            Self::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
            Self::TorrentNotFound(msg) => write!(f, "Torrent not found: {msg}"),
            Self::Timeout(msg) => write!(f, "Timeout: {msg}"),
            Self::CryptoError(msg) => write!(f, "Crypto error: {msg}"),
            Self::DatabaseError(msg) => write!(f, "Database error: {msg}"),
            Self::DebridError(msg) => write!(f, "Debrid error: {msg}"),
            Self::Other(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout(err.to_string())
        } else {
            Self::NetworkError(err.to_string())
        }
    }
}

impl From<sled::Error> for Error {
    fn from(err: sled::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}
