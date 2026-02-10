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
pub mod state;
pub mod torrent;
pub mod tracker;
pub mod utils;

// Re-exports
pub use error::{Error, Result};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize the application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "seedcore=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting SeedCore v{}", env!("CARGO_PKG_VERSION"));

    // Build and run Tauri application
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::new())
        .invoke_handler(tauri::generate_handler![
            // General commands
            commands::greet,
            commands::get_version,
            commands::get_settings,
            commands::update_settings,
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
