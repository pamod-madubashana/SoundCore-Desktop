use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const REPO: &str = "pamod-madubashana/SoundCore-Desktop";
const ASSET_NAME: &str = "SoundCore-Desktop.exe";
const INSTALLER_ASSET_NAME: &str = "SoundCore-Desktop-x64-setup.exe";
const API_URL_ENV: &str = "SOUNDCORE_UPDATE_API_URL";
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, serde::Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub digest: String,
    pub size: u64,
    pub installer_url: String,
    pub installer_digest: String,
    pub installer_size: u64,
}

#[derive(Clone, PartialEq, serde::Serialize)]
pub struct UpdateProgress {
    pub phase: String,
    pub percent: f64,
    pub message: String,
}

fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn parse_version(v: &str) -> Option<semver::Version> {
    let v = v.trim().strip_prefix('v').unwrap_or(v.trim());
    semver::Version::parse(v).ok()
}

/// Loop guard: the release tag must be strictly newer than the running binary.
/// Refuses a no-op install whose payload would still report the old version
/// (e.g. tag bumped without bumping the version files baked into the exe).
fn ensure_newer(current_str: &str, latest_str: &str) -> Result<(), String> {
    let current = parse_version(current_str)
        .ok_or_else(|| format!("Invalid current version: {current_str}"))?;
    let latest =
        parse_version(latest_str).ok_or_else(|| format!("Invalid latest version: {latest_str}"))?;
    if latest <= current {
        return Err(format!(
            "Already up to date (current v{current}, latest v{latest}); refusing no-op install"
        ));
    }
    Ok(())
}

fn get_updates_dir() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    let dir = base.join("SoundCore-Desktop").join("updates");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create updates dir: {e}"))?;
    Ok(dir)
}

fn exe_paths(current_exe: &Path) -> (PathBuf, PathBuf) {
    let name = current_exe
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "soundcore-desktop.exe".to_string());
    (
        current_exe.with_file_name(format!("{name}.old")),
        current_exe.with_file_name(format!("{name}.update")),
    )
}

fn exe_dir_is_writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".soundcore-write-{}", std::process::id()));
    let _ = fs::remove_file(&probe);
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(file) => {
            drop(file);
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

pub fn needs_elevated_installer() -> bool {
    match std::env::current_exe() {
        Ok(exe) => exe
            .parent()
            .map(|dir| !exe_dir_is_writable(dir))
            .unwrap_or(true),
        Err(_) => true,
    }
}

pub fn cleanup_old_exe() {
    if let Ok(exe) = std::env::current_exe() {
        let (old_exe, _) = exe_paths(&exe);
        if old_exe.exists() {
            let _ = fs::remove_file(&old_exe);
        }
    }
}

fn build_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent("SoundCore-Updater")
        .no_gzip()
        .no_brotli()
        .no_deflate()
        .timeout(CLIENT_TIMEOUT)
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())
}

fn asset_fields(assets: &[serde_json::Value], name: &str) -> Option<(String, String, u64)> {
    let asset = assets.iter().find(|a| a["name"].as_str() == Some(name))?;
    let url = asset["browser_download_url"].as_str()?.to_string();
    let digest = asset["digest"].as_str().unwrap_or("").to_string();
    let size = asset["size"].as_u64().unwrap_or(0);
    Some((url, digest, size))
}

pub async fn check_for_update() -> Result<UpdateInfo, String> {
    let current_version_str = get_current_version();
    let current_version = parse_version(&current_version_str)
        .ok_or_else(|| format!("Invalid current version: {current_version_str}"))?;

    let url = std::env::var(API_URL_ENV)
        .unwrap_or_else(|_| format!("https://api.github.com/repos/{REPO}/releases/latest"));
    let client = build_client()?;

    let response = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Failed to check for updates: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()));
    }

    let release: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    let tag_name = release["tag_name"]
        .as_str()
        .ok_or("Missing tag_name in release")?;
    let latest_version =
        parse_version(tag_name).ok_or_else(|| format!("Invalid version in tag: {tag_name}"))?;

    if latest_version <= current_version {
        return Err("No update available".to_string());
    }

    let assets = release["assets"]
        .as_array()
        .ok_or("Missing assets in release")?;

    let (download_url, digest, size) = asset_fields(assets, ASSET_NAME)
        .ok_or_else(|| format!("{ASSET_NAME} not found in release assets"))?;
    let (installer_url, installer_digest, installer_size) =
        asset_fields(assets, INSTALLER_ASSET_NAME)
            .ok_or_else(|| format!("{INSTALLER_ASSET_NAME} not found in release assets"))?;

    Ok(UpdateInfo {
        current_version: current_version_str,
        latest_version: latest_version.to_string(),
        download_url,
        digest,
        size,
        installer_url,
        installer_digest,
        installer_size,
    })
}

