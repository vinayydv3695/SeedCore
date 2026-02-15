/// Torrent download/upload engine
/// Coordinates peers, pieces, disk I/O, and trackers
use crate::database::{Database, TorrentSession};
use crate::disk::DiskManager;
use crate::peer::{PeerManager, PeerManagerCommand};
use crate::piece::{PieceManager, SelectionStrategy};
use crate::torrent::Metainfo;
use crate::tracker::http::HttpTracker;
use crate::tracker::{AnnounceRequest, AnnounceEvent};
use crate::utils;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time;
use tokio_util::sync::CancellationToken;

/// Maximum number of concurrent peer connections
const MAX_PEERS: usize = 50;

/// Interval for tracker announces (30 minutes)
const TRACKER_ANNOUNCE_INTERVAL: Duration = Duration::from_secs(1800);

/// Interval for saving progress to database (30 seconds)
const PROGRESS_SAVE_INTERVAL: Duration = Duration::from_secs(30);

/// Engine state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    Stopped,
    Starting,
    Downloading,
    Seeding,
    Paused,
    Error,
}

/// Statistics about current download/upload
#[derive(Debug, Clone)]
pub struct EngineStats {
    pub state: EngineState,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub download_speed: f64,  // bytes per second
    pub upload_speed: f64,    // bytes per second
    pub connected_peers: usize,
    pub total_peers: usize,
    pub progress: f64,        // 0.0 to 1.0
    pub eta_seconds: Option<u64>,
    pub completed_at: Option<i64>,
}

/// Command to control the engine
#[derive(Debug)]
pub enum EngineCommand {
    Start,
    Pause,
    Stop,
    SetStrategy(SelectionStrategy),
    GetStats(oneshot::Sender<EngineStats>),
}

/// Main torrent engine
pub struct TorrentEngine {
    /// Torrent metadata
    metainfo: Arc<Metainfo>,
    /// Piece manager
    piece_manager: Arc<RwLock<PieceManager>>,
    /// Disk I/O manager
    disk_manager: Arc<RwLock<DiskManager>>,
    /// Peer manager
    peer_manager_tx: Option<mpsc::Sender<PeerManagerCommand>>,
    /// Available peer addresses
    peer_addresses: Arc<RwLock<HashSet<SocketAddr>>>,
    /// Tracker client
    tracker: Arc<HttpTracker>,
    /// Tracker information for UI
    tracker_info: Arc<RwLock<Vec<crate::tracker::TrackerInfo>>>,
    /// Engine state
    state: Arc<RwLock<EngineState>>,
    /// Statistics
    stats: Arc<RwLock<EngineStats>>,
    /// Our peer ID
    peer_id: [u8; 20],
    /// Command channel receiver
    command_rx: mpsc::UnboundedReceiver<EngineCommand>,
    /// Command channel sender (for cloning)
    command_tx: mpsc::UnboundedSender<EngineCommand>,
    /// Database for persistence
    database: Option<Arc<Database>>,
    /// Download directory
    download_dir: PathBuf,
    /// Cancellation token for cooperative shutdown
    /// Cancellation token for cooperative shutdown
    cancel_token: CancellationToken,
    /// Tauri App Handle for events
    app_handle: Option<tauri::AppHandle>,
    /// Time when download completed
    completed_at: Option<i64>,
}

