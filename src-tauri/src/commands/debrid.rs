//! Debrid commands: cloud torrents, cache checking, debrid torrent management

use crate::state::AppState;
use crate::debrid::types::{CacheStatus, DebridFile, DebridProgress};
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use tauri::State;

/// Add and download a torrent using cloud debrid service
#[tauri::command]
pub async fn add_cloud_torrent(
    state: State<'_, AppState>,
    magnet_or_hash: String,
    provider: String,
    save_path: String,
) -> Result<String, String> {
    tracing::info!("Adding cloud torrent via {}: {}", provider, magnet_or_hash);

    let provider_type = super::parse_provider(&provider)?;

    // Convert to magnet URI if just hash
    let magnet_uri = if magnet_or_hash.starts_with("magnet:") {
        magnet_or_hash.clone()
    } else {
        format!("magnet:?xt=urn:btih:{}", magnet_or_hash)
    };

    // Add to debrid service
    let debrid_manager = state.debrid_manager.read().await;
    let request = crate::debrid::AddTorrentRequest::Magnet(magnet_uri.clone());
    let torrent_id_result = debrid_manager.add_to_cloud(provider_type, request)
        .await
        .map_err(|e| format!("Failed to add to debrid: {}", e))?;

    tracing::info!("Added to debrid service: {}", torrent_id_result.id);

    // For Real-Debrid, we need to check if file selection is required
    match debrid_manager.get_progress(provider_type, &torrent_id_result.id).await {
        Ok(progress) => {
            tracing::info!("Torrent status: {:?}", progress.status);

            if matches!(progress.status, crate::debrid::types::DebridStatus::WaitingFilesSelection) {
                tracing::info!("Torrent waiting for file selection, selecting all files");

                if let Err(e) = debrid_manager.select_files(provider_type, &torrent_id_result.id, &[]).await {
                    tracing::error!("Failed to select files: {}", e);
                    return Err(format!("Failed to select files: {}", e));
                }

                tracing::info!("Successfully selected all files for torrent");
            }
        }
        Err(e) => {
            tracing::warn!("Could not get torrent progress immediately after adding: {}", e);
        }
    }

    // Parse the magnet to get info hash
    let info_hash = if magnet_or_hash.starts_with("magnet:") {
        let magnet = crate::magnet::MagnetLink::parse(&magnet_or_hash)
            .map_err(|e| format!("Failed to parse magnet: {}", e))?;
        hex::encode(magnet.info_hash)
    } else {
        magnet_or_hash.clone()
    };

    // Create a TorrentInfo entry for UI tracking
    let torrent_info = crate::state::TorrentInfo {
        id: info_hash.clone(),
        name: format!("Cloud Download ({})", torrent_id_result.id),
        size: 0,
        downloaded: 0,
        uploaded: 0,
        state: crate::state::TorrentState::Downloading,
        download_speed: 0,
        upload_speed: 0,
        peers: 0,
        seeds: 0,
        source: crate::debrid::types::DownloadSource::Debrid {
            provider: provider_type,
            torrent_id: torrent_id_result.id.clone(),
        },
    };

    // Store in torrents map
    state.torrents.write().await.insert(info_hash.clone(), torrent_info);

    // Drop the debrid_manager read lock before spawning the task
    drop(debrid_manager);

    // Start background download task with cancellation support
    let cancel_token = tokio_util::sync::CancellationToken::new();
    crate::cloud::CloudDownloadManager::start_download_task(
        info_hash.clone(),
        torrent_id_result.id.clone(),
        provider_type,
        PathBuf::from(&save_path),
        Arc::clone(&state.torrents),
        Arc::clone(&state.debrid_manager),
        Arc::clone(&state.cloud_file_progress),
        cancel_token,
    ).await;

    tracing::info!("Cloud download task started for: {}", info_hash);
    Ok(info_hash)
}

/// Check torrent cache status across all providers
#[tauri::command]
pub async fn check_torrent_cache(
    info_hash: String,
    state: State<'_, AppState>,
) -> Result<HashMap<String, CacheStatus>, String> {
    tracing::info!("Checking cache for info_hash: {}", info_hash);

    let debrid_manager = state.debrid_manager.read().await;
    let cache_results = debrid_manager.check_cache_all(&info_hash)
        .await
        .map_err(|e| format!("Failed to check cache: {}", e))?;

    let results: HashMap<String, CacheStatus> = cache_results
        .into_iter()
        .map(|(provider, status)| (provider.as_str().to_string(), status))
        .collect();

    Ok(results)
}

