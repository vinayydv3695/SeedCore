//! Peer-to-peer communication module
//! 
//! Implements the BitTorrent wire protocol for communicating with peers.

pub mod handshake;
pub mod manager;
pub mod message;

pub use handshake::Handshake;
pub use manager::{PeerManager, PeerManagerCommand, PeerManagerStats};
pub use message::{Message, MessageId};

use serde::{Deserialize, Serialize};

/// Detailed peer information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// IP address
    pub ip: String,
    /// Port number
    pub port: u16,
    /// Client name (parsed from peer_id)
    pub client: String,
    /// Connection flags (D=downloading, U=uploading, etc.)
    pub flags: String,
    /// Peer's download progress (0.0-100.0)
    pub progress: f64,
    /// Download speed from this peer (bytes/sec)
    pub download_speed: u64,
    /// Upload speed to this peer (bytes/sec)
    pub upload_speed: u64,
    /// Total downloaded from this peer (bytes)
    pub downloaded: u64,
    /// Total uploaded to this peer (bytes)
    pub uploaded: u64,
}

use crate::error::Result;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Peer connection
pub struct PeerConnection {
    /// TCP stream
    stream: TcpStream,
    
    /// Peer address
    pub addr: SocketAddr,
    
    /// Peer ID (from handshake)
    pub peer_id: Option<[u8; 20]>,
    
    /// Whether we are choked by the peer
    pub peer_choking: bool,
    
    /// Whether the peer is interested in us
    pub peer_interested: bool,
    
    /// Whether we are choking the peer
    pub am_choking: bool,
    
    /// Whether we are interested in the peer
    pub am_interested: bool,
    
    /// Bitfield of pieces the peer has
    pub bitfield: Option<Vec<u8>>,
}

impl PeerConnection {
    /// Create a new peer connection from a TCP stream
    pub fn new(stream: TcpStream, addr: SocketAddr) -> Self {
        Self {
            stream,
            addr,
            peer_id: None,
            peer_choking: true,
            peer_interested: false,
            am_choking: true,
            am_interested: false,
            bitfield: None,
        }
    }
    
    /// Connect to a peer
    pub async fn connect(addr: SocketAddr) -> Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| crate::error::Error::NetworkError(format!("Failed to connect: {}", e)))?;
        
        Ok(Self::new(stream, addr))
    }
    
    /// Perform handshake with peer
    pub async fn handshake(
        &mut self,
        info_hash: [u8; 20],
        our_peer_id: [u8; 20],
    ) -> Result<Handshake> {
        // Send our handshake
        let our_handshake = Handshake::new(info_hash, our_peer_id);
        let handshake_bytes = our_handshake.to_bytes();
        
        self.stream.write_all(&handshake_bytes)
            .await
            .map_err(|e| crate::error::Error::NetworkError(format!("Failed to send handshake: {}", e)))?;
        
        tracing::debug!("Sent handshake to {}", self.addr);
        
        // Read peer's handshake
        let mut buf = vec![0u8; 68]; // Handshake is 68 bytes
        self.stream.read_exact(&mut buf)
            .await
            .map_err(|e| crate::error::Error::NetworkError(format!("Failed to read handshake: {}", e)))?;
        
        let peer_handshake = Handshake::from_bytes(&buf)?;
        
        // Verify info hash matches
        if peer_handshake.info_hash != info_hash {
            return Err(crate::error::Error::NetworkError(
                "Info hash mismatch".to_string()
            ));
        }
        
        self.peer_id = Some(peer_handshake.peer_id);
        
        tracing::debug!("Received handshake from {}", self.addr);
        
        Ok(peer_handshake)
    }
    
    /// Send a message to the peer
    pub async fn send_message(&mut self, message: &Message) -> Result<()> {
        let bytes = message.to_bytes();
        
        self.stream.write_all(&bytes)
            .await
            .map_err(|e| crate::error::Error::NetworkError(format!("Failed to send message: {}", e)))?;
        
        Ok(())
    }
    
    /// Receive a message from the peer
    pub async fn recv_message(&mut self) -> Result<Message> {
        // Read message length (4 bytes, big-endian)
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf)
            .await
            .map_err(|e| crate::error::Error::NetworkError(format!("Failed to read message length: {}", e)))?;
        
        let length = u32::from_be_bytes(len_buf);
        
        // Handle keep-alive (length = 0)
        if length == 0 {
            return Ok(Message::KeepAlive);
        }
        
        // Read message payload
        let mut payload = vec![0u8; length as usize];
        self.stream.read_exact(&mut payload)
            .await
            .map_err(|e| crate::error::Error::NetworkError(format!("Failed to read message payload: {}", e)))?;
        
        Message::from_bytes(&payload)
    }
    
    /// Send keep-alive message
    pub async fn send_keep_alive(&mut self) -> Result<()> {
        self.send_message(&Message::KeepAlive).await
    }
    
    /// Send interested message
    pub async fn send_interested(&mut self) -> Result<()> {
        self.am_interested = true;
        self.send_message(&Message::Interested).await
    }
    
    /// Send not interested message
    pub async fn send_not_interested(&mut self) -> Result<()> {
        self.am_interested = false;
        self.send_message(&Message::NotInterested).await
    }
    
    /// Send choke message
    pub async fn send_choke(&mut self) -> Result<()> {
        self.am_choking = true;
        self.send_message(&Message::Choke).await
    }
    
    /// Send unchoke message
    pub async fn send_unchoke(&mut self) -> Result<()> {
        self.am_choking = false;
        self.send_message(&Message::Unchoke).await
    }
}
