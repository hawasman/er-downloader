use crate::helpers::{format_size, format_speed, DropboxResponse, GLOBAL_APP_HANDLE};
use curl::easy::{Easy, WriteError};
use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::json;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use tauri::Emitter;

#[derive(serde::Serialize, Clone)]
struct Progress {
    total_size: String,
    current_size: String,
    speed: String,
    progress: String,
}

pub async fn download_file(link: &str, out_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let url = link;
    let output = PathBuf::from(out_path);
    fs::create_dir_all(output.parent().unwrap())?;

    // First get the total file size
    let mut easy = Easy::new();
    easy.url(url)?;
    easy.nobody(true)?;
    easy.perform()?;
    let total_size = easy.content_length_download()? as u64;

    println!("Total file size: {}", format_size(total_size));
    if total_size == 0 {
        return Err("Could not determine file size".into());
    }

    // Open the file in append mode to support resuming
    let mut file = File::options().create(true).append(true).open(&output)?;
    let current_size = file.metadata()?.len();

    if current_size >= total_size {
        println!(
            "File already exists and is complete ({}). Skipping download.",
            format_size(current_size)
        );
        return Ok(());
    }

    // Reset easy handle for actual download
    let mut easy = Easy::new();
    easy.url(url)?;
    easy.follow_location(true)?;
    easy.resume_from(current_size)?;

    let start_size = current_size;
    easy.progress(true)?;
    let start_time = std::time::Instant::now();
    easy.progress_function(move |dltotal, dlnow, _ultotal, _ulnow| {
        if dltotal > 0.0 {
            let current = start_size + dlnow as u64;
            let percentage = (current as f64 / total_size as f64) * 100.0;

            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { dlnow / elapsed } else { 0.0 };
            println!(
                "Downloading: {:.2}% ({} / {}) - {}",
                percentage,
                format_size(current),
                format_size(total_size),
                format_speed(speed)
            );
            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                app_handle
                    .emit(
                        "download_progress",
                        Progress {
                            total_size: format_size(total_size),
                            current_size: format_size(current),
                            speed: format_speed(speed),
                            progress: format!("{:.2}%", percentage),
                        },
                    )
                    .unwrap();
            }
        }
        true
    })?;

    // Write the downloaded data to the file
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            file.write_all(data).map_err(|_| WriteError::Pause)?;
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    // Verify download completion
    let final_size = file.metadata()?.len();
    if final_size == total_size {
        println!("Download completed successfully");
        Ok(())
    } else {
        println!(
            "Download incomplete: {} / {}",
            format_size(final_size),
            format_size(total_size)
        );
        Err("Download incomplete".into())
    }
}

pub async fn generate_download_link(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
    println!("Generating download link...");
    let dropbox_token =
        env::var("DROPBOX_TOKEN").expect("DROPBOX_TOKEN must be set in the environment");
    // let file_path = "/ConvergenceER.zip";
    let url = "https://api.dropboxapi.com/2/files/get_temporary_link";
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    let auth_token: String = format!("Bearer {}", dropbox_token);
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&auth_token).expect("Invalid header value"),
    );
    let body = json!({
    "path": file_path,
    });
    let req = client.post(url).headers(headers).json(&body).send().await?;
    let status = req.status();
    if status.is_success() {
        let response: DropboxResponse = req.json().await?;
        return Ok(response.link);
    } else {
        let error_message = req.text().await?;
        return Err(error_message.into());
    }
}
