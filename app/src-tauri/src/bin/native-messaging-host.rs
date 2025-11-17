// Separate binary for Native Messaging Host
// This runs as a standalone process when invoked by Firefox

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, BufReader, Write};

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

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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

        // Parse message
        let message: NativeMessage = match serde_json::from_slice(&buffer) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
                send_response(&mut stdout, false, Some("Invalid message format".to_string()))?;
                continue;
            }
        };

        // In production, this should communicate with the main GripDL app via:
        // - Unix domain socket
        // - HTTP localhost server
        // - Named pipe
        // For now, we'll just acknowledge receipt
        // The main app should be listening for these requests
        
        tracing::info!("Received download request: {}", message.url);
        
        send_response(&mut stdout, true, None)?;
    }

    Ok(())
}

