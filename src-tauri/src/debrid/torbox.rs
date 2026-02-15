use super::{provider::DebridProvider, types::*, request_queue::RequestQueue};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use anyhow::{anyhow, Result};

const BASE_URL: &str = "https://api.torbox.app/v1/api";
const MIN_REQUEST_INTERVAL_MS: u64 = 200; // Conservative rate limit

/// Torbox API provider implementation
pub struct TorboxProvider {
    api_key: String,
    client: Client,
    queue: RequestQueue,
}

impl TorboxProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            queue: RequestQueue::new(MIN_REQUEST_INTERVAL_MS, "Torbox".to_string()),
        }
    }

    /// Helper method to execute HTTP requests with rate limiting and retries
    async fn get<T>(&self, endpoint: &str, params: Option<&[(&str, &str)]>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url_base = format!("{}{}", BASE_URL, endpoint);
        // Convert params to owned Vec for cloning
        let query_params_base = params.map(|p| p.to_vec());
        
        let max_retries = 3;
        let mut retry_count = 0;

        loop {
            let url = url_base.clone();
            let api_key = self.api_key.clone();
            let client = self.client.clone();
            let query_params = query_params_base.clone();

            let result = self.queue
                .execute(async move {
                    let mut request = client
                        .get(&url)
                        .header("Authorization", format!("Bearer {}", api_key));

                    if let Some(params) = query_params {
                        request = request.query(&params);
                    }

                    let response = request.send().await?;

                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        
                        if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            return Err(anyhow!("Transient error {}: {}", status, error_text));
                        }

                        return Err(anyhow!("Torbox API error {}: {}", status, error_text));
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
                        tracing::warn!("Torbox request failed (attempt {}/{}), retrying in {:?}: {}", retry_count, max_retries, delay, e);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Helper method to execute POST requests with rate limiting and retries
    async fn post<T>(&self, endpoint: &str, json_body: Option<serde_json::Value>) -> Result<T>
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
            let body = json_body.clone();

            let result = self.queue
                .execute(async move {
                    let mut request = client
                        .post(&url)
                        .header("Authorization", format!("Bearer {}", api_key));

                    if let Some(b) = body {
                        request = request.json(&b);
                    }

                    let response = request.send().await?;

                    if !response.status().is_success() {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        
                        if status.is_server_error() || status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                            return Err(anyhow!("Transient error {}: {}", status, error_text));
                        }

                        return Err(anyhow!("Torbox API error {}: {}", status, error_text));
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
                        tracing::warn!("Torbox request failed (attempt {}/{}), retrying in {:?}: {}", retry_count, max_retries, delay, e);
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

                        return Err(anyhow!("Torbox API error {}: {}", status, error_text));
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
                        tracing::warn!("Torbox request failed (attempt {}/{}), retrying in {:?}: {}", retry_count, max_retries, delay, e);
                        tokio::time::sleep(delay).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }
}

// Torbox API response types
#[derive(Debug, Deserialize)]
struct TorboxResponse<T> {
    data: Option<T>,
    error: Option<String>,
    detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TorboxDownload {
    id: i64,
    name: String,
    hash: Option<String>,
    #[serde(default)]
    cached: bool,
    #[serde(default)]
    files: Vec<TorboxFile>,
}

#[derive(Debug, Deserialize)]
struct TorboxFile {
    id: i64,
    #[serde(default)]
    short_name: String,
    #[serde(default)]
    name: String,
    size: u64,
    #[serde(default)]
    mimetype: String,
}

#[async_trait]
impl DebridProvider for TorboxProvider {
    fn provider_type(&self) -> DebridProviderType {
        DebridProviderType::Torbox
    }

    async fn validate_credentials(&self) -> Result<bool> {
        // Try user info endpoint first - more reliable for validation
        tracing::info!("Validating Torbox credentials with /user/me endpoint");
        
        // Try the user/me endpoint which should be simpler and more reliable
        match self.get::<serde_json::Value>("/user/me", None).await {
            Ok(response) => {
                tracing::info!("Torbox /user/me succeeded: {:?}", response);
                Ok(true)
            }
            Err(e) => {
                tracing::warn!("Torbox /user/me failed ({}), trying /torrents/mylist", e);
                
                // Fallback to torrents list
                match self.get::<TorboxResponse<Vec<TorboxDownload>>>(
                    "/torrents/mylist",
                    Some(&[("limit", "1"), ("offset", "0")]),
                ).await {
                    Ok(_response) => {
                        tracing::info!("Torbox /torrents/mylist succeeded");
                        // Even if the response has data: null, it means the API key is valid
                        // (just no torrents in the account)
                        Ok(true)
                    }
                    Err(e) => {
                        tracing::error!("Torbox validation failed on both endpoints: {}", e);
                        Ok(false)
                    }
                }
            }
        }
    }

    async fn check_instant_availability(&self, info_hash: &str) -> Result<CacheStatus> {
        // Torbox doesn't have a direct instant availability check API
        // We check if the hash exists in the user's downloads
        let response: TorboxResponse<Vec<TorboxDownload>> = self.get(
            "/torrents/mylist",
            Some(&[("limit", "1000"), ("offset", "0"), ("bypass_cache", "true")]),
        ).await?;

        if let Some(downloads) = response.data {
            for download in downloads {
                if let Some(hash) = &download.hash {
                    if hash.eq_ignore_ascii_case(info_hash) {
                        // Found the torrent - it's cached
                        let files = download.files.into_iter().enumerate().map(|(idx, file)| {
                            CachedFile {
                                id: idx,
                                name: if !file.short_name.is_empty() {
                                    file.short_name
                                } else {
                                    file.name
                                },
                                size: file.size,
                                selected: false,
                            }
                        }).collect();

                        return Ok(CacheStatus::cached(files));
                    }
                }
            }
        }

        // Not found in user's downloads
        Ok(CacheStatus::not_cached())
    }

    async fn add_magnet(&self, _magnet: &str) -> Result<TorrentId> {
        // Torbox API doesn't expose torrent adding via their documented endpoints
        // This would require different API endpoints that aren't in the media center
        Err(anyhow!("Torbox add_magnet not yet implemented - API endpoint not documented"))
    }

    async fn add_torrent_file(&self, _torrent_data: &[u8]) -> Result<TorrentId> {
        // Torbox API doesn't expose torrent file upload via their documented endpoints
        Err(anyhow!("Torbox add_torrent_file not yet implemented - API endpoint not documented"))
    }

    async fn select_files(&self, _torrent_id: &str, _file_indices: Vec<usize>) -> Result<()> {
        // Torbox doesn't require file selection - all files are available
        Ok(())
    }

    async fn get_torrent_info(&self, torrent_id: &str) -> Result<DebridProgress> {
        // Get torrent info from the list
        let response: TorboxResponse<Vec<TorboxDownload>> = self.get(
            "/torrents/mylist",
            Some(&[("limit", "1000"), ("offset", "0"), ("bypass_cache", "true")]),
        ).await?;

        if let Some(downloads) = response.data {
            // Parse torrent_id as i64
            let id: i64 = torrent_id.parse()
                .map_err(|_| anyhow!("Invalid torrent ID format"))?;

            for download in downloads {
                if download.id == id {
                    // Torbox doesn't provide detailed progress info
                    // If cached, assume it's downloaded
                    let status = if download.cached {
                        DebridStatus::Downloaded
                    } else {
                        DebridStatus::Downloading
                    };

                    let total_size: u64 = download.files.iter().map(|f| f.size).sum();

                    return Ok(DebridProgress {
                        torrent_id: download.id.to_string(),
                        status,
                        progress: if download.cached { 100.0 } else { 0.0 },
                        speed: 0,
                        downloaded: if download.cached { total_size } else { 0 },
                        total_size,
                        seeders: None,
                        eta: None,
                    });
                }
            }
        }

        Err(anyhow!("Torrent not found"))
    }

    async fn get_download_links(&self, torrent_id: &str) -> Result<Vec<DebridFile>> {
        // Get download from list
        let response: TorboxResponse<Vec<TorboxDownload>> = self.get(
            "/torrents/mylist",
            Some(&[("limit", "1000"), ("offset", "0")]),
        ).await?;

        if let Some(downloads) = response.data {
            let id: i64 = torrent_id.parse()
                .map_err(|_| anyhow!("Invalid torrent ID format"))?;

            for download in downloads {
                if download.id == id {
                    let mut files = Vec::new();

                    for file in download.files {
                        // Construct download URL
                        let download_url = format!(
                            "{}/torrents/requestdl?token={}&torrent_id={}&file_id={}&redirect=true",
                            BASE_URL, self.api_key, download.id, file.id
                        );

                        files.push(DebridFile {
                            id: file.id.to_string(),
                            name: if !file.short_name.is_empty() {
                                file.short_name
                            } else {
                                file.name
                            },
                            size: file.size,
                            download_link: Some(download_url.clone()),
                            stream_link: Some(download_url), // Same URL for streaming
                            mime_type: if !file.mimetype.is_empty() {
                                Some(file.mimetype)
                            } else {
                                None
                            },
                        });
                    }

                    return Ok(files);
                }
            }
        }

        Err(anyhow!("Torrent not found"))
    }

    async fn unrestrict_link(&self, _link: &str) -> Result<String> {
        // Torbox doesn't have a generic link unrestriction feature like Real-Debrid
        Err(anyhow!("Torbox doesn't support generic link unrestriction"))
    }

    async fn delete_torrent(&self, _torrent_id: &str) -> Result<()> {
        // Torbox API doesn't expose torrent deletion via their documented endpoints
        Err(anyhow!("Torbox delete_torrent not yet implemented - API endpoint not documented"))
    }

    async fn list_torrents(&self) -> Result<Vec<DebridProgress>> {
        let response: TorboxResponse<Vec<TorboxDownload>> = self.get(
            "/torrents/mylist",
            Some(&[("limit", "1000"), ("offset", "0"), ("bypass_cache", "true")]),
        ).await?;

        let mut progress_list = Vec::new();

        if let Some(downloads) = response.data {
            for download in downloads {
                let status = if download.cached {
                    DebridStatus::Downloaded
                } else {
                    DebridStatus::Downloading
                };

                let total_size: u64 = download.files.iter().map(|f| f.size).sum();

                progress_list.push(DebridProgress {
                    torrent_id: download.id.to_string(),
                    status,
                    progress: if download.cached { 100.0 } else { 0.0 },
                    speed: 0,
                    downloaded: if download.cached { total_size } else { 0 },
                    total_size,
                    seeders: None,
                    eta: None,
                });
            }
        }

        Ok(progress_list)
    }

    async fn get_user_info(&self) -> Result<UserInfo> {
        // Torbox doesn't provide user info in their documented endpoints
        // Return minimal info
        Ok(UserInfo {
            username: "Torbox User".to_string(),
            email: None,
            is_premium: true, // Assume premium if API key works
            premium_expires: None,
            points: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = TorboxProvider::new("test_key".to_string());
        assert_eq!(provider.provider_type(), DebridProviderType::Torbox);
    }

    #[test]
    fn test_torbox_download_parsing() {
        let json = r#"{
            "data": [{
                "id": 123,
                "name": "My Torrent",
                "hash": "abcdef1234567890",
                "cached": true,
                "files": [{
                    "id": 456,
                    "short_name": "video.mkv",
                    "name": "folder/video.mkv",
                    "size": 1073741824,
                    "mimetype": "video/x-matroska"
                }]
            }]
        }"#;

        let parsed: TorboxResponse<Vec<TorboxDownload>> = serde_json::from_str(json).unwrap();
        assert!(parsed.data.is_some());
        
        let downloads = parsed.data.unwrap();
        assert_eq!(downloads.len(), 1);
        assert_eq!(downloads[0].id, 123);
        assert_eq!(downloads[0].name, "My Torrent");
        assert!(downloads[0].cached);
        assert_eq!(downloads[0].files.len(), 1);
        assert_eq!(downloads[0].files[0].short_name, "video.mkv");
    }
}