impl TorrentEngine {
    /// Create a new torrent engine
    pub fn new(metainfo: Metainfo, download_dir: PathBuf, app_handle: Option<tauri::AppHandle>) -> Self {
        let peer_id = utils::generate_peer_id();
        let num_pieces = metainfo.info.piece_count;
        let piece_length = metainfo.info.piece_length as usize;
        
        // Calculate last piece length
        let total_size = metainfo.info.total_size;
        let last_piece_length = if total_size % piece_length as u64 == 0 {
            piece_length
        } else {
            (total_size % piece_length as u64) as usize
        };

        // Extract piece hashes
        let piece_hashes: Vec<Vec<u8>> = (0..num_pieces)
            .map(|i| {
                let start = i * 20;
                let end = start + 20;
                metainfo.info.pieces[start..end].to_vec()
            })
            .collect();

        let piece_manager = PieceManager::new(
            num_pieces,
            piece_length,
            last_piece_length,
            piece_hashes,
            SelectionStrategy::RarestFirst,
        );

        let disk_manager = DiskManager::new(&metainfo, download_dir.clone());
        let tracker = HttpTracker::new();

        let (command_tx, command_rx) = mpsc::unbounded_channel();

        let stats = EngineStats {
            state: EngineState::Stopped,
            downloaded_bytes: 0,
            uploaded_bytes: 0,
            download_speed: 0.0,
            upload_speed: 0.0,
            connected_peers: 0,
            total_peers: 0,
            progress: 0.0,
            eta_seconds: None,
            completed_at: None,
        };

        Self {
            metainfo: Arc::new(metainfo),
            piece_manager: Arc::new(RwLock::new(piece_manager)),
            disk_manager: Arc::new(RwLock::new(disk_manager)),
            peer_manager_tx: None,
            peer_addresses: Arc::new(RwLock::new(HashSet::new())),
            tracker: Arc::new(tracker),
            tracker_info: Arc::new(RwLock::new(Vec::new())),
            state: Arc::new(RwLock::new(EngineState::Stopped)),
            stats: Arc::new(RwLock::new(stats)),
            peer_id,
            command_rx,
            command_tx,
            database: None,
            download_dir,
            cancel_token: CancellationToken::new(),
            app_handle,
            completed_at: None,
        }
    }

    /// Set completed_at timestamp (used when restoring state)
    pub fn set_completed_at(&mut self, timestamp: Option<i64>) {
        self.completed_at = timestamp;
    }

    /// Set database for persistence
    pub fn set_database(&mut self, database: Arc<Database>) {
        self.database = Some(database);
    }

    /// Get a command sender for controlling the engine
    pub fn command_sender(&self) -> mpsc::UnboundedSender<EngineCommand> {
        self.command_tx.clone()
    }

    /// Get the cancellation token for this engine
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Set priority for a specific file
    pub async fn set_file_priority(&mut self, file_index: usize, priority: crate::piece::PiecePriority) -> Result<(), String> {
        // Calculate file range in bytes
        let files = &self.metainfo.info.files;
        if file_index >= files.len() {
            return Err(format!("Invalid file index: {}", file_index));
        }

        let mut offset = 0;
        for i in 0..file_index {
            offset += files[i].length;
        }
        let length = files[file_index].length;
        let end = offset + length;

        // Calculate piece range
        let piece_length = self.metainfo.info.piece_length as u64;
        let start_piece = (offset / piece_length) as usize;
        let end_piece = ((end + piece_length - 1) / piece_length) as usize;

        // Update piece priorities
        let mut piece_manager = self.piece_manager.write().await;
        for piece_idx in start_piece..end_piece {
            if piece_idx < piece_manager.stats().total_pieces {
                piece_manager.set_piece_priority(piece_idx, priority);
            }
        }
        
        Ok(())
    }

