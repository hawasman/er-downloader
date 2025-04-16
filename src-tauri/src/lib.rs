mod downloader;
mod helpers;
use dotenv::dotenv;
use downloader::download_updates;
use helpers::GLOBAL_APP_HANDLE;
use std::{env, fs::File};
use tauri::Emitter;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

// #[tauri::command]
// async fn download_er(directory: &str) -> Result<(), String> {
//     println!("checking if version.txt exists...");
//     let file_path = Path::new(directory).join("version.txt");
//     let version_file = File::open(file_path);
//     if version_file.is_ok() {
//         println!("Checking and downloading updates");
//         check_for_updates(true, directory)
//             .await
//             .expect("Something went wrong while updating.");
//         return Ok(());
//     }
//     print!("Downloading file...");
//     let download_link = downloader::generate_download_link("/ConvergenceER.zip")
//         .await
//         .map_err(|e| e.to_string())?;
//     println!("Download link: {}", download_link);
//     let mut retries = 0;
//     let max_retries = 3;

//     loop {
//         match downloader::download_file(&download_link, "Download/Convergence.zip").await {
//             Ok(_) => return Ok(()),
//             Err(_) => {
//                 retries += 1;
//                 if retries >= max_retries {
//                     return Err(format!("Failed to download after {} attempts", max_retries));
//                 }
//                 println!("Download failed, retrying ({}/{})", retries, max_retries);
//             }
//         }
//     }
// }

#[tauri::command]
async fn extract_file(extract_path: &str) -> Result<(), String> {
    let file = File::open("Download/Convergence.zip").map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;
    println!("Extracting file...");
    println!("Extracting to: {}", extract_path);
    archive.extract(extract_path).map_err(|e| e.to_string())?;
    println!("File extracted successfully!");
    Ok(())
}

#[tauri::command]
async fn get_patch_notes() -> Result<(), String> {
    dotenv().ok();
    let patch_notes_url =
        env::var("PATCH_NOTES_URL").expect("PATCH_NOTES_URL must be set in the environment");
    let client = reqwest::Client::new();
    println!("Getting update info");
    let response = client.get(patch_notes_url).send().await;

    if let Err(err) = response {
        eprintln!("Error getting update info: {}", err);
        return Err(format!(
            "Error getting patch notes info: {}",
            err.to_string()
        ));
    }
    let response = response.unwrap();
    if !response.status().is_success() {
        eprintln!("Error getting patch notes info: {}", response.status());
        return Err(format!("HTTP error: {}", response.status()));
    }
    let patch_notes = response
        .text()
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>);

    if let Ok(patch_notes) = patch_notes {
        if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
            app_handle.emit("get_patch_notes", patch_notes).unwrap();
        }
    }
    Ok(())
}

#[tauri::command]
async fn check_for_updates(downloading: bool, directory: &str) -> Result<(), String> {
    let updates = match helpers::check_updates(downloading, directory).await {
        Ok(updates) => updates,
        Err(e) => return Err(e.to_string()),
    };

    if downloading {
        download_updates(updates, directory)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // download_er,
            extract_file,
            get_patch_notes,
            check_for_updates
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            tauri::async_runtime::block_on(async {
                match event {
                    tauri::RunEvent::Ready => {
                        println!("Window loaded");
                        GLOBAL_APP_HANDLE
                            .set(_app_handle.clone())
                            .expect("Failed to set global app handle");
                    }
                    _ => {}
                }
            });
        });
}
