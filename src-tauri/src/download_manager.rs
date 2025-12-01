use crate::nxm_protocol::NxmUrl;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, Semaphore};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: String,
    pub nxm_url: NxmUrl,
    pub mod_name: Option<String>,
    pub file_name: String,
    pub status: DownloadStatus,
    pub file_path: Option<PathBuf>,
    pub bytes_downloaded: u64,
    pub bytes_total: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed { error: String },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub download_id: String,
    pub bytes_downloaded: u64,
    pub bytes_total: Option<u64>,
    pub speed_bps: u64,
    pub eta_seconds: Option<u64>,
    pub progress_percent: f64,
}

#[derive(Clone)]
pub struct DownloadManager {
    queue: Arc<Mutex<VecDeque<DownloadTask>>>,
    active: Arc<Mutex<HashMap<String, DownloadTask>>>,
    semaphore: Arc<Semaphore>,
    download_dir: PathBuf,
    app_handle: AppHandle,
    client: Client,
}

impl DownloadManager {
    pub fn new(app_handle: AppHandle, download_dir: PathBuf, max_concurrent: usize) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(300)) // 5 minutes timeout
            .build()
            .unwrap();

        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            active: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            download_dir,
            app_handle,
            client,
        }
    }

    /// Add a download to the queue
    pub async fn add_to_queue(&self, nxm_url: NxmUrl) -> Result<String, String> {
        // Check if mod is already installed
        let settings = crate::settings::Settings::load(&self.app_handle)
            .map_err(|e| format!("Failed to load settings: {}", e))?;
        
        if !settings.game_path.is_empty() {
            let game_path = PathBuf::from(&settings.game_path);
            let installed_mods = crate::mod_installer::scan_mods(&game_path);
            
            for mod_info in installed_mods {
                if let (Some(mid), Some(fid)) = (mod_info.nexus_mod_id, mod_info.nexus_file_id) {
                    if mid == nxm_url.mod_id && fid == nxm_url.file_id {
                        return Err(format!(
                            "Mod '{}' (version {}) is already installed and up to date.",
                            mod_info.name, mod_info.version
                        ));
                    }
                }
            }
        }

        let download_id = Uuid::new_v4().to_string();

        // Generate filename from mod_id and file_id
        let file_name = format!("mod_{}_file_{}.zip", nxm_url.mod_id, nxm_url.file_id);

        let task = DownloadTask {
            id: download_id.clone(),
            nxm_url: nxm_url.clone(),
            mod_name: None, // Will be fetched later if needed
            file_name: file_name.clone(),
            status: DownloadStatus::Queued,
            file_path: None,
            bytes_downloaded: 0,
            bytes_total: None,
        };

        // Add to queue
        {
            let mut queue = self.queue.lock().await;
            queue.push_back(task.clone());
        }

        // Emit event to frontend
        let _ = self.app_handle.emit("download-queued", &task);

        // Start processing if permits available
        // Start processing if permits available
        self.start_next_download();

        Ok(download_id)
    }

    /// Start the next download from the queue if permits available
    /// Start the next download from the queue if permits available
    fn start_next_download(&self) {
        let manager = self.clone();
        tokio::spawn(async move {
            manager.process_next_download().await;
        });
    }

    /// Internal async function to process the next download
    async fn process_next_download(&self) {
        // Try to acquire a permit without blocking
        if let Ok(permit) = self.semaphore.clone().try_acquire_owned() {
            // Get next queued download
            let task = {
                let mut queue = self.queue.lock().await;
                queue.iter_mut().find(|t| matches!(t.status, DownloadStatus::Queued)).cloned()
            };

            if let Some(mut task) = task {
                task.status = DownloadStatus::Downloading;

                // Move to active
                {
                    let mut active = self.active.lock().await;
                    active.insert(task.id.clone(), task.clone());
                }

                // Update queue status
                {
                    let mut queue = self.queue.lock().await;
                    if let Some(t) = queue.iter_mut().find(|t| t.id == task.id) {
                        t.status = DownloadStatus::Downloading;
                    }
                }

                // Spawn download task
                let manager = DownloadManagerHandle {
                    queue: self.queue.clone(),
                    active: self.active.clone(),
                    download_dir: self.download_dir.clone(),
                    app_handle: self.app_handle.clone(),
                    client: self.client.clone(),
                };

                // Clone self to trigger next download
                let next_trigger = self.clone();

                tokio::spawn(async move {
                    let result = manager.execute_download(task.clone()).await;

                    // Release permit when done
                    drop(permit);

                    // Handle completion
                    match result {
                        Ok(file_path) => {
                            manager.complete_download(task.id, file_path).await;
                        }
                        Err(e) => {
                            manager.fail_download(task.id, e).await;
                        }
                    }

                    // Try to start next download
                    next_trigger.start_next_download();
                });
            }
        }
    }

    /// Get current queue state
    pub async fn get_queue_state(&self) -> Vec<DownloadTask> {
        let queue = self.queue.lock().await;
        queue.iter().cloned().collect()
    }

    /// Cancel a download
    pub async fn cancel_download(&self, download_id: &str) -> Result<(), String> {
        // Remove from queue if queued
        {
            let mut queue = self.queue.lock().await;
            if let Some(pos) = queue.iter().position(|t| t.id == download_id) {
                queue.remove(pos);
                let _ = self.app_handle.emit("download-cancelled", download_id);
                return Ok(());
            }
        }

        // If active, we need to implement cancellation token (TODO for now)
        // For now, just mark as failed
        {
            let mut active = self.active.lock().await;
            if let Some(task) = active.get_mut(download_id) {
                task.status = DownloadStatus::Failed {
                    error: "Cancelled by user".to_string(),
                };
            }
        }

        let _ = self.app_handle.emit("download-cancelled", download_id);
        Ok(())
    }

    /// Remove completed/failed downloads from queue
    pub async fn clear_completed(&self) -> Result<(), String> {
        let mut queue = self.queue.lock().await;
        queue.retain(|t| !matches!(t.status, DownloadStatus::Completed | DownloadStatus::Failed { .. }));
        Ok(())
    }
}

