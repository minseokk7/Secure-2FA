use crate::{github, InstallerProgress, VersionInfo};
use tauri::Emitter;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;
use reqwest::Client;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const REPO_NAME: &str = "minseokk7/Secure-2FA";

pub async fn check_latest_version() -> anyhow::Result<VersionInfo> {
    let release = github::get_latest_release(REPO_NAME).await?;
    let latest = release.tag_name.trim_start_matches('v').to_string();

    // Try to find the setup exe
    let download_url = release
        .assets
        .iter()
        .find(|a| a.name.ends_with("setup.exe") || a.name.ends_with(".exe"))
        .map(|a| a.browser_download_url.clone())
        .ok_or_else(|| anyhow::anyhow!("설치 파일을 릴리즈에서 찾을 수 없습니다."))?;

    Ok(VersionInfo {
        latest,
        release_notes: release.body,
        download_url,
    })
}

pub async fn download_and_install(
    app: &tauri::AppHandle,
    install_path: &str,
    download_url: &str,
) -> anyhow::Result<()> {
    let emit_progress = |stage: &str, progress: u32, message: &str| {
        let _ = app.emit("install-progress", InstallerProgress {
            stage: stage.to_string(),
            progress,
            message: message.to_string(),
        });
    };

    emit_progress("prepare", 5, "설치 준비 중...");

    let target_dir = Path::new(install_path);
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    // Download the setup file to a temp folder
    let temp_dir = std::env::temp_dir();
    let setup_file_path = temp_dir.join("Secure2FA_Setup.exe");

    emit_progress("download", 10, "설치 파일 다운로드 중...");

    let client = Client::builder()
        .user_agent("Secure2FA-Installer/1.0")
        .build()?;
    let mut response = client.get(download_url).send().await?;
    
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("다운로드 실패: {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    
    let mut file = fs::File::create(&setup_file_path)?;

    // Read chunks
    use futures_util::StreamExt;
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        
        if total_size > 0 {
            let percentage = (downloaded as f64 / total_size as f64 * 80.0) as u32; // up to 80%
            emit_progress("download", 10 + percentage, &format!("다운로드 중... ({}MB)", downloaded / 1024 / 1024));
        }
    }
    
    file.sync_all()?;
    drop(file);

    emit_progress("install", 90, "설치 중...");

    // Run the NSIS installer silently with specific target directory
    #[cfg(target_os = "windows")]
    {
        let target_dir_str = target_dir.to_string_lossy();
        let d_arg = format!("/D={}", target_dir_str);
        
        let output = std::process::Command::new(&setup_file_path)
            .arg("/S")
            .arg(&d_arg)
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output()?;
            
        if !output.status.success() {
            println!("설치 프로세스가 오류 코드를 반환했습니다: {:?}", output.status);
            // It might succeed or fail depending on UAC
        }
    }

    // Clean up
    let _ = fs::remove_file(setup_file_path);

    emit_progress("complete", 100, "설치 완료!");

    Ok(())
}
