use crate::helpers::{format_size, format_speed, DropboxResponse, GLOBAL_APP_HANDLE};
use curl::easy::{Easy, WriteError};
use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::json;
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use tauri::Emitter;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

#[derive(serde::Serialize, Clone)]
struct Progress {
    name: String,
    total_size: String,
    current_size: String,
    speed: String,
    progress: String,
}

pub async fn download_file(
    link: &str,
    download_to: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = link;
    let output = PathBuf::from(download_to);
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

    // Curl handle for actual download
    let mut easy = Easy::new();
    easy.url(url)?;
    easy.follow_location(true)?;
    easy.resume_from(current_size)?;

    let start_size = current_size;
    easy.progress(true)?;

    let start_time = std::time::Instant::now();
    let file_name = download_to
        .rsplit_once("/")
        .map(|(_, name)| name.to_string())
        .unwrap_or_else(|| download_to.to_string());
    easy.progress_function(move |total, now, _, _| {
        if total > 0.0 {
            let current = start_size + now as u64;
            let percentage = (current as f64 / total_size as f64) * 100.0;

            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { now / elapsed } else { 0.0 };
            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                app_handle
                    .emit(
                        "download_progress",
                        Progress {
                            name: format!("{}", file_name.replace(".zip", "")),
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
        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
            app_handle
                .emit(
                    "download_progress",
                    Progress {
                        name: format!("Download completed successfully"),
                        total_size: format_size(total_size),
                        current_size: format_size(final_size),
                        speed: "N/A".to_string(),
                        progress: "100%".to_string(),
                    },
                )
                .unwrap();
        }

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
        Ok(response.link)
    } else {
        let error_message = req.text().await?;
        Err(error_message.into())
    }
}

pub async fn download_updates(
    updates: Vec<String>,
    extract_path: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut latest_version = String::from("v0.0.0");
    if updates.is_empty() {
        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
            let answer = app_handle
                .dialog()
                .message("Directory doesn't contain a version.txt\nDo you want to download The full mod?")
                .title("Tauri is Awesome")
                .buttons(MessageDialogButtons::OkCancel)
                .blocking_show();
            if answer {
                print!("Downloading file...");
                let download_link = generate_download_link("/ConvergenceER.zip")
                    .await
                    .map_err(|e| e.to_string())?;
                println!("Download link: {}", download_link);
                let mut retries = 0;
                let max_retries = 3;

                loop {
                    match download_file(&download_link, "Download/Convergence.zip").await {
                        Ok(_) => {
                            let file = File::open("Download/Convergence.zip")
                                .map_err(|e| e.to_string())?;
                            let mut archive =
                                zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
                            println!("Extracting file...");
                            println!("Extracting to: {}", extract_path);
                            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                app_handle
                                    .emit(
                                        "download_progress",
                                        Progress {
                                            name: format!("Extracting to: {}", extract_path),
                                            total_size: "N/A".to_string(),
                                            current_size: "N/A".to_string(),
                                            speed: "N/A".to_string(),
                                            progress: "100%".to_string(),
                                        },
                                    )
                                    .unwrap();
                            }
                            archive.extract(extract_path).map_err(|e| e.to_string())?;
                            println!("File extracted successfully!");
                            if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                                app_handle
                                    .dialog()
                                    .message("File extracted successfully!")
                                    .blocking_show();
                            }

                            return Ok(true);
                        }
                        Err(_) => {
                            retries += 1;
                            if retries >= max_retries {
                                return Ok(false);
                            }
                            println!("Download failed, retrying ({}/{})", retries, max_retries);
                        }
                    }
                }
            }
        }
        return Ok(false);
    }
    println!("Downloading updates...");
    for update in updates {
        let (_, update_name) = update.rsplit_once('/').unwrap();
        println!("Downloading update: {}", update_name);
        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
            app_handle
                .emit(
                    "download_progress",
                    Progress {
                        name: format!("{}", update_name.replace(".zip", "")),
                        total_size: "N/A".to_string(),
                        current_size: "N/A".to_string(),
                        speed: "N/A".to_string(),
                        progress: "N/A".to_string(),
                    },
                )
                .unwrap();
        }
        let download_link = generate_download_link(&update)
            .await
            .map_err(|e| e.to_string())?;
        let output_path = format!("Download/{}", update);
        let mut retries = 0;
        let max_retries = 3;

        loop {
            match download_file(&download_link, &output_path).await {
                Ok(_) => {
                    let file_path = format!("Download/{}", update);
                    let file = File::open(file_path).map_err(|e| e.to_string())?;
                    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
                    archive.extract(extract_path).map_err(|e| e.to_string())?;
                    break;
                }
                Err(_) => {
                    retries += 1;
                    if retries >= max_retries {
                        return Ok(false);
                    }
                    println!("Download failed, retrying ({}/{})", retries, max_retries);
                }
            }
        }

        latest_version = update.replace("/updates/", "").replace(".zip", "");
    }

    let file_path = Path::new(extract_path).join("version.txt");

    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| e.to_string())?;
    }

    let mut version_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&file_path)
        .map_err(|e| e.to_string())?;

    if let Err(e) = write!(version_file, "{}", latest_version) {
        eprintln!("Couldn't write to file: {}", e);
    }
    println!("Version file updated: {}", latest_version);

    println!("All updates downloaded and extracted successfully!");
    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
        app_handle
            .dialog()
            .message("All updates downloaded and extracted successfully!")
            .blocking_show();
        app_handle
            .emit(
                "download_progress",
                Progress {
                    name: "All updates downloaded".to_string(),
                    total_size: "N/A".to_string(),
                    current_size: "N/A".to_string(),
                    speed: "N/A".to_string(),
                    progress: "N/A".to_string(),
                },
            )
            .unwrap();
    }

    Ok(false)
}
