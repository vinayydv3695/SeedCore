//! Torrent commands: add, remove, start, pause, load saved torrents

use crate::state::{AppState, TorrentInfo, TorrentState};
use crate::torrent::Metainfo;
use crate::engine::TorrentEngine;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock as TokioRwLock;

/// Parse torrent metadata from .torrent file without adding it
#[tauri::command]
pub fn parse_torrent_file(file_path: String) -> Result<super::TorrentMetadata, String> {
    tracing::info!("Parsing torrent file: {}", file_path);

    // Read .torrent file
    let path = PathBuf::from(&file_path);
    let data = std::fs::read(&path)
        .map_err(|e| format!("Failed to read torrent file: {}", e))?;

    // Parse metainfo
    let metainfo = Metainfo::from_bytes(&data)
        .map_err(|e| format!("Failed to parse torrent: {}", e))?;

    // Convert files to UI format
    let files: Vec<crate::torrent::FileInfoUI> = metainfo.info.files
        .iter()
        .map(|f| crate::torrent::FileInfoUI {
            path: f.path.join("/"),
            size: f.length,
            downloaded: 0,
            priority: crate::torrent::FilePriority::Normal,
            is_folder: false,
        })
        .collect();

    Ok(super::TorrentMetadata {
        name: metainfo.info.name.clone(),
        info_hash: metainfo.info_hash_hex(),
        total_size: metainfo.info.total_size,
        files,
        announce: metainfo.announce.clone(),
        creation_date: metainfo.creation_date,
        comment: metainfo.comment.clone(),
        created_by: metainfo.created_by.clone(),
    })
}

/// Parse torrent metadata from magnet link without adding it
#[tauri::command]
pub fn parse_magnet_link(magnet_uri: String) -> Result<super::TorrentMetadata, String> {
    tracing::info!("Parsing magnet link: {}", magnet_uri);

    // Parse the magnet link
    let magnet = crate::magnet::MagnetLink::parse(&magnet_uri)
        .map_err(|e| format!("Failed to parse magnet link: {}", e))?;

    // For magnet links, we don't have full metadata yet
    Ok(super::TorrentMetadata {
        name: magnet.display_name.clone().unwrap_or_else(|| "Unknown".to_string()),
        info_hash: magnet.info_hash_hex(),
        total_size: 0, // Unknown until we get metadata
        files: vec![], // Unknown until we get metadata
        announce: magnet.trackers.first().cloned().unwrap_or_default(),
        creation_date: None,
        comment: None,
        created_by: None,
    })
}

/// Add a torrent from a .torrent file
#[tauri::command]
pub async fn add_torrent_file(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    file_path: String,
) -> Result<String, String> {
    tracing::info!("Adding torrent from file: {}", file_path);

    // Read .torrent file
    let path = PathBuf::from(&file_path);
    let data = std::fs::read(&path)
        .map_err(|e| format!("Failed to read torrent file: {}", e))?;

    // Parse metainfo
    let metainfo = Metainfo::from_bytes(&data)
        .map_err(|e| format!("Failed to parse torrent: {}", e))?;

    // Generate torrent ID from info hash
    let torrent_id = metainfo.info_hash_hex();

    // Create torrent info
    let torrent_info = TorrentInfo {
        id: torrent_id.clone(),
        name: metainfo.info.name.clone(),
        size: metainfo.info.total_size,
        downloaded: 0,
        uploaded: 0,
        state: TorrentState::Paused,
        download_speed: 0,
        upload_speed: 0,
        peers: 0,
        seeds: 0,
        source: crate::debrid::types::DownloadSource::P2P,
    };

    // Add to state
    state.torrents.write().await.insert(torrent_id.clone(), torrent_info);

    // Save to database
    let db_session = crate::database::TorrentSession {
        id: torrent_id.clone(),
        metainfo: metainfo.clone(),
        bitfield: vec![],
        num_pieces: metainfo.info.piece_count,
        downloaded: 0,
        uploaded: 0,
        state: "paused".to_string(),
        download_dir: dirs::download_dir()
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
            .to_string_lossy()
            .to_string(),
        added_at: chrono::Utc::now().timestamp(),
        last_activity: chrono::Utc::now().timestamp(),
        source: crate::debrid::types::DownloadSource::P2P,
        completed_at: None,
    };

    state.database
        .save_torrent(&db_session)
        .map_err(|e| format!("Failed to save torrent to database: {}", e))?;

    // Create TorrentEngine instance (in paused state)
    let download_dir = PathBuf::from(&db_session.download_dir);
    let mut engine = TorrentEngine::new(metainfo.clone(), download_dir, Some(app));
    engine.set_database(state.database.clone());

    // Store engine in state
    let engine_arc = Arc::new(TokioRwLock::new(engine));
    state.engines.write().await.insert(torrent_id.clone(), engine_arc);

    tracing::info!("Added torrent: {} ({})", metainfo.info.name, torrent_id);

    Ok(torrent_id)
}

