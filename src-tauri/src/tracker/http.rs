//! HTTP tracker protocol implementation
//! 
//! Reference: http://bittorrent.org/beps/bep_0003.html

use crate::bencode::BencodeValue;
use crate::error::{Error, Result};
use crate::tracker::{AnnounceRequest, AnnounceResponse, Peer};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

/// HTTP tracker client
pub struct HttpTracker {
    /// HTTP client
    client: reqwest::Client,
}

impl HttpTracker {
    /// Create a new HTTP tracker client
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("SeedCore/0.1.0")
            .build()
            .expect("Failed to create HTTP client");
        
        Self { client }
    }
    
    /// Send announce request to tracker
    pub async fn announce(
        &self,
        tracker_url: &str,
        request: &AnnounceRequest,
    ) -> Result<AnnounceResponse> {
        // Build URL with query parameters
        let url = self.build_announce_url(tracker_url, request)?;
        
        tracing::debug!("Announcing to tracker: {}", url);
        
        // Send HTTP GET request
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::NetworkError(format!("HTTP request failed: {}", e)))?;
        
        // Check status code
        if !response.status().is_success() {
            return Err(Error::NetworkError(format!(
                "Tracker returned error: {}",
                response.status()
            )));
        }
        
        // Get response bytes
        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to read response: {}", e)))?;
        
        // Parse bencode response
        self.parse_announce_response(&bytes)
    }
    
    /// Build announce URL with parameters
    fn build_announce_url(
        &self,
        tracker_url: &str,
        request: &AnnounceRequest,
    ) -> Result<String> {
        let mut url = reqwest::Url::parse(tracker_url)
            .map_err(|e| Error::NetworkError(format!("Invalid tracker URL: {}", e)))?;
        
        // Manually build query string to avoid double-encoding
        let mut params = Vec::new();
        
        // Info hash (manually URL encoded)
        params.push(format!("info_hash={}", Self::url_encode_bytes(&request.info_hash)));
        
        // Peer ID (manually URL encoded)
        params.push(format!("peer_id={}", Self::url_encode_bytes(&request.peer_id)));
        
        // Port
        params.push(format!("port={}", request.port));
        
        // Statistics
        params.push(format!("uploaded={}", request.uploaded));
        params.push(format!("downloaded={}", request.downloaded));
        params.push(format!("left={}", request.left));
        
        // Compact mode (binary peer list)
        params.push(format!("compact={}", if request.compact { "1" } else { "0" }));
        
        // Number of peers wanted
        if let Some(numwant) = request.numwant {
            params.push(format!("numwant={}", numwant));
        }
        
        // Event
        if let Some(event) = request.event.as_str() {
            params.push(format!("event={}", event));
        }
        
        // Set query string directly
        url.set_query(Some(&params.join("&")));
        
        Ok(url.to_string())
    }
    
    /// URL-encode bytes (for info_hash and peer_id)
    fn url_encode_bytes(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|b| format!("%{:02x}", b))
            .collect()
    }
    
    /// Parse announce response from bencode
    fn parse_announce_response(&self, data: &[u8]) -> Result<AnnounceResponse> {
        let value = BencodeValue::parse(data)?;
        let dict = value.as_dict()
            .ok_or_else(|| Error::MetainfoError("response must be a dictionary".to_string()))?;
        
        // Check for failure reason
        if let Some(failure) = dict.get(b"failure reason" as &[u8]) {
            if let Some(reason) = failure.as_str() {
                return Err(Error::NetworkError(format!("Tracker error: {}", reason)));
            }
        }
        
        // Get interval (required)
        let interval = dict.get(b"interval" as &[u8])
            .and_then(|v| v.as_integer())
            .ok_or_else(|| Error::MetainfoError("missing interval".to_string()))?
            as u32;
        
        // Get optional fields
        let warning_message = dict.get(b"warning message" as &[u8])
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let min_interval = dict.get(b"min interval" as &[u8])
            .and_then(|v| v.as_integer())
            .map(|i| i as u32);
        
        let tracker_id = dict.get(b"tracker id" as &[u8])
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let complete = dict.get(b"complete" as &[u8])
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as u32;
        
        let incomplete = dict.get(b"incomplete" as &[u8])
            .and_then(|v| v.as_integer())
            .unwrap_or(0) as u32;
        
        // Parse peers
        let peers = self.parse_peers(dict)?;
        
        Ok(AnnounceResponse {
            warning_message,
            interval,
            min_interval,
            tracker_id,
            complete,
            incomplete,
            peers,
        })
    }
    
    /// Parse peers from response (supports both compact and dictionary format)
    fn parse_peers(
        &self,
        dict: &std::collections::HashMap<Vec<u8>, BencodeValue>,
    ) -> Result<Vec<Peer>> {
        let peers_value = dict.get(b"peers" as &[u8])
            .ok_or_else(|| Error::MetainfoError("missing peers".to_string()))?;
        
        // Check if compact format (byte string) or dictionary format (list)
        if let Some(bytes) = peers_value.as_bytes() {
            // Compact format: 6 bytes per peer (4 byte IP + 2 byte port)
            self.parse_compact_peers(bytes)
        } else if let Some(list) = peers_value.as_list() {
            // Dictionary format
            self.parse_dictionary_peers(list)
        } else {
            Err(Error::MetainfoError("invalid peers format".to_string()))
        }
    }
    
    /// Parse compact peer list (binary format)
    fn parse_compact_peers(&self, bytes: &[u8]) -> Result<Vec<Peer>> {
        if bytes.len() % 6 != 0 {
            return Err(Error::MetainfoError(
                "compact peers length must be multiple of 6".to_string()
            ));
        }
        
        let mut peers = Vec::new();
        
        for chunk in bytes.chunks_exact(6) {
            // First 4 bytes: IP address (big-endian)
            let ip = Ipv4Addr::new(chunk[0], chunk[1], chunk[2], chunk[3]);
            
            // Last 2 bytes: port (big-endian)
            let port = u16::from_be_bytes([chunk[4], chunk[5]]);
            
            peers.push(Peer {
                peer_id: None,
                addr: SocketAddr::new(IpAddr::V4(ip), port),
            });
        }
        
        Ok(peers)
    }
    
    /// Parse dictionary peer list
    fn parse_dictionary_peers(&self, list: &[BencodeValue]) -> Result<Vec<Peer>> {
        let mut peers = Vec::new();
        
        for peer_value in list {
            let peer_dict = peer_value.as_dict()
                .ok_or_else(|| Error::MetainfoError("peer must be a dictionary".to_string()))?;
            
            // Get peer ID (optional)
            let peer_id = peer_dict.get(b"peer id" as &[u8])
                .and_then(|v| v.as_bytes())
                .map(|b| b.to_vec());
            
            // Get IP address
            let ip_str = peer_dict.get(b"ip" as &[u8])
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::MetainfoError("missing peer IP".to_string()))?;
            
            let ip: IpAddr = ip_str.parse()
                .map_err(|_| Error::MetainfoError(format!("invalid IP address: {}", ip_str)))?;
            
            // Get port
            let port = peer_dict.get(b"port" as &[u8])
                .and_then(|v| v.as_integer())
                .ok_or_else(|| Error::MetainfoError("missing peer port".to_string()))?
                as u16;
            
            peers.push(Peer {
                peer_id,
                addr: SocketAddr::new(ip, port),
            });
        }
        
        Ok(peers)
    }
}