    /// Run the engine (main event loop)
    pub async fn run(&mut self) {
        let mut tracker_timer = time::interval(TRACKER_ANNOUNCE_INTERVAL);
        let mut stats_timer = time::interval(Duration::from_secs(1));
        let mut save_timer = time::interval(PROGRESS_SAVE_INTERVAL);

        loop {
            tokio::select! {
                biased;

                // Cooperative cancellation — highest priority
                _ = self.cancel_token.cancelled() => {
                    tracing::info!("Engine received cancellation signal");
                    self.handle_stop().await;
                    break;
                }

                // Handle commands
                Some(cmd) = self.command_rx.recv() => {
                    match cmd {
                        EngineCommand::Start => self.handle_start().await,
                        EngineCommand::Pause => self.handle_pause().await,
                        EngineCommand::Stop => {
                            self.handle_stop().await;
                            break;
                        }
                        EngineCommand::SetStrategy(strategy) => {
                            self.piece_manager.write().await.set_strategy(strategy);
                        }
                        EngineCommand::GetStats(tx) => {
                            let stats = self.get_stats().await;
                            let _ = tx.send(stats);
                        }
                    }
                }

                // Periodic tracker announces
                _ = tracker_timer.tick() => {
                    let current_state = *self.state.read().await;
                    if current_state == EngineState::Downloading
                        || current_state == EngineState::Seeding
                    {
                        self.announce_to_tracker().await;
                    }
                }

                // Update statistics
                _ = stats_timer.tick() => {
                    self.update_stats().await;
                    
                    // Emit update event
                    if let Some(app) = &self.app_handle {
                        use tauri::Emitter;
                        // Construct TorrentInfo for UI
                        let stats = self.stats.read().await;
                        let state = match stats.state {
                            EngineState::Downloading => crate::state::TorrentState::Downloading,
                            EngineState::Seeding => crate::state::TorrentState::Seeding,
                            EngineState::Paused => crate::state::TorrentState::Paused,
                            EngineState::Stopped => crate::state::TorrentState::Paused,
                            EngineState::Starting => crate::state::TorrentState::Checking,
                            EngineState::Error => crate::state::TorrentState::Error,
                        };
                        
                        let info = crate::state::TorrentInfo {
                            id: self.metainfo.info_hash_hex(),
                            name: self.metainfo.info.name.clone(),
                            size: self.metainfo.info.total_size,
                            downloaded: stats.downloaded_bytes,
                            uploaded: stats.uploaded_bytes,
                            state,
                            download_speed: stats.download_speed as u64,
                            upload_speed: stats.upload_speed as u64,
                            peers: stats.connected_peers as u32,
                            seeds: 0, // TODO: Get from tracker stats
                            source: crate::debrid::types::DownloadSource::P2P,
                        };
                        
                        if let Err(e) = app.emit("torrent-update", info) {
                            tracing::error!("Failed to emit torrent-update event: {}", e);
                        }
                    }
                }

                // Save progress to database
                _ = save_timer.tick() => {
                    if *self.state.read().await != EngineState::Stopped {
                        self.save_progress().await;
                    }
                }
            }
        }

        tracing::info!("Torrent engine stopped");
    }

    /// Handle start command
    async fn handle_start(&mut self) {
        // Check if we are resuming from pause (PeerManager already exists)
        if let Some(ref tx) = self.peer_manager_tx {
            tracing::info!("Resuming torrent engine");
            
            // Determine state based on completion
            let pm = self.piece_manager.read().await;
            let new_state = if pm.is_complete() {
                EngineState::Seeding
            } else {
                EngineState::Downloading
            };
            drop(pm);

            *self.state.write().await = new_state;
            
            // Resume peer manager
            let _ = tx.send(PeerManagerCommand::Resume).await;
            return;
        }

        tracing::info!("Starting torrent engine");
        *self.state.write().await = EngineState::Starting;

        // Check if we have metadata (for magnet links)
        if self.metainfo.info.total_size == 0 || self.metainfo.info.piece_count == 0 {
            tracing::warn!("Cannot start download: metadata not yet fetched (magnet link)");
            tracing::warn!("Metadata exchange (BEP 9) not yet implemented");
            *self.state.write().await = EngineState::Error;
            return;
        }

        // Allocate files on disk
        if let Err(e) = self.disk_manager.read().await.allocate_files().await {
            tracing::error!("Failed to allocate files: {}", e);
            *self.state.write().await = EngineState::Error;
            return;
        }

        // Start peer manager with a child cancellation token
        let peer_cancel = self.cancel_token.child_token();
        let peer_manager = PeerManager::new(
            self.metainfo.info_hash,
            self.peer_id,
            self.piece_manager.clone(),
            self.disk_manager.clone(),
            peer_cancel,
        );
        
        let peer_manager_tx = peer_manager.command_sender();
        self.peer_manager_tx = Some(peer_manager_tx.clone());

        // Spawn peer manager task
        tokio::spawn(async move {
            peer_manager.run().await;
        });

        // Announce to tracker and get peers
        self.announce_to_tracker().await;

        // Connect to peers
        self.connect_to_peers().await;

        *self.state.write().await = EngineState::Downloading;
        tracing::info!("Torrent engine started");
    }

