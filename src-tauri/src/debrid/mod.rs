// Debrid services integration module

pub mod provider;
pub mod types;
pub mod request_queue;
pub mod real_debrid;
pub mod torbox;

use std::collections::HashMap;
use std::sync::Arc;
use std::path::PathBuf;
use anyhow::{anyhow, Result};

pub use provider::DebridProvider;
pub use types::*;
pub use request_queue::RequestQueue;

/// Request to add a torrent (magnet or file)
pub enum AddTorrentRequest {
    Magnet(String),
    File(PathBuf),
}

/// Manages multiple debrid providers
pub struct DebridManager {
    /// Torbox provider (if configured)
    torbox: Option<Arc<dyn DebridProvider>>,
    /// Real-Debrid provider (if configured)
    real_debrid: Option<Arc<dyn DebridProvider>>,
    /// Provider preference order
    preference_order: Vec<DebridProviderType>,
}

impl DebridManager {
    /// Create a new DebridManager
    pub fn new() -> Self {
        Self {
            torbox: None,
            real_debrid: None,
            preference_order: vec![DebridProviderType::Torbox, DebridProviderType::RealDebrid],
        }
    }

    /// Set Torbox provider
    pub fn set_torbox(&mut self, provider: Arc<dyn DebridProvider>) {
        self.torbox = Some(provider);
    }

    /// Set Real-Debrid provider
    pub fn set_real_debrid(&mut self, provider: Arc<dyn DebridProvider>) {
        self.real_debrid = Some(provider);
    }

    /// Set provider preference order
    pub fn set_preference(&mut self, order: Vec<DebridProviderType>) {
        self.preference_order = order;
    }

    /// Initialize a provider with API key
    pub async fn initialize_provider(&mut self, provider_type: DebridProviderType, api_key: String) -> Result<()> {
        match provider_type {
            DebridProviderType::Torbox => {
                let provider = Arc::new(torbox::TorboxProvider::new(api_key));
                self.torbox = Some(provider);
            }
            DebridProviderType::RealDebrid => {
                let provider = Arc::new(real_debrid::RealDebridProvider::new(api_key));
                self.real_debrid = Some(provider);
            }
        }
        Ok(())
    }

    /// Validate a provider's credentials
    pub async fn validate_provider(&self, provider_type: DebridProviderType, api_key: &str) -> Result<bool> {
        let provider: Arc<dyn DebridProvider> = match provider_type {
            DebridProviderType::Torbox => Arc::new(torbox::TorboxProvider::new(api_key.to_string())),
            DebridProviderType::RealDebrid => Arc::new(real_debrid::RealDebridProvider::new(api_key.to_string())),
        };
        
        provider.validate_credentials().await
    }

    /// Get a provider by type
    pub fn get_provider(&self, provider_type: DebridProviderType) -> Option<&Arc<dyn DebridProvider>> {
        match provider_type {
            DebridProviderType::Torbox => self.torbox.as_ref(),
            DebridProviderType::RealDebrid => self.real_debrid.as_ref(),
        }
    }

    /// Check if a provider is configured
    pub fn is_configured(&self, provider_type: DebridProviderType) -> bool {
        self.get_provider(provider_type).is_some()
    }

    /// Check cache on all configured providers
    pub async fn check_cache_all(&self, info_hash: &str) -> Result<CacheCheckResult> {
        let mut results = HashMap::new();

        // Check Torbox
        if let Some(provider) = &self.torbox {
            match provider.check_instant_availability(info_hash).await {
                Ok(status) => {
                    results.insert(DebridProviderType::Torbox, status);
                }
                Err(e) => {
                    tracing::warn!("Torbox cache check failed: {}", e);
                    results.insert(DebridProviderType::Torbox, CacheStatus::not_cached());
                }
            }
        }

        // Check Real-Debrid
        if let Some(provider) = &self.real_debrid {
            match provider.check_instant_availability(info_hash).await {
                Ok(status) => {
                    results.insert(DebridProviderType::RealDebrid, status);
                }
                Err(e) => {
                    tracing::warn!("Real-Debrid cache check failed: {}", e);
                    results.insert(DebridProviderType::RealDebrid, CacheStatus::not_cached());
                }
            }
        }

        Ok(results)
    }