/// Get preferred cached provider based on user preference
#[tauri::command]
pub async fn get_preferred_cached_provider(
    info_hash: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    tracing::info!("Getting preferred cached provider for: {}", info_hash);

    let debrid_manager = state.debrid_manager.read().await;
    let preferred = debrid_manager.get_preferred_cached(&info_hash)
        .await
        .map_err(|e| format!("Failed to get preferred provider: {}", e))?;

    Ok(preferred.map(|p| p.as_str().to_string()))
}

/// Add magnet link to debrid provider
#[tauri::command]
pub async fn add_magnet_to_debrid(
    magnet: String,
    provider: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    tracing::info!("Adding magnet to {}", provider);

    let provider_type = super::parse_provider(&provider)?;

    let debrid_manager = state.debrid_manager.read().await;
    let torrent_id = debrid_manager.add_to_cloud(provider_type, crate::debrid::AddTorrentRequest::Magnet(magnet))
        .await
        .map_err(|e| format!("Failed to add magnet: {}", e))?;

    Ok(torrent_id.id)
}

/// Add torrent file to debrid provider
#[tauri::command]
pub async fn add_torrent_file_to_debrid(
    file_path: String,
    provider: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    tracing::info!("Adding torrent file to {}: {}", provider, file_path);

    let provider_type = super::parse_provider(&provider)?;

    let debrid_manager = state.debrid_manager.read().await;
    let torrent_id = debrid_manager.add_to_cloud(provider_type, crate::debrid::AddTorrentRequest::File(PathBuf::from(file_path)))
        .await
        .map_err(|e| format!("Failed to add torrent file: {}", e))?;

    Ok(torrent_id.id)
}

/// Select files for download from debrid torrent
#[tauri::command]
pub async fn select_debrid_files(
    torrent_id: String,
    provider: String,
    file_indices: Vec<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Selecting {} files in torrent {} on {}", file_indices.len(), torrent_id, provider);

    let provider_type = super::parse_provider(&provider)?;

    let debrid_manager = state.debrid_manager.read().await;
    debrid_manager.select_files(provider_type, &torrent_id, &file_indices)
        .await
        .map_err(|e| format!("Failed to select files: {}", e))?;

    Ok(())
}

/// Get download links for debrid torrent
#[tauri::command]
pub async fn get_debrid_download_links(
    torrent_id: String,
    provider: String,
    state: State<'_, AppState>,
) -> Result<Vec<DebridFile>, String> {
    tracing::info!("Getting download links for torrent {} on {}", torrent_id, provider);

    let provider_type = super::parse_provider(&provider)?;

    let debrid_manager = state.debrid_manager.read().await;
    let files = debrid_manager.get_download_links(provider_type, &torrent_id)
        .await
        .map_err(|e| format!("Failed to get download links: {}", e))?;

    Ok(files)
}

/// List all torrents on debrid provider
#[tauri::command]
pub async fn list_debrid_torrents(
    provider: String,
    state: State<'_, AppState>,
) -> Result<Vec<DebridProgress>, String> {
    tracing::info!("Listing torrents on {}", provider);

    let provider_type = super::parse_provider(&provider)?;

    let debrid_manager = state.debrid_manager.read().await;
    let torrents = debrid_manager.list_torrents(provider_type)
        .await
        .map_err(|e| format!("Failed to list torrents: {}", e))?;

    Ok(torrents)
}

/// Delete torrent from debrid provider
#[tauri::command]
pub async fn delete_debrid_torrent(
    torrent_id: String,
    provider: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Deleting torrent {} from {}", torrent_id, provider);

    let provider_type = super::parse_provider(&provider)?;

    let debrid_manager = state.debrid_manager.read().await;
    debrid_manager.delete_torrent(provider_type, &torrent_id)
        .await
        .map_err(|e| format!("Failed to delete torrent: {}", e))?;

    Ok(())
}

/// Get cloud file download progress for a torrent
#[tauri::command]
pub async fn get_cloud_file_progress(
    torrent_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::state::CloudFileProgress>, String> {
    tracing::debug!("Getting cloud file progress for torrent: {}", torrent_id);

    let progress_map = state.cloud_file_progress.read().await;
    let files = progress_map.get(&torrent_id)
        .map(|file_map| {
            let mut files: Vec<_> = file_map.values().cloned().collect();
            files.sort_by(|a, b| a.name.cmp(&b.name));
            files
        })
        .unwrap_or_default();
    Ok(files)
}
