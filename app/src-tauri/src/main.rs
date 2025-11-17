// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod downloader;
mod native_messaging;
mod persistence;
mod state;

use downloader::DownloadManager;
use native_messaging::NativeMessagingHost;
use state::AppState;
use tauri::{Manager, State};
use tokio::sync::RwLock;

#[tauri::command]
async fn start_download(
    url: String,
    cookies: Option<String>,
    referrer: Option<String>,
    user_agent: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let manager = state.download_manager.read().await;
    manager
        .start_download(url, cookies, referrer, user_agent)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn pause_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.download_manager.read().await;
    manager.pause_download(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn resume_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.download_manager.read().await;
    manager.resume_download(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn cancel_download(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let manager = state.download_manager.read().await;
    manager.cancel_download(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_downloads(state: State<'_, AppState>) -> Result<Vec<downloader::DownloadInfo>, String> {
    let manager = state.download_manager.read().await;
    Ok(manager.get_all_downloads().await)
}

#[tauri::command]
async fn get_download_info(
    id: String,
    state: State<'_, AppState>,
) -> Result<downloader::DownloadInfo, String> {
    let manager = state.download_manager.read().await;
    manager
        .get_download_info(&id)
        .await
        .ok_or_else(|| "Download not found".to_string())
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Initialize download manager
            let download_manager = DownloadManager::new(app_handle.clone());
            let app_state = AppState {
                download_manager: RwLock::new(download_manager),
            };
            app.manage(app_state);

            // Note: Native messaging host should run as a separate process
            // For now, we'll handle native download requests via events
            // In production, create a separate binary for native messaging

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_download,
            pause_download,
            resume_download,
            cancel_download,
            get_downloads,
            get_download_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

