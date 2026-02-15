/// Peer manager - handles multiple peer connections and download coordination
use super::{PeerConnection, Message};
use crate::piece::{Bitfield, BlockInfo, PieceManager};
use crate::disk::DiskManager;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time;
use tokio_util::sync::CancellationToken;

/// Maximum number of concurrent block requests per peer
const MAX_PENDING_REQUESTS: usize = 5;

/// Timeout for block requests (30 seconds)
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Keep-alive interval (2 minutes)
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(120);

/// Choking algorithm interval (10 seconds)
const CHOKING_INTERVAL: Duration = Duration::from_secs(10);

/// Optimistic unchoke interval (30 seconds)
const OPTIMISTIC_UNCHOKE_INTERVAL: Duration = Duration::from_secs(30);

/// Number of peers to unchoke
const NUM_UNCHOKED: usize = 4;

/// Peer session state
struct PeerSession {
    /// Peer connection
    connection: PeerConnection,
    /// Last activity timestamp
    last_activity: Instant,
    /// Pending block requests (block_info -> request_time)
    pending_requests: HashMap<BlockInfo, Instant>,
    /// Peer's bitfield
    peer_bitfield: Option<Bitfield>,
    /// Download statistics
    downloaded_bytes: u64,
    uploaded_bytes: u64,
    /// Download speed (bytes/sec)
    download_speed: f64,
    /// Upload speed (bytes/sec)
    upload_speed: f64,
    /// Bytes downloaded at last stats update
    last_downloaded_bytes: u64,
    /// Bytes uploaded at last stats update
    last_uploaded_bytes: u64,
}

impl PeerSession {
    fn new(connection: PeerConnection) -> Self {
        Self {
            connection,
            last_activity: Instant::now(),
            pending_requests: HashMap::new(),
            peer_bitfield: None,
            downloaded_bytes: 0,
            uploaded_bytes: 0,
            download_speed: 0.0,
            upload_speed: 0.0,
            last_downloaded_bytes: 0,
            last_uploaded_bytes: 0,
        }
    }

    /// Check if we can send more requests to this peer
    fn can_request(&self) -> bool {
        !self.connection.peer_choking && self.pending_requests.len() < MAX_PENDING_REQUESTS
    }

    /// Mark a request as pending
    fn add_pending_request(&mut self, block: BlockInfo) {
        self.pending_requests.insert(block, Instant::now());
    }

    /// Remove a completed request
    fn remove_pending_request(&mut self, block: &BlockInfo) -> bool {
        self.pending_requests.remove(block).is_some()
    }

    /// Get timed-out requests
    fn get_timed_out_requests(&self) -> Vec<BlockInfo> {
        let now = Instant::now();
        self.pending_requests
            .iter()
            .filter(|(_, request_time)| now.duration_since(**request_time) > REQUEST_TIMEOUT)
            .map(|(block, _)| *block)
            .collect()
    }

    /// Check if keep-alive is needed
    fn needs_keep_alive(&self) -> bool {
        Instant::now().duration_since(self.last_activity) > KEEP_ALIVE_INTERVAL
    }
}

/// Command to the peer manager
pub enum PeerManagerCommand {
    /// Add a peer address to connect to
    AddPeer(SocketAddr),
    /// Remove a peer
    RemovePeer(SocketAddr),
    /// Get peer statistics
    GetStats(oneshot::Sender<PeerManagerStats>),
    /// Get peer list for UI
    GetPeerList(oneshot::Sender<Vec<crate::peer::PeerInfo>>),
    /// Broadcast that we have a piece
    BroadcastHave(usize),
    /// Pause peer manager (stop requesting blocks)
    Pause,
    /// Resume peer manager
    Resume,
}

/// Peer manager statistics
#[derive(Debug, Clone)]
pub struct PeerManagerStats {
    pub connected_peers: usize,
    pub total_downloaded: u64,
    pub total_uploaded: u64,
    pub download_speed: f64,
    pub upload_speed: f64,
}

