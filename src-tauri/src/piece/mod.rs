/// Piece manager for coordinating piece downloads and verification
pub mod bitfield;
pub mod strategy;

pub use bitfield::Bitfield;
pub use strategy::{PieceSelector, SelectionStrategy};

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::collections::{HashMap, HashSet};

/// Pieces information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiecesInfo {
    /// Total number of pieces
    pub total_pieces: usize,
    /// Number of pieces we have
    pub pieces_have: usize,
    /// Number of pieces currently downloading
    pub pieces_downloading: usize,
    /// Bitfield state (0=missing, 1=have, 2=downloading)
    pub bitfield: Vec<u8>,
    /// Piece availability (number of peers that have each piece)
    pub availability: Vec<usize>,
}

/// Standard block size for piece requests (16KB)
pub const BLOCK_SIZE: usize = 16384;

/// Represents a block within a piece
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockInfo {
    pub piece_index: usize,
    pub offset: usize,
    pub length: usize,
}

impl BlockInfo {
    pub fn new(piece_index: usize, offset: usize, length: usize) -> Self {
        Self {
            piece_index,
            offset,
            length,
        }
    }
}

/// State of a piece being downloaded
#[derive(Debug, Clone)]
struct PieceState {
    /// Data buffer for this piece
    data: Vec<u8>,
    /// Which blocks have been downloaded
    downloaded_blocks: HashSet<usize>, // block offset
    /// Total blocks in this piece
    total_blocks: usize,
}

impl PieceState {
    fn new(piece_length: usize) -> Self {
        let total_blocks = (piece_length + BLOCK_SIZE - 1) / BLOCK_SIZE;
        Self {
            data: vec![0; piece_length],
            downloaded_blocks: HashSet::new(),
            total_blocks,
        }
    }

    fn write_block(&mut self, offset: usize, data: &[u8]) {
        let end = offset + data.len();
        if end <= self.data.len() {
            self.data[offset..end].copy_from_slice(data);
            self.downloaded_blocks.insert(offset);
        }
    }

    fn is_complete(&self) -> bool {
        self.downloaded_blocks.len() == self.total_blocks
    }

    fn missing_blocks(&self) -> Vec<usize> {
        (0..self.total_blocks)
            .map(|i| i * BLOCK_SIZE)
            .filter(|offset| !self.downloaded_blocks.contains(offset))
            .collect()
    }
}

/// Manages all piece-related operations for a torrent
pub struct PieceManager {
    /// Our bitfield tracking which pieces we have
    our_bitfield: Bitfield,
    /// Piece selection strategy
    selector: PieceSelector,
    /// SHA1 hashes for each piece (from metainfo)
    piece_hashes: Vec<Vec<u8>>,
    /// Length of each piece in bytes
    piece_length: usize,
    /// Length of the last piece (may be shorter)
    last_piece_length: usize,
    /// Total number of pieces
    num_pieces: usize,
    /// Pieces currently being downloaded
    in_progress: HashMap<usize, PieceState>,
    /// Pieces that have been verified and are complete
    verified_pieces: HashSet<usize>,
    /// Track which pieces we've requested from which peers
    /// peer_id -> set of piece indices
    peer_requests: HashMap<String, HashSet<usize>>,
}

impl PieceManager {
    pub fn new(
        num_pieces: usize,
        piece_length: usize,
        last_piece_length: usize,
        piece_hashes: Vec<Vec<u8>>,
        strategy: SelectionStrategy,
    ) -> Self {
        assert_eq!(
            piece_hashes.len(),
            num_pieces,
            "Number of piece hashes must match number of pieces"
        );

        Self {
            our_bitfield: Bitfield::new(num_pieces),
            selector: PieceSelector::new(strategy),
            piece_hashes,
            piece_length,
            last_piece_length,
            num_pieces,
            in_progress: HashMap::new(),
            verified_pieces: HashSet::new(),
            peer_requests: HashMap::new(),
        }
    }

    /// Add a peer's bitfield to tracking
    pub fn add_peer(&mut self, peer_id: String, peer_bitfield: &Bitfield) {
        self.selector.add_peer(peer_bitfield);
        self.peer_requests.insert(peer_id, HashSet::new());
    }

