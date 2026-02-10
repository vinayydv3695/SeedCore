//! Tauri commands - Frontend to Backend communication

use crate::state::{AppState, TorrentInfo, TorrentState};
use crate::torrent::{Metainfo, FilePriority};
use crate::peer::PeerInfo;
use crate::tracker::TrackerInfo;
use crate::piece::PiecesInfo;
use crate::engine::TorrentEngine;
use crate::debrid::types::{DebridProviderType, CacheStatus, DebridFile, DebridProgress};
use crate::crypto::{self, CryptoManager};
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use tauri::State;
use tokio::sync::RwLock as TokioRwLock;
use serde::{Serialize, Deserialize};

/// Simple greeting command (for testing)
#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! Welcome to SeedCore.")
}

/// Get application version
#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Get application settings
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<crate::state::Settings, String> {
    state
        .settings
        .read()
        .map(|s| s.clone())
        .map_err(|e| e.to_string())
}

/// Update application settings
#[tauri::command]
pub fn update_settings(
    state: State<AppState>,
    settings: crate::state::Settings,
) -> Result<(), String> {
    state
        .settings
        .write()
        .map(|mut s| *s = settings)
        .map_err(|e| e.to_string())
}

/// Get list of all torrents
#[tauri::command]
pub fn get_torrents(state: State<AppState>) -> Result<Vec<crate::state::TorrentInfo>, String> {
    state
        .torrents
        .read()
        .map(|torrents| torrents.values().cloned().collect())
        .map_err(|e| e.to_string())
}

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

