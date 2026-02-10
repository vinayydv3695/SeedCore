//! BitTorrent handshake protocol
//!
//! Format:
//! - 1 byte: protocol name length (19)
//! - 19 bytes: protocol name ("BitTorrent protocol")
//! - 8 bytes: reserved (extension bits)
//! - 20 bytes: info hash
//! - 20 bytes: peer ID

use crate::error::{Error, Result};

const PROTOCOL_NAME: &[u8] = b"BitTorrent protocol";
const HANDSHAKE_LENGTH: usize = 68;

/// BitTorrent handshake message
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handshake {
    /// Protocol name (should be "BitTorrent protocol")
    pub protocol: Vec<u8>,

    /// Reserved bytes (extension bits)
    pub reserved: [u8; 8],

    /// Info hash (20 bytes)
    pub info_hash: [u8; 20],

    /// Peer ID (20 bytes)
    pub peer_id: [u8; 20],
}

impl Handshake {
    /// Create a new handshake
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            protocol: PROTOCOL_NAME.to_vec(),
            reserved: [0u8; 8],
            info_hash,
            peer_id,
        }
    }

    /// Parse handshake from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() != HANDSHAKE_LENGTH {
            return Err(Error::InvalidData(format!(
                "handshake must be {} bytes, got {}",
                HANDSHAKE_LENGTH,
                data.len()
            )));
        }

        // Read protocol name length
        let pstr_len = data[0] as usize;

        if pstr_len != 19 {
            return Err(Error::InvalidData(format!(
                "protocol name length must be 19, got {}",
                pstr_len
            )));
        }

        // Read protocol name
        let protocol = data[1..20].to_vec();

        if protocol != PROTOCOL_NAME {
            return Err(Error::InvalidData(
                "protocol name must be 'BitTorrent protocol'".to_string(),
            ));
        }

        // Read reserved bytes
        let mut reserved = [0u8; 8];
        reserved.copy_from_slice(&data[20..28]);

        // Read info hash
        let mut info_hash = [0u8; 20];
        info_hash.copy_from_slice(&data[28..48]);

        // Read peer ID
        let mut peer_id = [0u8; 20];
        peer_id.copy_from_slice(&data[48..68]);

        Ok(Self {
            protocol,
            reserved,
            info_hash,
            peer_id,
        })
    }

    /// Convert handshake to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HANDSHAKE_LENGTH);

        // Protocol name length
        bytes.push(self.protocol.len() as u8);

        // Protocol name
        bytes.extend_from_slice(&self.protocol);

        // Reserved bytes
        bytes.extend_from_slice(&self.reserved);

        // Info hash
        bytes.extend_from_slice(&self.info_hash);

        // Peer ID
        bytes.extend_from_slice(&self.peer_id);

        bytes
    }

    /// Check if extension is supported (from reserved bytes)
    pub fn supports_extension(&self, bit: u8) -> bool {
        let byte_idx = (bit / 8) as usize;
        let bit_idx = bit % 8;

        if byte_idx >= 8 {
            return false;
        }

        (self.reserved[byte_idx] & (1 << bit_idx)) != 0
    }

    /// Enable an extension bit
    pub fn enable_extension(&mut self, bit: u8) {
        let byte_idx = (bit / 8) as usize;
        let bit_idx = bit % 8;

        if byte_idx < 8 {
            self.reserved[byte_idx] |= 1 << bit_idx;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_serialize_deserialize() {
        let info_hash = [1u8; 20];
        let peer_id = [2u8; 20];

        let handshake = Handshake::new(info_hash, peer_id);
        let bytes = handshake.to_bytes();

        assert_eq!(bytes.len(), HANDSHAKE_LENGTH);

        let parsed = Handshake::from_bytes(&bytes).unwrap();

        assert_eq!(parsed, handshake);
        assert_eq!(parsed.info_hash, info_hash);
        assert_eq!(parsed.peer_id, peer_id);
    }

    #[test]
    fn test_handshake_format() {
        let info_hash = [0xABu8; 20];
        let peer_id = [0xCDu8; 20];

        let handshake = Handshake::new(info_hash, peer_id);
        let bytes = handshake.to_bytes();

        // Check protocol name length
        assert_eq!(bytes[0], 19);

        // Check protocol name
        assert_eq!(&bytes[1..20], b"BitTorrent protocol");

        // Check reserved bytes are zero
        assert_eq!(&bytes[20..28], &[0u8; 8]);

        // Check info hash
        assert_eq!(&bytes[28..48], &[0xABu8; 20]);

        // Check peer ID
        assert_eq!(&bytes[48..68], &[0xCDu8; 20]);
    }

    #[test]
    fn test_extension_bits() {
        let mut handshake = Handshake::new([0u8; 20], [0u8; 20]);

        // Initially no extensions
        assert!(!handshake.supports_extension(0));
        assert!(!handshake.supports_extension(20));

        // Enable extension bit 20 (DHT)
        handshake.enable_extension(20);
        assert!(handshake.supports_extension(20));

        // Check it persists through serialization
        let bytes = handshake.to_bytes();
        let parsed = Handshake::from_bytes(&bytes).unwrap();
        assert!(parsed.supports_extension(20));
    }
}
