//! Cloud download manager for debrid services
//! 
//! Handles downloading torrents through Real-Debrid, Torbox, etc.

use crate::debrid::types::{DebridProviderType, DebridFile};
use crate::debrid::DebridManager;
use crate::error::Result;
use crate::state::TorrentState;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};

/// Polling interval for checking debrid download status (in seconds)
const POLL_INTERVAL: u64 = 10;

/// Cloud download manager
pub struct CloudDownloadManager {
    /// Debrid manager for API calls
    debrid_manager: Arc<RwLock<DebridManager>>,
    /// HTTP client for downloading files
    client: reqwest::Client,
}

impl CloudDownloadManager {
    /// Create a new cloud download manager
    pub fn new(debrid_manager: Arc<RwLock<DebridManager>>) -> Self {
        Self {
            debrid_manager,
            client: reqwest::Client::new(),
        }
    }

    /// Add torrent to debrid service and start downloading
    /// Returns the debrid torrent ID
    pub async fn add_torrent(
        &self,
        magnet_or_hash: &str,
        provider: DebridProviderType,
    ) -> Result<String> {
        tracing::info!("Adding torrent to {:?}: {}", provider, magnet_or_hash);
        
        let debrid_manager = self.debrid_manager.read().await;
        let request = crate::debrid::AddTorrentRequest::Magnet(magnet_or_hash.to_string());
        let torrent_id = debrid_manager.add_to_cloud(provider, request).await?;
        
        tracing::info!("Added torrent to debrid service: {}", torrent_id.id);
        Ok(torrent_id.id)
    }

    /// Get download links for a torrent from debrid service
    pub async fn get_download_links(
        &self,
        torrent_id: &str,
        provider: DebridProviderType,
    ) -> Result<Vec<DebridFile>> {
        tracing::info!("Getting download links for torrent: {}", torrent_id);
        
        let debrid_manager = self.debrid_manager.read().await;
        let files = debrid_manager.get_download_links(provider, torrent_id).await?;
        
        Ok(files)
    }

    /// Download a file from a direct URL to the specified path
    pub async fn download_file(
        &self,
        url: &str,
        destination: PathBuf,
    ) -> Result<()> {
        tracing::info!("Downloading file from {} to {:?}", url, destination);
        
        // Create parent directories if they don't exist
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Download the file
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(crate::error::Error::NetworkError(format!(
                "Failed to download file: HTTP {}",
                response.status()
            )));
        }

        let bytes = response.bytes().await?;
        
        // Write to file
        let mut file = File::create(&destination).await?;
        file.write_all(&bytes).await?;
        file.flush().await?;
        
