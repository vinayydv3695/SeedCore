use crate::state::AppState;
use tauri::{Manager, Emitter};
use tokio::time::{self, Duration};
use crate::engine::EngineState;

pub async fn start_cleanup_task(app_handle: tauri::AppHandle) {
    let mut interval = time::interval(Duration::from_secs(60)); // Check every minute

    loop {
        interval.tick().await;

        let state_guard = app_handle.state::<AppState>();
        
        // Load settings from database
        let settings = match state_guard.database.load_settings() {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Cleanup task failed to load settings: {}", e);
                continue;
            }
        };

        if !settings.cleanup_enabled {
            continue;
        }

        // Get snapshot of engines (cloning the map to avoid holding lock while iterating and locking engines)
        let engines_map = state_guard.engines.read().await.clone();
        
        for (id, engine_arc) in engines_map {
            // Read stats
            let engine = engine_arc.read().await;
            let stats = engine.get_stats().await;
            
            // Only consider Seeding torrents
            if stats.state != EngineState::Seeding {
                continue;
            }

            let metainfo = engine.metainfo();
            let total_size = metainfo.info.total_size;
            let torrent_name = metainfo.info.name.clone();
            let completed_at = stats.completed_at;
            let uploaded = stats.uploaded_bytes;
            drop(engine); // Release read lock

            let mut should_cleanup = false;
            let mut reason = String::new();

            // Check Ratio
            if settings.cleanup_ratio > 0.0 && total_size > 0 {
                let ratio = uploaded as f64 / total_size as f64;
                if ratio >= settings.cleanup_ratio as f64 {
                    should_cleanup = true;
                    reason = format!("Ratio reached {:.2} (limit {:.2})", ratio, settings.cleanup_ratio);
                }
            }

            // Check Time
            if !should_cleanup && settings.cleanup_time > 0 {
                if let Some(ts) = completed_at {
                    let now = chrono::Utc::now().timestamp();
                    let seeded_seconds = now - ts;
                    if seeded_seconds >= settings.cleanup_time as i64 {
                        should_cleanup = true;
                        reason = format!("Seeding time reached {}s (limit {}s)", seeded_seconds, settings.cleanup_time);
                    }
                }
            }

            if should_cleanup {
                tracing::info!("Auto-cleanup triggered for {} ({}): {}", torrent_name, id, reason);
                
                match settings.cleanup_mode.as_str() {
                    "Pause" => {
                         let engine = engine_arc.read().await; 
                         let _ = engine.command_sender().send(crate::engine::EngineCommand::Pause);
                         drop(engine);

                         // Update UI state
                         let mut torrents = state_guard.torrents.write().await;
                         if let Some(torrent) = torrents.get_mut(&id) {
                             torrent.state = crate::state::TorrentState::Paused;
                         }
                    }
                    "Remove" => {
                        let _ = crate::commands::remove_torrent_internal(&state_guard, id.clone(), false).await;
                    }
                    "Delete" => {
                        let _ = crate::commands::remove_torrent_internal(&state_guard, id.clone(), true).await;
                    }
                    _ => {}
                }
                
               if let Err(e) = app_handle.emit("cleanup-triggered", format!("Cleaned up {}: {}", torrent_name, reason)) {
                   tracing::error!("Failed to emit cleanup-triggered event: {}", e);
               }
            }
        }
    }
}
