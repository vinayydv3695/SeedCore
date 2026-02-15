// SeedCore - Modern BitTorrent Client
// Core library and Tauri integration

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

// Module declarations
pub mod bencode;
pub mod cloud;
pub mod commands;
pub mod crypto;
pub mod database;
pub mod debrid;
pub mod disk;
pub mod download;
pub mod engine;
pub mod error;
pub mod magnet;
pub mod peer;
pub mod piece;
pub mod scheduler;
pub mod state;
pub mod torrent;
pub mod tracker;
pub mod utils;
pub mod cleanup;

// Re-exports
pub use error::{Error, Result};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Shared references for graceful shutdown (populated in setup, used in on_window_event)
struct ShutdownState {
    engines: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, std::sync::Arc<tokio::sync::RwLock<engine::TorrentEngine>>>>>,
    engine_tasks: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, tokio::task::JoinHandle<()>>>>,
    cloud_download_tasks: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, tokio::task::JoinHandle<()>>>>,
    master_password: std::sync::Arc<tokio::sync::RwLock<Option<String>>>,
    database: std::sync::Arc<database::Database>,
    _tracing_guard: std::sync::Arc<std::sync::Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>>,
}

/// Initialize the application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    let log_dir = dirs::config_dir()
        .map(|d| d.join("seedcore").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));
    
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Failed to create log directory: {}", e);
    }

    let file_appender = tracing_appender::rolling::daily(&log_dir, "seedcore.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    // Store guard in Arc<Mutex> so it can be properly dropped on shutdown
    let guard_arc = std::sync::Arc::new(std::sync::Mutex::new(Some(guard)));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "seedcore=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();

    tracing::info!("Starting SeedCore v{}", env!("CARGO_PKG_VERSION"));

    // Initialize application state
    let app_state = match state::AppState::new() {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Failed to initialize application state: {}", e);
            eprintln!("FATAL ERROR: {}", e);
            eprintln!("The application cannot start without a working database.");
            eprintln!("Please check file permissions and disk space.");
            std::process::exit(1);
        }
    };

    // Clone Arc refs before moving app_state into manage()
    let shutdown_state = std::sync::Arc::new(ShutdownState {
        engines: app_state.engines.clone(),
        engine_tasks: app_state.engine_tasks.clone(),
        cloud_download_tasks: app_state.cloud_download_tasks.clone(),
        master_password: app_state.master_password.clone(),
        database: app_state.database.clone(),
        _tracing_guard: guard_arc,
    });

    // Build and run Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .setup(|app| {
            // Start auto-cleanup task
            let cleanup_app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                cleanup::start_cleanup_task(cleanup_app).await;
            });

            // Start bandwidth scheduler task
            let scheduler_app = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                scheduler::start_scheduler_task(scheduler_app).await;
            });

            Ok(())
        })
        .on_window_event(move |_win, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                tracing::info!("Window close requested, performing graceful shutdown...");

                let ss = shutdown_state.clone();

                // Spawn shutdown task
                tauri::async_runtime::spawn(async move {
                    // 1. Cancel all engine tokens and send Stop commands
                    {
                        let engines_map = ss.engines.read().await;
                        for (id, engine_arc) in engines_map.iter() {
                            tracing::info!("Stopping engine: {}", id);
                            let engine = engine_arc.read().await;
                            engine.cancel_token().cancel();
                            let _ = engine.command_sender().send(
                                engine::EngineCommand::Stop,
                            );
                        }
                    }

                    // 2. Abort all cloud download tasks
                    {
                        let mut cloud_tasks = ss.cloud_download_tasks.write().await;
                        for (id, task) in cloud_tasks.drain() {
                            tracing::info!("Aborting cloud download: {}", id);
                            task.abort();
                        }
                    }

                    // 3. Wait briefly for engine tasks to finish
                    {
                        let mut tasks = ss.engine_tasks.write().await;
                        for (id, task) in tasks.drain() {
                            tracing::info!("Waiting for engine task: {}", id);
                            let _ = tokio::time::timeout(
                                std::time::Duration::from_secs(3),
                                task,
                            ).await;
                        }
                    }

                    // 4. Clear master password from memory
                    {
                        let mut pw = ss.master_password.write().await;
                        *pw = None;
                        tracing::info!("Master password cleared from memory");
                    }

                    // 5. Flush database
                    if let Err(e) = ss.database.flush() {
                        tracing::error!("Failed to flush database on shutdown: {}", e);
                    } else {
                        tracing::info!("Database flushed successfully");
                    }

                    // 6. Drop the tracing guard to ensure proper cleanup
                    if let Ok(mut guard_opt) = ss._tracing_guard.lock() {
                        if let Some(guard) = guard_opt.take() {
                            drop(guard);
                            tracing::info!("Tracing guard dropped");
                        }
                    }

                    tracing::info!("Graceful shutdown complete");
                    
                    // Give async tasks time to finish logging
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            // General commands
            commands::greet,
            commands::get_version,
            commands::get_settings,
            commands::update_settings,
            commands::backup_data,
            commands::restore_data,
            commands::export_backup,
            commands::import_backup,
            // Torrent commands
            commands::get_torrents,
            commands::parse_torrent_file,
            commands::parse_magnet_link,
            commands::add_torrent_file,
            commands::add_magnet_link,
            commands::add_cloud_torrent,
            commands::remove_torrent,
            commands::start_torrent,
            commands::pause_torrent,
            commands::get_torrent_details,
            commands::load_saved_torrents,
            // Torrent info commands
            commands::get_peer_list,
            commands::get_tracker_list,
            commands::get_pieces_info,
            commands::get_file_list,
            commands::set_file_priority,
            commands::get_available_disk_space,
            // Master password commands
            commands::check_master_password_set,
            commands::set_master_password,
            commands::unlock_with_master_password,
            commands::change_master_password,
            commands::lock_debrid_services,
            // Credential management commands
            commands::save_debrid_credentials,
            commands::get_debrid_credentials_status,
            commands::delete_debrid_credentials,
            commands::validate_debrid_provider,
            // Cache check commands
            commands::check_torrent_cache,
            commands::get_preferred_cached_provider,
            // Torrent management commands
            commands::add_magnet_to_debrid,
            commands::add_torrent_file_to_debrid,
            commands::select_debrid_files,
            commands::get_debrid_download_links,
            commands::list_debrid_torrents,
            commands::delete_debrid_torrent,
            commands::get_cloud_file_progress,
            // Settings commands
            commands::get_debrid_settings,
            commands::update_debrid_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