    /// Remove a peer from tracking
    pub fn remove_peer(&mut self, peer_id: &str, peer_bitfield: &Bitfield) {
        self.selector.remove_peer(peer_bitfield);
        self.peer_requests.remove(peer_id);
    }

    /// Update when peer announces they have a new piece
    pub fn peer_has_piece(&mut self, piece_index: usize) {
        self.selector.mark_piece_available(piece_index);
    }

    /// Get our current bitfield
    pub fn our_bitfield(&self) -> &Bitfield {
        &self.our_bitfield
    }

    /// Check if we have a specific piece
    pub fn has_piece(&self, piece_index: usize) -> bool {
        self.our_bitfield.has_piece(piece_index)
    }

    /// Get download completion percentage
    pub fn completion(&self) -> f64 {
        self.our_bitfield.completion()
    }

    /// Check if download is complete
    pub fn is_complete(&self) -> bool {
        self.our_bitfield.is_complete()
    }

    /// Get the length of a specific piece
    pub fn piece_len(&self, piece_index: usize) -> usize {
        if piece_index == self.num_pieces - 1 {
            self.last_piece_length
        } else {
            self.piece_length
        }
    }

    /// Get list of blocks to request for a piece
    pub fn get_blocks_for_piece(&self, piece_index: usize) -> Vec<BlockInfo> {
        let piece_len = self.piece_len(piece_index);
        let num_blocks = (piece_len + BLOCK_SIZE - 1) / BLOCK_SIZE;

        (0..num_blocks)
            .map(|i| {
                let offset = i * BLOCK_SIZE;
                let length = std::cmp::min(BLOCK_SIZE, piece_len - offset);
                BlockInfo::new(piece_index, offset, length)
            })
            .collect()
    }

    /// Select next piece to download from a peer
    /// Returns piece index and list of blocks to request
    pub fn select_next_piece(
        &mut self,
        peer_id: &str,
        peer_bitfield: &Bitfield,
    ) -> Option<(usize, Vec<BlockInfo>)> {
        let pending: Vec<usize> = self.in_progress.keys().copied().collect();

        let piece_index =
            self.selector
                .select_piece(&self.our_bitfield, peer_bitfield, &pending)?;

        // Initialize piece state if not already in progress
        if !self.in_progress.contains_key(&piece_index) {
            let piece_len = self.piece_len(piece_index);
            self.in_progress
                .insert(piece_index, PieceState::new(piece_len));
        }

        // Track that this peer is working on this piece
        if let Some(peer_pieces) = self.peer_requests.get_mut(peer_id) {
            peer_pieces.insert(piece_index);
        }

        let blocks = self.get_blocks_for_piece(piece_index);
        Some((piece_index, blocks))
    }

    /// Get missing blocks for a piece that's in progress
    pub fn get_missing_blocks(&self, piece_index: usize) -> Option<Vec<BlockInfo>> {
        let state = self.in_progress.get(&piece_index)?;
        let missing_offsets = state.missing_blocks();

        let blocks: Vec<BlockInfo> = missing_offsets
            .into_iter()
            .map(|offset| {
                let piece_len = self.piece_len(piece_index);
                let length = std::cmp::min(BLOCK_SIZE, piece_len - offset);
                BlockInfo::new(piece_index, offset, length)
            })
            .collect();

        Some(blocks)
    }

    /// Write received block data to piece buffer
    pub fn write_block(&mut self, block: BlockInfo, data: &[u8]) -> Result<bool, String> {
        if block.length != data.len() {
            return Err(format!(
                "Block data length mismatch: expected {}, got {}",
                block.length,
                data.len()
            ));
        }

        let state = self
            .in_progress
            .get_mut(&block.piece_index)
            .ok_or_else(|| format!("Piece {} not in progress", block.piece_index))?;

        state.write_block(block.offset, data);

        // Check if piece is now complete
        Ok(state.is_complete())
    }

