use super::{provider::DebridProvider, types::*, request_queue::RequestQueue};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use anyhow::{anyhow, Result};
use std::collections::HashMap;

const BASE_URL: &str = "https://api.real-debrid.com/rest/1.0";
const MIN_REQUEST_INTERVAL_MS: u64 = 240; // 250 requests/minute = ~240ms between requests

/// Real-Debrid API provider implementation
pub struct RealDebridProvider {
    api_key: String,
    client: Client,
    queue: RequestQueue,
}

impl RealDebridProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            queue: RequestQueue::new(MIN_REQUEST_INTERVAL_MS, "Real-Debrid".to_string()),
        }
    }

    /// Helper method to execute HTTP requests with rate limiting and retries
    async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url_base = format!("{}{}", BASE_URL, endpoint);
        let max_retries = 3;
        let mut retry_count = 0;

        loop {
            let url = url_base.clone();
            let api_key = self.api_key.clone();
            let client = self.client.clone();

            let result = self.queue
                .execute(async move {
                    let response = client
                        .get(&url)
                        .header("Authorization", format!("Bearer {}", api_key))
                        .send()
                        .await?;

                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        
                        // Retry on server errors (5xx) or too many requests (429) if somehow leaked
                        if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            return Err(anyhow!("Transient error {}: {}", status, error_text));
                        }
                        
                        return Err(anyhow!("Real-Debrid API error {}: {}", status, error_text));
                    }

                    Ok(response.json().await?)
                })
                .await;

            match result {
                Ok(val) => return Ok(val),
                Err(e) => {
                    let err_str = e.to_string();
                    // Retry on network errors or transient API errors
                    // "Transient error" is our own marker from above
                    // reqwest errors are also retried
                    let is_transient = err_str.contains("Transient error") 
                        || err_str.contains("connection closed") 
                        || err_str.contains("timed out")
                        || err_str.contains("connect error");

                    if is_transient && retry_count < max_retries {
                        retry_count += 1;
                        let delay = std::time::Duration::from_secs(2u64.pow(retry_count));
                        tracing::warn!("Real-Debrid request failed (attempt {}/{}), retrying in {:?}: {}", retry_count, max_retries, delay, e);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Helper method to execute POST requests with rate limiting and retries
    async fn post<T>(&self, endpoint: &str, form: Option<HashMap<String, String>>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url_base = format!("{}{}", BASE_URL, endpoint);
        let max_retries = 3;
        let mut retry_count = 0;

        loop {
            let url = url_base.clone();
            let api_key = self.api_key.clone();
            let client = self.client.clone();
            let form_data = form.clone();

            let result = self.queue
                .execute(async move {
                    let mut request = client
                        .post(&url)
                        .header("Authorization", format!("Bearer {}", api_key));

                    if let Some(data) = form_data {
                        request = request.form(&data);
                    }

                    let response = request.send().await?;

                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        
                        if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            return Err(anyhow!("Transient error {}: {}", status, error_text));
                        }

                        return Err(anyhow!("Real-Debrid API error {}: {}", status, error_text));
                    }

                    Ok(response.json().await?)
                })
                .await;

            match result {
                Ok(val) => return Ok(val),
                Err(e) => {
                    let err_str = e.to_string();
                    let is_transient = err_str.contains("Transient error") 
                        || err_str.contains("connection closed") 
                        || err_str.contains("timed out")
                        || err_str.contains("connect error");

                    if is_transient && retry_count < max_retries {
                        retry_count += 1;
                        let delay = std::time::Duration::from_secs(2u64.pow(retry_count));
                        tracing::warn!("Real-Debrid request failed (attempt {}/{}), retrying in {:?}: {}", retry_count, max_retries, delay, e);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Helper method to execute DELETE requests with rate limiting and retries
    async fn delete(&self, endpoint: &str) -> Result<()> {
        let url_base = format!("{}{}", BASE_URL, endpoint);
        let max_retries = 3;
        let mut retry_count = 0;

        loop {
            let url = url_base.clone();
            let api_key = self.api_key.clone();
            let client = self.client.clone();

            let result = self.queue
                .execute(async move {
                    let response = client
                        .delete(&url)
                        .header("Authorization", format!("Bearer {}", api_key))
                        .send()
                        .await?;

                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        
                        if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            return Err(anyhow!("Transient error {}: {}", status, error_text));
                        }

                        return Err(anyhow!("Real-Debrid API error {}: {}", status, error_text));
                    }

                    Ok(())
                })
                .await;

            match result {
                Ok(_) => return Ok(()),
                Err(e) => {
                    let err_str = e.to_string();
                    let is_transient = err_str.contains("Transient error") 
                        || err_str.contains("connection closed") 
                        || err_str.contains("timed out")
                        || err_str.contains("connect error");

                    if is_transient && retry_count < max_retries {
                        retry_count += 1;
                        let delay = std::time::Duration::from_secs(2u64.pow(retry_count));
                        tracing::warn!("Real-Debrid request failed (attempt {}/{}), retrying in {:?}: {}", retry_count, max_retries, delay, e);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }
}

// Real-Debrid API response types
#[derive(Debug, Deserialize)]
struct RDUser {
    id: i64,
    username: String,
    email: String,
    points: i64,
    premium: i64, // Unix timestamp
}

/// Real-Debrid instant availability response
/// Format: { "hash": { "rd": [{ "1": { "filename": "...", "filesize": 123 }, "2": {...} }] } }
#[derive(Debug, Deserialize)]
struct RDFileVariant {
    filename: String,
    filesize: u64,
}

type RDVariant = HashMap<String, RDFileVariant>; // file_id -> file info
type RDHostVariants = Vec<RDVariant>; // Array of variants
type RDHostAvailability = HashMap<String, RDHostVariants>; // "rd" -> variants
type RDInstantAvailability = HashMap<String, RDHostAvailability>; // hash -> host availability

#[derive(Debug, Deserialize)]
struct RDAddMagnetResponse {
    id: String,
    uri: String,
}

#[derive(Debug, Deserialize)]
struct RDTorrentInfo {
    id: String,
    filename: String,
    hash: String,
    bytes: u64,
    host: String,
    split: u64,
    progress: f64,
    status: String,
    added: String,
    files: Vec<RDFile>,
    links: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RDFile {
    id: u64,
    path: String,
    bytes: u64,
    selected: u64, // 0 or 1
}

#[derive(Debug, Deserialize)]
struct RDUnrestrictResponse {
    id: String,
    filename: String,
    filesize: u64,
    link: String,
    download: String,
}

#[async_trait]
impl DebridProvider for RealDebridProvider {
    fn provider_type(&self) -> DebridProviderType {
        DebridProviderType::RealDebrid
    }

    async fn validate_credentials(&self) -> Result<bool> {
        match self.get::<RDUser>("/user").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn check_instant_availability(&self, info_hash: &str) -> Result<CacheStatus> {
        // Real-Debrid instant availability endpoint
        let endpoint = format!("/torrents/instantAvailability/{}", info_hash.to_lowercase());
        
        let response: RDInstantAvailability = self.get(&endpoint).await?;
        
        // Check if our hash exists in the response
        let hash_lower = info_hash.to_lowercase();
        if let Some(host_availability) = response.get(&hash_lower) {
            // Check if "rd" (Real-Debrid) has cached this torrent
            if let Some(variants) = host_availability.get("rd") {
                if !variants.is_empty() {
                    // Get the first variant (usually the complete file set)
                    let first_variant = &variants[0];
                    
                    // Convert to CachedFile structs
                    let mut files = Vec::new();
                    for (file_id_str, file_info) in first_variant.iter() {
                        // Parse file ID from string
                        if let Ok(file_id) = file_id_str.parse::<usize>() {
                            files.push(CachedFile {
                                id: file_id,
                                name: file_info.filename.clone(),
                                size: file_info.filesize,
                                selected: false, // Not selected by default
                            });
                        }
                    }
                    
                    // Sort files by ID for consistent ordering
                    files.sort_by_key(|f| f.id);
                    
                    return Ok(CacheStatus::cached(files));
                }
            }
        }
        
        // Not cached
        Ok(CacheStatus::not_cached())
    }

    async fn add_magnet(&self, magnet: &str) -> Result<TorrentId> {
        let mut form = HashMap::new();
        form.insert("magnet".to_string(), magnet.to_string());

        let response: RDAddMagnetResponse = self.post("/torrents/addMagnet", Some(form)).await?;
        Ok(TorrentId {
            id: response.id,
            uri: Some(response.uri),
        })
    }

    async fn add_torrent_file(&self, torrent_data: &[u8]) -> Result<TorrentId> {
        let url = format!("{}/torrents/addTorrent", BASE_URL);
        let api_key = self.api_key.clone();
        let client = self.client.clone();
        let data = torrent_data.to_vec();

        self.queue
            .execute(async move {
                // Create multipart form with the torrent file
                let part = reqwest::multipart::Part::bytes(data)
                    .file_name("torrent.torrent")
                    .mime_str("application/x-bittorrent")?;

                let form = reqwest::multipart::Form::new()
                    .part("file", part);

                let response = client
                    .put(&url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .multipart(form)
                    .send()
                    .await?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_text = response.text().await.unwrap_or_default();
                    return Err(anyhow!("Real-Debrid API error {}: {}", status, error_text));
                }

                let result: RDAddMagnetResponse = response.json().await?;
                Ok(TorrentId {
                    id: result.id,
                    uri: Some(result.uri),
                })
            })
            .await
    }

    async fn select_files(&self, torrent_id: &str, file_indices: Vec<usize>) -> Result<()> {
        let files_str = if file_indices.is_empty() {
            "all".to_string()
        } else {
            file_indices
                .iter()
                .map(|i| (i + 1).to_string()) // Real-Debrid uses 1-based indexing
                .collect::<Vec<_>>()
                .join(",")
        };

        let mut form = HashMap::new();
        form.insert("files".to_string(), files_str);

        let endpoint = format!("/torrents/selectFiles/{}", torrent_id);
        let _: serde_json::Value = self.post(&endpoint, Some(form)).await?;
        Ok(())
    }

    async fn get_torrent_info(&self, torrent_id: &str) -> Result<DebridProgress> {
        let endpoint = format!("/torrents/info/{}", torrent_id);
        let info: RDTorrentInfo = self.get(&endpoint).await?;

        // Map Real-Debrid status to our DebridStatus enum
        let status = match info.status.as_str() {
            "waiting_files_selection" => DebridStatus::WaitingFilesSelection,
            "queued" => DebridStatus::Queued,
            "downloading" => DebridStatus::Downloading,
            "downloaded" => DebridStatus::Downloaded,
            "error" => DebridStatus::Error,
            "virus" => DebridStatus::Error, // Treat virus as error
            "dead" => DebridStatus::Dead,
            "magnet_conversion" => DebridStatus::MagnetConversion,
            "compressing" => DebridStatus::Compressing,
            "uploading" => DebridStatus::Uploading,
            _ => DebridStatus::Error, // Unknown states treated as errors
        };

        Ok(DebridProgress {
            torrent_id: info.id,
            status,
            progress: info.progress as f32,
            speed: 0, // Real-Debrid doesn't provide this in the response
            downloaded: (info.bytes as f64 * (info.progress / 100.0)) as u64,
            total_size: info.bytes,
            seeders: None,
            eta: None,
        })
    }

    async fn get_download_links(&self, torrent_id: &str) -> Result<Vec<DebridFile>> {
        let endpoint = format!("/torrents/info/{}", torrent_id);
        let info: RDTorrentInfo = self.get(&endpoint).await?;

        let mut debrid_files = Vec::new();

        for (idx, link) in info.links.iter().enumerate() {
            // Unrestrict each link to get the direct download URL
            let mut form = HashMap::new();
            form.insert("link".to_string(), link.to_string());

            match self.post::<RDUnrestrictResponse>("/unrestrict/link", Some(form)).await {
                Ok(unrestrict) => {
                    debrid_files.push(DebridFile {
                        id: idx.to_string(),
                        name: unrestrict.filename,
                        size: unrestrict.filesize,
                        download_link: Some(unrestrict.download.clone()),
                        stream_link: Some(unrestrict.download), // Real-Debrid uses same link for both
                        mime_type: None, // Real-Debrid doesn't provide MIME type directly
                    });
                }
                Err(e) => {
                    eprintln!("Failed to unrestrict link {}: {}", link, e);
                }
            }
        }

        Ok(debrid_files)
    }

    async fn unrestrict_link(&self, link: &str) -> Result<String> {
        let mut form = HashMap::new();
        form.insert("link".to_string(), link.to_string());

        let response: RDUnrestrictResponse = self.post("/unrestrict/link", Some(form)).await?;
        Ok(response.download)
    }

    async fn delete_torrent(&self, torrent_id: &str) -> Result<()> {
        let endpoint = format!("/torrents/delete/{}", torrent_id);
        self.delete(&endpoint).await
    }

    async fn list_torrents(&self) -> Result<Vec<DebridProgress>> {
        // Get list of all torrents (limited to 100 per request by default)
        let torrents: Vec<RDTorrentInfo> = self.get("/torrents").await?;

        let mut progress_list = Vec::new();
        for torrent in torrents {
            // Map Real-Debrid status to our DebridStatus enum
            let status = match torrent.status.as_str() {
                "waiting_files_selection" => DebridStatus::WaitingFilesSelection,
                "queued" => DebridStatus::Queued,
                "downloading" => DebridStatus::Downloading,
                "downloaded" => DebridStatus::Downloaded,
                "error" => DebridStatus::Error,
                "virus" => DebridStatus::Error,
                "dead" => DebridStatus::Dead,
                "magnet_conversion" => DebridStatus::MagnetConversion,
                "compressing" => DebridStatus::Compressing,
                "uploading" => DebridStatus::Uploading,
                _ => DebridStatus::Error,
            };

            progress_list.push(DebridProgress {
                torrent_id: torrent.id,
                status,
                progress: torrent.progress as f32,
                speed: 0, // Not provided in list view
                downloaded: (torrent.bytes as f64 * (torrent.progress / 100.0)) as u64,
                total_size: torrent.bytes,
                seeders: None,
                eta: None,
            });
        }

        Ok(progress_list)
    }

    async fn get_user_info(&self) -> Result<UserInfo> {
        let user: RDUser = self.get("/user").await?;

        Ok(UserInfo {
            username: user.username,
            email: Some(user.email),
            is_premium: user.premium > 0,
            premium_expires: if user.premium > 0 {
                Some(user.premium)
            } else {
                None
            },
            points: Some(user.points),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = RealDebridProvider::new("test_key".to_string());
        assert_eq!(provider.provider_type(), DebridProviderType::RealDebrid);
    }

    #[test]
    fn test_file_variant_parsing() {
        // Test parsing the instant availability response format
        let json = r#"{
            "abcdef1234567890": {
                "rd": [
                    {
                        "1": {"filename": "movie.mkv", "filesize": 1073741824},
                        "2": {"filename": "subtitle.srt", "filesize": 50000}
                    }
                ]
            }
        }"#;

        let parsed: RDInstantAvailability = serde_json::from_str(json).unwrap();
        assert!(parsed.contains_key("abcdef1234567890"));
        
        let host_avail = &parsed["abcdef1234567890"];
        assert!(host_avail.contains_key("rd"));
        
        let variants = &host_avail["rd"];
        assert_eq!(variants.len(), 1);
        
        let first_variant = &variants[0];
        assert!(first_variant.contains_key("1"));
        assert_eq!(first_variant["1"].filename, "movie.mkv");
        assert_eq!(first_variant["1"].filesize, 1073741824);
    }

    #[tokio::test]
    async fn test_cache_status_parsing() {
        // Test that CacheStatus is created correctly from response
        let provider = RealDebridProvider::new("test_key".to_string());
        
        // Note: This test would need a mock HTTP server to fully test
        // For now, we just verify the types are correct
        let cache_status = CacheStatus::cached(vec![
            CachedFile {
                id: 1,
                name: "test.mkv".to_string(),
                size: 1000000,
                selected: false,
            }
        ]);
        
        assert!(cache_status.is_cached);
        assert_eq!(cache_status.files.len(), 1);
        assert!(cache_status.instant_download);
    }

    #[test]
    fn test_torrent_info_response_parsing() {
        let json = r#"{
            "id": "ABC123",
            "filename": "test.torrent",
            "hash": "abcdef1234567890",
            "bytes": 1073741824,
            "host": "real-debrid.com",
            "split": 2000,
            "progress": 75,
            "status": "downloading",
            "added": "2024-01-01T12:00:00.000Z",
            "files": [
                {
                    "id": 1,
                    "path": "/movie.mkv",
                    "bytes": 1000000000,
                    "selected": 1
                }
            ],
            "links": [
                "https://example.com/link1"
            ]
        }"#;

        let parsed: RDTorrentInfo = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.id, "ABC123");
        assert_eq!(parsed.progress, 75.0);
        assert_eq!(parsed.status, "downloading");
        assert_eq!(parsed.bytes, 1073741824);
        assert_eq!(parsed.links.len(), 1);
    }

    #[test]
    fn test_user_info_parsing() {
        let json = r#"{
            "id": 42,
            "username": "testuser",
            "email": "test@example.com",
            "points": 10000,
            "premium": 1735689600
        }"#;

        let parsed: RDUser = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.username, "testuser");
        assert_eq!(parsed.email, "test@example.com");
        assert_eq!(parsed.points, 10000);
        assert!(parsed.premium > 0);
    }

    #[test]
    fn test_add_magnet_response_parsing() {
        let json = r#"{
            "id": "TORRENT123",
            "uri": "https://real-debrid.com/torrents/info/TORRENT123"
        }"#;

        let parsed: RDAddMagnetResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.id, "TORRENT123");
        assert_eq!(parsed.uri, "https://real-debrid.com/torrents/info/TORRENT123");
    }

    #[test]
    fn test_unrestrict_response_parsing() {
        let json = r#"{
            "id": "DL123",
            "filename": "movie.mkv",
            "filesize": 1073741824,
            "link": "https://hoster.com/file123",
            "download": "https://real-debrid.com/download/xyz"
        }"#;

        let parsed: RDUnrestrictResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.filename, "movie.mkv");
        assert_eq!(parsed.filesize, 1073741824);
        assert_eq!(parsed.download, "https://real-debrid.com/download/xyz");
    }
}
