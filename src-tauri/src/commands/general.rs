//! General commands: app info, settings, greeting

use crate::state::AppState;
use tauri::State;

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
pub async fn get_settings(state: State<'_, AppState>) -> Result<crate::state::Settings, String> {
    Ok(state.settings.read().await.clone())
}

/// Update application settings
#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: crate::state::Settings,
) -> Result<(), String> {
    // Update memory state
    *state.settings.write().await = settings.clone();

    // Persist to database
    let mut db_settings = state.database.load_settings()
        .map_err(|e| format!("Failed to load settings: {}", e))?;
    
    db_settings.max_download_speed = settings.download_limit;
    db_settings.max_upload_speed = settings.upload_limit;
    db_settings.max_concurrent_downloads = settings.max_active_downloads as usize;
    db_settings.listen_port = settings.listen_port;
    db_settings.enable_dht = settings.enable_dht;
    db_settings.enable_pex = settings.enable_pex;
    db_settings.bandwidth_scheduler_enabled = settings.bandwidth_scheduler_enabled;
    db_settings.bandwidth_schedule = settings.bandwidth_schedule;

    state.database.save_settings(&db_settings)
        .map_err(|e| format!("Failed to save settings: {}", e))?;

    Ok(())
}

/// Get list of all torrents
#[tauri::command]
pub async fn get_torrents(state: State<'_, AppState>) -> Result<Vec<crate::state::TorrentInfo>, String> {
    Ok(state.torrents.read().await.values().cloned().collect())
}

/// Get debrid settings
#[tauri::command]
pub async fn get_debrid_settings(state: State<'_, AppState>) -> Result<super::DebridSettings, String> {
    let app_settings = state.database
        .load_settings()
        .map_err(|e| format!("Failed to load settings: {}", e))?;

    Ok(super::DebridSettings {
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
    settings: super::DebridSettings,
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

    // Parse provider preference using shared helper
    let mut preference = Vec::new();
    for provider_str in settings.debrid_preference {
        if let Ok(provider_type) = super::parse_provider(&provider_str) {
            preference.push(provider_type);
        }
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

/// Backup all database data to a JSON string
#[tauri::command]
pub async fn backup_data(state: State<'_, AppState>) -> Result<String, String> {
    state.database
        .dump_all()
        .map_err(|e| format!("Failed to create backup: {}", e))
}

/// Export backup to a file
#[tauri::command]
pub async fn export_backup(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let json = state.database
        .dump_all()
        .map_err(|e| format!("Failed to create backup: {}", e))?;
    
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write backup file: {}", e))?;
    
    tracing::info!("Backup exported successfully to: {}", path);
    Ok(())
}

/// Restore database data from a JSON string
#[tauri::command]
pub async fn restore_data(state: State<'_, AppState>, json: String) -> Result<(), String> {
    state.database
        .restore(&json)
        .map_err(|e| format!("Failed to restore backup: {}", e))?;
    
    if let Ok(settings) = state.database.load_settings() {
        *state.settings.write().await = settings.into();
    }
    
    Ok(())
}

/// Import backup from a file
#[tauri::command]
pub async fn import_backup(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read backup file: {}", e))?;
    
    state.database
        .restore(&json)
        .map_err(|e| format!("Failed to restore backup: {}", e))?;
    
    if let Ok(settings) = state.database.load_settings() {
        *state.settings.write().await = settings.into();
    }
    
    tracing::info!("Backup imported successfully from: {}", path);
    Ok(())
}
