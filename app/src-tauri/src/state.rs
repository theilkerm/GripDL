use crate::downloader::DownloadManager;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub download_manager: Arc<RwLock<DownloadManager>>,
}

