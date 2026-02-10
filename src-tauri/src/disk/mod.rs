/// Disk I/O manager for reading and writing torrent pieces
/// Handles both single-file and multi-file torrents
use crate::torrent::Metainfo;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use std::io::SeekFrom;

/// A write request for the disk manager
#[derive(Debug)]
pub struct WriteRequest {
    pub piece_index: usize,
    pub data: Vec<u8>,
}



/// Information about a file in the torrent
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub length: u64,
    pub offset: u64, // Byte offset from start of torrent
}

/// Manages disk I/O operations for torrents
pub struct DiskManager {
    /// Root directory for downloads
    download_dir: PathBuf,
    /// Information about each file in the torrent
    files: Vec<FileInfo>,
    /// Piece length in bytes
    piece_length: usize,
    /// Total size of torrent in bytes
    total_size: u64,
    /// Queue of pending write operations
    write_queue: VecDeque<WriteRequest>,
    /// Maximum number of queued writes before applying backpressure
    max_queue_size: usize,
}

impl DiskManager {
    /// Create a new disk manager from torrent metainfo
    pub fn new(metainfo: &Metainfo, download_dir: PathBuf) -> Self {
        let files = Self::build_file_list(metainfo, &download_dir);
        let total_size = metainfo.info.total_size;

        Self {
            download_dir,
            files,
            piece_length: metainfo.info.piece_length as usize,
            total_size,
            write_queue: VecDeque::new(),
            max_queue_size: 100,
        }
    }

    /// Build list of files with their absolute paths and byte offsets
    fn build_file_list(metainfo: &Metainfo, download_dir: &Path) -> Vec<FileInfo> {
        let mut files = Vec::new();
        let mut offset = 0u64;

        if metainfo.info.is_single_file {
            // Single file torrent
            let path = download_dir.join(&metainfo.info.name);
            files.push(FileInfo {
                path,
                length: metainfo.info.total_size,
                offset,
            });
        } else {
            // Multi-file torrent
            let torrent_dir = download_dir.join(&metainfo.info.name);
            
            for file_info in &metainfo.info.files {
                let file_path = file_info.path.iter().fold(
                    torrent_dir.clone(),
                    |acc, component| acc.join(component)
                );
                
                files.push(FileInfo {
                    path: file_path,
                    length: file_info.length,
                    offset,
                });
                
                offset += file_info.length;
            }
        }

        files
    }

    /// Pre-allocate all files for the torrent
    pub async fn allocate_files(&self) -> Result<(), String> {
        for file_info in &self.files {
            // Create parent directories
            if let Some(parent) = file_info.path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            // Create/open file
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&file_info.path)
                .await
                .map_err(|e| format!("Failed to create file {:?}: {}", file_info.path, e))?;

            // Set file length (pre-allocate space)
            file.set_len(file_info.length)
                .await
                .map_err(|e| format!("Failed to allocate file space: {}", e))?;

            tracing::info!(
                "Allocated file: {:?} ({} bytes)",
                file_info.path,
                file_info.length
            );
        }

