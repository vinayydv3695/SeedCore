/// Bitfield implementation for tracking piece availability
/// Each bit represents whether we have a specific piece (1) or not (0)
use std::fmt;

#[derive(Clone, Debug)]
pub struct Bitfield {
    /// Bytes storing the bitfield, each byte represents 8 pieces
    bytes: Vec<u8>,
    /// Total number of pieces (may not be multiple of 8)
    num_pieces: usize,
}

impl Bitfield {
    /// Create a new empty bitfield for the given number of pieces
    pub fn new(num_pieces: usize) -> Self {
        let num_bytes = (num_pieces + 7) / 8; // Round up to nearest byte
        Self {
            bytes: vec![0; num_bytes],
            num_pieces,
        }
    }

    /// Create a bitfield from raw bytes (e.g., from a peer's bitfield message)
    pub fn from_bytes(bytes: Vec<u8>, num_pieces: usize) -> Self {
        let expected_bytes = (num_pieces + 7) / 8;
        assert!(
            bytes.len() >= expected_bytes,
            "Bitfield bytes length {} is less than expected {}",
            bytes.len(),
            expected_bytes
        );
        Self { bytes, num_pieces }
    }

    /// Create a complete bitfield (all pieces available)
    pub fn complete(num_pieces: usize) -> Self {
        let mut bitfield = Self::new(num_pieces);
        for i in 0..num_pieces {
            bitfield.set_piece(i);
        }
        bitfield
    }

    /// Check if we have a specific piece
    pub fn has_piece(&self, index: usize) -> bool {
        if index >= self.num_pieces {
            return false;
        }
        let byte_index = index / 8;
        let bit_offset = 7 - (index % 8); // MSB first
        (self.bytes[byte_index] & (1 << bit_offset)) != 0
    }

    /// Mark a piece as available (set bit to 1)
    pub fn set_piece(&mut self, index: usize) {
        if index >= self.num_pieces {
            return;
        }
        let byte_index = index / 8;
        let bit_offset = 7 - (index % 8); // MSB first
        self.bytes[byte_index] |= 1 << bit_offset;
    }

    /// Mark a piece as unavailable (set bit to 0)
    pub fn clear_piece(&mut self, index: usize) {
        if index >= self.num_pieces {
            return;
        }
        let byte_index = index / 8;
        let bit_offset = 7 - (index % 8); // MSB first
        self.bytes[byte_index] &= !(1 << bit_offset);
    }

    /// Get the total number of pieces
    pub fn num_pieces(&self) -> usize {
        self.num_pieces
    }

    /// Count how many pieces we have
    pub fn count_pieces(&self) -> usize {
        (0..self.num_pieces).filter(|&i| self.has_piece(i)).count()
    }

    /// Check if we have all pieces
    pub fn is_complete(&self) -> bool {
        self.count_pieces() == self.num_pieces
    }

    /// Check if we have no pieces
    pub fn is_empty(&self) -> bool {
        self.count_pieces() == 0
    }

    /// Get completion percentage (0.0 to 1.0)
    pub fn completion(&self) -> f64 {
        if self.num_pieces == 0 {
            return 1.0;
        }
        self.count_pieces() as f64 / self.num_pieces as f64
    }

    /// Get raw bytes (for sending to peers in bitfield message)
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Get indices of all pieces we have
    pub fn available_pieces(&self) -> Vec<usize> {
        (0..self.num_pieces)
            .filter(|&i| self.has_piece(i))
            .collect()
    }

    /// Get indices of all pieces we don't have
    pub fn missing_pieces(&self) -> Vec<usize> {
        (0..self.num_pieces)
            .filter(|&i| !self.has_piece(i))
            .collect()
    }

    /// Check which pieces a peer has that we don't
    pub fn pieces_to_request(&self, peer_bitfield: &Bitfield) -> Vec<usize> {
        (0..self.num_pieces)
            .filter(|&i| !self.has_piece(i) && peer_bitfield.has_piece(i))
            .collect()
    }

    /// Bitwise OR with another bitfield (union of available pieces)
    pub fn union(&self, other: &Bitfield) -> Bitfield {
        assert_eq!(
            self.num_pieces, other.num_pieces,
            "Bitfields must have same number of pieces"
        );
        let bytes = self
            .bytes
            .iter()
            .zip(&other.bytes)
            .map(|(a, b)| a | b)
            .collect();
        Bitfield {
            bytes,
            num_pieces: self.num_pieces,
        }
    }

    /// Bitwise AND with another bitfield (intersection of available pieces)
    pub fn intersection(&self, other: &Bitfield) -> Bitfield {
        assert_eq!(
            self.num_pieces, other.num_pieces,
            "Bitfields must have same number of pieces"
        );
        let bytes = self
            .bytes
            .iter()
            .zip(&other.bytes)
            .map(|(a, b)| a & b)
            .collect();
        Bitfield {
            bytes,
            num_pieces: self.num_pieces,
        }
    }
}