/// Add a torrent from a magnet link
#[tauri::command]
pub async fn add_magnet_link(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    magnet_uri: String,
) -> Result<String, String> {
    tracing::info!("Adding magnet link: {}", magnet_uri);

    // Parse the magnet link
    let magnet = crate::magnet::MagnetLink::parse(&magnet_uri)
        .map_err(|e| format!("Failed to parse magnet link: {}", e))?;

    let torrent_id = magnet.info_hash_hex();

    tracing::info!(
        "Parsed magnet link - ID: {}, Name: {:?}, Trackers: {}",
        torrent_id,
        magnet.display_name,
        magnet.trackers.len()
    );

    // Create minimal Metainfo from magnet link
    let metainfo = crate::torrent::Metainfo::from_magnet(
        magnet.info_hash,
        magnet.display_name.clone(),
        magnet.trackers.clone(),
    );

    tracing::debug!("Loading settings from database");
    let download_dir = {
        let db_settings = state.database
            .load_settings()
            .map_err(|e| format!("Failed to load settings: {}", e))?;
        PathBuf::from(db_settings.download_dir)
    };

    tracing::debug!("Creating TorrentEngine for magnet");
    let mut engine = TorrentEngine::new(metainfo.clone(), download_dir.clone(), Some(app));
    engine.set_database(state.database.clone());

    tracing::debug!("Storing engine in state");
    let engine_arc = Arc::new(TokioRwLock::new(engine));
    state.engines.write().await.insert(torrent_id.clone(), engine_arc);

    tracing::debug!("Creating TorrentInfo for UI");
    let torrent_info = TorrentInfo {
        id: torrent_id.clone(),
        name: magnet.display_name.unwrap_or_else(|| format!("Magnet {}", &torrent_id[..8])),
        size: 0,
        downloaded: 0,
        uploaded: 0,
        state: TorrentState::Paused,
        download_speed: 0,
        upload_speed: 0,
        peers: 0,
        seeds: 0,
        source: crate::debrid::types::DownloadSource::P2P,
    };

    tracing::debug!("Adding to in-memory state");
    state.torrents.write().await.insert(torrent_id.clone(), torrent_info);

    tracing::debug!("Saving to database");
    let db_session = crate::database::TorrentSession {
        id: torrent_id.clone(),
        metainfo: metainfo.clone(),
        download_dir: download_dir.to_string_lossy().to_string(),
        state: "paused".to_string(),
        downloaded: 0,
        uploaded: 0,
        last_activity: chrono::Utc::now().timestamp(),
        bitfield: Vec::new(),
        num_pieces: 0,
        added_at: chrono::Utc::now().timestamp(),
        source: crate::debrid::types::DownloadSource::P2P,
        completed_at: None,
    };

    state.database
        .save_torrent(&db_session)
        .map_err(|e| format!("Failed to save torrent to database: {}", e))?;

    tracing::info!("Successfully added magnet link: {} ({})", metainfo.info.name, torrent_id);

    Ok(torrent_id)
}

/// Remove a torrent
#[tauri::command]
pub async fn remove_torrent(
    state: State<'_, AppState>,
    torrent_id: String,
    delete_files: bool,
) -> Result<(), String> {
    remove_torrent_internal(&state, torrent_id, delete_files).await
}

pub async fn remove_torrent_internal(
    state: &AppState,
    torrent_id: String,
    delete_files: bool,
) -> Result<(), String> {
    tracing::info!("Removing torrent: {} (delete_files: {})", torrent_id, delete_files);

    // Stop the engine if running — cancel token + stop command
    {
        let engines = state.engines.read().await;
        if let Some(engine_arc) = engines.get(&torrent_id) {
            let engine = engine_arc.read().await;
            engine.cancel_token().cancel();
            let _ = engine.command_sender().send(crate::engine::EngineCommand::Stop);
        }
    }

    // Wait for task to complete and remove from task tracker
    if let Some(task_handle) = state.engine_tasks.write().await.remove(&torrent_id) {
        task_handle.abort();
    }

    // Remove from engines HashMap
    state.engines.write().await.remove(&torrent_id);

    // Remove from torrents HashMap
    state.torrents.write().await.remove(&torrent_id);

    // Delete downloaded files if requested
    if delete_files {
        // Get download directory from database before deleting the entry
        if let Ok(Some(session)) = state.database.load_torrent(&torrent_id) {
            let download_dir = PathBuf::from(&session.download_dir);
            let torrent_name = &session.metainfo.info.name;
            let torrent_path = download_dir.join(torrent_name);

            if torrent_path.exists() {
                if torrent_path.is_dir() {
                    if let Err(e) = std::fs::remove_dir_all(&torrent_path) {
                        tracing::error!("Failed to delete torrent directory {:?}: {}", torrent_path, e);
                    } else {
                        tracing::info!("Deleted torrent directory: {:?}", torrent_path);
                    }
                } else {
                    if let Err(e) = std::fs::remove_file(&torrent_path) {
                        tracing::error!("Failed to delete torrent file {:?}: {}", torrent_path, e);
                    } else {
                        tracing::info!("Deleted torrent file: {:?}", torrent_path);
                    }
                }
            } else {
                tracing::warn!("Torrent path not found for deletion: {:?}", torrent_path);
            }
        }
    }

    // Delete from database
    state.database
        .delete_torrent(&torrent_id)
        .map_err(|e| format!("Failed to delete torrent from database: {}", e))?;

    tracing::info!("Removed torrent: {}", torrent_id);
    Ok(())
}

