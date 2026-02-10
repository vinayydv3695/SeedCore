// Debrid provider trait - abstract interface for all debrid services

use super::types::*;
use anyhow::Result;
use async_trait::async_trait;

/// Trait that all debrid providers must implement
#[async_trait]
pub trait DebridProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> DebridProviderType;

    /// Validate API credentials
    async fn validate_credentials(&self) -> Result<bool>;

    /// Get user information
    async fn get_user_info(&self) -> Result<UserInfo>;

    /// Check if a torrent is instantly available (cached)
    /// 
    /// # Arguments
    /// * `info_hash` - The torrent info hash (hex string, 40 characters)
    async fn check_instant_availability(&self, info_hash: &str) -> Result<CacheStatus>;

    /// Add a magnet link to the debrid service
    /// 
    /// # Arguments
    /// * `magnet_uri` - The magnet link
    /// 
    /// # Returns
    /// Torrent ID assigned by the service
    async fn add_magnet(&self, magnet_uri: &str) -> Result<TorrentId>;

    /// Add a torrent file to the debrid service
    /// 
    /// # Arguments
    /// * `torrent_data` - The raw .torrent file bytes
    /// 
    /// # Returns
    /// Torrent ID assigned by the service
    async fn add_torrent_file(&self, torrent_data: &[u8]) -> Result<TorrentId>;

    /// Select specific files from a torrent
    /// 
    /// # Arguments
    /// * `torrent_id` - The torrent ID from the service
    /// * `file_ids` - List of file IDs to download (or "all" for all files)
    async fn select_files(&self, torrent_id: &str, file_ids: Vec<usize>) -> Result<()>;

    /// Get information about a torrent
    /// 
    /// # Arguments
    /// * `torrent_id` - The torrent ID from the service
    async fn get_torrent_info(&self, torrent_id: &str) -> Result<DebridProgress>;

    /// Get download links for a torrent's files
    /// 
    /// # Arguments
    /// * `torrent_id` - The torrent ID from the service
    /// 
    /// # Returns
    /// List of files with download/stream links
    async fn get_download_links(&self, torrent_id: &str) -> Result<Vec<DebridFile>>;

    /// Unrestrict a link to get direct download URL
    /// 
    /// # Arguments
    /// * `link` - The hoster link to unrestrict
    /// 
    /// # Returns
    /// Direct download URL
    async fn unrestrict_link(&self, link: &str) -> Result<String>;

    /// Delete a torrent from the service
    /// 
    /// # Arguments
    /// * `torrent_id` - The torrent ID to delete
    async fn delete_torrent(&self, torrent_id: &str) -> Result<()>;

    /// Get list of active torrents
    async fn list_torrents(&self) -> Result<Vec<DebridProgress>>;
}
