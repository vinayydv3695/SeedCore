//! BitTorrent wire protocol messages
//!
//! All messages follow this format:
//! - 4 bytes: message length (big-endian)
//! - 1 byte: message ID
//! - N bytes: payload

use crate::error::{Error, Result};

/// Message ID constants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageId {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

impl MessageId {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Choke),
            1 => Ok(Self::Unchoke),
            2 => Ok(Self::Interested),
            3 => Ok(Self::NotInterested),
            4 => Ok(Self::Have),
            5 => Ok(Self::Bitfield),
            6 => Ok(Self::Request),
            7 => Ok(Self::Piece),
            8 => Ok(Self::Cancel),
            _ => Err(Error::InvalidData(format!("unknown message ID: {}", value))),
        }
    }
}

/// BitTorrent wire protocol message
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Keep-alive message (no payload)
    KeepAlive,

    /// Choke the peer
    Choke,

    /// Unchoke the peer
    Unchoke,

    /// Express interest in the peer
    Interested,

    /// Express lack of interest
    NotInterested,

    /// Notify peer we have a piece
    Have { piece_index: u32 },

    /// Send bitfield of pieces we have
    Bitfield { bitfield: Vec<u8> },

    /// Request a block of a piece
    Request { index: u32, begin: u32, length: u32 },

    /// Send a block of data
    Piece {
        index: u32,
        begin: u32,
        data: Vec<u8>,
    },

    /// Cancel a request
    Cancel { index: u32, begin: u32, length: u32 },
}