    /// Handle pause command
    async fn handle_pause(&mut self) {
        tracing::info!("Pausing torrent engine");
        *self.state.write().await = EngineState::Paused;

        // Pause peer manager
        if let Some(ref tx) = self.peer_manager_tx {
            let _ = tx.send(PeerManagerCommand::Pause).await;
        }
    }

    /// Handle stop command
    async fn handle_stop(&mut self) {
        tracing::info!("Stopping torrent engine");
        *self.state.write().await = EngineState::Stopped;

        // Cancel all child tasks (peer manager, etc.)
        self.cancel_token.cancel();

        // Flush pending writes
        if let Err(e) = self.disk_manager.write().await.flush_writes().await {
            tracing::error!("Failed to flush writes: {}", e);
        }

        // Save final progress
        self.save_progress().await;

        // Peer manager will exit via its cancellation token
        self.peer_manager_tx = None;

        // Final tracker announce (stopped)
        // TODO: Implement stopped event
    }

    /// Announce to tracker and update peer list
    async fn announce_to_tracker(&mut self) {
        let pm = self.piece_manager.read().await;
        let downloaded = (pm.completion() * self.metainfo.info.total_size as f64) as u64;
        let left = self.metainfo.info.total_size - downloaded;

        drop(pm); // Release lock

        let request = AnnounceRequest {
            info_hash: self.metainfo.info_hash,
            peer_id: self.peer_id,
            port: 6881,
            uploaded: self.stats.read().await.uploaded_bytes,
            downloaded,
            left,
            compact: true,
            numwant: Some(50),
            event: AnnounceEvent::None,
        };

        // Collect all trackers to try (primary + announce-list)
        let mut trackers_to_try = vec![self.metainfo.announce.clone()];
        
        // Add announce-list trackers (flatten the tiers)
        for tier in &self.metainfo.announce_list {
            for tracker_url in tier {
                if !trackers_to_try.contains(tracker_url) {
                    trackers_to_try.push(tracker_url.clone());
                }
            }
        }
        
        // Filter to only HTTP/HTTPS trackers (UDP not yet supported)
        trackers_to_try.retain(|url| url.starts_with("http://") || url.starts_with("https://"));
        
        tracing::debug!("Trying {} HTTP/HTTPS trackers", trackers_to_try.len());
        
        // Try each tracker until one succeeds
        let mut announce_succeeded = false;
        for tracker_url in &trackers_to_try {
            // Update tracker status to "Updating"
            let mut tracker_list = self.tracker_info.write().await;
            let tracker_idx = tracker_list.iter().position(|t| &t.url == tracker_url);
            if tracker_idx.is_none() {
                tracker_list.push(crate::tracker::TrackerInfo {
                    url: tracker_url.clone(),
                    status: crate::tracker::TrackerStatus::Updating,
                    message: "Announcing...".to_string(),
                    peers: 0,
                    seeds: 0,
                    leechers: 0,
                    downloaded: 0,
                    last_announce: None,
                    next_announce: None,
                });
            } else if let Some(idx) = tracker_idx {
                tracker_list[idx].status = crate::tracker::TrackerStatus::Updating;
                tracker_list[idx].message = "Announcing...".to_string();
            }
            drop(tracker_list);

            match self
                .tracker
                .announce(tracker_url, &request)
                .await
            {
                Ok(response) => {
                    tracing::info!(
                        "Tracker announce successful ({}): {} peers, interval {}s",
                        tracker_url,
                        response.peers.len(),
                        response.interval
                    );

                    // Add new peer addresses
                    let mut addresses = self.peer_addresses.write().await;
                    for peer in &response.peers {
                        addresses.insert(peer.addr);
                    }

                    // Update stats
                    self.stats.write().await.total_peers = addresses.len();
                    drop(addresses);

                    // Update tracker info with success
                    let mut tracker_list = self.tracker_info.write().await;
                    if let Some(tracker) = tracker_list.iter_mut().find(|t| &t.url == tracker_url) {
                        tracker.status = crate::tracker::TrackerStatus::Working;
                        tracker.message = "Announce OK".to_string();
                        tracker.peers = response.peers.len() as u32;
                        tracker.seeds = response.complete;
                        tracker.leechers = response.incomplete;
                        tracker.last_announce = Some(chrono::Utc::now().timestamp());
                        tracker.next_announce = Some(chrono::Utc::now().timestamp() + response.interval as i64);
                    }
                    
                    announce_succeeded = true;
                    break; // Success! No need to try other trackers
                }
                Err(e) => {
                    tracing::warn!("Tracker announce failed ({}): {}", tracker_url, e);
                    
                    // Update tracker info with error
                    let mut tracker_list = self.tracker_info.write().await;
                    if let Some(tracker) = tracker_list.iter_mut().find(|t| &t.url == tracker_url) {
                        tracker.status = crate::tracker::TrackerStatus::Error;
                        tracker.message = format!("Error: {}", e);
                    }
                    
                    // Continue to next tracker
                }
            }
        }
        
        if !announce_succeeded {
            tracing::error!("All trackers failed to announce");
        }
    }

