//! Torrent metainfo parsing
//!
//! Parses .torrent files according to the BitTorrent specification.
//! Reference: http://bittorrent.org/beps/bep_0003.html

use crate::bencode::BencodeValue;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

/// Parsed torrent metainfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metainfo {
    /// Tracker announce URL
    pub announce: String,

    /// Optional list of announce URLs (BEP 12)
    pub announce_list: Vec<Vec<String>>,

    /// Info dictionary
    pub info: TorrentInfo,

    /// Info hash (SHA1 of the bencode info dictionary)
    pub info_hash: [u8; 20],

    /// Creation date (Unix timestamp)
    pub creation_date: Option<i64>,

    /// Comment
    pub comment: Option<String>,

    /// Created by
    pub created_by: Option<String>,
}

/// Torrent info dictionary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    /// Piece length in bytes
    pub piece_length: u64,

    /// Concatenated SHA1 hashes of all pieces
    pub pieces: Vec<u8>,

    /// Number of pieces
    pub piece_count: usize,

    /// Files in the torrent
    pub files: Vec<FileInfo>,

    /// Torrent name
    pub name: String,

    /// Total size of all files
    pub total_size: u64,

    /// Whether this is a single-file torrent
    pub is_single_file: bool,
}

/// File information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// File path components
    pub path: Vec<String>,

    /// File length in bytes
    pub length: u64,
}

/// File priority for selective downloading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilePriority {
    /// Don't download this file
    Skip,
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
}

/// Enhanced file information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfoUI {
    /// Full file path as string
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// Downloaded bytes for this file
    pub downloaded: u64,
    /// Download priority
    pub priority: FilePriority,
    /// Whether this is a folder entry
    pub is_folder: bool,
}

impl Metainfo {
    /// Parse a .torrent file from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let root = BencodeValue::parse(data)?;

        let dict = root
            .as_dict()
            .ok_or_else(|| Error::MetainfoError("root must be a dictionary".to_string()))?;

