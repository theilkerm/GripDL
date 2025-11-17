use anyhow::{Context, Result};
use bytes::Bytes;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::persistence::DownloadPersistence;
use std::sync::Arc;

const MAX_SEGMENTS: usize = 32;
const MIN_SEGMENT_SIZE: u64 = 1024 * 1024; // 1MB minimum per segment

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub id: String,
    pub url: String,
    pub file_path: PathBuf,
    pub file_name: String,
    pub total_size: Option<u64>,
    pub downloaded_size: u64,
    pub status: DownloadStatus,
    pub cookies: Option<String>,
    pub referrer: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
struct Segment {
    index: usize,
    start: u64,
    end: u64,
    downloaded: u64,
}

pub struct DownloadManager {
    app_handle: AppHandle,
    persistence: DownloadPersistence,
    active_downloads: Arc<Mutex<HashMap<String, mpsc::Sender<DownloadCommand>>>>,
}

enum DownloadCommand {
    Pause,
    Resume,
    Cancel,
}

impl DownloadManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let persistence = DownloadPersistence::new(&app_handle)
            .expect("Failed to initialize persistence");
        
        Self {
            app_handle,
            persistence,
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start_download(
        &self,
        url: String,
        cookies: Option<String>,
        referrer: Option<String>,
        user_agent: Option<String>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        
        // Create download directory
        let downloads_dir = self
            .app_handle
            .path()
            .download_dir()
            .context("Failed to get download directory")?;
        
        let file_name = self.extract_filename(&url).unwrap_or_else(|| {
            format!("download_{}", id.chars().take(8).collect::<String>())
        });
        
        let file_path = downloads_dir.join(&file_name);
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let info = DownloadInfo {
            id: id.clone(),
            url: url.clone(),
            file_path: file_path.clone(),
            file_name: file_name.clone(),
            total_size: None,
            downloaded_size: 0,
            status: DownloadStatus::Pending,
            cookies: cookies.clone(),
            referrer: referrer.clone(),
            user_agent: user_agent.clone(),
            created_at: now,
            updated_at: now,
        };

        self.persistence.save_download(&info)?;

        // Start download task
        let (tx, mut rx) = mpsc::channel(10);
        self.active_downloads.lock().insert(id.clone(), tx);

        let manager_clone = self.clone_for_task();
        let app_handle_clone = self.app_handle.clone();
        let id_clone = id.clone();

        tokio::spawn(async move {
            let mut paused = false;
            let mut cancelled = false;

            loop {
                tokio::select! {
                    cmd = rx.recv() => {
                        match cmd {
                            Some(DownloadCommand::Pause) => paused = true,
                            Some(DownloadCommand::Resume) => paused = false,
                            Some(DownloadCommand::Cancel) => {
                                cancelled = true;
                                break;
                            }
                            None => break,
                        }
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                        if !paused && !cancelled {
                            if let Err(e) = manager_clone.download_file(
                                &id_clone,
                                &url,
                                &file_path,
                                cookies.as_deref(),
                                referrer.as_deref(),
                                user_agent.as_deref(),
                            ).await {
                                tracing::error!("Download error: {}", e);
                                let mut info = manager_clone.get_download_info(&id_clone).await.unwrap();
                                info.status = DownloadStatus::Failed(e.to_string());
                                info.updated_at = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64;
                                let _ = manager_clone.persistence.save_download(&info);
                                manager_clone.emit_download_update(&info).await;
                                break;
                            } else {
                                // Download completed
                                break;
                            }
                        }
                    }
                }
            }

            manager_clone.active_downloads.lock().remove(&id_clone);
        });

        self.emit_download_update(&info).await;

        Ok(id)
    }

    async fn download_file(
        &self,
        id: &str,
        url: &str,
        file_path: &Path,
        cookies: Option<&str>,
        referrer: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<()> {
        let client = self.build_client(cookies, referrer, user_agent)?;

        // Head request to get file size and check Range support
        let head_response = client.head(url).send().await?;
        let total_size = head_response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        let supports_range = head_response
            .headers()
            .get("accept-ranges")
            .and_then(|v| v.to_str().ok())
            .map(|s| s == "bytes")
            .unwrap_or(false);

        // Update download info
        let mut info = self.get_download_info(id).await.unwrap();
        info.total_size = total_size;
        info.status = DownloadStatus::Downloading;
        self.persistence.save_download(&info)?;
        self.emit_download_update(&info).await;

        if !supports_range || total_size.is_none() {
            // Single-threaded download
            return self.download_single_threaded(&client, url, file_path, id).await;
        }

        let total_size = total_size.unwrap();
        let num_segments = self.calculate_segments(total_size);
        
        if num_segments <= 1 {
            return self.download_single_threaded(&client, url, file_path, id).await;
        }

        // Multi-threaded segmented download
        let self_arc = Arc::new(self.clone_for_task());
        self_arc.download_segmented(&client, url, file_path, total_size, num_segments, id).await
    }

    fn calculate_segments(&self, total_size: u64) -> usize {
        let max_segments = MAX_SEGMENTS.min((total_size / MIN_SEGMENT_SIZE) as usize);
        max_segments.max(1)
    }

    async fn download_segmented(
        self: Arc<Self>,
        client: &reqwest::Client,
        url: &str,
        file_path: &Path,
        total_size: u64,
        num_segments: usize,
        id: &str,
    ) -> Result<()> {
        let segment_size = total_size / num_segments as u64;
        let mut handles = Vec::new();

        // Create temporary files for each segment
        let temp_dir = file_path.parent().unwrap();
        let temp_base = format!("{}.part", file_path.file_name().unwrap().to_string_lossy());

        for i in 0..num_segments {
            let start = i as u64 * segment_size;
            let end = if i == num_segments - 1 {
                total_size - 1
            } else {
                (i + 1) as u64 * segment_size - 1
            };

            let segment_file = temp_dir.join(format!("{}.{}", temp_base, i));
            let url = url.to_string();
            let client = client.clone();
            let id = id.to_string();
            let manager = Arc::clone(&self);

            let handle = tokio::spawn(async move {
                manager
                    .download_segment(&client, &url, &segment_file, start, end, &id, i)
                    .await
            });

            handles.push(handle);
        }

        // Wait for all segments to complete
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await??);
        }

        // Merge segments
        self.merge_segments(file_path, &temp_dir, &temp_base, num_segments).await?;

        // Update final status
        let mut info = self.get_download_info(id).await.unwrap();
        info.status = DownloadStatus::Completed;
        info.downloaded_size = total_size;
        info.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.persistence.save_download(&info)?;
        self.emit_download_update(&info).await;

        Ok(())
    }

    async fn download_segment(
        self: Arc<Self>,
        client: &reqwest::Client,
        url: &str,
        segment_file: &Path,
        start: u64,
        end: u64,
        id: &str,
        segment_index: usize,
    ) -> Result<u64> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(segment_file)
            .await?;

        let range_header = format!("bytes={}-{}", start, end);
        let mut response = client
            .get(url)
            .header("Range", range_header)
            .send()
            .await?;

        let mut downloaded = 0u64;
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Update progress periodically
            if downloaded % (1024 * 1024) == 0 {
                let mut info = self.get_download_info(id).await.unwrap();
                info.downloaded_size += chunk.len() as u64;
                info.updated_at = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                self.persistence.save_download(&info)?;
                self.emit_download_update(&info).await;
            }
        }

        Ok(downloaded)
    }

    async fn merge_segments(
        &self,
        final_path: &Path,
        temp_dir: &Path,
        temp_base: &str,
        num_segments: usize,
    ) -> Result<()> {
        let mut final_file = File::create(final_path).await?;

        for i in 0..num_segments {
            let segment_path = temp_dir.join(format!("{}.{}", temp_base, i));
            let mut segment_file = File::open(&segment_path).await?;
            tokio::io::copy(&mut segment_file, &mut final_file).await?;
            tokio::fs::remove_file(&segment_path).await?;
        }

        Ok(())
    }

    async fn download_single_threaded(
        &self,
        client: &reqwest::Client,
        url: &str,
        file_path: &Path,
        id: &str,
    ) -> Result<()> {
        let mut response = client.get(url).send().await?;
        let mut file = File::create(file_path).await?;
        let mut downloaded = 0u64;

        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Update progress
            let mut info = self.get_download_info(id).await.unwrap();
            info.downloaded_size = downloaded;
            info.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            self.persistence.save_download(&info)?;
            self.emit_download_update(&info).await;
        }

        let mut info = self.get_download_info(id).await.unwrap();
        info.status = DownloadStatus::Completed;
        info.downloaded_size = downloaded;
        info.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.persistence.save_download(&info)?;
        self.emit_download_update(&info).await;

        Ok(())
    }

    fn build_client(
        &self,
        cookies: Option<&str>,
        referrer: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<reqwest::Client> {
        let mut builder = reqwest::Client::builder();

        if let Some(ua) = user_agent {
            builder = builder.user_agent(ua);
        } else {
            builder = builder.user_agent("GripDL/1.0");
        }

        if let Some(ref_str) = referrer {
            builder = builder.referer(true);
        }

        let client = builder.build()?;

        // Set cookies if provided
        if let Some(cookie_str) = cookies {
            // Parse and set cookies
            // This is simplified - you might want to use a cookie jar
        }

        Ok(client)
    }

    fn extract_filename(&self, url: &str) -> Option<String> {
        url.split('/').last().and_then(|s| {
            s.split('?').next().filter(|s| !s.is_empty()).map(|s| s.to_string())
        })
    }

    pub async fn pause_download(&self, id: &str) -> Result<()> {
        if let Some(tx) = self.active_downloads.lock().get(id) {
            tx.send(DownloadCommand::Pause).await?;
            
            let mut info = self.get_download_info(id).await.unwrap();
            info.status = DownloadStatus::Paused;
            info.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            self.persistence.save_download(&info)?;
            self.emit_download_update(&info).await;
        }
        Ok(())
    }

    pub async fn resume_download(&self, id: &str) -> Result<()> {
        if let Some(tx) = self.active_downloads.lock().get(id) {
            tx.send(DownloadCommand::Resume).await?;
            
            let mut info = self.get_download_info(id).await.unwrap();
            info.status = DownloadStatus::Downloading;
            info.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            self.persistence.save_download(&info)?;
            self.emit_download_update(&info).await;
        }
        Ok(())
    }

    pub async fn cancel_download(&self, id: &str) -> Result<()> {
        if let Some(tx) = self.active_downloads.lock().get(id) {
            tx.send(DownloadCommand::Cancel).await?;
            
            let mut info = self.get_download_info(id).await.unwrap();
            info.status = DownloadStatus::Cancelled;
            info.updated_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            self.persistence.save_download(&info)?;
            self.emit_download_update(&info).await;
        }
        Ok(())
    }

    pub async fn get_download_info(&self, id: &str) -> Option<DownloadInfo> {
        self.persistence
            .load_downloads()
            .ok()?
            .into_iter()
            .find(|d| d.id == id)
    }

    pub async fn get_all_downloads(&self) -> Vec<DownloadInfo> {
        self.persistence.load_downloads().unwrap_or_default()
    }

    async fn emit_download_update(&self, info: &DownloadInfo) {
        let _ = self.app_handle.emit("download-update", info);
    }

    fn clone_for_task(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
            persistence: DownloadPersistence::new(&self.app_handle)
                .expect("Failed to create persistence"),
            active_downloads: self.active_downloads.clone(),
        }
    }
}

