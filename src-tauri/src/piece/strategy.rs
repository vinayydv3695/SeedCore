/// Piece selection strategies for downloading torrents
/// Different strategies optimize for different goals (speed, availability, streaming)
use super::bitfield::Bitfield;
use rand::seq::SliceRandom;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionStrategy {
    /// Select rarest pieces first (standard BitTorrent strategy)
    RarestFirst,
    /// Select pieces sequentially (good for streaming/previewing)
    Sequential,
    /// Random selection (useful for initial pieces)
    Random,
    /// Endgame mode: request all remaining pieces from all peers
    Endgame,
}

/// Manages piece selection based on strategy
pub struct PieceSelector {
    strategy: SelectionStrategy,
    /// Track piece rarity across all connected peers
    /// piece_index -> count of peers that have it
    piece_availability: HashMap<usize, usize>,
}

impl PieceSelector {
    pub fn new(strategy: SelectionStrategy) -> Self {
        Self {
            strategy,
            piece_availability: HashMap::new(),
        }
    }

    /// Update piece availability based on a peer's bitfield
    pub fn update_peer_availability(&mut self, peer_bitfield: &Bitfield, increment: bool) {
        for piece_idx in peer_bitfield.available_pieces() {
            let count = self.piece_availability.entry(piece_idx).or_insert(0);
            if increment {
                *count += 1;
            } else if *count > 0 {
                *count -= 1;
            }
        }
    }

    /// Add a peer's bitfield to availability tracking
    pub fn add_peer(&mut self, peer_bitfield: &Bitfield) {
        self.update_peer_availability(peer_bitfield, true);
    }

    /// Remove a peer's bitfield from availability tracking
    pub fn remove_peer(&mut self, peer_bitfield: &Bitfield) {
        self.update_peer_availability(peer_bitfield, false);
    }

    /// Mark a piece as available from a peer (when receiving HAVE message)
    pub fn mark_piece_available(&mut self, piece_idx: usize) {
        *self.piece_availability.entry(piece_idx).or_insert(0) += 1;
    }

    /// Select next piece to download from available pieces
    /// Returns None if no suitable piece is available
    pub fn select_piece(
        &self,
        our_bitfield: &Bitfield,
        peer_bitfield: &Bitfield,
        pending_pieces: &[usize],
    ) -> Option<usize> {
        // Get pieces we need that the peer has
        let mut candidates = our_bitfield.pieces_to_request(peer_bitfield);

        // Filter out pieces we're already requesting
        candidates.retain(|piece| !pending_pieces.contains(piece));

        if candidates.is_empty() {
            return None;
        }

        match self.strategy {
            SelectionStrategy::RarestFirst => self.select_rarest(&candidates),
            SelectionStrategy::Sequential => self.select_sequential(&candidates),
            SelectionStrategy::Random => self.select_random(&candidates),
            SelectionStrategy::Endgame => self.select_endgame(&candidates),
        }
    }

    /// Select the rarest piece (fewest peers have it)
    fn select_rarest(&self, candidates: &[usize]) -> Option<usize> {
        candidates
            .iter()
            .min_by_key(|&&piece_idx| {
                self.piece_availability
                    .get(&piece_idx)
                    .copied()
                    .unwrap_or(0)
            })
            .copied()
    }

    /// Select the piece with lowest index (sequential download)
    fn select_sequential(&self, candidates: &[usize]) -> Option<usize> {
        candidates.iter().min().copied()
    }

    /// Select a random piece
    fn select_random(&self, candidates: &[usize]) -> Option<usize> {
        let mut rng = rand::thread_rng();
        candidates.choose(&mut rng).copied()
    }

    /// In endgame mode, request any remaining piece
    /// (In practice, endgame requests all pieces from all peers)
    fn select_endgame(&self, candidates: &[usize]) -> Option<usize> {
        candidates.first().copied()
    }

    /// Check if we should enter endgame mode
    /// Typically when we have very few pieces left (< 5% or < 10 pieces)
    pub fn should_enter_endgame(&self, our_bitfield: &Bitfield) -> bool {
        let missing = our_bitfield.missing_pieces();
        let total = our_bitfield.num_pieces();

        // Enter endgame if missing < 10 pieces or < 5% remaining
        missing.len() < 10 || (missing.len() as f64 / total as f64) < 0.05
    }

    /// Get the current strategy
    pub fn strategy(&self) -> SelectionStrategy {
        self.strategy
    }

    /// Set a new strategy
    pub fn set_strategy(&mut self, strategy: SelectionStrategy) {
        self.strategy = strategy;
    }

    /// Get piece availability count for a specific piece
    pub fn get_availability(&self, piece_idx: usize) -> usize {
        self.piece_availability
            .get(&piece_idx)
            .copied()
            .unwrap_or(0)
    }

    /// Get average piece availability across all pieces
    pub fn average_availability(&self) -> f64 {
        if self.piece_availability.is_empty() {
            return 0.0;
        }
        let sum: usize = self.piece_availability.values().sum();
        sum as f64 / self.piece_availability.len() as f64
    }

