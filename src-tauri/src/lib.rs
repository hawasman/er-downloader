mod downloader;
mod helpers;
use std::{fs::File, io::Read};
use tauri::Emitter;

use helpers::GLOBAL_APP_HANDLE;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn download_er() -> Result<(), String> {
    print!("Downloading file...");
    let download_link = downloader::generate_download_link("/ConvergenceER.zip")
        .await
        .map_err(|e| e.to_string())?;
    println!("Download link: {}", download_link);
    downloader::download_file(&download_link, "Download/Convergence.zip")
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

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
    println!("Checking if patch notes exist...");
    let mut content = String::new();

    if let Ok(mut file) = File::open("Download/path_notes.md") {
        println!("File exists, reading content...");
        file.read_to_string(&mut content)
            .map_err(|e| e.to_string())?;
    } else {
        println!("Downloading Patch Notes...");
        let download_link = downloader::generate_download_link("/path_notes.md")
            .await
            .map_err(|e| e.to_string())?;

        println!("Download link: {}", download_link);
        downloader::download_file(&download_link, "Download/path_notes.md")
            .await
            .map_err(|e| e.to_string())?;

        let mut file = File::open("Download/path_notes.md").map_err(|e| e.to_string())?;
        file.read_to_string(&mut content)
            .map_err(|e| e.to_string())?;
    }

    if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
        app_handle.emit("get_patch_notes", content).unwrap();
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
            greet,
            download_er,
            extract_file,
            get_patch_notes
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::Ready => {
                println!("Window loaded");
                helpers::GLOBAL_APP_HANDLE
                    .set(_app_handle.clone())
                    .expect("Failed to set global app handle");
            }
            _ => {}
        });
}