        // Get announce URL
        let announce = dict
            .get(b"announce" as &[u8])
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::MetainfoError("missing announce field".to_string()))?
            .to_string();

        // Get optional announce-list (BEP 12)
        let announce_list = dict
            .get(b"announce-list" as &[u8])
            .and_then(|v| v.as_list())
            .map(|list| {
                list.iter()
                    .filter_map(|tier| tier.as_list())
                    .map(|tier| {
                        tier.iter()
                            .filter_map(|url| url.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Get info dictionary
        let info_value = dict
            .get(b"info" as &[u8])
            .ok_or_else(|| Error::MetainfoError("missing info field".to_string()))?;

        // Calculate info hash
        let info_hash = Self::calculate_info_hash(data)?;

        // Parse info dictionary
        let info = TorrentInfo::parse(info_value)?;

        // Get optional fields
        let creation_date = dict
            .get(b"creation date" as &[u8])
            .and_then(|v| v.as_integer());

        let comment = dict
            .get(b"comment" as &[u8])
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let created_by = dict
            .get(b"created by" as &[u8])
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(Self {
            announce,
            announce_list,
            info,
            info_hash,
            creation_date,
            comment,
            created_by,
        })
    }

    /// Calculate the info hash from raw .torrent data
    fn calculate_info_hash(data: &[u8]) -> Result<[u8; 20]> {
        // Find the info dictionary in the bencode data
        // This is a simplified approach - we need to hash the exact bytes
        let root = BencodeValue::parse(data)?;
        let dict = root
            .as_dict()
            .ok_or_else(|| Error::MetainfoError("root must be a dictionary".to_string()))?;

        // For now, we'll return a placeholder
        // TODO: Implement proper info dictionary extraction and hashing
        let info_value = dict
            .get(b"info" as &[u8])
            .ok_or_else(|| Error::MetainfoError("missing info field".to_string()))?;

        // Hash the info dictionary
        // Note: This is a simplified version - in production, we need to hash
        // the exact bytes of the info dictionary from the original data
        let info_bytes = format!("{:?}", info_value).into_bytes();
        let mut hasher = Sha1::new();
        hasher.update(&info_bytes);
        let result = hasher.finalize();

        let mut hash = [0u8; 20];
        hash.copy_from_slice(&result);
        Ok(hash)
    }

    /// Get the info hash as a hex string
    pub fn info_hash_hex(&self) -> String {
        self.info_hash
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    /// Get the info hash as a URL-encoded string (for tracker requests)
    pub fn info_hash_urlencoded(&self) -> String {
        self.info_hash
            .iter()
            .map(|b| format!("%{:02x}", b))
            .collect()
    }
}

impl TorrentInfo {
    /// Parse the info dictionary
    fn parse(value: &BencodeValue) -> Result<Self> {
        let dict = value
            .as_dict()
            .ok_or_else(|| Error::MetainfoError("info must be a dictionary".to_string()))?;

        // Get piece length
        let piece_length = dict
            .get(b"piece length" as &[u8])
            .or_else(|| dict.get(b"piece_length" as &[u8])) // Try underscore version too
            .and_then(|v| v.as_integer())
            .ok_or_else(|| Error::MetainfoError("missing piece length".to_string()))?
            as u64;

        // Get pieces (concatenated SHA1 hashes)
        let pieces = dict
            .get(b"pieces" as &[u8])
            .and_then(|v| v.as_bytes())
            .ok_or_else(|| Error::MetainfoError("missing pieces".to_string()))?
            .to_vec();

        if pieces.len() % 20 != 0 {
            return Err(Error::MetainfoError(
                "pieces length must be multiple of 20".to_string(),
            ));
        }

        let piece_count = pieces.len() / 20;

        // Get name
        let name = dict
            .get(b"name" as &[u8])
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::MetainfoError("missing name".to_string()))?
            .to_string();

        // Check if single-file or multi-file torrent
        let (files, total_size, is_single_file) = if let Some(length) = dict.get(b"length" as &[u8])
        {
            // Single file torrent
            let length = length
                .as_integer()
                .ok_or_else(|| Error::MetainfoError("invalid length".to_string()))?
                as u64;

            let file = FileInfo {
                path: vec![name.clone()],
                length,
            };

            (vec![file], length, true)
        } else if let Some(files_value) = dict.get(b"files" as &[u8]) {
            // Multi-file torrent
            let files_list = files_value
                .as_list()
                .ok_or_else(|| Error::MetainfoError("files must be a list".to_string()))?;

            let mut files = Vec::new();
            let mut total = 0u64;

            for file_value in files_list {
                let file_dict = file_value
                    .as_dict()
                    .ok_or_else(|| Error::MetainfoError("file must be a dictionary".to_string()))?;

                let length = file_dict
                    .get(b"length" as &[u8])
                    .and_then(|v| v.as_integer())
                    .ok_or_else(|| Error::MetainfoError("file missing length".to_string()))?
                    as u64;

                let path_list = file_dict
                    .get(b"path" as &[u8])
                    .and_then(|v| v.as_list())
                    .ok_or_else(|| Error::MetainfoError("file missing path".to_string()))?;

                let path: Vec<String> = path_list
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                if path.is_empty() {
                    return Err(Error::MetainfoError("empty file path".to_string()));
                }

                total += length;
                files.push(FileInfo { path, length });
            }

            (files, total, false)
        } else {
            return Err(Error::MetainfoError(
                "missing length or files field".to_string(),
            ));
        };

        Ok(Self {
            piece_length,
            pieces,
            piece_count,
            files,
            name,
            total_size,
            is_single_file,
        })
    }

    /// Get the SHA1 hash for a specific piece
    pub fn piece_hash(&self, index: usize) -> Option<&[u8]> {
        if index >= self.piece_count {
            return None;
        }

        let start = index * 20;
        let end = start + 20;
        Some(&self.pieces[start..end])
    }
}

impl Metainfo {
    /// Create a minimal Metainfo from a magnet link (for metadata exchange)
    ///
    /// This creates a stub Metainfo that can be used to start the torrent engine.
    /// The actual metadata will be fetched from peers via BEP 9.
    pub fn from_magnet(
        info_hash: [u8; 20],
        display_name: Option<String>,
        trackers: Vec<String>,
    ) -> Self {
        // Use first tracker as primary, rest as announce_list
        let announce = trackers
            .first()
            .map(|s| s.to_string())
            .unwrap_or_else(|| String::from(""));

        let announce_list = if trackers.len() > 1 {
            trackers.iter().skip(1).map(|t| vec![t.clone()]).collect()
        } else {
            Vec::new()
        };

        // Create a minimal TorrentInfo with placeholder data
        // This will be replaced once we fetch the actual metadata from peers
        let info = TorrentInfo {
            piece_length: 262144, // Default: 256KB pieces
            pieces: Vec::new(),   // Empty until we get metadata
            piece_count: 0,       // Unknown until metadata
            files: vec![FileInfo {
                path: vec![display_name
                    .clone()
                    .unwrap_or_else(|| "Unknown".to_string())],
                length: 0, // Unknown until metadata
            }],
            name: display_name.unwrap_or_else(|| hex::encode(&info_hash[..8])),
            total_size: 0,        // Unknown until metadata
            is_single_file: true, // Assume single file for now
        };

        Metainfo {
            announce,
            announce_list,
            info,
            info_hash,
            creation_date: None,
            comment: Some("Created from magnet link".to_string()),
            created_by: Some("SeedCore".to_string()),
        }
    }
}

/// Get file list with UI metadata for a torrent
/// TODO: Track per-file progress based on piece completion
pub fn get_file_list(metainfo: &Metainfo) -> Vec<FileInfoUI> {
    let mut files = Vec::new();

    for file in &metainfo.info.files {
        // Join path components with forward slash
        let path = file.path.join("/");

        files.push(FileInfoUI {
            path,
            size: file.length,
            downloaded: 0, // TODO: Calculate from piece completion
            priority: FilePriority::Normal,
            is_folder: false,
        });
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_file_torrent() {
        // Create a minimal single-file torrent using underscore naming
        let mut data = Vec::new();
        data.extend_from_slice(b"d"); // Start dictionary
        data.extend_from_slice(b"8:announce14:http://tracker"); // announce URL (14 bytes!)
        data.extend_from_slice(b"4:info"); // info key
        data.extend_from_slice(b"d"); // Start info dictionary
        data.extend_from_slice(b"6:lengthi1234e"); // length
        data.extend_from_slice(b"4:name9:test.file"); // name
        data.extend_from_slice(b"12:piece_lengthi16384e"); // piece_length is 12 bytes!
        data.extend_from_slice(b"6:pieces20:12345678901234567890"); // pieces
        data.extend_from_slice(b"e"); // End info dictionary
        data.extend_from_slice(b"e"); // End root dictionary

        let metainfo = Metainfo::from_bytes(&data).unwrap();

        assert_eq!(metainfo.announce, "http://tracker");
        assert_eq!(metainfo.info.name, "test.file");
        assert_eq!(metainfo.info.total_size, 1234);
        assert_eq!(metainfo.info.piece_length, 16384);
        assert_eq!(metainfo.info.piece_count, 1);
        assert!(metainfo.info.is_single_file);
        assert_eq!(metainfo.info.files.len(), 1);
    }

    #[test]
    fn test_piece_hash_extraction() {
        // Format with 2 pieces (40 bytes of hash data)
        let mut data = Vec::new();
        data.extend_from_slice(b"d");
        data.extend_from_slice(b"8:announce14:http://tracker"); // announce URL (14 bytes!)
        data.extend_from_slice(b"4:info");
        data.extend_from_slice(b"d");
        data.extend_from_slice(b"6:lengthi1234e");
        data.extend_from_slice(b"4:name4:test");
        data.extend_from_slice(b"12:piece_lengthi16384e"); // piece_length is 12 bytes!
        data.extend_from_slice(b"6:pieces40:1234567890123456789012345678901234567890");
        data.extend_from_slice(b"e");
        data.extend_from_slice(b"e");

        let metainfo = Metainfo::from_bytes(&data).unwrap();

        assert_eq!(metainfo.info.piece_count, 2);

        let piece0 = metainfo.info.piece_hash(0).unwrap();
        assert_eq!(piece0, b"12345678901234567890");

        let piece1 = metainfo.info.piece_hash(1).unwrap();
        assert_eq!(piece1, b"12345678901234567890");

        assert!(metainfo.info.piece_hash(2).is_none());
    }
}