/// Manages all peer connections for a torrent
pub struct PeerManager {
    /// Torrent info hash
    info_hash: [u8; 20],
    /// Our peer ID
    peer_id: [u8; 20],
    /// Active peer sessions
    sessions: Arc<RwLock<HashMap<SocketAddr, PeerSession>>>,
    /// Piece manager (shared with engine)
    piece_manager: Arc<RwLock<PieceManager>>,
    /// Disk manager (shared with engine)
    disk_manager: Arc<RwLock<DiskManager>>,
    /// Command receiver
    command_rx: mpsc::Receiver<PeerManagerCommand>,
    /// Command sender (for cloning)
    command_tx: mpsc::Sender<PeerManagerCommand>,
    /// Statistics
    stats: Arc<RwLock<PeerManagerStats>>,
    /// Cancellation token for cooperative shutdown
    cancel_token: CancellationToken,
    /// Paused state
    paused: bool,
}

impl PeerManager {
    /// Create a new peer manager
    pub fn new(
        info_hash: [u8; 20],
        peer_id: [u8; 20],
        piece_manager: Arc<RwLock<PieceManager>>,
        disk_manager: Arc<RwLock<DiskManager>>,
        cancel_token: CancellationToken,
    ) -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        let stats = PeerManagerStats {
            connected_peers: 0,
            total_downloaded: 0,
            total_uploaded: 0,
            download_speed: 0.0,
            upload_speed: 0.0,
        };

