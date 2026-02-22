use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub latest: String,
    pub release_notes: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerProgress {
    pub stage: String,
    pub progress: u32,
    pub message: String,
}

mod github;
mod installer;

#[tauri::command]
fn get_default_install_path() -> Result<String, String> {
    // Default to LocalAppData/Secure 2FA
    let local_app_data = std::env::var("LOCALAPPDATA")
        .unwrap_or_else(|_| "C:\\".to_string());
    
    let path = std::path::Path::new(&local_app_data).join("Secure 2FA");
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn check_latest_version() -> Result<VersionInfo, String> {
    installer::check_latest_version().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn run_install(
    app: tauri::AppHandle,
    install_path: String,
    download_url: String,
) -> Result<(), String> {
    installer::download_and_install(&app, &install_path, &download_url)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_default_install_path,
            check_latest_version,
            run_install,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
