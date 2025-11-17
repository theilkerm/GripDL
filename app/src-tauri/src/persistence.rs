use crate::downloader::{DownloadInfo, DownloadStatus};
use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use tauri::AppHandle;

pub struct DownloadPersistence {
    db_path: PathBuf,
}

impl DownloadPersistence {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .context("Failed to get app data directory")?;
        
        std::fs::create_dir_all(&app_data_dir)
            .context("Failed to create app data directory")?;

        let db_path = app_data_dir.join("downloads.db");
        
        let persistence = Self { db_path };
        persistence.init_db()?;
        
        Ok(persistence)
    }

    fn init_db(&self) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS downloads (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                total_size INTEGER,
                downloaded_size INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL,
                cookies TEXT,
                referrer TEXT,
                user_agent TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS download_segments (
                download_id TEXT NOT NULL,
                segment_index INTEGER NOT NULL,
                start_byte INTEGER NOT NULL,
                end_byte INTEGER NOT NULL,
                downloaded_bytes INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (download_id, segment_index),
                FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
            )",
            [],
        )?;

        Ok(())
    }

    pub fn save_download(&self, info: &DownloadInfo) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        
        let status_str = match info.status {
            DownloadStatus::Pending => "pending",
            DownloadStatus::Downloading => "downloading",
            DownloadStatus::Paused => "paused",
            DownloadStatus::Completed => "completed",
            DownloadStatus::Failed(_) => "failed",
            DownloadStatus::Cancelled => "cancelled",
        };

        conn.execute(
            "INSERT OR REPLACE INTO downloads 
            (id, url, file_path, file_name, total_size, downloaded_size, status, cookies, referrer, user_agent, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                info.id,
                info.url,
                info.file_path.to_string_lossy(),
                info.file_name,
                info.total_size,
                info.downloaded_size,
                status_str,
                info.cookies,
                info.referrer,
                info.user_agent,
                info.created_at,
                info.updated_at
            ],
        )?;

        Ok(())
    }

    pub fn load_downloads(&self) -> Result<Vec<DownloadInfo>> {
        let conn = Connection::open(&self.db_path)?;
        
        let mut stmt = conn.prepare(
            "SELECT id, url, file_path, file_name, total_size, downloaded_size, status, cookies, referrer, user_agent, created_at, updated_at
             FROM downloads"
        )?;

        let download_iter = stmt.query_map([], |row| {
            let status_str: String = row.get(6)?;
            let status = match status_str.as_str() {
                "pending" => DownloadStatus::Pending,
                "downloading" => DownloadStatus::Downloading,
                "paused" => DownloadStatus::Paused,
                "completed" => DownloadStatus::Completed,
                "failed" => DownloadStatus::Failed("Unknown error".to_string()),
                "cancelled" => DownloadStatus::Cancelled,
                _ => DownloadStatus::Pending,
            };

            Ok(DownloadInfo {
                id: row.get(0)?,
                url: row.get(1)?,
                file_path: PathBuf::from(row.get::<_, String>(2)?),
                file_name: row.get(3)?,
                total_size: row.get(4)?,
                downloaded_size: row.get(5)?,
                status,
                cookies: row.get(7)?,
                referrer: row.get(8)?,
                user_agent: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })?;

        let mut downloads = Vec::new();
        for download in download_iter {
            downloads.push(download?);
        }

        Ok(downloads)
    }

    pub fn delete_download(&self, id: &str) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute("DELETE FROM downloads WHERE id = ?1", params![id])?;
        Ok(())
    }
}