fn verify_sha256(file_path: &Path, expected_digest: &str) -> Result<(), String> {
    if expected_digest.is_empty() {
        return Ok(());
    }

    let expected = expected_digest
        .strip_prefix("sha256:")
        .unwrap_or(expected_digest);

    let bytes = fs::read(file_path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual = format!("{:x}", hasher.finalize());

    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!(
            "SHA-256 mismatch: expected {expected}, got {actual} (size: {} bytes)",
            bytes.len()
        ));
    }

    Ok(())
}

async fn download_asset(
    url: &str,
    digest: &str,
    size: u64,
    file_name: &str,
    label: &str,
    progress: &dyn Fn(UpdateProgress),
) -> Result<PathBuf, String> {
    let updates_dir = get_updates_dir()?;
    let download_path = updates_dir.join(format!("{file_name}.download"));

    if download_path.exists() {
        fs::remove_file(&download_path).map_err(|e| e.to_string())?;
    }

    progress(UpdateProgress {
        phase: "downloading".to_string(),
        percent: 0.0,
        message: format!("Starting {label} download..."),
    });

    let client = build_client()?;
    let response = client
        .get(url)
        .header("Accept", "application/octet-stream")
        .send()
        .await
        .map_err(|e| format!("Failed to start {label} download: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "{label} download failed with status: {}",
            response.status()
        ));
    }

    let mut downloaded: u64 = 0;
    let mut file = fs::File::create(&download_path).map_err(|e| e.to_string())?;
    let mut stream = response;
    let mut last_emit = std::time::Instant::now();

    use std::io::Write;
    while let Some(chunk) = stream
        .chunk()
        .await
        .map_err(|e| format!("Download error: {e}"))?
    {
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaded += chunk.len() as u64;

        if size > 0 && last_emit.elapsed().as_millis() > 200 {
            let percent = (downloaded as f64 / size as f64) * 100.0;
            progress(UpdateProgress {
                phase: "downloading".to_string(),
                percent,
                message: format!("Downloading {label}... {percent:.0}%"),
            });
            last_emit = std::time::Instant::now();
        }
    }

    file.flush().map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);

    progress(UpdateProgress {
        phase: "verifying".to_string(),
        percent: 100.0,
        message: format!("Verifying {label}..."),
    });

    verify_sha256(&download_path, digest)?;
    Ok(download_path)
}

pub async fn download_update(
    info: &UpdateInfo,
    progress: &dyn Fn(UpdateProgress),
) -> Result<PathBuf, String> {
    download_asset(
        &info.download_url,
        &info.digest,
        info.size,
        ASSET_NAME,
        "update",
        progress,
    )
    .await
}

pub async fn download_installer(
    info: &UpdateInfo,
    progress: &dyn Fn(UpdateProgress),
) -> Result<PathBuf, String> {
    download_asset(
        &info.installer_url,
        &info.installer_digest,
        info.installer_size,
        INSTALLER_ASSET_NAME,
        "installer",
        progress,
    )
    .await
}

pub fn launch_installer_elevated(installer_path: &Path) -> Result<(), String> {
    let status = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Start-Process -FilePath $env:SOC_INSTALLER -Verb RunAs",
        ])
        .env("SOC_INSTALLER", installer_path)
        .status()
        .map_err(|e| format!("Failed to launch elevated installer: {e}"))?;
    if !status.success() {
        return Err("Elevated installer launch was cancelled or failed".to_string());
    }
    Ok(())
}