/// Helper struct for executing downloads (can be cloned and sent to tokio tasks)
#[derive(Clone)]
struct DownloadManagerHandle {
    queue: Arc<Mutex<VecDeque<DownloadTask>>>,
    active: Arc<Mutex<HashMap<String, DownloadTask>>>,
    download_dir: PathBuf,
    app_handle: AppHandle,
    client: Client,
}

impl DownloadManagerHandle {
    async fn execute_download(&self, task: DownloadTask) -> Result<PathBuf, String> {
        // Load Nexus Mods API key from settings
        let settings = crate::settings::Settings::load(&self.app_handle)
            .map_err(|e| format!("Failed to load settings: {}", e))?;

        if settings.nexus_api_key.is_empty() {
            return Err("Nexus Mods API key not configured. Please add your API key in Settings.".to_string());
        }

        // Step 1: Get the actual download link from Nexus Mods API
        let api_url = format!(
            "https://api.nexusmods.com/v1/games/{}/mods/{}/files/{}/download_link.json",
            task.nxm_url.game,
            task.nxm_url.mod_id,
            task.nxm_url.file_id
        );

        println!("🔍 Fetching download link from API: {}", api_url);
        println!("   Key: {}", task.nxm_url.key);
        println!("   Expires: {:?}", task.nxm_url.expires);
        println!("   User ID: {:?}", task.nxm_url.user_id);
        println!("   Using Nexus API key: {}...", &settings.nexus_api_key.chars().take(8).collect::<String>());

        // Call API to get download link
        // Build query parameters
        let mut query_params = vec![
            ("key", task.nxm_url.key.clone()),
            ("expires", task.nxm_url.expires.unwrap_or(0).to_string()),
        ];

        // Add user_id if present
        if let Some(user_id) = task.nxm_url.user_id {
            query_params.push(("user_id", user_id.to_string()));
        }

        println!("   📋 Query parameters: {:?}", query_params);

        let api_response = self
            .client
            .get(&api_url)
            .query(&query_params)
            .header("User-Agent", "Treasure Chest Mod Manager/0.1.0")
            .header("apikey", settings.nexus_api_key.clone())
            .send()
            .await
            .map_err(|e| {
                eprintln!("❌ API request error: {:?}", e);
                format!("API request failed: {}", e)
            })?;

        let api_status = api_response.status();
        println!("📡 API Response status: {}", api_status);

        // Track API usage from response headers
        let headers = api_response.headers().clone();
        if let Some(tracker) = self.app_handle.try_state::<crate::api_usage_tracker::ApiUsageTracker>() {
            tracker.update_from_headers(&headers).await;
        }

        if !api_status.is_success() {
            let error_body = api_response.text().await.unwrap_or_default();
            eprintln!("❌ API error response: {}", error_body);
            return Err(format!("API error {}: {}", api_status, error_body));
        }

        // Get response text for debugging
        let response_text = api_response.text().await
            .map_err(|e| format!("Failed to read API response: {}", e))?;

        println!("📄 API Response body: {}", response_text);

        // Parse JSON response
        let cdn_links: Vec<serde_json::Value> = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse API response as JSON: {}. Response was: {}", e, response_text))?;

        println!("📦 Parsed {} CDN link(s)", cdn_links.len());

        // Get the CDN URI from the first link
        let download_url = cdn_links
            .first()
            .and_then(|link| {
                println!("🔗 Link object: {:?}", link);
                link.get("URI")
            })
            .and_then(|uri| uri.as_str())
            .ok_or_else(|| format!("No download link in API response. Response was: {}", response_text))?
            .to_string();

        println!("✅ Got CDN URL: {}", download_url);

        // Make request with proper headers
        let response = self
            .client
            .get(&download_url)
            .header("User-Agent", "Treasure Chest Mod Manager/0.1.0")
            .send()
            .await
            .map_err(|e| {
                eprintln!("❌ Request error: {:?}", e);
                format!("Request failed: {}", e)
            })?;

        let status = response.status();
        println!("📡 Response status: {}", status);

        // Check content type
        if let Some(content_type) = response.headers().get("content-type") {
            println!("📄 Content-Type: {:?}", content_type);
        }

        // Check if this is an HTML page (redirect) instead of a file
        if let Some(content_type) = response.headers().get("content-type") {
            let content_type_str = content_type.to_str().unwrap_or("");
            if content_type_str.contains("text/html") {
                eprintln!("⚠️  Received HTML instead of file! Nexus might be returning a download page.");
                let html_body = response.text().await.unwrap_or_default();
                eprintln!("📄 HTML preview: {}", &html_body[..html_body.len().min(500)]);
                return Err("Received HTML page instead of file. The download URL might need Nexus Mods API access.".to_string());
            }
        }

        if !status.is_success() {
            // Try to get the response body for debugging
            let error_body = response.text().await.unwrap_or_else(|_| "Could not read response body".to_string());
            eprintln!("❌ HTTP error response body: {}", error_body);
            return Err(format!("HTTP error {}: {}", status,
                if error_body.len() > 200 { &error_body[..200] } else { &error_body }));
        }

        // Get total size if available
        let total_size = response.content_length();
        println!("📊 Content length: {:?}", total_size);

        // Update task with total size
        {
            let mut queue = self.queue.lock().await;
            if let Some(t) = queue.iter_mut().find(|t| t.id == task.id) {
                t.bytes_total = total_size;
            }
        }
        {
            let mut active = self.active.lock().await;
            if let Some(t) = active.get_mut(&task.id) {
                t.bytes_total = total_size;
            }
        }

        // Create download directory if it doesn't exist
        tokio::fs::create_dir_all(&self.download_dir)
            .await
            .map_err(|e| format!("Failed to create download directory: {}", e))?;

        // Create file
        let file_path = self.download_dir.join(&task.file_name);
        let mut file = File::create(&file_path)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?;

        // Download with progress tracking
        let mut downloaded: u64 = 0;
        let mut last_progress_time = Instant::now();
        let mut last_progress_bytes = 0u64;

        use futures::StreamExt;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Stream error: {}", e))?;

            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Write error: {}", e))?;

            downloaded += chunk.len() as u64;

            // Update progress every 100ms
            let now = Instant::now();
            if now.duration_since(last_progress_time) > Duration::from_millis(100) {
                let elapsed = now.duration_since(last_progress_time).as_secs_f64();
                let bytes_diff = downloaded - last_progress_bytes;
                let speed_bps = (bytes_diff as f64 / elapsed) as u64;

                let eta_seconds = if speed_bps > 0 && total_size.is_some() {
                    let remaining = total_size.unwrap() - downloaded;
                    Some(remaining / speed_bps)
                } else {
                    None
                };

                let progress_percent = if let Some(total) = total_size {
                    (downloaded as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                let progress = DownloadProgress {
                    download_id: task.id.clone(),
                    bytes_downloaded: downloaded,
                    bytes_total: total_size,
                    speed_bps,
                    eta_seconds,
                    progress_percent,
                };

                // Update task in queue
                {
                    let mut queue = self.queue.lock().await;
                    if let Some(t) = queue.iter_mut().find(|t| t.id == task.id) {
                        t.bytes_downloaded = downloaded;
                    }
                }

                // Emit progress event
                let _ = self.app_handle.emit("download-progress", &progress);

                last_progress_time = now;
                last_progress_bytes = downloaded;
            }
        }

        file.flush()
            .await
            .map_err(|e| format!("Flush error: {}", e))?;

        Ok(file_path)
    }

    async fn complete_download(&self, download_id: String, file_path: PathBuf) {
        // Update in queue
        {
            let mut queue = self.queue.lock().await;
            if let Some(task) = queue.iter_mut().find(|t| t.id == download_id) {
                task.status = DownloadStatus::Completed;
                task.file_path = Some(file_path.clone());
            }
        }

        // Remove from active
        {
            let mut active = self.active.lock().await;
            active.remove(&download_id);
        }

        // Emit completion event
        let _ = self.app_handle.emit("download-completed", download_id);
    }

    async fn fail_download(&self, download_id: String, error: String) {
        // Update in queue
        {
            let mut queue = self.queue.lock().await;
            if let Some(task) = queue.iter_mut().find(|t| t.id == download_id) {
                task.status = DownloadStatus::Failed { error: error.clone() };
            }
        }

        // Remove from active
        {
            let mut active = self.active.lock().await;
            active.remove(&download_id);
        }

        // Emit failure event
        #[derive(Serialize, Clone)]
        struct FailurePayload {
            download_id: String,
            error: String,
        }

        let _ = self.app_handle.emit(
            "download-failed",
            FailurePayload {
                download_id,
                error,
            },
        );
    }
}