/// Start/resume a torrent
#[tauri::command]
pub async fn start_torrent(state: State<'_, AppState>, torrent_id: String) -> Result<(), String> {
    tracing::info!("Starting torrent: {}", torrent_id);

    // Check if engine exists
    let engines = state.engines.read().await;
    let engine_arc = engines.get(&torrent_id)
        .ok_or_else(|| format!("Torrent not found: {}", torrent_id))?
        .clone();
    drop(engines);

    // Check if already running
    let engine_tasks = state.engine_tasks.read().await;
    if engine_tasks.contains_key(&torrent_id) {
        tracing::warn!("Torrent {} is already running", torrent_id);
        return Ok(());
    }
    drop(engine_tasks);

    // Send Start command to engine
    {
        let engine = engine_arc.read().await;
        let cmd_tx = engine.command_sender();
        cmd_tx.send(crate::engine::EngineCommand::Start)
            .map_err(|e| format!("Failed to send start command: {}", e))?;
    }

    // Spawn the engine's event loop
    let task_handle = tokio::spawn(async move {
        let mut engine = engine_arc.write().await;
        engine.run().await;
    });

    // Store task handle
    state.engine_tasks.write().await.insert(torrent_id.clone(), task_handle);

    // Update torrent state in UI
    {
        let mut torrents = state.torrents.write().await;
        if let Some(torrent) = torrents.get_mut(&torrent_id) {
            torrent.state = TorrentState::Downloading;
        }
    }

    tracing::info!("Started torrent: {}", torrent_id);
    Ok(())
}

/// Pause a torrent
#[tauri::command]
pub async fn pause_torrent(state: State<'_, AppState>, torrent_id: String) -> Result<(), String> {
    tracing::info!("Pausing torrent: {}", torrent_id);

    // Get engine
    let engines = state.engines.read().await;
    let engine_arc = engines.get(&torrent_id)
        .ok_or_else(|| format!("Torrent not found: {}", torrent_id))?
        .clone();
    drop(engines);

    // Send Pause command to engine
    {
        let engine = engine_arc.read().await;
        let cmd_tx = engine.command_sender();
        cmd_tx.send(crate::engine::EngineCommand::Pause)
            .map_err(|e| format!("Failed to send pause command: {}", e))?;
    }

    // Update torrent state in UI
    {
        let mut torrents = state.torrents.write().await;
        if let Some(torrent) = torrents.get_mut(&torrent_id) {
            torrent.state = TorrentState::Paused;
        }
    }

    tracing::info!("Paused torrent: {}", torrent_id);
    Ok(())
}

/// Get detailed info about a specific torrent
#[tauri::command]
pub async fn get_torrent_details(
    state: State<'_, AppState>,
    torrent_id: String,
) -> Result<TorrentInfo, String> {
    state.torrents.read().await
        .get(&torrent_id)
        .cloned()
        .ok_or_else(|| format!("Torrent not found: {}", torrent_id))
}