    /// Get piece availability for all pieces (for UI display)
    /// Returns a vector where each index represents a piece and the value is the number of peers that have it
    pub fn get_piece_availability(&self, num_pieces: usize) -> Vec<usize> {
        (0..num_pieces).map(|i| self.get_availability(i)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rarest_first_selection() {
        let mut selector = PieceSelector::new(SelectionStrategy::RarestFirst);

        // Create peer bitfields with different piece availability
        let mut peer1 = Bitfield::new(10);
        peer1.set_piece(0);
        peer1.set_piece(1);
        peer1.set_piece(2);

        let mut peer2 = Bitfield::new(10);
        peer2.set_piece(1); // piece 1 is common
        peer2.set_piece(3);

        let mut peer3 = Bitfield::new(10);
        peer3.set_piece(1); // piece 1 is very common
        peer3.set_piece(2);

        selector.add_peer(&peer1);
        selector.add_peer(&peer2);
        selector.add_peer(&peer3);

        // piece 0: 1 peer, piece 1: 3 peers, piece 2: 2 peers, piece 3: 1 peer
        // Rarest should be 0 or 3 (both have 1 peer)

        let our_bf = Bitfield::new(10);
        let peer_bf = peer1; // Has pieces 0, 1, 2
        let pending = vec![];

        let selected = selector.select_piece(&our_bf, &peer_bf, &pending);
        assert!(selected == Some(0) || selected == Some(2));
    }

    #[test]
    fn test_sequential_selection() {
        let selector = PieceSelector::new(SelectionStrategy::Sequential);

        let our_bf = Bitfield::new(10);
        let mut peer_bf = Bitfield::new(10);
        peer_bf.set_piece(3);
        peer_bf.set_piece(5);
        peer_bf.set_piece(1);

        let pending = vec![];
        let selected = selector.select_piece(&our_bf, &peer_bf, &pending);

        // Should select lowest index
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn test_random_selection() {
        let selector = PieceSelector::new(SelectionStrategy::Random);

        let our_bf = Bitfield::new(10);
        let mut peer_bf = Bitfield::new(10);
        peer_bf.set_piece(2);
        peer_bf.set_piece(5);
        peer_bf.set_piece(8);

        let pending = vec![];
        let selected = selector.select_piece(&our_bf, &peer_bf, &pending);

        // Should be one of the available pieces
        assert!(selected == Some(2) || selected == Some(5) || selected == Some(8));
    }

    #[test]
    fn test_pending_pieces_exclusion() {
        let selector = PieceSelector::new(SelectionStrategy::Sequential);

        let our_bf = Bitfield::new(10);
        let mut peer_bf = Bitfield::new(10);
        peer_bf.set_piece(1);
        peer_bf.set_piece(2);
        peer_bf.set_piece(3);

        // Piece 1 is already being requested
        let pending = vec![1];
        let selected = selector.select_piece(&our_bf, &peer_bf, &pending);

        // Should skip piece 1 and select piece 2
        assert_eq!(selected, Some(2));
    }

    #[test]
    fn test_no_available_pieces() {
        let selector = PieceSelector::new(SelectionStrategy::RarestFirst);

        let our_bf = Bitfield::complete(10); // We have everything
        let peer_bf = Bitfield::complete(10);
        let pending = vec![];

        let selected = selector.select_piece(&our_bf, &peer_bf, &pending);
        assert_eq!(selected, None);
    }

    #[test]
    fn test_endgame_mode_detection() {
        let selector = PieceSelector::new(SelectionStrategy::RarestFirst);

        let mut bf = Bitfield::new(100);
        // Complete most pieces
        for i in 0..97 {
            bf.set_piece(i);
        }

        // 3 pieces left (< 5%)
        assert!(selector.should_enter_endgame(&bf));

        // Add more missing pieces
        let bf2 = Bitfield::new(100); // 0% complete
        assert!(!selector.should_enter_endgame(&bf2));
    }

    #[test]
    fn test_availability_tracking() {
        let mut selector = PieceSelector::new(SelectionStrategy::RarestFirst);

        let mut peer1 = Bitfield::new(10);
        peer1.set_piece(0);
        peer1.set_piece(1);

        let mut peer2 = Bitfield::new(10);
        peer2.set_piece(1);
        peer2.set_piece(2);

        selector.add_peer(&peer1);
        selector.add_peer(&peer2);

        assert_eq!(selector.get_availability(0), 1);
        assert_eq!(selector.get_availability(1), 2);
        assert_eq!(selector.get_availability(2), 1);
        assert_eq!(selector.get_availability(9), 0);

        // Remove peer1
        selector.remove_peer(&peer1);
        assert_eq!(selector.get_availability(0), 0);
        assert_eq!(selector.get_availability(1), 1);
    }

    #[test]
    fn test_mark_piece_available() {
        let mut selector = PieceSelector::new(SelectionStrategy::RarestFirst);

        assert_eq!(selector.get_availability(5), 0);

        selector.mark_piece_available(5);
        assert_eq!(selector.get_availability(5), 1);

        selector.mark_piece_available(5);
        assert_eq!(selector.get_availability(5), 2);
    }
}