impl Message {
    /// Parse message from bytes (without the length prefix)
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Ok(Self::KeepAlive);
        }

        let id = MessageId::from_u8(data[0])?;
        let payload = &data[1..];

        match id {
            MessageId::Choke => {
                if !payload.is_empty() {
                    return Err(Error::InvalidData("choke must have no payload".to_string()));
                }
                Ok(Self::Choke)
            }

            MessageId::Unchoke => {
                if !payload.is_empty() {
                    return Err(Error::InvalidData(
                        "unchoke must have no payload".to_string(),
                    ));
                }
                Ok(Self::Unchoke)
            }

            MessageId::Interested => {
                if !payload.is_empty() {
                    return Err(Error::InvalidData(
                        "interested must have no payload".to_string(),
                    ));
                }
                Ok(Self::Interested)
            }

            MessageId::NotInterested => {
                if !payload.is_empty() {
                    return Err(Error::InvalidData(
                        "not interested must have no payload".to_string(),
                    ));
                }
                Ok(Self::NotInterested)
            }

            MessageId::Have => {
                if payload.len() != 4 {
                    return Err(Error::InvalidData("have must be 4 bytes".to_string()));
                }
                let piece_index =
                    u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                Ok(Self::Have { piece_index })
            }

            MessageId::Bitfield => Ok(Self::Bitfield {
                bitfield: payload.to_vec(),
            }),

            MessageId::Request => {
                if payload.len() != 12 {
                    return Err(Error::InvalidData("request must be 12 bytes".to_string()));
                }

                let index = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                let begin = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
                let length = u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]);

                Ok(Self::Request {
                    index,
                    begin,
                    length,
                })
            }

            MessageId::Piece => {
                if payload.len() < 8 {
                    return Err(Error::InvalidData(
                        "piece must be at least 8 bytes".to_string(),
                    ));
                }

                let index = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                let begin = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
                let data = payload[8..].to_vec();

                Ok(Self::Piece { index, begin, data })
            }

            MessageId::Cancel => {
                if payload.len() != 12 {
                    return Err(Error::InvalidData("cancel must be 12 bytes".to_string()));
                }

                let index = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
                let begin = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
                let length = u32::from_be_bytes([payload[8], payload[9], payload[10], payload[11]]);

                Ok(Self::Cancel {
                    index,
                    begin,
                    length,
                })
            }
        }
    }

    /// Convert message to bytes (including length prefix)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match self {
            Self::KeepAlive => {
                // Length: 0
                bytes.extend_from_slice(&0u32.to_be_bytes());
            }

            Self::Choke => {
                bytes.extend_from_slice(&1u32.to_be_bytes());
                bytes.push(MessageId::Choke as u8);
            }

            Self::Unchoke => {
                bytes.extend_from_slice(&1u32.to_be_bytes());
                bytes.push(MessageId::Unchoke as u8);
            }

            Self::Interested => {
                bytes.extend_from_slice(&1u32.to_be_bytes());
                bytes.push(MessageId::Interested as u8);
            }

            Self::NotInterested => {
                bytes.extend_from_slice(&1u32.to_be_bytes());
                bytes.push(MessageId::NotInterested as u8);
            }

            Self::Have { piece_index } => {
                bytes.extend_from_slice(&5u32.to_be_bytes()); // Length: 1 + 4
                bytes.push(MessageId::Have as u8);
                bytes.extend_from_slice(&piece_index.to_be_bytes());
            }

            Self::Bitfield { bitfield } => {
                let length = 1 + bitfield.len() as u32;
                bytes.extend_from_slice(&length.to_be_bytes());
                bytes.push(MessageId::Bitfield as u8);
                bytes.extend_from_slice(bitfield);
            }

            Self::Request {
                index,
                begin,
                length,
            } => {
                bytes.extend_from_slice(&13u32.to_be_bytes()); // Length: 1 + 12
                bytes.push(MessageId::Request as u8);
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(&length.to_be_bytes());
            }

            Self::Piece { index, begin, data } => {
                let length = 1 + 8 + data.len() as u32;
                bytes.extend_from_slice(&length.to_be_bytes());
                bytes.push(MessageId::Piece as u8);
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(data);
            }

            Self::Cancel {
                index,
                begin,
                length,
            } => {
                bytes.extend_from_slice(&13u32.to_be_bytes()); // Length: 1 + 12
                bytes.push(MessageId::Cancel as u8);
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(&length.to_be_bytes());
            }
        }

        bytes
    }

    /// Get the message length (for the length prefix)
    pub fn length(&self) -> u32 {
        match self {
            Self::KeepAlive => 0,
            Self::Choke | Self::Unchoke | Self::Interested | Self::NotInterested => 1,
            Self::Have { .. } => 5,
            Self::Bitfield { bitfield } => 1 + bitfield.len() as u32,
            Self::Request { .. } | Self::Cancel { .. } => 13,
            Self::Piece { data, .. } => 1 + 8 + data.len() as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keep_alive() {
        let msg = Message::KeepAlive;
        let bytes = msg.to_bytes();

        assert_eq!(bytes, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_choke_unchoke() {
        let choke = Message::Choke;
        let choke_bytes = choke.to_bytes();

        assert_eq!(choke_bytes, vec![0, 0, 0, 1, 0]);

        let parsed = Message::from_bytes(&choke_bytes[4..]).unwrap();
        assert_eq!(parsed, choke);

        let unchoke = Message::Unchoke;
        let unchoke_bytes = unchoke.to_bytes();

        assert_eq!(unchoke_bytes, vec![0, 0, 0, 1, 1]);
    }

    #[test]
    fn test_have() {
        let msg = Message::Have { piece_index: 42 };
        let bytes = msg.to_bytes();

        // Length (4 bytes) + ID (1 byte) + piece_index (4 bytes)
        assert_eq!(bytes.len(), 9);
        assert_eq!(&bytes[0..4], &[0, 0, 0, 5]); // Length = 5
        assert_eq!(bytes[4], MessageId::Have as u8);

        let parsed = Message::from_bytes(&bytes[4..]).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_request() {
        let msg = Message::Request {
            index: 10,
            begin: 16384,
            length: 16384,
        };

        let bytes = msg.to_bytes();
        assert_eq!(bytes.len(), 17); // 4 + 1 + 12

        let parsed = Message::from_bytes(&bytes[4..]).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_piece() {
        let data = vec![1, 2, 3, 4, 5];
        let msg = Message::Piece {
            index: 0,
            begin: 0,
            data: data.clone(),
        };

        let bytes = msg.to_bytes();
        assert_eq!(bytes.len(), 4 + 1 + 8 + 5); // length + id + index + begin + data

        let parsed = Message::from_bytes(&bytes[4..]).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn test_bitfield() {
        let bitfield = vec![0xFF, 0x00, 0xAB];
        let msg = Message::Bitfield {
            bitfield: bitfield.clone(),
        };

        let bytes = msg.to_bytes();
        let parsed = Message::from_bytes(&bytes[4..]).unwrap();

        assert_eq!(parsed, msg);
    }
}
