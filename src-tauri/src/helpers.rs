use serde::{Deserialize, Serialize};

use once_cell::sync::OnceCell;
use tauri::AppHandle;
pub static GLOBAL_APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

#[derive(Serialize, Deserialize, Debug)]
pub struct DropboxResponse {
    #[serde(rename = "metadata")]
    pub metadata: Metadata,

    #[serde(rename = "link")]
    pub link: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "path_lower")]
    path_lower: String,

    #[serde(rename = "path_display")]
    path_display: String,

    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "client_modified")]
    client_modified: String,

    #[serde(rename = "server_modified")]
    server_modified: String,

    #[serde(rename = "rev")]
    rev: String,

    #[serde(rename = "size")]
    size: i64,

    #[serde(rename = "is_downloadable")]
    is_downloadable: bool,

    #[serde(rename = "content_hash")]
    content_hash: String,
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

pub fn format_speed(bytes_per_sec: f64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    if bytes_per_sec >= GB {
        format!("{:.2} GB/s", bytes_per_sec / GB)
    } else if bytes_per_sec >= MB {
        format!("{:.2} MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.2} KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}