    /// Verify and finalize a completed piece
    /// Returns the piece data if verification succeeds
    pub fn verify_piece(&mut self, piece_index: usize) -> Result<Vec<u8>, String> {
        let state = self
            .in_progress
            .remove(&piece_index)
            .ok_or_else(|| format!("Piece {} not in progress", piece_index))?;

        if !state.is_complete() {
            // Put it back if not complete
            self.in_progress.insert(piece_index, state);
            return Err("Piece not complete".to_string());
        }

        // Calculate SHA1 hash
        let mut hasher = Sha1::new();
        hasher.update(&state.data);
        let hash = hasher.finalize().to_vec();

        // Compare with expected hash
        let expected_hash = &self.piece_hashes[piece_index];
        if hash != *expected_hash {
            // Hash mismatch - put piece back for re-download
            self.in_progress
                .insert(piece_index, PieceState::new(state.data.len()));
            return Err(format!(
                "Piece {} hash verification failed: expected {:?}, got {:?}",
                piece_index, expected_hash, hash
            ));
        }

        // Mark piece as verified and available
        self.our_bitfield.set_piece(piece_index);
        self.verified_pieces.insert(piece_index);

        // Remove from peer request tracking
        for peer_pieces in self.peer_requests.values_mut() {
            peer_pieces.remove(&piece_index);
        }

        Ok(state.data)
    }

    /// Cancel a piece download (e.g., if peer disconnects)
    pub fn cancel_piece(&mut self, piece_index: usize) {
        self.in_progress.remove(&piece_index);
    }

    /// Get statistics about current download state
    pub fn stats(&self) -> PieceStats {
        PieceStats {
            total_pieces: self.num_pieces,
            completed_pieces: self.our_bitfield.count_pieces(),
            in_progress_pieces: self.in_progress.len(),
            verified_pieces: self.verified_pieces.len(),
            completion_percent: self.our_bitfield.completion() * 100.0,
        }
    }

    /// Check if we should enter endgame mode
    pub fn should_enter_endgame(&self) -> bool {
        self.selector.should_enter_endgame(&self.our_bitfield)
    }

    /// Change selection strategy
    pub fn set_strategy(&mut self, strategy: SelectionStrategy) {
        self.selector.set_strategy(strategy);
    }

    /// Get pieces that are currently being downloaded
    pub fn in_progress_pieces(&self) -> Vec<usize> {
        self.in_progress.keys().copied().collect()
    }