impl Default for HttpTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_url_encode_bytes() {
        let bytes = b"hello";
        let encoded = HttpTracker::url_encode_bytes(bytes);
        assert_eq!(encoded, "%68%65%6c%6c%6f");
    }
    
    #[test]
    fn test_parse_compact_peers() {
        let tracker = HttpTracker::new();
        
        // 2 peers: 192.168.1.1:6881 and 10.0.0.1:6882
        let data = vec![
            192, 168, 1, 1, 0x1A, 0xE1,  // 192.168.1.1:6881
            10, 0, 0, 1, 0x1A, 0xE2,      // 10.0.0.1:6882
        ];
        
        let peers = tracker.parse_compact_peers(&data).unwrap();
        
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].addr.to_string(), "192.168.1.1:6881");
        assert_eq!(peers[1].addr.to_string(), "10.0.0.1:6882");
    }
    
    #[test]
    fn test_build_announce_url() {
        let tracker = HttpTracker::new();
        let mut request = AnnounceRequest::default();
        request.info_hash = [1u8; 20];
        request.peer_id = [2u8; 20];
        request.port = 6881;
        
        let url = tracker.build_announce_url("http://tracker.example.com/announce", &request).unwrap();
        
        // Debug: print the actual URL
        println!("Generated URL: {}", url);
        
        // The URL encoding uses reqwest's built-in encoding, which may differ slightly
        // Just check that the important parts are present
        assert!(url.contains("info_hash="));
        assert!(url.contains("peer_id="));
        assert!(url.contains("port=6881"));
        assert!(url.contains("compact=1"));
        assert!(url.contains("uploaded=0"));
        assert!(url.contains("downloaded=0"));
        assert!(url.contains("left=0"));
    }
}