pub fn install_update(
    download_path: &Path,
    expected_version: &str,
    progress: &dyn Fn(UpdateProgress),
) -> Result<(), String> {
    ensure_newer(&get_current_version(), expected_version)?;

    let staged_len = fs::metadata(download_path)
        .map_err(|e| format!("Failed to stat staged update: {e}"))?
        .len();
    if staged_len == 0 {
        return Err("Staged update is empty; aborting install".to_string());
    }

    let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let target_exe = current_exe.clone();
    let exe_dir = target_exe
        .parent()
        .ok_or_else(|| "Could not resolve executable directory".to_string())?;
    let current_name = target_exe
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (old_exe, temp_exe) = exe_paths(&current_exe);
    let wrong_exe = exe_dir.join(format!("{current_name}.wrong"));

    if !exe_dir_is_writable(exe_dir) {
        return Err(format!(
            "Executable directory '{}' is not writable; installer must be used",
            exe_dir.display()
        ));
    }

    progress(UpdateProgress {
        phase: "installing".to_string(),
        percent: 0.0,
        message: "Preparing update...".to_string(),
    });

    if old_exe.exists() {
        fs::remove_file(&old_exe).map_err(|e| format!("Failed to remove old backup: {e}"))?;
    }
    if temp_exe.exists() {
        fs::remove_file(&temp_exe).map_err(|e| format!("Failed to remove old temp: {e}"))?;
    }
    if wrong_exe.exists() {
        fs::remove_file(&wrong_exe)
            .map_err(|e| format!("Failed to remove old failed update: {e}"))?;
    }

    fs::copy(download_path, &temp_exe).map_err(|e| format!("Failed to prepare new exe: {e}"))?;

    fs::rename(&current_exe, &old_exe).map_err(|e| {
        let _ = fs::remove_file(&temp_exe);
        format!("Failed to backup current exe: {e}")
    })?;

    fs::rename(&temp_exe, &target_exe).map_err(|e| {
        let _ = fs::rename(&old_exe, &current_exe);
        let _ = fs::remove_file(&temp_exe);
        format!("Failed to replace exe: {e}")
    })?;

    let target_len = fs::metadata(&target_exe)
        .map_err(|e| format!("Update vanished after replace: {e}"))?
        .len();
    if target_len == 0 || target_len != staged_len {
        fs::rename(&target_exe, &wrong_exe).map_err(|e| {
            format!(
                "Installed exe is invalid and could not be isolated: {e}; original remains at {}",
                old_exe.display()
            )
        })?;
        return match fs::rename(&old_exe, &current_exe) {
            Ok(()) => {
                let _ = fs::remove_file(&wrong_exe);
                Err(format!(
                    "Installed exe looks wrong (expected {staged_len} bytes, got {target_len}); rolled back"
                ))
            }
            Err(err) => Err(format!(
                "Installed exe looks wrong (expected {staged_len} bytes, got {target_len}); rollback failed: {err}; original remains at {}",
                old_exe.display()
            )),
        };
    }

    let _ = fs::remove_file(download_path);

    progress(UpdateProgress {
        phase: "restarting".to_string(),
        percent: 100.0,
        message: "Restarting...".to_string(),
    });

    let mut cfg = crate::config::load();
    cfg.update_restart = true;
    if let Err(err) = crate::config::save(&cfg) {
        tracing::warn!("failed to persist update_restart flag: {err}");
    }

    Command::new(&target_exe)
        .spawn()
        .map_err(|e| format!("Failed to launch new version: {e}"))?;

    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("soundcore-updater-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parses_release_assets() {
        let assets = serde_json::json!([
            {
                "name": ASSET_NAME,
                "browser_download_url": "https://example.test/update.exe",
                "digest": "sha256:abc",
                "size": 123
            },
            {
                "name": INSTALLER_ASSET_NAME,
                "browser_download_url": "https://example.test/installer.exe",
                "digest": "sha256:def",
                "size": 456
            }
        ]);
        let assets = assets.as_array().unwrap();

        let (url, digest, size) = asset_fields(assets, ASSET_NAME).unwrap();
        assert_eq!(url, "https://example.test/update.exe");
        assert_eq!(digest, "sha256:abc");
        assert_eq!(size, 123);

        let (url, digest, size) = asset_fields(assets, INSTALLER_ASSET_NAME).unwrap();
        assert_eq!(url, "https://example.test/installer.exe");
        assert_eq!(digest, "sha256:def");
        assert_eq!(size, 456);

        assert!(asset_fields(assets, "missing.exe").is_none());
    }

    #[test]
    fn derives_exe_paths_from_current_name() {
        let exe = PathBuf::from(r"C:\Apps\soundcore-desktop.exe");
        let (old_exe, temp_exe) = exe_paths(&exe);
        assert_eq!(old_exe, PathBuf::from(r"C:\Apps\soundcore-desktop.exe.old"));
        assert_eq!(
            temp_exe,
            PathBuf::from(r"C:\Apps\soundcore-desktop.exe.update")
        );
    }

    #[test]
    fn verifies_sha256_digest() {
        let dir = temp_dir("sha");
        let file = dir.join("payload.bin");
        fs::write(&file, b"update payload").unwrap();

        let mut hasher = Sha256::new();
        hasher.update(b"update payload");
        let digest = format!("sha256:{:x}", hasher.finalize());

        verify_sha256(&file, &digest).unwrap();
        assert!(verify_sha256(&file, "sha256:00").is_err());
        verify_sha256(&file, "").unwrap();
    }

    #[tokio::test]
    async fn reads_newer_release_from_api_override() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let body = serde_json::json!({
            "tag_name": "v1.9.0",
            "assets": [
                {
                    "name": ASSET_NAME,
                    "browser_download_url": format!("http://{addr}/update.exe"),
                    "digest": "sha256:abc",
                    "size": 123
                },
                {
                    "name": INSTALLER_ASSET_NAME,
                    "browser_download_url": format!("http://{addr}/installer.exe"),
                    "digest": "sha256:def",
                    "size": 456
                }
            ]
        })
        .to_string();

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0u8; 2048];
            let _ = stream.read(&mut request);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        unsafe {
            std::env::set_var(API_URL_ENV, format!("http://{addr}/release.json"));
        }
        let info = check_for_update().await.unwrap();
        unsafe {
            std::env::remove_var(API_URL_ENV);
        }
        server.join().unwrap();

        assert_eq!(info.latest_version, "1.9.0");
        assert_eq!(info.download_url, format!("http://{addr}/update.exe"));
        assert_eq!(info.installer_url, format!("http://{addr}/installer.exe"));
    }
}