        tracing::info!("Downloaded file to {:?} ({} bytes)", destination, bytes.len());
        Ok(())
    }

    /// Download a file with progress reporting
    pub async fn download_file_with_progress<F>(
        &self,
        url: &str,
        destination: PathBuf,
        mut progress_callback: F,
    ) -> Result<()>
    where
        F: FnMut(u64, u64) + Send,
    {
        tracing::info!("Downloading file from {} to {:?}", url, destination);
        
        // Create parent directories if they don't exist
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Download the file with streaming
        let response = self.client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(crate::error::Error::NetworkError(format!(
                "Failed to download file: HTTP {}",
                response.status()
            )));
        }

        let total_size = response.content_length().unwrap_or(0);
        let mut file = File::create(&destination).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }
        
        file.flush().await?;
        
        tracing::info!("Downloaded file to {:?} ({} bytes)", destination, downloaded);
        Ok(())
    }

    /// Start a background task to poll debrid service and download files
    /// 
    /// This task will:
    /// 1. Poll the debrid service every POLL_INTERVAL seconds
    /// 2. Get download links when the torrent is ready
    /// 3. Download all files to the specified directory
    /// 4. Update torrent state in AppState
    pub async fn start_download_task(
        info_hash: String,
        debrid_torrent_id: String,
        provider: DebridProviderType,
        save_path: PathBuf,
        torrents: Arc<std::sync::RwLock<std::collections::HashMap<String, crate::state::TorrentInfo>>>,
        debrid_manager: Arc<RwLock<DebridManager>>,
        file_progress: Arc<std::sync::RwLock<std::collections::HashMap<String, std::collections::HashMap<String, crate::state::CloudFileProgress>>>>,
    ) {
        let info_hash_clone = info_hash.clone();
        let debrid_torrent_id_clone = debrid_torrent_id.clone();
        
        tokio::spawn(async move {
            tracing::info!(
                "Starting cloud download task for {} (debrid_id: {})",
                info_hash_clone,
                debrid_torrent_id_clone
            );

            // Poll until torrent is ready to download
            let files = loop {
                tracing::debug!("Polling debrid service for torrent {}", debrid_torrent_id_clone);
                
                let manager = debrid_manager.read().await;
                
                // First, check torrent status/progress
                match manager.get_progress(provider, &debrid_torrent_id_clone).await {
                    Ok(progress) => {
                        tracing::debug!(
                            "Torrent {} status: {:?}, progress: {:.1}%",
                            debrid_torrent_id_clone,
                            progress.status,
                            progress.progress
                        );
                        
                        // Update torrent progress in UI
                        if let Ok(mut torrent_map) = torrents.write() {
                            if let Some(torrent) = torrent_map.get_mut(&info_hash_clone) {
                                torrent.size = progress.total_size;
                                // Don't update downloaded here - we'll track that during file download
                            }
                        }
                        
                        // Check if we need to select files
                        use crate::debrid::types::DebridStatus;
                        if matches!(progress.status, DebridStatus::WaitingFilesSelection) {
                            tracing::info!("Torrent waiting for file selection, selecting all files");
                            
                            if let Err(e) = manager.select_files(provider, &debrid_torrent_id_clone, &[]).await {
                                tracing::error!("Failed to select files: {}", e);
                                
                                // Update torrent state to error
                                if let Ok(mut torrent_map) = torrents.write() {
                                    if let Some(torrent) = torrent_map.get_mut(&info_hash_clone) {
                                        torrent.state = TorrentState::Error;
                                    }
                                }
                                return;
                            }
                            
                            tracing::info!("Successfully selected all files, waiting for download to complete");
                        }
                        
                        // If downloaded (or downloading with high progress), try to get download links
                        if matches!(progress.status, DebridStatus::Downloaded) 
                            || (matches!(progress.status, DebridStatus::Downloading) && progress.progress > 95.0) {
                            
                            tracing::info!("Torrent is ready, getting download links");
                            
                            match manager.get_download_links(provider, &debrid_torrent_id_clone).await {
                                Ok(files) if !files.is_empty() => {
                                    tracing::info!("Got {} download links for torrent {}", files.len(), debrid_torrent_id_clone);
                                    break files;
                                }
                                Ok(_) => {
                                    tracing::debug!("No download links yet, waiting...");
                                }
                                Err(e) => {
                                    tracing::error!("Error getting download links: {}", e);
                                }
                            }
                        } else {
                            tracing::debug!(
                                "Torrent not ready yet (status: {:?}, progress: {:.1}%), waiting...",
                                progress.status,
                                progress.progress
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Error getting torrent progress: {}", e);
                        
                        // Update torrent state to error
                        if let Ok(mut torrent_map) = torrents.write() {
                            if let Some(torrent) = torrent_map.get_mut(&info_hash_clone) {
                                torrent.state = TorrentState::Error;
                            }
                        }
                        return;
                    }
                }
                
                // Wait before polling again
                sleep(Duration::from_secs(POLL_INTERVAL)).await;
            };

            // Calculate total size
            let total_size: u64 = files.iter().map(|f| f.size).sum();
            
            // Initialize file progress for all files
            if let Ok(mut progress_map) = file_progress.write() {
                let mut file_map = std::collections::HashMap::new();
                for file in &files {
                    file_map.insert(file.name.clone(), crate::state::CloudFileProgress {
                        name: file.name.clone(),
                        size: file.size,
                        downloaded: 0,
                        speed: 0,
                        state: crate::state::CloudFileState::Queued,
                    });
                }
                progress_map.insert(info_hash_clone.clone(), file_map);
            }
            
            // Update torrent info with total size
            if let Ok(mut torrent_map) = torrents.write() {
                if let Some(torrent) = torrent_map.get_mut(&info_hash_clone) {
                    torrent.size = total_size;
                    // Use the first file's name as the torrent name
                    if let Some(first_file) = files.first() {
                        torrent.name = first_file.name.clone();
                    }
                }
            }

            // Download each file
            let client = reqwest::Client::new();
            let mut total_downloaded: u64 = 0;

            for file in files {
                // Mark file as downloading
                if let Ok(mut progress_map) = file_progress.write() {
                    if let Some(file_map) = progress_map.get_mut(&info_hash_clone) {
                        if let Some(progress) = file_map.get_mut(&file.name) {
                            progress.state = crate::state::CloudFileState::Downloading;
                        }
                    }
                }

                // Use the file name directly for the destination path
                let file_path = save_path.join(&file.name);
                tracing::info!("Downloading file: {} -> {:?}", file.name, file_path);

                // Get download URL (prefer download_link, fallback to stream_link)
                let download_url = match file.download_link.as_ref().or(file.stream_link.as_ref()) {
                    Some(url) => url,
                    None => {
                        tracing::error!("No download URL for file: {}", file.name);
                        
                        // Mark file as error
                        if let Ok(mut progress_map) = file_progress.write() {
                            if let Some(file_map) = progress_map.get_mut(&info_hash_clone) {
                                if let Some(progress) = file_map.get_mut(&file.name) {
                                    progress.state = crate::state::CloudFileState::Error;
                                }
                            }
                        }
                        continue;
                    }
                };

                // Create parent directories
                if let Some(parent) = file_path.parent() {
                    if let Err(e) = tokio::fs::create_dir_all(parent).await {
                        tracing::error!("Failed to create directory {:?}: {}", parent, e);
                        
                        // Mark file as error
                        if let Ok(mut progress_map) = file_progress.write() {
                            if let Some(file_map) = progress_map.get_mut(&info_hash_clone) {
                                if let Some(progress) = file_map.get_mut(&file.name) {
                                    progress.state = crate::state::CloudFileState::Error;
                                }
                            }
                        }
                        continue;
                    }
                }

                // Download file with progress updates
                match download_file_with_state_update(
                    &client,
                    download_url,
                    &file_path,
                    &info_hash_clone,
                    &file.name,
                    file.size,
                    &torrents,
                    &file_progress,
                    &mut total_downloaded,
                ).await {
                    Ok(_) => {
                        tracing::info!("Successfully downloaded: {}", file.name);
                        
                        // Mark file as complete
                        if let Ok(mut progress_map) = file_progress.write() {
                            if let Some(file_map) = progress_map.get_mut(&info_hash_clone) {
                                if let Some(progress) = file_map.get_mut(&file.name) {
                                    progress.state = crate::state::CloudFileState::Complete;
                                    progress.downloaded = file.size;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to download {}: {}", file.name, e);
                        
                        // Mark file as error
                        if let Ok(mut progress_map) = file_progress.write() {
                            if let Some(file_map) = progress_map.get_mut(&info_hash_clone) {
                                if let Some(progress) = file_map.get_mut(&file.name) {
                                    progress.state = crate::state::CloudFileState::Error;
                                }
                            }
                        }
                    }
                }
            }

            // Mark torrent as complete
            if let Ok(mut torrent_map) = torrents.write() {
                if let Some(torrent) = torrent_map.get_mut(&info_hash_clone) {
                    torrent.state = TorrentState::Seeding;
                    torrent.downloaded = total_size;
                }
            }

            tracing::info!("Cloud download task completed for {}", info_hash_clone);
        });
    }
}

/// Helper function to download a file with state updates
async fn download_file_with_state_update(
    client: &reqwest::Client,
    url: &str,
    destination: &PathBuf,
    info_hash: &str,
    file_name: &str,
    file_size: u64,
    torrents: &Arc<std::sync::RwLock<std::collections::HashMap<String, crate::state::TorrentInfo>>>,
    file_progress: &Arc<std::sync::RwLock<std::collections::HashMap<String, std::collections::HashMap<String, crate::state::CloudFileProgress>>>>,
    total_downloaded: &mut u64,
) -> Result<()> {
    let response = client.get(url).send().await?;
    
    if !response.status().is_success() {
        return Err(crate::error::Error::NetworkError(format!(
            "Failed to download file: HTTP {}",
            response.status()
        )));
    }

    let mut file = File::create(destination).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut last_update = std::time::Instant::now();
    let mut last_downloaded = 0u64;

    use futures::StreamExt;
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        *total_downloaded += chunk.len() as u64;
        
        // Update state every 100KB or 1 second
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(last_update).as_secs_f64();
        
        if downloaded % (100 * 1024) == 0 || elapsed >= 1.0 {
            // Calculate speed
            let speed = if elapsed > 0.0 {
                ((downloaded - last_downloaded) as f64 / elapsed) as u64
            } else {
                0
            };
            
            // Update file progress
            if let Ok(mut progress_map) = file_progress.write() {
                if let Some(file_map) = progress_map.get_mut(info_hash) {
                    if let Some(progress) = file_map.get_mut(file_name) {
                        progress.downloaded = downloaded;
                        progress.speed = speed;
                    }
                }
            }
            
            // Update torrent progress
            if let Ok(mut torrent_map) = torrents.write() {
                if let Some(torrent) = torrent_map.get_mut(info_hash) {
                    torrent.downloaded = *total_downloaded;
                    torrent.download_speed = speed;
                }
            }
            
            last_update = now;
            last_downloaded = downloaded;
        }
    }
    
    file.flush().await?;
    
    // Final state update
    if let Ok(mut progress_map) = file_progress.write() {
        if let Some(file_map) = progress_map.get_mut(info_hash) {
            if let Some(progress) = file_map.get_mut(file_name) {
                progress.downloaded = downloaded;
                progress.speed = 0;
            }
        }
    }
    
    if let Ok(mut torrent_map) = torrents.write() {
        if let Some(torrent) = torrent_map.get_mut(info_hash) {
            torrent.downloaded = *total_downloaded;
        }
    }
    
    Ok(())
}
