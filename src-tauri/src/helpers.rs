use dotenv::dotenv;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fs::File, io::Read, path::Path};
use tauri::AppHandle;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
use version_compare::{Cmp, Version};

pub static GLOBAL_APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

#[derive(Deserialize, Debug)]
pub struct UpdateInfo {
    pub latest: String,
    pub least: String,
    pub updates: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UpdateStatus {
    status: String,
    message: String,
}

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

pub async fn check_updates(
    downloading: bool,
    directory: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    dotenv().ok();
    let updates_url = env::var("UPDATES_URL").expect("UPDATES_URL must be set in the environment");
    println!("Checking for updates...");
    let file_path = Path::new(directory).join("version.txt");

    if !file_path.exists() {
        println!("Version file does not exist, skipping update check.");
        return Ok([].to_vec());
    }

    let version_string = File::open(file_path)
        .expect("Failed to open version file")
        .bytes()
        .map(|b| b.expect("Failed to read byte") as char)
        .collect::<String>();
    let client = reqwest::Client::new();
    println!("Getting update info");
    let response = client.get(updates_url).send().await;

    if let Err(err) = response {
        eprintln!("Error getting update info: {}", err);
        return Err(Box::new(err));
    }
    let response = response.unwrap();
    if !response.status().is_success() {
        eprintln!("Error getting update info: {}", response.status());
        return Err(Box::from(format!("HTTP error: {}", response.status())));
    }
    let update_info: UpdateInfo = response
        .json()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    let current = Version::from(version_string.as_str()).unwrap();
    let latest = Version::from(update_info.latest.as_str()).unwrap();
    let least = Version::from(update_info.least.as_str()).unwrap();
    let latest_str = update_info.latest.clone();

    if current.compare_to(least, Cmp::Lt) {
        let least_str = update_info.least.clone();

        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
            app_handle
                .dialog()
                .message(format!(
                    "You're using an older version: {} that is not supported by the downloader (last supported version: {})\n Your version can't be updated.",
                    current, least_str
                ))
                .kind(MessageDialogKind::Info)
                .title("Unsupported Version")
                .blocking_show();
        }

        return Ok([].to_vec());
    }

    match current.compare(latest) {
        Cmp::Lt => {
            if !downloading {
                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                    app_handle
                        .dialog()
                        .message(format!(
                            "A new version is available: {} (current: {})",
                            latest_str, current
                        ))
                        .kind(MessageDialogKind::Info)
                        .title("New version available")
                        .blocking_show();
                }
            }
            let mut updates: Vec<String> = Vec::new();
            for (key, value) in update_info.updates.iter() {
                let replaced = key.replace("v", "");
                let version = Version::from(replaced.as_str()).unwrap();
                if current < version {
                    updates.push(value.to_string());
                }
            }
            updates.sort_by(|a, b| {
                let a_str = a.replace("v", "");
                let version_a = Version::from(a_str.as_str()).unwrap();
                let b_str = b.replace("v", "");
                let version_b = Version::from(b_str.as_str()).unwrap();
                match version_a.compare(version_b) {
                    Cmp::Lt => std::cmp::Ordering::Less,
                    Cmp::Eq => std::cmp::Ordering::Equal,
                    Cmp::Gt => std::cmp::Ordering::Greater,
                    _ => unreachable!(),
                }
            });

            Ok(updates)
        }
        Cmp::Eq => {
            if !downloading {
                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                    app_handle
                        .dialog()
                        .message(format!("You are using the latest version: {}", current))
                        .kind(MessageDialogKind::Info)
                        .title("New version available")
                        .blocking_show();
                }
            }
            Ok([].to_vec())
        }
        Cmp::Gt => {
            if !downloading {
                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                    app_handle
                        .dialog()
                        .message(format!(
                            "You are using a newer version: {}, latest: {}.\n How..???",
                            current, latest_str
                        ))
                        .kind(MessageDialogKind::Info)
                        .title("New version available")
                        .blocking_show();
                }
            }
            panic!("You are using a newer version than expected")
        }
        _ => unreachable!(),
    }
}