/// Parse torrent metadata from .torrent file without adding it
#[tauri::command]
pub fn parse_torrent_file(file_path: String) -> Result<TorrentMetadata, String> {
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
            priority: FilePriority::Normal,
            is_folder: false,
        })
        .collect();
    
    Ok(TorrentMetadata {
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
pub fn parse_magnet_link(magnet_uri: String) -> Result<TorrentMetadata, String> {
    tracing::info!("Parsing magnet link: {}", magnet_uri);
    
    // Parse the magnet link
    let magnet = crate::magnet::MagnetLink::parse(&magnet_uri)
        .map_err(|e| format!("Failed to parse magnet link: {}", e))?;
    
    // For magnet links, we don't have full metadata yet
    // We'll return basic info and the UI will need to handle partial data
    Ok(TorrentMetadata {
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
        source: crate::debrid::types::DownloadSource::P2P, // Default to P2P
    };
    
    // Add to state
    state
        .torrents
        .write()
        .map(|mut torrents| {
            torrents.insert(torrent_id.clone(), torrent_info);
        })
        .map_err(|e| e.to_string())?;
    
    // Save to database
    let db_session = crate::database::TorrentSession {
        id: torrent_id.clone(),
        metainfo: metainfo.clone(),
        bitfield: vec![],  // Empty bitfield initially
        num_pieces: metainfo.info.piece_count,
        downloaded: 0,
        uploaded: 0,
        state: "paused".to_string(),
        download_dir: dirs::download_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap())
            .to_string_lossy()
            .to_string(),
        added_at: chrono::Utc::now().timestamp(),
        last_activity: chrono::Utc::now().timestamp(),
        source: crate::debrid::types::DownloadSource::P2P, // Default to P2P for now
    };
    
    state.database
        .save_torrent(&db_session)
        .map_err(|e| format!("Failed to save torrent to database: {}", e))?;
    
    // Create TorrentEngine instance (in paused state)
    let download_dir = PathBuf::from(&db_session.download_dir);
    let mut engine = TorrentEngine::new(metainfo.clone(), download_dir);
    engine.set_database(state.database.clone());
    
    // Store engine in state (wrapped in Arc<TokioRwLock> for async access)
    let engine_arc = Arc::new(TokioRwLock::new(engine));
    state.engines.write().await.insert(torrent_id.clone(), engine_arc);
    
    tracing::info!("Added torrent: {} ({})", metainfo.info.name, torrent_id);
    
    Ok(torrent_id)
}

/// Add a torrent from a magnet link
#[tauri::command]
pub async fn add_magnet_link(
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
    // Get download directory from settings
    let download_dir = {
        let db_settings = state.database
            .load_settings()
            .map_err(|e| format!("Failed to load settings: {}", e))?;
        PathBuf::from(db_settings.download_dir)
    };
    
    tracing::debug!("Creating TorrentEngine for magnet");
    // Create TorrentEngine instance (in paused state)
    let mut engine = TorrentEngine::new(metainfo.clone(), download_dir.clone());
    engine.set_database(state.database.clone());
    
    tracing::debug!("Storing engine in state");
    // Store engine in state
    let engine_arc = Arc::new(TokioRwLock::new(engine));
    state.engines.write().await.insert(torrent_id.clone(), engine_arc);
    
    tracing::debug!("Creating TorrentInfo for UI");
    // Create TorrentInfo for UI
    let torrent_info = TorrentInfo {
        id: torrent_id.clone(),
        name: magnet.display_name.unwrap_or_else(|| format!("Magnet {}", &torrent_id[..8])),
        size: 0, // Unknown until metadata is fetched
        downloaded: 0,
        uploaded: 0,
        state: TorrentState::Paused,
        download_speed: 0,
        upload_speed: 0,
        peers: 0,
        seeds: 0,
        source: crate::debrid::types::DownloadSource::P2P, // Default to P2P
    };
    
    tracing::debug!("Adding to in-memory state");
    // Add to in-memory state
    state.torrents.write()
        .map(|mut torrents| {
            torrents.insert(torrent_id.clone(), torrent_info);
        })
        .map_err(|e| e.to_string())?;
    
    tracing::debug!("Saving to database");
    // Save to database
    let db_session = crate::database::TorrentSession {
        id: torrent_id.clone(),
        metainfo: metainfo.clone(),
        download_dir: download_dir.to_string_lossy().to_string(),
        state: "paused".to_string(),
        downloaded: 0,
        uploaded: 0,
        last_activity: chrono::Utc::now().timestamp(),
        bitfield: Vec::new(),
        num_pieces: 0, // Unknown until metadata is fetched
        added_at: chrono::Utc::now().timestamp(),
        source: crate::debrid::types::DownloadSource::P2P, // Default to P2P for now
    };
    
    state.database
        .save_torrent(&db_session)
        .map_err(|e| format!("Failed to save torrent to database: {}", e))?;
    
    tracing::info!("Successfully added magnet link: {} ({})", metainfo.info.name, torrent_id);
    
    Ok(torrent_id)
}

/// Add and download a torrent using cloud debrid service
#[tauri::command]
pub async fn add_cloud_torrent(
    state: State<'_, AppState>,
    magnet_or_hash: String,
    provider: String,
    save_path: String,
) -> Result<String, String> {
    tracing::info!("Adding cloud torrent via {}: {}", provider, magnet_or_hash);
    
    // Parse provider type
    let provider_type = match provider.as_str() {
        "torbox" => crate::debrid::types::DebridProviderType::Torbox,
        "real-debrid" => crate::debrid::types::DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
    // Convert to magnet URI if just hash
    let magnet_uri = if magnet_or_hash.starts_with("magnet:") {
        magnet_or_hash.clone()
    } else {
        // Convert info hash to magnet URI
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
    // Get torrent info to check status
    match debrid_manager.get_progress(provider_type, &torrent_id_result.id).await {
        Ok(progress) => {
            tracing::info!("Torrent status: {:?}", progress.status);
            
            // If waiting for file selection, select all files
            if matches!(progress.status, crate::debrid::types::DebridStatus::WaitingFilesSelection) {
                tracing::info!("Torrent waiting for file selection, selecting all files");
                
                // Select all files (empty vec means "all files" for Real-Debrid)
                if let Err(e) = debrid_manager.select_files(provider_type, &torrent_id_result.id, &[]).await {
                    tracing::error!("Failed to select files: {}", e);
                    return Err(format!("Failed to select files: {}", e));
                }
                
                tracing::info!("Successfully selected all files for torrent");
            }
        }
        Err(e) => {
            tracing::warn!("Could not get torrent progress immediately after adding: {}", e);
            // Continue anyway - the polling loop will handle it
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
        size: 0, // Will be updated when we get file info
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
    state.torrents.write()
        .map(|mut torrents| {
            torrents.insert(info_hash.clone(), torrent_info);
        })
        .map_err(|e| e.to_string())?;
    
    // Drop the debrid_manager read lock before spawning the task
    drop(debrid_manager);
    
    // Start background download task
    crate::cloud::CloudDownloadManager::start_download_task(
        info_hash.clone(),
        torrent_id_result.id.clone(),
        provider_type,
        PathBuf::from(&save_path),
        Arc::clone(&state.torrents),
        Arc::clone(&state.debrid_manager),
        Arc::clone(&state.cloud_file_progress),
    ).await;
    
    tracing::info!("Cloud download task started for: {}", info_hash);
    Ok(info_hash)
}

/// Remove a torrent
#[tauri::command]
pub async fn remove_torrent(
    state: State<'_, AppState>,
    torrent_id: String,
    delete_files: bool,
) -> Result<(), String> {
    tracing::info!("Removing torrent: {} (delete_files: {})", torrent_id, delete_files);
    
    // Stop the engine if running
    {
        let engines = state.engines.read().await;
        if let Some(engine_arc) = engines.get(&torrent_id) {
            let engine = engine_arc.read().await;
            let cmd_tx = engine.command_sender();
            let _ = cmd_tx.send(crate::engine::EngineCommand::Stop);
        }
    }
    
    // Wait for task to complete and remove from task tracker
    if let Some(task_handle) = state.engine_tasks.write().await.remove(&torrent_id) {
        task_handle.abort();
    }
    
    // Remove from engines HashMap
    state.engines.write().await.remove(&torrent_id);
    
    // Remove from torrents HashMap
    state.torrents.write()
        .map(|mut torrents| {
            torrents.remove(&torrent_id);
        })
        .map_err(|e| e.to_string())?;
    
    // Delete from database
    state.database
        .delete_torrent(&torrent_id)
        .map_err(|e| format!("Failed to delete torrent from database: {}", e))?;
    
    // TODO: Delete files if requested
    if delete_files {
        tracing::warn!("File deletion not yet implemented");
    }
    
    tracing::info!("Removed torrent: {}", torrent_id);
    Ok(())
}

// ============================================================================
// DEBRID COMMANDS
// ============================================================================

// ----------------------------------------------------------------------------
// Master Password Commands
// ----------------------------------------------------------------------------

/// Check if master password is set
#[tauri::command]
pub async fn check_master_password_set(state: State<'_, AppState>) -> Result<bool, String> {
    state.database
        .has_master_password()
        .map_err(|e| format!("Failed to check master password: {}", e))
}

/// Set master password (first time setup)
#[tauri::command]
pub async fn set_master_password(
    password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Setting master password");
    
    // Check if already set
    if state.database.has_master_password()
        .map_err(|e| format!("Failed to check existing password: {}", e))? 
    {
        return Err("Master password already set. Use change_master_password instead.".to_string());
    }
    
    // Create password data
    let salt = crypto::generate_salt();
    let password_hash = crypto::hash_master_password(&password, &salt)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    
    let password_data = crate::database::MasterPasswordData {
        password_hash,
        salt,
    };
    
    // Save to database
    state.database
        .save_master_password(&password_data)
        .map_err(|e| format!("Failed to save password: {}", e))?;
    
    // Cache password in memory
    let mut cached_password = state.master_password.write().await;
    *cached_password = Some(password);
    
    tracing::info!("Master password set successfully");
    Ok(())
}

/// Unlock debrid services with master password
#[tauri::command]
pub async fn unlock_with_master_password(
    password: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    tracing::info!("Attempting to unlock with master password");
    
    // Load password data
    let password_data = state.database
        .load_master_password()
        .map_err(|e| format!("Failed to load password: {}", e))?
        .ok_or_else(|| "Master password not set".to_string())?;
    
    // Verify password
    let is_valid = crypto::verify_master_password(&password, &password_data.password_hash)
        .map_err(|e| format!("Failed to verify password: {}", e))?;
    
    if is_valid {
        // Cache password in memory
        let mut cached_password = state.master_password.write().await;
        *cached_password = Some(password);
        
        tracing::info!("Master password verified and cached");
        Ok(true)
    } else {
        tracing::warn!("Invalid master password attempt");
        Ok(false)
    }
}

/// Change master password
#[tauri::command]
pub async fn change_master_password(
    old_password: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Attempting to change master password");
    
    // Load current password data
    let password_data = state.database
        .load_master_password()
        .map_err(|e| format!("Failed to load password: {}", e))?
        .ok_or_else(|| "Master password not set".to_string())?;
    
    // Verify old password
    let is_valid = crypto::verify_master_password(&old_password, &password_data.password_hash)
        .map_err(|e| format!("Failed to verify password: {}", e))?;
    
    if !is_valid {
        return Err("Invalid old password".to_string());
    }
    
    // Load all credentials with old password
    let old_credentials = state.database
        .load_all_debrid_credentials()
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    
    // Decrypt all API keys with old password
    let mut decrypted_keys: HashMap<DebridProviderType, String> = HashMap::new();
    for cred in old_credentials {
        let old_crypto = CryptoManager::from_password(&old_password, &password_data.salt)
            .map_err(|e| format!("Failed to create crypto manager: {}", e))?;
        let api_key = old_crypto.decrypt(&cred.api_key_encrypted, &cred.nonce)
            .map_err(|e| format!("Failed to decrypt credentials for {}: {}", cred.provider.as_str(), e))?;
        decrypted_keys.insert(cred.provider, api_key);
    }
    
    // Create new password hash
    let new_salt = crypto::generate_salt();
    let new_password_hash = crypto::hash_master_password(&new_password, &new_salt)
        .map_err(|e| format!("Failed to hash new password: {}", e))?;
    
    // Re-encrypt all API keys with new password
    let new_crypto = CryptoManager::from_password(&new_password, &new_salt)
        .map_err(|e| format!("Failed to create crypto manager: {}", e))?;
    
    for (provider, api_key) in decrypted_keys {
        let (encrypted_api_key, nonce) = new_crypto.encrypt(&api_key)
            .map_err(|e| format!("Failed to encrypt credentials for {}: {}", provider.as_str(), e))?;
        
        let new_cred = crate::database::DebridCredentials {
            provider,
            api_key_encrypted: encrypted_api_key,
            nonce,
            created_at: chrono::Utc::now().timestamp(),
            last_validated: 0,
            is_valid: false,
        };
        
        state.database
            .save_debrid_credentials(&new_cred)
            .map_err(|e| format!("Failed to save re-encrypted credentials: {}", e))?;
    }
    
    // Save new password
    let new_password_data = crate::database::MasterPasswordData {
        password_hash: new_password_hash,
        salt: new_salt,
    };
    
    state.database
        .save_master_password(&new_password_data)
        .map_err(|e| format!("Failed to save new password: {}", e))?;
    
    // Update cached password
    let mut cached_password = state.master_password.write().await;
    *cached_password = Some(new_password);
    
    tracing::info!("Master password changed successfully");
    Ok(())
}

/// Lock debrid services (clear cached password)
#[tauri::command]
pub async fn lock_debrid_services(state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("Locking debrid services");
    
    let mut cached_password = state.master_password.write().await;
    *cached_password = None;
    
    Ok(())
}

// ----------------------------------------------------------------------------
// Credential Management Commands
// ----------------------------------------------------------------------------

/// Credential status for frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CredentialStatus {
    pub provider: String,
    pub is_configured: bool,
    pub is_valid: Option<bool>,
    pub last_validated: Option<i64>,
}

/// Save debrid provider credentials
#[tauri::command]
pub async fn save_debrid_credentials(
    provider: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Saving credentials for provider: {}", provider);
    
    // Parse provider type
    let provider_type = match provider.as_str() {
        "torbox" => DebridProviderType::Torbox,
        "real-debrid" => DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
    // Get cached master password
    let cached_password = state.master_password.read().await;
    let master_password = cached_password.as_ref()
        .ok_or_else(|| "Master password not unlocked. Please unlock first.".to_string())?;
    
    // Load master password data for salt
    let password_data = state.database
        .load_master_password()
        .map_err(|e| format!("Failed to load password data: {}", e))?
        .ok_or_else(|| "Master password not set".to_string())?;
    
    // Encrypt API key
    let crypto_manager = CryptoManager::from_password(master_password, &password_data.salt)
        .map_err(|e| format!("Failed to create crypto manager: {}", e))?;
    let (encrypted_api_key, nonce) = crypto_manager.encrypt(&api_key)
        .map_err(|e| format!("Failed to encrypt API key: {}", e))?;
    
    tracing::debug!("Successfully encrypted API key for {}", provider);
    
    // Create credentials struct
    let credentials = crate::database::DebridCredentials {
        provider: provider_type,
        api_key_encrypted: encrypted_api_key,
        nonce,
        created_at: chrono::Utc::now().timestamp(),
        last_validated: 0,
        is_valid: false,
    };
    
    // Save to database
    state.database
        .save_debrid_credentials(&credentials)
        .map_err(|e| {
            tracing::error!("Failed to save to database: {}", e);
            format!("Failed to save credentials: {}", e)
        })?;
    
    tracing::info!("Saved to database, now initializing provider");
    
    // Initialize provider in DebridManager
    let mut debrid_manager = state.debrid_manager.write().await;
    debrid_manager.initialize_provider(provider_type, api_key)
        .await
        .map_err(|e| {
            tracing::error!("Failed to initialize provider: {}", e);
            format!("Failed to initialize provider: {}", e)
        })?;
    
    tracing::info!("Credentials saved successfully for {}", provider);
    Ok(())
}

/// Get status of all configured debrid credentials
#[tauri::command]
pub async fn get_debrid_credentials_status(
    state: State<'_, AppState>,
) -> Result<Vec<CredentialStatus>, String> {
    let all_credentials = state.database
        .load_all_debrid_credentials()
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    
    let mut statuses = Vec::new();
    
    for cred in all_credentials {
        statuses.push(CredentialStatus {
            provider: cred.provider.as_str().to_string(),
            is_configured: true,
            is_valid: None, // Would need to call validate_debrid_provider to check
            last_validated: Some(cred.last_validated),
        });
    }
    
    Ok(statuses)
}

/// Delete debrid provider credentials
#[tauri::command]
pub async fn delete_debrid_credentials(
    provider: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Deleting credentials for provider: {}", provider);
    
    let provider_type = match provider.as_str() {
        "torbox" => DebridProviderType::Torbox,
        "real-debrid" => DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
    state.database
        .delete_debrid_credentials(provider_type)
        .map_err(|e| format!("Failed to delete credentials: {}", e))?;
    
    tracing::info!("Credentials deleted for {}", provider);
    Ok(())
}

/// Validate debrid provider credentials
#[tauri::command]
pub async fn validate_debrid_provider(
    provider: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    tracing::info!("Validating credentials for provider: {}", provider);
    
    let provider_type = match provider.as_str() {
        "torbox" => DebridProviderType::Torbox,
        "real-debrid" => DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
    // Get cached master password
    let cached_password = state.master_password.read().await;
    let master_password = cached_password.as_ref()
        .ok_or_else(|| "Master password not unlocked. Please unlock first.".to_string())?;
    
    // Load credentials
    let credentials = state.database
        .load_debrid_credentials(provider_type)
        .map_err(|e| {
            tracing::error!("Failed to load credentials for {}: {}", provider, e);
            format!("Failed to load credentials: {}", e)
        })?
        .ok_or_else(|| {
            tracing::warn!("No credentials found for {}", provider);
            format!("No credentials found for {}", provider)
        })?;
    
    // Load master password data for salt
    let password_data = state.database
        .load_master_password()
        .map_err(|e| format!("Failed to load password data: {}", e))?
        .ok_or_else(|| "Master password not set".to_string())?;
    
    // Decrypt API key
    let crypto_manager = CryptoManager::from_password(master_password, &password_data.salt)
        .map_err(|e| format!("Failed to create crypto manager: {}", e))?;
    let api_key = crypto_manager.decrypt(&credentials.api_key_encrypted, &credentials.nonce)
        .map_err(|e| format!("Failed to decrypt API key: {}", e))?;
    
    tracing::debug!("Validating {} with API (first 10 chars: {}...)", provider, &api_key.chars().take(10).collect::<String>());
    
    // Validate with provider
    let debrid_manager = state.debrid_manager.read().await;
    let is_valid = debrid_manager.validate_provider(provider_type, &api_key)
        .await
        .map_err(|e| {
            tracing::error!("Validation failed for {}: {}", provider, e);
            format!("Validation failed: {}", e)
        })?;
    
    tracing::info!("Validation result for {}: {}", provider, is_valid);
    
    if is_valid {
        // Update last_validated timestamp
        let mut updated_cred = credentials;
        updated_cred.last_validated = chrono::Utc::now().timestamp();
        updated_cred.is_valid = true;
        state.database
            .save_debrid_credentials(&updated_cred)
            .map_err(|e| format!("Failed to update validation timestamp: {}", e))?;
        
        tracing::info!("Updated validation timestamp for {}", provider);
    }
    
    Ok(is_valid)
}

// ----------------------------------------------------------------------------
// Cache Check Commands
// ----------------------------------------------------------------------------

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
    
    // Convert HashMap<DebridProviderType, CacheStatus> to HashMap<String, CacheStatus>
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

// ----------------------------------------------------------------------------
// Torrent Management Commands
// ----------------------------------------------------------------------------

/// Add magnet link to debrid provider
#[tauri::command]
pub async fn add_magnet_to_debrid(
    magnet: String,
    provider: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    tracing::info!("Adding magnet to {}", provider);
    
    let provider_type = match provider.as_str() {
        "torbox" => DebridProviderType::Torbox,
        "real-debrid" => DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
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
    
    let provider_type = match provider.as_str() {
        "torbox" => DebridProviderType::Torbox,
        "real-debrid" => DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
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
    
    let provider_type = match provider.as_str() {
        "torbox" => DebridProviderType::Torbox,
        "real-debrid" => DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
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
    
    let provider_type = match provider.as_str() {
        "torbox" => DebridProviderType::Torbox,
        "real-debrid" => DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
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
    
    let provider_type = match provider.as_str() {
        "torbox" => DebridProviderType::Torbox,
        "real-debrid" => DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
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
    
    let provider_type = match provider.as_str() {
        "torbox" => DebridProviderType::Torbox,
        "real-debrid" => DebridProviderType::RealDebrid,
        _ => return Err(format!("Unknown provider: {}", provider)),
    };
    
    let debrid_manager = state.debrid_manager.read().await;
    debrid_manager.delete_torrent(provider_type, &torrent_id)
        .await
        .map_err(|e| format!("Failed to delete torrent: {}", e))?;
    
    Ok(())
}

/// Get cloud file download progress for a torrent
#[tauri::command]
pub fn get_cloud_file_progress(
    torrent_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<crate::state::CloudFileProgress>, String> {
    tracing::debug!("Getting cloud file progress for torrent: {}", torrent_id);
    
    state.cloud_file_progress.read()
        .map(|progress_map| {
            progress_map.get(&torrent_id)
                .map(|file_map| {
                    let mut files: Vec<_> = file_map.values().cloned().collect();
                    // Sort by file name for consistent ordering
                    files.sort_by(|a, b| a.name.cmp(&b.name));
                    files
                })
                .unwrap_or_default()
        })
        .map_err(|e| e.to_string())
}

// ----------------------------------------------------------------------------
// Settings Commands
// ----------------------------------------------------------------------------

/// Debrid settings for frontend
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DebridSettings {
    pub enable_debrid: bool,
    pub debrid_preference: Vec<String>,
    pub smart_mode_enabled: bool,
}

/// Get debrid settings
#[tauri::command]
pub async fn get_debrid_settings(state: State<'_, AppState>) -> Result<DebridSettings, String> {
    let app_settings = state.database
        .load_settings()
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    
    Ok(DebridSettings {
        enable_debrid: app_settings.enable_debrid,
        debrid_preference: app_settings.debrid_preference
            .iter()
            .map(|p| p.as_str().to_string())
            .collect(),
        smart_mode_enabled: app_settings.smart_mode_enabled,
    })
}

/// Update debrid settings
#[tauri::command]
pub async fn update_debrid_settings(
    settings: DebridSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("Updating debrid settings");
    
    // Load current settings
    let mut app_settings = state.database
        .load_settings()
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    
    // Update debrid-related fields
    app_settings.enable_debrid = settings.enable_debrid;
    app_settings.smart_mode_enabled = settings.smart_mode_enabled;
    
    // Parse provider preference
    let mut preference = Vec::new();
    for provider_str in settings.debrid_preference {
        let provider_type = match provider_str.as_str() {
            "torbox" => DebridProviderType::Torbox,
            "real-debrid" => DebridProviderType::RealDebrid,
            _ => continue,
        };
        preference.push(provider_type);
    }
    app_settings.debrid_preference = preference;
    
    // Save settings
    state.database
        .save_settings(&app_settings)
        .map_err(|e| format!("Failed to save settings: {}", e))?;
    
    // Update debrid manager preference
    let mut debrid_manager = state.debrid_manager.write().await;
    debrid_manager.set_preference(app_settings.debrid_preference.clone());
    
    tracing::info!("Debrid settings updated successfully");
    Ok(())
}

// ============================================================================
// ORIGINAL TORRENT MANAGEMENT COMMANDS (continued)
// ============================================================================

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
    state.torrents.write()
        .map(|mut torrents| {
            if let Some(torrent) = torrents.get_mut(&torrent_id) {
                torrent.state = TorrentState::Downloading;
            }
        })
        .map_err(|e| e.to_string())?;
    
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
    state.torrents.write()
        .map(|mut torrents| {
            if let Some(torrent) = torrents.get_mut(&torrent_id) {
                torrent.state = TorrentState::Paused;
            }
        })
        .map_err(|e| e.to_string())?;
    
    tracing::info!("Paused torrent: {}", torrent_id);
    Ok(())
}

/// Get detailed info about a specific torrent
#[tauri::command]
pub fn get_torrent_details(
    state: State<AppState>,
    torrent_id: String,
) -> Result<TorrentInfo, String> {
    state
        .torrents
        .read()
        .map_err(|e| e.to_string())
        .and_then(|torrents| {
            torrents
                .get(&torrent_id)
                .cloned()
                .ok_or_else(|| format!("Torrent not found: {}", torrent_id))
        })
}

/// Load all saved torrents from database
#[tauri::command]
pub async fn load_saved_torrents(state: State<'_, AppState>) -> Result<Vec<TorrentInfo>, String> {
    tracing::info!("Loading saved torrents from database");
    
    let sessions = state.database
        .load_all_torrents()
        .map_err(|e| format!("Failed to load torrents from database: {}", e))?;
    
    let mut torrents = Vec::new();
    
    for session in sessions {
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
            source: session.source.clone(), // Use stored source from database
        };
        
        // Create engine for this torrent (if not already exists)
        let engine_exists = state.engines.read().await.contains_key(&session.id);
        
        if !engine_exists {
            let download_dir = PathBuf::from(&session.download_dir);
            let mut engine = TorrentEngine::new(session.metainfo.clone(), download_dir);
            engine.set_database(state.database.clone());
            
            // TODO: Restore bitfield from session.bitfield if available
            
            // Store engine
            let engine_arc = Arc::new(TokioRwLock::new(engine));
            state.engines.write().await.insert(session.id.clone(), engine_arc.clone());
            
            // Auto-start if it was downloading/seeding before
            if torrent_state == TorrentState::Downloading || torrent_state == TorrentState::Seeding {
                tracing::info!("Auto-starting torrent: {}", session.id);
                
                // Send Start command
                {
                    let engine = engine_arc.read().await;
                    let cmd_tx = engine.command_sender();
                    let _ = cmd_tx.send(crate::engine::EngineCommand::Start);
                }
                
                // Spawn the engine's event loop
                let task_handle = tokio::spawn(async move {
                    let mut engine = engine_arc.write().await;
                    engine.run().await;
                });
                
                // Store task handle
                state.engine_tasks.write().await.insert(session.id.clone(), task_handle);
            }
        }
        
        // Add to in-memory state
        state.torrents.write()
            .map(|mut t| {
                t.insert(session.id.clone(), torrent_info.clone());
            })
            .map_err(|e| e.to_string())?;
        
        torrents.push(torrent_info);
    }
    
    tracing::info!("Loaded {} torrents from database", torrents.len());
    
    Ok(torrents)
}

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
    
    Ok(crate::torrent::get_file_list(metainfo))
}

/// Set file priority for selective downloading
#[tauri::command]
pub async fn set_file_priority(
    _state: State<'_, AppState>,
    torrent_id: String,
    file_path: String,
    priority: FilePriority,
) -> Result<(), String> {
    tracing::info!("Setting priority for file {} in torrent {} to {:?}", file_path, torrent_id, priority);
    
    // TODO: Implement file priority storage and piece selection logic
    // This will affect which pieces are downloaded based on file priorities
    tracing::warn!("File priority setting not yet fully implemented");
    
    Ok(())
}

/// Get available disk space for a given path
#[tauri::command]
pub fn get_available_disk_space(path: String) -> Result<u64, String> {
    use fs2::statvfs;
    
    tracing::debug!("Getting disk space for path: {}", path);
    
    let path_buf = PathBuf::from(&path);
    
    // Get the actual path to check (if path doesn't exist, use parent or current dir)
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
    
    // Get filesystem stats
    let stats = statvfs(&check_path)
        .map_err(|e| format!("Failed to get disk space: {}", e))?;
    
    // Calculate available space in bytes
    // f_bavail = available blocks for unprivileged users
    // f_bsize = filesystem block size
    let available_bytes = stats.available_space();
    
    tracing::debug!("Available space for {}: {} bytes", check_path.display(), available_bytes);
    
    Ok(available_bytes)
}