    /// Get pieces information for UI display
    pub fn get_pieces_info(&self) -> PiecesInfo {
        let total = self.num_pieces;
        let have = self.our_bitfield.count_pieces();
        let downloading = self.in_progress.len();

        // Build bitfield state array (0=missing, 1=have, 2=downloading)
        let mut bitfield_state = vec![0u8; total];
        for i in 0..total {
            if self.our_bitfield.has_piece(i) {
                bitfield_state[i] = 1; // Have
            } else if self.in_progress.contains_key(&i) {
                bitfield_state[i] = 2; // Downloading
            }
            // else 0 = missing (default)
        }

        // Calculate piece availability from selector
        // This shows how many peers have each piece
        let availability = self.selector.get_piece_availability(total);

        PiecesInfo {
            total_pieces: total,
            pieces_have: have,
            pieces_downloading: downloading,
            bitfield: bitfield_state,
            availability,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PieceStats {
    pub total_pieces: usize,
    pub completed_pieces: usize,
    pub in_progress_pieces: usize,
    pub verified_pieces: usize,
    pub completion_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_hashes(num_pieces: usize) -> Vec<Vec<u8>> {
        (0..num_pieces)
            .map(|i| {
                let mut hasher = Sha1::new();
                hasher.update(format!("piece_{}", i).as_bytes());
                hasher.finalize().to_vec()
            })
            .collect()
    }

    #[test]
    fn test_piece_manager_creation() {
        let hashes = create_test_hashes(10);
        let pm = PieceManager::new(10, 16384, 8192, hashes, SelectionStrategy::RarestFirst);

        assert_eq!(pm.num_pieces, 10);
        assert_eq!(pm.piece_length, 16384);
        assert_eq!(pm.last_piece_length, 8192);
        assert!(!pm.is_complete());
        assert_eq!(pm.completion(), 0.0);
    }

    #[test]
    fn test_piece_length() {
        let hashes = create_test_hashes(10);
        let pm = PieceManager::new(10, 16384, 8192, hashes, SelectionStrategy::RarestFirst);

        assert_eq!(pm.piece_len(0), 16384);
        assert_eq!(pm.piece_len(5), 16384);
        assert_eq!(pm.piece_len(9), 8192); // Last piece
    }

    #[test]
    fn test_get_blocks_for_piece() {
        let hashes = create_test_hashes(3);
        let pm = PieceManager::new(3, 32768, 16384, hashes, SelectionStrategy::RarestFirst);

        // Piece 0: 32768 bytes = 2 blocks of 16384 bytes each
        let blocks = pm.get_blocks_for_piece(0);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].offset, 0);
        assert_eq!(blocks[0].length, 16384);
        assert_eq!(blocks[1].offset, 16384);
        assert_eq!(blocks[1].length, 16384);

        // Last piece: 16384 bytes = 1 block
        let blocks = pm.get_blocks_for_piece(2);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].length, 16384);
    }

    #[test]
    fn test_select_next_piece() {
        let hashes = create_test_hashes(5);
        let mut pm = PieceManager::new(5, 16384, 16384, hashes, SelectionStrategy::Sequential);

        let mut peer_bf = Bitfield::new(5);
        peer_bf.set_piece(0);
        peer_bf.set_piece(2);
        peer_bf.set_piece(4);

        pm.add_peer("peer1".to_string(), &peer_bf);

        let result = pm.select_next_piece("peer1", &peer_bf);
        assert!(result.is_some());

        let (piece_idx, blocks) = result.unwrap();
        assert_eq!(piece_idx, 0); // Sequential should pick 0 first
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_write_and_verify_piece() {
        let piece_data = b"Hello, this is a test piece!";
        let mut hasher = Sha1::new();
        hasher.update(piece_data);
        let correct_hash = hasher.finalize().to_vec();

        let hashes = vec![correct_hash];
        let mut pm = PieceManager::new(
            1,
            piece_data.len(),
            piece_data.len(),
            hashes,
            SelectionStrategy::RarestFirst,
        );

        // Start downloading piece 0
        let mut peer_bf = Bitfield::new(1);
        peer_bf.set_piece(0);
        pm.add_peer("peer1".to_string(), &peer_bf);

        let (piece_idx, blocks) = pm.select_next_piece("peer1", &peer_bf).unwrap();
        assert_eq!(piece_idx, 0);

        // Write the data
        for block in blocks {
            let data = &piece_data[block.offset..block.offset + block.length];
            let is_complete = pm.write_block(block, data).unwrap();
            if block.offset + block.length == piece_data.len() {
                assert!(is_complete);
            }
        }

        // Verify piece
        let verified_data = pm.verify_piece(0).unwrap();
        assert_eq!(verified_data, piece_data);
        assert!(pm.has_piece(0));
        assert_eq!(pm.completion(), 1.0);
    }

    #[test]
    fn test_verify_piece_hash_mismatch() {
        let correct_data = b"correct data";
        let mut hasher = Sha1::new();
        hasher.update(correct_data);
        let correct_hash = hasher.finalize().to_vec();

        let hashes = vec![correct_hash];
        let mut pm = PieceManager::new(1, 32, 32, hashes, SelectionStrategy::RarestFirst);

        let mut peer_bf = Bitfield::new(1);
        peer_bf.set_piece(0);
        pm.add_peer("peer1".to_string(), &peer_bf);

        pm.select_next_piece("peer1", &peer_bf);

        // Write wrong data
        let wrong_data = b"wrong data!!";
        let block = BlockInfo::new(0, 0, wrong_data.len());
        pm.write_block(block, wrong_data).unwrap();

        // Verification should fail
        let result = pm.verify_piece(0);
        assert!(result.is_err());
        assert!(!pm.has_piece(0));
    }

    #[test]
    fn test_piece_stats() {
        let hashes = create_test_hashes(10);
        let mut pm = PieceManager::new(10, 16384, 16384, hashes, SelectionStrategy::RarestFirst);

        let stats = pm.stats();
        assert_eq!(stats.total_pieces, 10);
        assert_eq!(stats.completed_pieces, 0);
        assert_eq!(stats.completion_percent, 0.0);

        // Manually mark some pieces as complete for testing
        pm.our_bitfield.set_piece(0);
        pm.our_bitfield.set_piece(1);
        pm.verified_pieces.insert(0);
        pm.verified_pieces.insert(1);

        let stats = pm.stats();
        assert_eq!(stats.completed_pieces, 2);
        assert_eq!(stats.verified_pieces, 2);
        assert_eq!(stats.completion_percent, 20.0);
    }
}
