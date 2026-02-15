//! Info commands: peers, trackers, pieces, files, disk space

use crate::state::AppState;
use crate::peer::PeerInfo;
use crate::tracker::TrackerInfo;
use crate::piece::PiecesInfo;
use std::path::PathBuf;
use tauri::State;

/// Get peer list for a torrent
#[tauri::command]
pub async fn get_peer_list(
    state: State<'_, AppState>,
    torrent_id: String,
) -> Result<Vec<PeerInfo>, String> {
    tracing::debug!("Getting peer list for torrent: {}", torrent_id);

    let engines = state.engines.read().await;
    let engine = engines.get(&torrent_id)
        .ok_or_else(|| format!("Torrent not found: {}", torrent_id))?;

    let engine_lock = engine.read().await;
    let peers = engine_lock.get_peer_list().await;

    Ok(peers)
}

/// Get tracker list for a torrent
#[tauri::command]
pub async fn get_tracker_list(
    state: State<'_, AppState>,
    torrent_id: String,
) -> Result<Vec<TrackerInfo>, String> {
    tracing::debug!("Getting tracker list for torrent: {}", torrent_id);

    let engines = state.engines.read().await;
    let engine = engines.get(&torrent_id)
        .ok_or_else(|| format!("Torrent not found: {}", torrent_id))?;

    let engine_lock = engine.read().await;
    let trackers = engine_lock.get_tracker_list().await;

    Ok(trackers)
}

/// Get pieces info for a torrent
#[tauri::command]
pub async fn get_pieces_info(
    state: State<'_, AppState>,
    torrent_id: String,
) -> Result<PiecesInfo, String> {
    tracing::debug!("Getting pieces info for torrent: {}", torrent_id);

    let engines = state.engines.read().await;
    let engine = engines.get(&torrent_id)
        .ok_or_else(|| format!("Torrent not found: {}", torrent_id))?;

    let engine_lock = engine.read().await;
    let piece_manager = engine_lock.piece_manager();
    let pm = piece_manager.read().await;

    Ok(pm.get_pieces_info())
}

/// Get file list for a torrent
#[tauri::command]
pub async fn get_file_list(
    state: State<'_, AppState>,
    torrent_id: String,
) -> Result<Vec<crate::torrent::FileInfoUI>, String> {
    tracing::debug!("Getting file list for torrent: {}", torrent_id);

    let engines = state.engines.read().await;
    let engine = engines.get(&torrent_id)
        .ok_or_else(|| format!("Torrent not found: {}", torrent_id))?;

    let engine_lock = engine.read().await;
    let metainfo = engine_lock.metainfo();
    let piece_manager = engine_lock.piece_manager();
    let pm = piece_manager.read().await;
    
    let progress = pm.calculate_file_progress(&metainfo.info.files);

    Ok(crate::torrent::get_file_list(&metainfo, Some(&progress)))
}

/// Get available disk space for a given path
#[tauri::command]
pub fn get_available_disk_space(path: String) -> Result<u64, String> {
    use fs2::statvfs;

    tracing::debug!("Getting disk space for path: {}", path);

    let path_buf = PathBuf::from(&path);

    // Get the actual path to check
    let check_path = if path_buf.exists() {
        path_buf
    } else if let Some(parent) = path_buf.parent() {
        if parent.exists() {
            parent.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?
        }
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
    };

    let stats = statvfs(&check_path)
        .map_err(|e| format!("Failed to get disk space: {}", e))?;

    let available_bytes = stats.available_space();

    tracing::debug!("Available space for {}: {} bytes", check_path.display(), available_bytes);

    Ok(available_bytes)
}
