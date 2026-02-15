use seedcore_lib::engine::{TorrentEngine, EngineState, EngineCommand};
use seedcore_lib::torrent::Metainfo;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_ubuntu_torrent_download() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    // 1. Download Ubuntu torrent file
    let torrent_url = "https://releases.ubuntu.com/24.04/ubuntu-24.04.4-live-server-amd64.iso.torrent";
    let torrent_path = PathBuf::from("ubuntu.torrent");
    
    // Use curl to download the file (reqwest is async and blocking in test setup is annoying without runtime)
    let status = std::process::Command::new("curl")
        .arg("-L")
        .arg("-o")
        .arg(&torrent_path)
        .arg(torrent_url)
        .status()
        .expect("Failed to execute curl");
        
    assert!(status.success(), "Failed to download torrent file");
    
    // 2. Read and parse torrent file
    let torrent_data = std::fs::read(&torrent_path).expect("Failed to read torrent file");
    let metainfo = Metainfo::from_bytes(&torrent_data).expect("Failed to parse torrent file");
    
    // 3. Setup download directory
    let download_dir = PathBuf::from("test_downloads");
    if download_dir.exists() {
        std::fs::remove_dir_all(&download_dir).unwrap_or(());
    }
    std::fs::create_dir_all(&download_dir).expect("Failed to create download dir");
    
    // 4. Create and start engine
    let mut engine = TorrentEngine::new(metainfo, download_dir.clone());
    
    // Create a channel to control the engine from a separate task
    let command_tx = engine.command_sender();
    
    // Spawn engine in background
    let engine_handle = tokio::spawn(async move {
        engine.run().await;
    });
    
    // Start the engine
    command_tx.send(EngineCommand::Start).expect("Failed to send start command");
    
    // 5. Monitor progress
    let mut success = false;
    let mut attempts = 0;
    while attempts < 120 { // Wait up to 120 seconds
        sleep(Duration::from_secs(1)).await;
        
        let (stats_tx, stats_rx) = tokio::sync::oneshot::channel();
        if command_tx.send(EngineCommand::GetStats(stats_tx)).is_err() {
            break;
        }
        
        match stats_rx.await {
            Ok(stats) => {
                println!(
                    "Time: {}s, State: {:?}, Peers: {}, Downloaded: {} bytes, Speed: {:.2} KB/s", 
                    attempts, stats.state, stats.connected_peers, stats.downloaded_bytes, stats.download_speed / 1024.0
                );
                
                if stats.downloaded_bytes > 0 {
                    success = true;
                    println!("Download started successfully!");
                    break;
                }
            },
            Err(_) => break,
        }
        
        attempts += 1;
    }
    
    // Cleanup
    let _ = command_tx.send(EngineCommand::Stop);
    let _ = engine_handle.await;
    
    // Clean up files
    let _ = std::fs::remove_file("ubuntu.torrent");
    let _ = std::fs::remove_dir_all("test_downloads");
    
    assert!(success, "Torrent failed to start downloading within 60 seconds");
}