/// Load all saved torrents from database
#[tauri::command]
pub async fn load_saved_torrents(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<Vec<TorrentInfo>, String> {
    tracing::info!("Loading saved torrents from database");

    let sessions = state.database
        .load_all_torrents()
        .map_err(|e| format!("Failed to load torrents from database: {}", e))?;

    let mut torrents = Vec::new();
    let mut new_engines = Vec::new();
    let mut new_tasks = Vec::new();

    // Check which engines already exist (single read lock)
    let existing_engines = {
        let engines = state.engines.read().await;
        sessions.iter()
            .map(|s| (s.id.clone(), engines.contains_key(&s.id)))
            .collect::<std::collections::HashMap<String, bool>>()
    };

    // Process all sessions and create engines/tasks
    for session in sessions {
        // Wrap in a catch to prevent one bad torrent from breaking all loading
        let process_result = async {
            // Convert database session to TorrentInfo
            let torrent_state = match session.state.as_str() {
                "downloading" => TorrentState::Downloading,
                "seeding" => TorrentState::Seeding,
                "paused" => TorrentState::Paused,
                "stopped" => TorrentState::Paused,
                _ => TorrentState::Paused,
            };

            let torrent_info = TorrentInfo {
                id: session.id.clone(),
                name: session.metainfo.info.name.clone(),
                size: session.metainfo.info.total_size,
                downloaded: session.downloaded,
                uploaded: session.uploaded,
                state: torrent_state,
                download_speed: 0,
                upload_speed: 0,
                peers: 0,
                seeds: 0,
                source: session.source.clone(),
            };

            // Create engine for this torrent (if not already exists)
            if !existing_engines.get(&session.id).unwrap_or(&false) {
                let download_dir = PathBuf::from(&session.download_dir);
                let mut engine = TorrentEngine::new(session.metainfo.clone(), download_dir, Some(app.clone()));
                engine.set_database(state.database.clone());
                engine.set_completed_at(session.completed_at);

                // Restore bitfield from saved session
                if !session.bitfield.is_empty() {
                    let pm = engine.piece_manager();
                    let mut pm_guard = pm.write().await;
                    pm_guard.restore_bitfield(&session.bitfield);
                }

                let engine_arc = Arc::new(TokioRwLock::new(engine));
                
                // Prepare for batch insertion
                new_engines.push((session.id.clone(), engine_arc.clone()));

                // Auto-start if it was downloading/seeding before
                if torrent_state == TorrentState::Downloading || torrent_state == TorrentState::Seeding {
                    tracing::info!("Auto-starting torrent: {}", session.id);

                    // Clone Arc for the spawned task
                    let engine_arc_clone = engine_arc.clone();
                    
                    // Send Start command
                    {
                        let engine = engine_arc.read().await;
                        let cmd_tx = engine.command_sender();
                        let _ = cmd_tx.send(crate::engine::EngineCommand::Start);
                    }

                    // Spawn the engine's event loop
                    let task_handle = tokio::spawn(async move {
                        let mut engine = engine_arc_clone.write().await;
                        engine.run().await;
                    });

                    // Prepare for batch insertion
                    new_tasks.push((session.id.clone(), task_handle));
                }
            }

            Ok::<_, String>((session.id.clone(), torrent_info))
        }.await;

        match process_result {
            Ok((id, info)) => {
                torrents.push((id, info));
            }
            Err(e) => {
                tracing::error!("Failed to load torrent {}: {}", session.id, e);
            }
        }
    }

    // Batch insert all new engines (single write lock)
    if !new_engines.is_empty() {
        let mut engines = state.engines.write().await;
        for (id, engine) in new_engines {
            engines.insert(id, engine);
        }
    }

    // Batch insert all new tasks (single write lock)
    if !new_tasks.is_empty() {
        let mut tasks = state.engine_tasks.write().await;
        for (id, task) in new_tasks {
            tasks.insert(id, task);
        }
    }

    // Batch insert all torrent info (single write lock)
    {
        let mut torrents_map = state.torrents.write().await;
        for (id, info) in &torrents {
            torrents_map.insert(id.clone(), info.clone());
        }
    }

    let result: Vec<TorrentInfo> = torrents.into_iter().map(|(_, info)| info).collect();
    
    tracing::info!("Loaded {} torrents from database", result.len());

    Ok(result)
}

/// Set priority for a file in a torrent
#[tauri::command]
pub async fn set_file_priority(
    state: State<'_, AppState>,
    torrent_id: String,
    file_index: usize,
    priority: u8,
) -> Result<(), String> {
    tracing::info!("Setting file priority - Torrent: {}, File: {}, Priority: {}", torrent_id, file_index, priority);

    // Convert u8 to PiecePriority
    let priority_enum = match priority {
        0 => crate::piece::PiecePriority::Skip,
        1 => crate::piece::PiecePriority::Low,
        2 => crate::piece::PiecePriority::Normal,
        3 => crate::piece::PiecePriority::High,
        4 => crate::piece::PiecePriority::Critical,
        _ => return Err(format!("Invalid priority value: {}", priority)),
    };

    // Get engine
    let engines = state.engines.read().await;
    let engine_arc = engines.get(&torrent_id)
        .ok_or_else(|| format!("Torrent not found: {}", torrent_id))?
        .clone();
    drop(engines);

    // Set priority
    let mut engine = engine_arc.write().await;
    engine.set_file_priority(file_index, priority_enum).await?;

    tracing::info!("Set priority for file {} to {:?}", file_index, priority_enum);
    Ok(())
}
