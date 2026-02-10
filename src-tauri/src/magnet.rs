//! Magnet URI parsing and handling (BEP 9)
//!
//! Supports magnet URIs like:
//! magnet:?xt=urn:btih:HASH&dn=Name&tr=http://tracker.example.com/announce

use std::collections::HashMap;

/// Parsed magnet link information
#[derive(Debug, Clone)]
pub struct MagnetLink {
    /// Info hash (20 bytes)
    pub info_hash: [u8; 20],

    /// Display name (optional)
    pub display_name: Option<String>,

    /// Tracker URLs
    pub trackers: Vec<String>,

    /// Web seed URLs (optional)
    pub web_seeds: Vec<String>,
}

impl MagnetLink {
    /// Parse a magnet URI string
    pub fn parse(uri: &str) -> Result<Self, String> {
        // Check if it starts with "magnet:?"
        if !uri.starts_with("magnet:?") {
            return Err("Invalid magnet URI: must start with 'magnet:?'".to_string());
        }

        // Remove "magnet:?" prefix
        let params_str = &uri[8..];

        // Parse query parameters
        let params = Self::parse_params(params_str)?;

        // Extract info hash (required)
        let info_hash = Self::extract_info_hash(&params)?;

        // Extract display name (optional)
        let display_name = params.get("dn").map(|s| s.to_string());

        // Extract trackers (optional, can be multiple)
        let trackers = params
            .iter()
            .filter_map(|(k, v)| {
                if k == "tr" || k.starts_with("tr_") {
                    Some(v.clone())
                } else {
                    None
                }
            })
            .collect();

        // Extract web seeds (optional)
        let web_seeds = params
            .iter()
            .filter_map(|(k, v)| {
                if k == "ws" || k.starts_with("ws_") {
                    Some(v.clone())
                } else {
                    None
                }
            })
            .collect();

        Ok(MagnetLink {
            info_hash,
            display_name,
            trackers,
            web_seeds,
        })
    }

    /// Parse query parameters from the magnet URI
    fn parse_params(params_str: &str) -> Result<HashMap<String, String>, String> {
        let mut params = HashMap::new();

        for param in params_str.split('&') {
            if let Some((key, value)) = param.split_once('=') {
                let decoded_value = urlencoding::decode(value)
                    .map_err(|e| format!("Failed to decode parameter: {}", e))?
                    .to_string();

                // Handle multiple values for same key (like multiple trackers)
                if params.contains_key(key) {
                    // For simplicity, we'll handle this in extract phase
                    // Store with a counter suffix
                    let mut counter = 1;
                    while params.contains_key(&format!("{}_{}", key, counter)) {
                        counter += 1;
                    }
                    params.insert(format!("{}_{}", key, counter), decoded_value);
                } else {
                    params.insert(key.to_string(), decoded_value);
                }
            }
        }

        Ok(params)
    }

    /// Extract and decode info hash from parameters
    fn extract_info_hash(params: &HashMap<String, String>) -> Result<[u8; 20], String> {
        // Look for "xt" parameter (exact topic)
        let xt = params
            .get("xt")
            .ok_or_else(|| "Missing 'xt' parameter (info hash)".to_string())?;

        // Should be in format "urn:btih:HASH"
        if !xt.starts_with("urn:btih:") {
            return Err("Invalid 'xt' parameter: must start with 'urn:btih:'".to_string());
        }

        let hash_str = &xt[9..]; // Remove "urn:btih:" prefix

        // Try to decode as hex (40 characters) or base32 (32 characters)
        if hash_str.len() == 40 {
            // Hex encoding
            Self::decode_hex(hash_str)
        } else if hash_str.len() == 32 {
            // Base32 encoding
            Self::decode_base32(hash_str)
        } else {
            Err(format!(
                "Invalid info hash length: expected 40 (hex) or 32 (base32), got {}",
                hash_str.len()
            ))
        }
    }

    /// Decode hex string to 20 bytes
    fn decode_hex(s: &str) -> Result<[u8; 20], String> {
        let mut bytes = [0u8; 20];

        for i in 0..20 {
            let hex_byte = &s[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(hex_byte, 16)
                .map_err(|e| format!("Invalid hex character: {}", e))?;
        }

        Ok(bytes)
    }

    /// Decode base32 string to 20 bytes
    fn decode_base32(s: &str) -> Result<[u8; 20], String> {
        // BitTorrent uses base32 without padding
        const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

        let s_upper = s.to_uppercase();
        let mut bits = Vec::new();

        for c in s_upper.chars() {
            let value = BASE32_ALPHABET
                .iter()
                .position(|&b| b as char == c)
                .ok_or_else(|| format!("Invalid base32 character: {}", c))?;

            // Each base32 char = 5 bits
            for i in (0..5).rev() {
                bits.push(((value >> i) & 1) as u8);
            }
        }

        // Convert bits to bytes (should be 160 bits = 20 bytes)
        if bits.len() < 160 {
            return Err(format!(
                "Invalid base32 hash: too short ({} bits)",
                bits.len()
            ));
        }

        let mut bytes = [0u8; 20];
        for i in 0..20 {
            let mut byte = 0u8;
            for j in 0..8 {
                byte = (byte << 1) | bits[i * 8 + j];
            }
            bytes[i] = byte;
        }

        Ok(bytes)
    }

    /// Get info hash as hex string
    pub fn info_hash_hex(&self) -> String {
        hex::encode(self.info_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_magnet() {
        let uri = "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567";
        let magnet = MagnetLink::parse(uri).unwrap();

        assert_eq!(
            magnet.info_hash_hex(),
            "0123456789abcdef0123456789abcdef01234567"
        );
        assert_eq!(magnet.display_name, None);
        assert_eq!(magnet.trackers.len(), 0);
    }

    #[test]
    fn test_parse_magnet_with_name() {
        let uri = "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=Test%20Torrent";
        let magnet = MagnetLink::parse(uri).unwrap();

        assert_eq!(magnet.display_name, Some("Test Torrent".to_string()));
    }

    #[test]
    fn test_parse_magnet_with_tracker() {
        let uri = "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&tr=http://tracker.example.com/announce";
        let magnet = MagnetLink::parse(uri).unwrap();

        assert_eq!(magnet.trackers.len(), 1);
        assert_eq!(magnet.trackers[0], "http://tracker.example.com/announce");
    }

    #[test]
    fn test_invalid_magnet() {
        let uri = "http://example.com";
        assert!(MagnetLink::parse(uri).is_err());
    }

    #[test]
    fn test_missing_info_hash() {
        let uri = "magnet:?dn=Test";
        assert!(MagnetLink::parse(uri).is_err());
    }
}