impl fmt::Display for Bitfield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Bitfield({}/{} pieces, {:.1}%)",
            self.count_pieces(),
            self.num_pieces,
            self.completion() * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_bitfield() {
        let bf = Bitfield::new(10);
        assert_eq!(bf.num_pieces(), 10);
        assert_eq!(bf.count_pieces(), 0);
        assert!(bf.is_empty());
        assert!(!bf.is_complete());
    }

    #[test]
    fn test_set_and_has_piece() {
        let mut bf = Bitfield::new(16);
        assert!(!bf.has_piece(0));
        assert!(!bf.has_piece(5));
        assert!(!bf.has_piece(15));

        bf.set_piece(0);
        bf.set_piece(5);
        bf.set_piece(15);

        assert!(bf.has_piece(0));
        assert!(bf.has_piece(5));
        assert!(bf.has_piece(15));
        assert!(!bf.has_piece(1));
        assert_eq!(bf.count_pieces(), 3);
    }

    #[test]
    fn test_clear_piece() {
        let mut bf = Bitfield::new(10);
        bf.set_piece(3);
        assert!(bf.has_piece(3));

        bf.clear_piece(3);
        assert!(!bf.has_piece(3));
    }

    #[test]
    fn test_complete_bitfield() {
        let bf = Bitfield::complete(10);
        assert_eq!(bf.count_pieces(), 10);
        assert!(bf.is_complete());
        assert!(!bf.is_empty());
        assert_eq!(bf.completion(), 1.0);

        for i in 0..10 {
            assert!(bf.has_piece(i));
        }
    }

    #[test]
    fn test_from_bytes() {
        // First byte: 11010000 = pieces 0, 1, 3 available
        // Second byte: 10000000 = piece 8 available
        let bytes = vec![0b11010000, 0b10000000];
        let bf = Bitfield::from_bytes(bytes, 12);

        assert!(bf.has_piece(0));
        assert!(bf.has_piece(1));
        assert!(!bf.has_piece(2));
        assert!(bf.has_piece(3));
        assert!(!bf.has_piece(4));
        assert!(bf.has_piece(8));
        assert!(!bf.has_piece(9));
        assert_eq!(bf.count_pieces(), 4);
    }

    #[test]
    fn test_available_and_missing_pieces() {
        let mut bf = Bitfield::new(8);
        bf.set_piece(0);
        bf.set_piece(3);
        bf.set_piece(7);

        assert_eq!(bf.available_pieces(), vec![0, 3, 7]);
        assert_eq!(bf.missing_pieces(), vec![1, 2, 4, 5, 6]);
    }

    #[test]
    fn test_pieces_to_request() {
        let mut our_bf = Bitfield::new(10);
        our_bf.set_piece(0);
        our_bf.set_piece(1);

        let mut peer_bf = Bitfield::new(10);
        peer_bf.set_piece(1);
        peer_bf.set_piece(2);
        peer_bf.set_piece(5);

        let to_request = our_bf.pieces_to_request(&peer_bf);
        assert_eq!(to_request, vec![2, 5]);
    }

    #[test]
    fn test_union() {
        let mut bf1 = Bitfield::new(10);
        bf1.set_piece(0);
        bf1.set_piece(2);

        let mut bf2 = Bitfield::new(10);
        bf2.set_piece(1);
        bf2.set_piece(2);

        let union = bf1.union(&bf2);
        assert!(union.has_piece(0));
        assert!(union.has_piece(1));
        assert!(union.has_piece(2));
        assert!(!union.has_piece(3));
        assert_eq!(union.count_pieces(), 3);
    }

    #[test]
    fn test_intersection() {
        let mut bf1 = Bitfield::new(10);
        bf1.set_piece(0);
        bf1.set_piece(1);
        bf1.set_piece(2);

        let mut bf2 = Bitfield::new(10);
        bf2.set_piece(1);
        bf2.set_piece(2);
        bf2.set_piece(3);

        let intersection = bf1.intersection(&bf2);
        assert!(!intersection.has_piece(0));
        assert!(intersection.has_piece(1));
        assert!(intersection.has_piece(2));
        assert!(!intersection.has_piece(3));
        assert_eq!(intersection.count_pieces(), 2);
    }

    #[test]
    fn test_completion() {
        let mut bf = Bitfield::new(10);
        assert_eq!(bf.completion(), 0.0);

        bf.set_piece(0);
        bf.set_piece(1);
        bf.set_piece(2);
        bf.set_piece(3);
        bf.set_piece(4);
        assert_eq!(bf.completion(), 0.5);

        for i in 5..10 {
            bf.set_piece(i);
        }
        assert_eq!(bf.completion(), 1.0);
    }
}