        Self {
            info_hash,
            peer_id,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            piece_manager,
            disk_manager,
            command_rx,
            command_tx,
            stats: Arc::new(RwLock::new(stats)),
            cancel_token,
            paused: false,
        }
    }

    /// Get command sender
    pub fn command_sender(&self) -> mpsc::Sender<PeerManagerCommand> {
        self.command_tx.clone()
    }

    /// Run the peer manager event loop
    pub async fn run(mut self) {
        let mut tick_interval = time::interval(Duration::from_secs(1));
        let mut keep_alive_interval = time::interval(Duration::from_secs(30));
        let mut choking_interval = time::interval(CHOKING_INTERVAL);
        let mut optimistic_interval = time::interval(OPTIMISTIC_UNCHOKE_INTERVAL);

        loop {
            tokio::select! {
                biased;

                // Cooperative cancellation — highest priority
                _ = self.cancel_token.cancelled() => {
                    tracing::info!("PeerManager received cancellation signal, shutting down");
                    break;
                }

                // Handle commands
                Some(cmd) = self.command_rx.recv() => {
                    match cmd {
                        PeerManagerCommand::AddPeer(addr) => {
                            if !self.paused {
                                self.connect_to_peer(addr).await;
                            }
                        }
                        PeerManagerCommand::RemovePeer(addr) => {
                            self.sessions.write().await.remove(&addr);
                        }
                        PeerManagerCommand::GetStats(tx) => {
                            let stats = self.stats.read().await.clone();
                            let _ = tx.send(stats);
                        }
                        PeerManagerCommand::GetPeerList(tx) => {
                            let peer_list = self.get_peer_list().await;
                            let _ = tx.send(peer_list);
                        }
                        PeerManagerCommand::BroadcastHave(piece_index) => {
                            // Can still broadcast haves while paused? Probably yes, to keep state in sync
                            self.broadcast_have(piece_index).await;
                        }
                        PeerManagerCommand::Pause => {
                            tracing::info!("PeerManager paused");
                            self.paused = true;
                            // Optionally disconnect peers or just stop requesting?
                            // For now, let's just stop requesting and processing new connections
                        }
                        PeerManagerCommand::Resume => {
                            tracing::info!("PeerManager resumed");
                            self.paused = false;
                        }
                    }
                }

                // Periodic tasks
                _ = tick_interval.tick() => {
                    if !self.paused {
                        self.handle_pending_requests().await;
                    }
                    self.update_stats().await; // Always update stats
                }

                // Send keep-alive messages
                _ = keep_alive_interval.tick() => {
                    self.send_keep_alives().await;
                }

                // Choking algorithm
                _ = choking_interval.tick() => {
                    if !self.paused {
                        self.update_choking().await;
                    }
                }

                // Optimistic unchoke
                _ = optimistic_interval.tick() => {
                    if !self.paused {
                        self.optimistic_unchoke().await;
                    }
                }
            }
        }

        // Clean up: disconnect all peers
        let mut sessions = self.sessions.write().await;
        tracing::info!("PeerManager shutting down, disconnecting {} peers", sessions.len());
        sessions.clear();
    }

    /// Connect to a peer and start download loop
    async fn connect_to_peer(&self, addr: SocketAddr) {
        tracing::info!("Connecting to peer: {}", addr);

        // Connect
        let connection = match PeerConnection::connect(addr).await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::warn!("Failed to connect to {}: {}", addr, e);
                return;
            }
        };

        let mut session = PeerSession::new(connection);

        // Perform handshake
        if let Err(e) = session
            .connection
            .handshake(self.info_hash, self.peer_id)
            .await
        {
            tracing::warn!("Handshake failed with {}: {}", addr, e);
            return;
        }

        tracing::info!("Handshake successful with {}", addr);

        // CRITICAL FIX: Send our bitfield immediately after handshake
        // This tells the peer what pieces we have
        let our_bitfield = {
            let pm = self.piece_manager.read().await;
            pm.our_bitfield().as_bytes().to_vec()
        };
        
        if let Err(e) = session.connection.send_message(&Message::Bitfield {
            bitfield: our_bitfield
        }).await {
            tracing::warn!("Failed to send bitfield to {}: {}", addr, e);
            return;
        }
        
        tracing::debug!("Sent our bitfield to {}", addr);

        // Store session
        let sessions = self.sessions.clone();
        sessions.write().await.insert(addr, session);

        // Spawn peer handler
        let sessions_clone = sessions.clone();
        let piece_manager = self.piece_manager.clone();
        let disk_manager = self.disk_manager.clone();
        let peer_id_str = format!("{:?}", addr); // Use for tracking

        tokio::spawn(async move {
            if let Err(e) = Self::handle_peer(
                addr,
                sessions_clone,
                piece_manager,
                disk_manager,
                peer_id_str,
            )
            .await
            {
                tracing::error!("Peer handler error for {}: {}", addr, e);
            }
        });
    }

    /// Handle communication with a single peer
    async fn handle_peer(
        addr: SocketAddr,
        sessions: Arc<RwLock<HashMap<SocketAddr, PeerSession>>>,
        piece_manager: Arc<RwLock<PieceManager>>,
        disk_manager: Arc<RwLock<DiskManager>>,
        peer_id: String,
    ) -> Result<(), String> {
        loop {
            // CRITICAL FIX: Extract connection from sessions to avoid holding lock during I/O
            // We temporarily remove the session, do I/O, then re-insert it
            
            // Step 1: Extract the entire session (including connection)
            let mut session = {
                let mut sessions_guard = sessions.write().await;
                match sessions_guard.remove(&addr) {
                    None => return Err("Session not found".to_string()),
                    Some(s) => s,
                }
            };
            // Lock is now released - other peers can proceed
            
            // Step 2: Do network I/O without holding any lock
            let message = match session.connection.recv_message().await {
                Ok(msg) => {
                    session.last_activity = Instant::now();
                    msg
                },
                Err(e) => {
                    // Don't re-insert session on error - just exit
                    return Err(format!("Failed to receive message: {}", e));
                }
            };
            
            // Step 3: Re-insert session before processing message
            {
                let mut sessions_guard = sessions.write().await;
                sessions_guard.insert(addr, session);
            }
            // Lock released again

            // Step 4: Handle message (may need to update session state)
            match message {
                Message::KeepAlive => {
                    tracing::debug!("Received keep-alive from {}", addr);
                }

                Message::Choke => {
                    tracing::debug!("Choked by {}", addr);
                    let mut sessions_guard = sessions.write().await;
                    if let Some(session) = sessions_guard.get_mut(&addr) {
                        session.connection.peer_choking = true;
                    }
                }

                Message::Unchoke => {
                    tracing::info!("Unchoked by {}", addr);
                    {
                        let mut sessions_guard = sessions.write().await;
                        if let Some(session) = sessions_guard.get_mut(&addr) {
                            session.connection.peer_choking = false;
                        }
                    }

                    // Start requesting pieces
                    Self::request_pieces(addr, sessions.clone(), piece_manager.clone(), &peer_id)
                        .await?;
                    continue;
                }

                Message::Interested => {
                    let mut sessions_guard = sessions.write().await;
                    if let Some(session) = sessions_guard.get_mut(&addr) {
                        session.connection.peer_interested = true;
                    }
                }

                Message::NotInterested => {
                    let mut sessions_guard = sessions.write().await;
                    if let Some(session) = sessions_guard.get_mut(&addr) {
                        session.connection.peer_interested = false;
                    }
                }

                Message::Have { piece_index } => {
                    tracing::debug!("Peer {} has piece {}", addr, piece_index);
                    piece_manager.write().await.peer_has_piece(piece_index as usize);

                    let mut sessions_guard = sessions.write().await;
                    if let Some(session) = sessions_guard.get_mut(&addr) {
                        if let Some(ref mut bitfield) = session.peer_bitfield {
                            bitfield.set_piece(piece_index as usize);
                        }
                    }
                }

                Message::Bitfield { bitfield } => {
                    tracing::debug!("Received bitfield from {} ({} bytes)", addr, bitfield.len());
                    
                    let num_pieces = piece_manager.read().await.our_bitfield().num_pieces();
                    let peer_bf = Bitfield::from_bytes(bitfield, num_pieces);
                    
                    // Add peer to piece manager
                    piece_manager.write().await.add_peer(peer_id.clone(), &peer_bf);
                    
                    // Send interested if they have pieces we need
                    let our_bf = piece_manager.read().await.our_bitfield().clone();
                    let pieces_we_need = our_bf.pieces_to_request(&peer_bf);
                    
                    // Update session and send interested message
                    let send_interested = !pieces_we_need.is_empty();
                    {
                        let mut sessions_guard = sessions.write().await;
                        if let Some(session) = sessions_guard.get_mut(&addr) {
                            session.peer_bitfield = Some(peer_bf.clone());
                        }
                    }
                    
                    if send_interested {
                        tracing::info!("Peer {} has {} pieces we need, sending interested", addr, pieces_we_need.len());
                        
                        // Extract connection again to send message
                        let mut session = sessions.write().await.remove(&addr)
                            .ok_or_else(|| "Session not found".to_string())?;
                        
                        if let Err(e) = session.connection.send_interested().await {
                            return Err(format!("Failed to send interested: {}", e));
                        }
                        
                        sessions.write().await.insert(addr, session);
                    } else {
                        tracing::debug!("Peer {} has no pieces we need", addr);
                    }
                }

                Message::Request { index, begin, length } => {
                    tracing::debug!(
                        "Received request from {} for piece {} offset {} length {}",
                        addr, index, begin, length
                    );

                    // Check if we're choking this peer
                    let am_choking = {
                        let sessions_guard = sessions.read().await;
                        sessions_guard.get(&addr)
                            .map(|s| s.connection.am_choking)
                            .unwrap_or(true)
                    };
                    
                    if am_choking {
                        tracing::debug!("Ignoring request from {} (we are choking them)", addr);
                        continue;
                    }

                    // Check if we have this piece
                    let has_piece = {
                        let pm = piece_manager.read().await;
                        pm.has_piece(index as usize)
                    };

                    if !has_piece {
                        tracing::warn!(
                            "Peer {} requested piece {} that we don't have",
                            addr, index
                        );
                        continue;
                    }

                    // Read the piece data from disk and send
                    if let Err(e) = Self::handle_upload_request(
                        addr,
                        sessions.clone(),
                        disk_manager.clone(),
                        index as usize,
                        begin as usize,
                        length as usize,
                    )
                    .await
                    {
                        tracing::error!("Failed to handle upload request: {}", e);
                    }
                    continue;
                }

                Message::Piece {
                    index,
                    begin,
                    data,
                } => {
                    let block = BlockInfo::new(index as usize, begin as usize, data.len());
                    
                    // Mark request as complete and update stats
                    let (was_pending, can_request) = {
                        let mut sessions_guard = sessions.write().await;
                        if let Some(session) = sessions_guard.get_mut(&addr) {
                            let was_pending = session.remove_pending_request(&block);
                            if was_pending {
                                session.downloaded_bytes += data.len() as u64;
                            }
                            (was_pending, session.can_request())
                        } else {
                            (false, false)
                        }
                    };
                    
                    if !was_pending {
                        tracing::warn!("Received unrequested block from {}", addr);
                        continue;
                    }

                    tracing::debug!(
                        "Received piece {} offset {} ({} bytes) from {}",
                        index,
                        begin,
                        data.len(),
                        addr
                    );

                    // Write block to piece manager
                    let mut pm = piece_manager.write().await;
                    match pm.write_block(block, &data) {
                        Ok(is_complete) => {
                            if is_complete {
                                // Piece is complete - verify and write to disk
                                drop(pm);
                                Self::handle_piece_complete(
                                    index as usize,
                                    piece_manager.clone(),
                                    disk_manager.clone(),
                                )
                                .await?;
                                continue;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to write block: {}", e);
                        }
                    }

                    drop(pm);

                    // Request more pieces if we can
                    if can_request {
                        Self::request_pieces(addr, sessions.clone(), piece_manager.clone(), &peer_id)
                            .await?;
                        continue;
                    }
                }

                Message::Cancel { .. } => {
                    tracing::debug!("Received cancel from {}", addr);
                }
            }
        }
    }

    /// Request pieces from a peer
    async fn request_pieces(
        addr: SocketAddr,
        sessions: Arc<RwLock<HashMap<SocketAddr, PeerSession>>>,
        piece_manager: Arc<RwLock<PieceManager>>,
        peer_id: &str,
    ) -> Result<(), String> {
        let mut sessions_lock = sessions.write().await;
        let session = sessions_lock
            .get_mut(&addr)
            .ok_or_else(|| "Session not found".to_string())?;

        if !session.can_request() {
            return Ok(());
        }

        let peer_bitfield = match &session.peer_bitfield {
            Some(bf) => bf.clone(),
            None => return Ok(()), // Don't have bitfield yet
        };

        let mut pm = piece_manager.write().await;

        // Try to select a new piece
        if let Some((piece_idx, blocks)) = pm.select_next_piece(peer_id, &peer_bitfield) {
            tracing::debug!("Selected piece {} for download from {}", piece_idx, addr);

            // Request blocks
            let blocks_to_request: Vec<_> = blocks
                .into_iter()
                .take(MAX_PENDING_REQUESTS - session.pending_requests.len())
                .collect();

            for block in blocks_to_request {
                let request_msg = Message::Request {
                    index: block.piece_index as u32,
                    begin: block.offset as u32,
                    length: block.length as u32,
                };

                if let Err(e) = session.connection.send_message(&request_msg).await {
                    return Err(format!("Failed to send request: {}", e));
                }

                session.add_pending_request(block);
                tracing::debug!(
                    "Requested piece {} offset {} from {}",
                    block.piece_index,
                    block.offset,
                    addr
                );
            }
        } else {
            // Try to get missing blocks from in-progress pieces
            for piece_idx in pm.in_progress_pieces() {
                if let Some(missing_blocks) = pm.get_missing_blocks(piece_idx) {
                    if let Some(peer_bf) = &session.peer_bitfield {
                        if peer_bf.has_piece(piece_idx) {
                            let blocks_to_request: Vec<_> = missing_blocks
                                .into_iter()
                                .take(MAX_PENDING_REQUESTS - session.pending_requests.len())
                                .collect();

                            for block in blocks_to_request {
                                let request_msg = Message::Request {
                                    index: block.piece_index as u32,
                                    begin: block.offset as u32,
                                    length: block.length as u32,
                                };

                                if let Err(e) = session.connection.send_message(&request_msg).await {
                                    return Err(format!("Failed to send request: {}", e));
                                }

                                session.add_pending_request(block);
                            }

                            if session.pending_requests.len() >= MAX_PENDING_REQUESTS {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a completed piece
    async fn handle_piece_complete(
        piece_index: usize,
        piece_manager: Arc<RwLock<PieceManager>>,
        disk_manager: Arc<RwLock<DiskManager>>,
    ) -> Result<(), String> {
        tracing::info!("Piece {} completed, verifying...", piece_index);

        let mut pm = piece_manager.write().await;
        let piece_data = match pm.verify_piece(piece_index) {
            Ok(data) => {
                tracing::info!("Piece {} verified successfully!", piece_index);
                data
            }
            Err(e) => {
                tracing::error!("Piece {} verification failed: {}", piece_index, e);
                return Err(e);
            }
        };

        drop(pm);

        // Write to disk
        let mut dm = disk_manager.write().await;
        if let Err(e) = dm.write_piece(piece_index, piece_data).await {
            tracing::error!("Failed to write piece {} to disk: {}", piece_index, e);
            return Err(e);
        }

        tracing::info!("Piece {} written to disk successfully", piece_index);
        
        // Note: Broadcasting HAVE messages is handled per-peer in their loops
        // Each peer will be notified when they send/receive messages
        
        Ok(())
    }

    /// Handle an upload request from a peer
    async fn handle_upload_request(
        addr: SocketAddr,
        sessions: Arc<RwLock<HashMap<SocketAddr, PeerSession>>>,
        disk_manager: Arc<RwLock<DiskManager>>,
        piece_index: usize,
        offset: usize,
        length: usize,
    ) -> Result<(), String> {
        // Read piece from disk
        let dm = disk_manager.read().await;
        let piece_data = dm.read_piece(piece_index).await?;
        drop(dm);

        // Extract requested block
        if offset + length > piece_data.len() {
            return Err(format!(
                "Invalid block request: offset {} + length {} > piece size {}",
                offset,
                length,
                piece_data.len()
            ));
        }

        let block_data = piece_data[offset..offset + length].to_vec();

        // Send piece message
        let mut sessions_lock = sessions.write().await;
        let session = sessions_lock
            .get_mut(&addr)
            .ok_or_else(|| "Session not found".to_string())?;

        let piece_msg = Message::Piece {
            index: piece_index as u32,
            begin: offset as u32,
            data: block_data,
        };

        if let Err(e) = session.connection.send_message(&piece_msg).await {
            return Err(format!("Failed to send piece: {}", e));
        }

        session.uploaded_bytes += length as u64;

        tracing::debug!(
            "Uploaded piece {} offset {} ({} bytes) to {}",
            piece_index,
            offset,
            length,
            addr
        );

        Ok(())
    }

    /// Handle timed-out requests
    /// Handle timed-out block requests
    /// Removes timed-out blocks from pending and marks them for re-request
    async fn handle_pending_requests(&self) {
        let mut sessions = self.sessions.write().await;

        for (addr, session) in sessions.iter_mut() {
            let timed_out = session.get_timed_out_requests();
            
            for block in &timed_out {
                tracing::warn!(
                    "Request timed out for piece {} offset {} from {}, will re-request",
                    block.piece_index,
                    block.offset,
                    addr
                );
                session.remove_pending_request(block);
                
                // Mark block as failed in piece manager so it can be re-requested
                // This ensures the block will be picked up again by request_pieces
                let mut pm = self.piece_manager.write().await;
                if let Err(e) = pm.mark_block_failed(*block) {
                    tracing::debug!("Could not mark block as failed (piece may be complete): {}", e);
                }
                drop(pm);
            }
        }
    }

    /// Send keep-alive to all peers
    async fn send_keep_alives(&self) {
        let mut sessions = self.sessions.write().await;

        for (addr, session) in sessions.iter_mut() {
            if session.needs_keep_alive() {
                if let Err(e) = session.connection.send_keep_alive().await {
                    tracing::warn!("Failed to send keep-alive to {}: {}", addr, e);
                } else {
                    session.last_activity = Instant::now();
                }
            }
        }
    }

    /// Update statistics
    /// Update statistics
    async fn update_stats(&mut self) { // Changed to mutable self to be explicit, though we use interior mutability
        // We need write lock on sessions to update per-session stats
        let mut sessions = self.sessions.write().await;
        let mut connected_peers = 0;
        let mut total_downloaded = 0;
        let mut total_uploaded = 0;
        let mut download_speed = 0.0;
        let mut upload_speed = 0.0;
        
        // Update per-peer stats
        for session in sessions.values_mut() {
            connected_peers += 1;
            total_downloaded += session.downloaded_bytes;
            total_uploaded += session.uploaded_bytes;
            
            // Calculate speed (simple 1-second window since this runs every second)
            let diff_down = session.downloaded_bytes.saturating_sub(session.last_downloaded_bytes);
            let diff_up = session.uploaded_bytes.saturating_sub(session.last_uploaded_bytes);
            
            session.download_speed = diff_down as f64;
            session.upload_speed = diff_up as f64;
            
            session.last_downloaded_bytes = session.downloaded_bytes;
            session.last_uploaded_bytes = session.uploaded_bytes;
            
            download_speed += session.download_speed;
            upload_speed += session.upload_speed;
        }
        
        let mut stats = self.stats.write().await;
        stats.connected_peers = connected_peers;
        stats.total_downloaded = total_downloaded;
        stats.total_uploaded = total_uploaded;
        stats.download_speed = download_speed;
        stats.upload_speed = upload_speed;
    }

    /// Update choking algorithm
    /// Unchokes the best uploaders and chokes the rest
    async fn update_choking(&self) {
        let mut sessions = self.sessions.write().await;

        // Get peers sorted by download rate (how much they've sent to us)
        let mut peer_stats: Vec<(SocketAddr, u64)> = sessions
            .iter()
            .filter(|(_, s)| s.connection.peer_interested)
            .map(|(addr, s)| (*addr, s.downloaded_bytes))
            .collect();

        // Sort by download amount (best uploaders first)
        peer_stats.sort_by(|a, b| b.1.cmp(&a.1));

        // Unchoke top N peers
        let mut unchoked = 0;
        for (addr, _) in &peer_stats {
            if unchoked < NUM_UNCHOKED {
                if let Some(session) = sessions.get_mut(addr) {
                    if session.connection.am_choking {
                        tracing::debug!("Unchoking peer {} (good uploader)", addr);
                        if session.connection.send_unchoke().await.is_err() {
                            tracing::warn!("Failed to send unchoke to {}", addr);
                        }
                    }
                }
                unchoked += 1;
            } else {
                // Choke the rest
                if let Some(session) = sessions.get_mut(addr) {
                    if !session.connection.am_choking {
                        tracing::debug!("Choking peer {}", addr);
                        if session.connection.send_choke().await.is_err() {
                            tracing::warn!("Failed to send choke to {}", addr);
                        }
                    }
                }
            }
        }
    }

    /// Optimistically unchoke a random peer
    /// This gives new peers a chance to show their upload rate
    async fn optimistic_unchoke(&self) {
        use rand::seq::SliceRandom;
        
        let mut sessions = self.sessions.write().await;

        // Find choked peers that are interested
        let choked_interested: Vec<SocketAddr> = sessions
            .iter()
            .filter(|(_, s)| s.connection.am_choking && s.connection.peer_interested)
            .map(|(addr, _)| *addr)
            .collect();

        if choked_interested.is_empty() {
            return;
        }

        // Pick a random one (scope the RNG to avoid holding it across await)
        let chosen_addr = {
            let mut rng = rand::thread_rng();
            choked_interested.choose(&mut rng).copied()
        };

        if let Some(addr) = chosen_addr {
            if let Some(session) = sessions.get_mut(&addr) {
                tracing::info!("Optimistically unchoking peer {}", addr);
                if let Err(e) = session.connection.send_unchoke().await {
                    tracing::warn!("Failed to optimistically unchoke {}: {}", addr, e);
                }
            }
        }
    }

    /// Broadcast HAVE message to all connected peers
    async fn broadcast_have(&self, piece_index: usize) {
        let mut sessions = self.sessions.write().await;

        let have_msg = Message::Have {
            piece_index: piece_index as u32,
        };

        for (addr, session) in sessions.iter_mut() {
            if let Err(e) = session.connection.send_message(&have_msg).await {
                tracing::warn!("Failed to send HAVE to {}: {}", addr, e);
            } else {
                tracing::debug!("Sent HAVE {} to {}", piece_index, addr);
            }
        }
    }

    /// Get list of all connected peers with their info
    pub async fn get_peer_list(&self) -> Vec<super::PeerInfo> {
        let sessions = self.sessions.read().await;
        
        sessions.iter().map(|(addr, session)| {
            let client = parse_peer_id(session.connection.peer_id);
            let flags = calculate_flags(session);
            let progress = calculate_progress(session);
            
            super::PeerInfo {
                ip: addr.ip().to_string(),
                port: addr.port(),
                client,
                flags,
                progress,
                download_speed: session.download_speed as u64,
                upload_speed: session.upload_speed as u64,
                downloaded: session.downloaded_bytes,
                uploaded: session.uploaded_bytes,
            }
        }).collect()
    }
}

/// Parse peer ID to detect client name
fn parse_peer_id(peer_id: Option<[u8; 20]>) -> String {
    let peer_id = match peer_id {
        Some(id) => id,
        None => return "Unknown".to_string(),
    };
    
    // Try to convert to string for Azureus-style encoding
    if let Ok(s) = std::str::from_utf8(&peer_id[0..8]) {
        if s.starts_with('-') && s.len() >= 7 {
            // Azureus-style: -AB1234-
            let client_code = &s[1..3];
            let version = &s[3..7];
            
            let client_name = match client_code {
                "qB" => "qBittorrent",
                "TR" => "Transmission",
                "DE" => "Deluge",
                "UT" => "µTorrent",
                "LT" => "libtorrent",
                "Az" => "Azureus",
                "BI" => "BiglyBT",
                "SD" => "SeedCore",
                _ => return format!("{} {}", client_code, version),
            };
            
            // Parse version: "4500" -> "4.5.0"
            if version.len() == 4 {
                if let (Ok(major), Ok(minor), Ok(patch)) = (
                    version[0..1].parse::<u8>(),
                    version[1..2].parse::<u8>(),
                    version[2..4].parse::<u8>(),
                ) {
                    return format!("{} {}.{}.{}", client_name, major, minor, patch);
                }
            }
            
            return format!("{} {}", client_name, version);
        }
    }
    
    // Shadow-style: first byte is ASCII char
    if peer_id[0].is_ascii_alphabetic() {
        let client_char = peer_id[0] as char;
        let client_name = match client_char {
            'M' => "BitTorrent",
            'A' => "ABC",
            'O' => "Osprey",
            'Q' => "BTQueue",
            'R' => "Tribler",
            'S' => "Shadow",
            'T' => "BitTornado",
            _ => return "Unknown".to_string(),
        };
        
        // Try to extract version from next bytes
        if peer_id[1].is_ascii_digit() && peer_id[2].is_ascii_digit() && peer_id[3].is_ascii_digit() {
            let version = format!(
                "{}.{}.{}",
                (peer_id[1] as char),
                (peer_id[2] as char),
                (peer_id[3] as char)
            );
            return format!("{} {}", client_name, version);
        }
        
        return client_name.to_string();
    }
    
    "Unknown".to_string()
}

/// Calculate connection flags for a peer
fn calculate_flags(session: &PeerSession) -> String {
    let mut flags = String::new();
    
    // D = Downloading from peer (we are receiving data)
    if !session.connection.peer_choking && session.connection.am_interested {
        flags.push('D');
    }
    
    // U = Uploading to peer (we are sending data)
    if !session.connection.am_choking && session.connection.peer_interested {
        flags.push('U');
    }
    
    // I = We are interested in peer
    if session.connection.am_interested {
        flags.push('I');
    }
    
    // C = We are choking peer
    if session.connection.am_choking {
        flags.push('C');
    }
    
    // O = Optimistic unchoke
    // This would need to be tracked separately in PeerSession
    // For now, we'll skip this flag
    
    // E = Encrypted connection
    // Not yet implemented
    
    // S = Snubbed (peer hasn't sent data in a while)
    // Not yet implemented
    
    flags
}

/// Calculate peer's download progress from their bitfield
fn calculate_progress(session: &PeerSession) -> f64 {
    match &session.peer_bitfield {
        Some(bitfield) => bitfield.completion() * 100.0,
        None => 0.0,
    }
}