    /// Connect to available peers
    async fn connect_to_peers(&self) {
        if let Some(ref peer_manager_tx) = self.peer_manager_tx {
            let addresses = self.peer_addresses.read().await;
            
            // Connect to up to MAX_PEERS
            for (i, addr) in addresses.iter().enumerate() {
                if i >= MAX_PEERS {
                    break;
                }
                
                tracing::info!("Requesting connection to peer: {}", addr);
                let _ = peer_manager_tx.send(PeerManagerCommand::AddPeer(*addr)).await;
            }
        }
    }

    /// Get current engine statistics
    pub async fn get_stats(&self) -> EngineStats {
        self.stats.read().await.clone()
    }

    /// Get current engine state
    pub async fn get_state(&self) -> EngineState {
        *self.state.read().await
    }

    /// Get torrent metainfo
    pub fn metainfo(&self) -> Arc<Metainfo> {
        self.metainfo.clone()
    }

    /// Get list of trackers and their status
    pub async fn get_tracker_list(&self) -> Vec<crate::tracker::TrackerInfo> {
        self.tracker_info.read().await.clone()
    }

    /// Get list of peers from peer manager
    pub async fn get_peer_list(&self) -> Vec<crate::peer::PeerInfo> {
        if let Some(ref tx) = self.peer_manager_tx {
            let (resp_tx, resp_rx) = oneshot::channel();
            if tx.send(PeerManagerCommand::GetPeerList(resp_tx)).await.is_ok() {
                return resp_rx.await.unwrap_or_default();
            }
        }
        Vec::new()
    }

    /// Get the peer manager command sender if available
    pub fn peer_manager_tx(&self) -> Option<mpsc::Sender<PeerManagerCommand>> {
        self.peer_manager_tx.clone()
    }

    /// Get the piece manager
    pub fn piece_manager(&self) -> Arc<RwLock<PieceManager>> {
        self.piece_manager.clone()
    }

    /// Update engine statistics
    async fn update_stats(&mut self) {
        let mut stats = self.stats.write().await;
        let pm = self.piece_manager.read().await;

        stats.state = *self.state.read().await;
        stats.progress = pm.completion();

        // Get peer stats from peer manager if available
        if let Some(ref peer_manager_tx) = self.peer_manager_tx {
            let (tx, rx) = oneshot::channel();
            if peer_manager_tx.send(PeerManagerCommand::GetStats(tx)).await.is_ok() {
                if let Ok(peer_stats) = rx.await {
                    stats.connected_peers = peer_stats.connected_peers;
                    stats.downloaded_bytes = peer_stats.total_downloaded;
                    stats.uploaded_bytes = peer_stats.total_uploaded;
                    stats.download_speed = peer_stats.download_speed;
                    stats.upload_speed = peer_stats.upload_speed;
                }
            }
        }

        // Calculate ETA
        if stats.download_speed > 0.0 {
            let remaining = self.metainfo.info.total_size - stats.downloaded_bytes;
            stats.eta_seconds = Some((remaining as f64 / stats.download_speed) as u64);
        } else {
            stats.eta_seconds = None;
        }

        stats.completed_at = self.completed_at;

        // Check if we're complete
        if pm.is_complete() {
             if stats.state == EngineState::Downloading {
                drop(stats); // Release lock before modifying state
                drop(pm);
                *self.state.write().await = EngineState::Seeding;
                if self.completed_at.is_none() {
                    self.completed_at = Some(chrono::Utc::now().timestamp());
                    tracing::info!("Download complete! Now seeding. Completed at: {:?}", self.completed_at);
                }
            } else if self.completed_at.is_none() {
                // If we started as Seeding but didn't have completed_at set
                self.completed_at = Some(chrono::Utc::now().timestamp());
            }
        }
    }