    /// Get the preferred cached provider based on preference order
    pub async fn get_preferred_cached(&self, info_hash: &str) -> Result<Option<DebridProviderType>> {
        let cache_results = self.check_cache_all(info_hash).await?;
        
        for provider_type in &self.preference_order {
            if let Some(status) = cache_results.get(provider_type) {
                if status.is_cached && status.instant_download {
                    return Ok(Some(*provider_type));
                }
            }
        }
        Ok(None)
    }

    /// Add a torrent to a specific provider
    pub async fn add_to_cloud(
        &self,
        provider_type: DebridProviderType,
        request: AddTorrentRequest,
    ) -> Result<TorrentId> {
        let provider = self
            .get_provider(provider_type)
            .ok_or_else(|| anyhow!("Provider {} not configured", provider_type.display_name()))?;

        match request {
            AddTorrentRequest::Magnet(magnet) => provider.add_magnet(&magnet).await,
            AddTorrentRequest::File(path) => {
                let data = std::fs::read(&path)?;
                provider.add_torrent_file(&data).await
            }
        }
    }

    /// Get download links from a provider
    pub async fn get_download_links(
        &self,
        provider_type: DebridProviderType,
        torrent_id: &str,
    ) -> Result<Vec<DebridFile>> {
        let provider = self
            .get_provider(provider_type)
            .ok_or_else(|| anyhow!("Provider {} not configured", provider_type.display_name()))?;

        provider.get_download_links(torrent_id).await
    }

    /// Select files for a torrent
    pub async fn select_files(
        &self,
        provider_type: DebridProviderType,
        torrent_id: &str,
        file_ids: &[usize],
    ) -> Result<()> {
        let provider = self
            .get_provider(provider_type)
            .ok_or_else(|| anyhow!("Provider {} not configured", provider_type.display_name()))?;

        provider.select_files(torrent_id, file_ids.to_vec()).await
    }

    /// Get torrent progress
    pub async fn get_progress(
        &self,
        provider_type: DebridProviderType,
        torrent_id: &str,
    ) -> Result<DebridProgress> {
        let provider = self
            .get_provider(provider_type)
            .ok_or_else(|| anyhow!("Provider {} not configured", provider_type.display_name()))?;

        provider.get_torrent_info(torrent_id).await
    }

    /// Delete a torrent from a provider
    pub async fn delete_torrent(
        &self,
        provider_type: DebridProviderType,
        torrent_id: &str,
    ) -> Result<()> {
        let provider = self
            .get_provider(provider_type)
            .ok_or_else(|| anyhow!("Provider {} not configured", provider_type.display_name()))?;

        provider.delete_torrent(torrent_id).await
    }

    /// List all torrents from a provider
    pub async fn list_torrents(
        &self,
        provider_type: DebridProviderType,
    ) -> Result<Vec<DebridProgress>> {
        let provider = self
            .get_provider(provider_type)
            .ok_or_else(|| anyhow!("Provider {} not configured", provider_type.display_name()))?;

        provider.list_torrents().await
    }

    /// Validate all configured providers
    pub async fn validate_all(&self) -> HashMap<DebridProviderType, bool> {
        let mut results = HashMap::new();

        if let Some(provider) = &self.torbox {
            let valid = provider.validate_credentials().await.unwrap_or(false);
            results.insert(DebridProviderType::Torbox, valid);
        }

        if let Some(provider) = &self.real_debrid {
            let valid = provider.validate_credentials().await.unwrap_or(false);
            results.insert(DebridProviderType::RealDebrid, valid);
        }

        results
    }
}

impl Default for DebridManager {
    fn default() -> Self {
        Self::new()
    }
}
