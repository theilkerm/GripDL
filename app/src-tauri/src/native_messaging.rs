use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Write};
use tauri::AppHandle;

#[derive(Debug, Deserialize)]
struct NativeMessage {
    url: String,
    cookies: Option<String>,
    referrer: Option<String>,
    user_agent: Option<String>,
}

#[derive(Debug, Serialize)]
struct NativeResponse {
    success: bool,
    message: Option<String>,
}

pub struct NativeMessagingHost;

impl NativeMessagingHost {
    /// Start the native messaging host as a standalone process
    /// This should be called from a separate binary or when invoked as native messaging host
    pub async fn start(app_handle: AppHandle) -> Result<()> {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        let mut stdout = io::stdout();

        loop {
            // Read message length (4 bytes, little-endian)
            let mut length_bytes = [0u8; 4];
            if reader.read_exact(&mut length_bytes).is_err() {
                break; // EOF or error
            }
            let length = u32::from_le_bytes(length_bytes) as usize;

            if length == 0 {
                continue;
            }

            // Read message content
            let mut buffer = vec![0u8; length];
            if reader.read_exact(&mut buffer).is_err() {
                break;
            }

            // Parse JSON message
            let message: NativeMessage = match serde_json::from_slice(&buffer) {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("Failed to parse native message: {}", e);
                    Self::send_response(&mut stdout, false, Some("Invalid message format".to_string()))?;
                    continue;
                }
            };

            // Emit event to Tauri app - the app will handle starting the download
            // In a production setup, you might want to use IPC or a local server
            let app_handle_clone = app_handle.clone();
            let url = message.url.clone();
            let cookies = message.cookies.clone();
            let referrer = message.referrer.clone();
            let user_agent = message.user_agent.clone();

            // Emit event that the frontend can listen to
            let _ = app_handle_clone.emit("native-download-request", serde_json::json!({
                "url": url,
                "cookies": cookies,
                "referrer": referrer,
                "user_agent": user_agent,
            }));

            Self::send_response(&mut stdout, true, None)?;
        }

        Ok(())
    }

    fn send_response(
        stdout: &mut io::Stdout,
        success: bool,
        message: Option<String>,
    ) -> Result<()> {
        let response = NativeResponse { success, message };
        let json = serde_json::to_string(&response)?;
        let length = json.len() as u32;

        stdout.write_all(&length.to_le_bytes())?;
        stdout.write_all(json.as_bytes())?;
        stdout.flush()?;

        Ok(())
    }
}