        Ok(())
    }

    /// Write a piece to disk
    pub async fn write_piece(&mut self, piece_index: usize, data: Vec<u8>) -> Result<(), String> {
        let piece_offset = (piece_index * self.piece_length) as u64;
        let piece_size = data.len() as u64;

        // Find which file(s) this piece spans
        let files_to_write = self.get_files_for_range(piece_offset, piece_size);

        let mut data_offset = 0usize;
        
        for (file_info, file_offset, write_size) in files_to_write {
            let mut file = OpenOptions::new()
                .write(true)
                .open(&file_info.path)
                .await
                .map_err(|e| format!("Failed to open file {:?}: {}", file_info.path, e))?;

            // Seek to the correct position
            file.seek(SeekFrom::Start(file_offset))
                .await
                .map_err(|e| format!("Failed to seek in file: {}", e))?;

            // Write the data chunk
            let chunk = &data[data_offset..data_offset + write_size];
            file.write_all(chunk)
                .await
                .map_err(|e| format!("Failed to write to file: {}", e))?;

            // Ensure data is flushed to disk
            file.flush()
                .await
                .map_err(|e| format!("Failed to flush file: {}", e))?;

            data_offset += write_size;
        }

        tracing::debug!(
            "Wrote piece {} ({} bytes) to disk",
            piece_index,
            piece_size
        );

        Ok(())
    }

    /// Read a piece from disk
    pub async fn read_piece(&self, piece_index: usize) -> Result<Vec<u8>, String> {
        let piece_offset = (piece_index * self.piece_length) as u64;
        
        // Calculate piece size (last piece may be smaller)
        let piece_size = if piece_offset + self.piece_length as u64 > self.total_size {
            (self.total_size - piece_offset) as usize
        } else {
            self.piece_length
        };

        let mut piece_data = vec![0u8; piece_size];
        let files_to_read = self.get_files_for_range(piece_offset, piece_size as u64);

        let mut data_offset = 0usize;

        for (file_info, file_offset, read_size) in files_to_read {
            let mut file = File::open(&file_info.path)
                .await
                .map_err(|e| format!("Failed to open file {:?}: {}", file_info.path, e))?;

            // Seek to the correct position
            file.seek(SeekFrom::Start(file_offset))
                .await
                .map_err(|e| format!("Failed to seek in file: {}", e))?;

            // Read the data chunk
            let chunk = &mut piece_data[data_offset..data_offset + read_size];
            file.read_exact(chunk)
                .await
                .map_err(|e| format!("Failed to read from file: {}", e))?;

            data_offset += read_size;
        }

        Ok(piece_data)
    }

    /// Queue a write operation (for batching)
    pub fn queue_write(&mut self, piece_index: usize, data: Vec<u8>) -> Result<(), String> {
        if self.write_queue.len() >= self.max_queue_size {
            return Err("Write queue is full".to_string());
        }

        self.write_queue.push_back(WriteRequest { piece_index, data });
        Ok(())
    }

    /// Flush all queued writes to disk
    pub async fn flush_writes(&mut self) -> Result<(), String> {
        while let Some(write_req) = self.write_queue.pop_front() {
            self.write_piece(write_req.piece_index, write_req.data).await?;
        }
        Ok(())
    }

    /// Get which files a byte range spans
    /// Returns: Vec<(FileInfo, offset_in_file, bytes_to_read)>
    fn get_files_for_range(&self, offset: u64, size: u64) -> Vec<(&FileInfo, u64, usize)> {
        let mut result = Vec::new();
        let end_offset = offset + size;

        for file_info in &self.files {
            let file_start = file_info.offset;
            let file_end = file_info.offset + file_info.length;

            // Check if this file overlaps with the requested range
            if file_start < end_offset && file_end > offset {
                let read_start = offset.saturating_sub(file_start);
                let read_end = std::cmp::min(end_offset - file_start, file_info.length);
                let read_size = (read_end - read_start) as usize;

                result.push((file_info, read_start, read_size));
            }
        }

        result
    }

    /// Get total size of torrent in bytes
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Get piece length
    pub fn piece_length(&self) -> usize {
        self.piece_length
    }

    /// Get number of pieces
    pub fn num_pieces(&self) -> usize {
        ((self.total_size + self.piece_length as u64 - 1) / self.piece_length as u64) as usize
    }

    /// Get list of files
    pub fn files(&self) -> &[FileInfo] {
        &self.files
    }

    /// Check if all files exist
    pub async fn files_exist(&self) -> bool {
        for file_info in &self.files {
            if !tokio::fs::try_exists(&file_info.path).await.unwrap_or(false) {
                return false;
            }
        }
        true
    }

    /// Delete all files associated with this torrent
    pub async fn delete_files(&self) -> Result<(), String> {
        for file_info in &self.files {
            tokio::fs::remove_file(&file_info.path)
                .await
                .map_err(|e| format!("Failed to delete file {:?}: {}", file_info.path, e))?;
        }

        // Try to remove empty directories
        if let Some(first_file) = self.files.first() {
            if let Some(parent) = first_file.path.parent() {
                let _ = tokio::fs::remove_dir_all(parent).await;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::torrent::{TorrentInfo, FileInfo as TorrentFileInfo};

    fn create_test_metainfo_single() -> Metainfo {
        Metainfo {
            announce: "http://tracker.example.com".to_string(),
            announce_list: vec![],
            info: TorrentInfo {
                piece_length: 16384,
                pieces: vec![0u8; 40], // 2 pieces * 20 bytes
                piece_count: 2,
                files: vec![TorrentFileInfo {
                    path: vec!["test_file.txt".to_string()],
                    length: 20000,
                }],
                name: "test_file.txt".to_string(),
                total_size: 20000,
                is_single_file: true,
            },
            info_hash: [0u8; 20],
            creation_date: None,
            comment: None,
            created_by: None,
        }
    }

    fn create_test_metainfo_multi() -> Metainfo {
        Metainfo {
            announce: "http://tracker.example.com".to_string(),
            announce_list: vec![],
            info: TorrentInfo {
                piece_length: 16384,
                pieces: vec![0u8; 40],
                piece_count: 2,
                files: vec![
                    TorrentFileInfo {
                        path: vec!["file1.txt".to_string()],
                        length: 10000,
                    },
                    TorrentFileInfo {
                        path: vec!["subdir".to_string(), "file2.txt".to_string()],
                        length: 10000,
                    },
                ],
                name: "test_torrent".to_string(),
                total_size: 20000,
                is_single_file: false,
            },
            info_hash: [0u8; 20],
            creation_date: None,
            comment: None,
            created_by: None,
        }
    }

    #[test]
    fn test_build_file_list_single() {
        let metainfo = create_test_metainfo_single();
        let download_dir = PathBuf::from("/tmp/downloads");
        let files = DiskManager::build_file_list(&metainfo, &download_dir);

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("/tmp/downloads/test_file.txt"));
        assert_eq!(files[0].length, 20000);
        assert_eq!(files[0].offset, 0);
    }

    #[test]
    fn test_build_file_list_multi() {
        let metainfo = create_test_metainfo_multi();
        let download_dir = PathBuf::from("/tmp/downloads");
        let files = DiskManager::build_file_list(&metainfo, &download_dir);

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, PathBuf::from("/tmp/downloads/test_torrent/file1.txt"));
        assert_eq!(files[0].length, 10000);
        assert_eq!(files[0].offset, 0);

        assert_eq!(files[1].path, PathBuf::from("/tmp/downloads/test_torrent/subdir/file2.txt"));
        assert_eq!(files[1].length, 10000);
        assert_eq!(files[1].offset, 10000);
    }

    #[test]
    fn test_get_files_for_range_single_file() {
        let metainfo = create_test_metainfo_single();
        let download_dir = PathBuf::from("/tmp/downloads");
        let dm = DiskManager::new(&metainfo, download_dir);

        // Read first 1000 bytes
        let files = dm.get_files_for_range(0, 1000);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].1, 0); // offset in file
        assert_eq!(files[0].2, 1000); // bytes to read
    }

    #[test]
    fn test_get_files_for_range_multi_file() {
        let metainfo = create_test_metainfo_multi();
        let download_dir = PathBuf::from("/tmp/downloads");
        let dm = DiskManager::new(&metainfo, download_dir);

        // Read bytes 9000-11000 (spans both files)
        let files = dm.get_files_for_range(9000, 2000);
        assert_eq!(files.len(), 2);
        
        // First file: read from offset 9000, size 1000 bytes
        assert_eq!(files[0].1, 9000);
        assert_eq!(files[0].2, 1000);
        
        // Second file: read from offset 0, size 1000 bytes
        assert_eq!(files[1].1, 0);
        assert_eq!(files[1].2, 1000);
    }

    #[test]
    fn test_disk_manager_properties() {
        let metainfo = create_test_metainfo_single();
        let download_dir = PathBuf::from("/tmp/downloads");
        let dm = DiskManager::new(&metainfo, download_dir);

        assert_eq!(dm.total_size(), 20000);
        assert_eq!(dm.piece_length(), 16384);
        assert_eq!(dm.num_pieces(), 2);
    }

    #[tokio::test]
    async fn test_write_and_read_piece() {
        let metainfo = create_test_metainfo_single();
        let download_dir = PathBuf::from("/tmp/seedcore_test");
        let mut dm = DiskManager::new(&metainfo, download_dir.clone());

        // Allocate files
        dm.allocate_files().await.unwrap();

        // Write piece 0
        let piece_data = vec![42u8; 16384];
        dm.write_piece(0, piece_data.clone()).await.unwrap();

        // Read it back
        let read_data = dm.read_piece(0).await.unwrap();
        assert_eq!(read_data, piece_data);

        // Cleanup
        dm.delete_files().await.unwrap();
        let _ = tokio::fs::remove_dir_all(download_dir).await;
    }

    #[tokio::test]
    async fn test_queue_and_flush_writes() {
        let metainfo = create_test_metainfo_single();
        let download_dir = PathBuf::from("/tmp/seedcore_test_queue");
        let mut dm = DiskManager::new(&metainfo, download_dir.clone());

        dm.allocate_files().await.unwrap();

        // Queue multiple writes
        let piece0 = vec![1u8; 16384];
        let piece1 = vec![2u8; 3616]; // Last piece is smaller

        dm.queue_write(0, piece0.clone()).unwrap();
        dm.queue_write(1, piece1.clone()).unwrap();

        // Flush to disk
        dm.flush_writes().await.unwrap();

        // Verify
        let read0 = dm.read_piece(0).await.unwrap();
        let read1 = dm.read_piece(1).await.unwrap();
        
        assert_eq!(read0, piece0);
        assert_eq!(read1, piece1);

        // Cleanup
        dm.delete_files().await.unwrap();
        let _ = tokio::fs::remove_dir_all(download_dir).await;
    }
}