    // ... (existing methods until save_progress)

    /// Save progress to database
    async fn save_progress(&self) {
        if let Some(ref database) = self.database {
            let pm = self.piece_manager.read().await;
            let stats = self.stats.read().await;
            let state = *self.state.read().await;
            let id = hex::encode(self.metainfo.info_hash);

            // Preserve original added_at from existing DB entry
            let added_at = database
                .load_torrent(&id)
                .ok()
                .flatten()
                .map(|s| s.added_at)
                .unwrap_or_else(|| chrono::Utc::now().timestamp());

            let session = TorrentSession {
                id: id.clone(),
                metainfo: (*self.metainfo).clone(),
                bitfield: pm.our_bitfield().as_bytes().to_vec(),
                num_pieces: pm.our_bitfield().num_pieces(),
                downloaded: stats.downloaded_bytes,
                uploaded: stats.uploaded_bytes,
                state: format!("{:?}", state).to_lowercase(),
                download_dir: self.download_dir.to_string_lossy().to_string(),
                added_at,
                last_activity: chrono::Utc::now().timestamp(),
                source: crate::debrid::types::DownloadSource::P2P, // Default to P2P
                completed_at: self.completed_at,
            };

            if let Err(e) = database.save_torrent(&session) {
                tracing::error!("Failed to save progress to database: {}", e);
            } else {
                tracing::debug!("Progress saved to database for torrent {}", session.id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::{TorrentInfo, FileInfo};

    fn create_test_metainfo() -> Metainfo {
        Metainfo {
            announce: "http://tracker.example.com/announce".to_string(),
            announce_list: vec![],
            info: TorrentInfo {
                piece_length: 16384,
                pieces: vec![0u8; 40], // 2 pieces
                piece_count: 2,
                files: vec![FileInfo {
                    path: vec!["test.txt".to_string()],
                    length: 20000,
                }],
                name: "test.txt".to_string(),
                total_size: 20000,
                is_single_file: true,
            },
            info_hash: [0u8; 20],
            creation_date: None,
            comment: None,
            created_by: None,
        }
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let metainfo = create_test_metainfo();
        let download_dir = PathBuf::from("/tmp/test_engine");
        let engine = TorrentEngine::new(metainfo, download_dir);

        assert_eq!(engine.get_state().await, EngineState::Stopped);
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.progress, 0.0);
        assert_eq!(stats.connected_peers, 0);
    }

    #[tokio::test]
    async fn test_engine_command_sender() {
        let metainfo = create_test_metainfo();
        let download_dir = PathBuf::from("/tmp/test_engine2");
        let engine = TorrentEngine::new(metainfo, download_dir);

        let tx = engine.command_sender();
        
        // Send a command
        let (stats_tx, _stats_rx) = oneshot::channel();
        tx.send(EngineCommand::GetStats(stats_tx)).unwrap();

        // The command was sent successfully
        // (we can't test receiving without running the engine)
    }

    #[test]
    fn test_engine_stats() {
        let stats = EngineStats {
            state: EngineState::Downloading,
            downloaded_bytes: 10000,
            uploaded_bytes: 5000,
            download_speed: 1024.0,
            upload_speed: 512.0,
            connected_peers: 5,
            total_peers: 10,
            progress: 0.5,
            eta_seconds: Some(120),
            completed_at: None,
        };

        assert_eq!(stats.state, EngineState::Downloading);
        assert_eq!(stats.progress, 0.5);
        assert_eq!(stats.connected_peers, 5);
    }
}
