use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubRelease {
    pub tag_name: String,
    pub body: String,
    pub assets: Vec<ReleaseAsset>,
}

pub async fn get_latest_release(repo: &str) -> anyhow::Result<GithubRelease> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    
    let client = reqwest::Client::builder()
        .user_agent("Secure-2FA-Installer/1.0")
        .build()?;
        
    let res = client.get(&url).send().await?;
    
    if !res.status().is_success() {
        return Err(anyhow::anyhow!("GitHub API 요청 실패: {}", res.status()));
    }
    
    let release: GithubRelease = res.json().await?;
    Ok(release)
}
