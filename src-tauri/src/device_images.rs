use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;
use std::sync::LazyLock;

const CDN_DOMAIN: &str = "https://d2htfo7ft368vg.cloudfront.net";

/// Track failed downloads so we don't retry endlessly.
static FAILED: LazyLock<std::sync::Mutex<HashSet<String>>> =
    LazyLock::new(|| std::sync::Mutex::new(HashSet::new()));

fn color_code_to_name(code: &str) -> &'static str {
    match code.to_lowercase().as_str() {
        "1" => "black",
        "2" => "white",
        "3" => "blue",
        "4" => "silver",
        "5" => "pink",
        "6" => "green",
        "7" => "yellow",
        "8" => "brown",
        "9" => "red",
        "a" => "gray",
        "b" => "golden",
        "c" => "sand_white",
        "d" => "navy_blue",
        "e" => "coral",
        "f" => "sky_blue",
        "g" => "watermelonred",
        "h" => "khaki",
        "i" => "babyblue",
        "k" => "lightgray",
        "l" => "bluesilver",
        "m" => "redgolden",
        "n" => "lightblue",
        "o" => "purple",
        "p" => "camouflage",
        "q" => "creamywhite",
        "r" => "black_se",
        "s" => "white_red",
        "t" => "purple_yellow",
        "u" => "black_golden",
        "v" => "yellow_green",
        "w" => "black_pure",
        "x" => "green_pure",
        "y" => "blue_pure",
        _ => "black",
    }
}

/// Normalize model string to CDN product code.
/// E.g., "SoundcoreA3959" -> "a3959", "A3939WEU" -> "a3939"
fn normalize_model(model: &str) -> String {
    let s = model.trim().to_lowercase();
    // Strip "soundcore" prefix if present
    let s = s.strip_prefix("soundcore").unwrap_or(&s);
    // Strip trailing region suffixes like "weu", "r50"
    let s = if let Some(pos) = s.find(|c: char| !c.is_ascii_alphanumeric()) {
        &s[..pos]
    } else {
        s
    };
    let mut result = s.to_string();
    if result.ends_with("weu") || result.ends_with("r50") {
        result.truncate(result.len() - 3);
    }
    result
}

fn cache_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("soundcore-desktop").join("device_images")
}

fn cached_image_path(model: &str, color_code: &str) -> Option<PathBuf> {
    let product = normalize_model(model);
    let color_name = color_code_to_name(color_code);
    let path = cache_dir().join(format!("{product}_{color_name}_com_device.png"));
    if path.exists() { Some(path) } else { None }
}

fn zip_url(model: &str, color_code: &str) -> String {
    let product = normalize_model(model);
    let color_name = color_code_to_name(color_code);
    format!(
        "{CDN_DOMAIN}/media/prod/{product}/{color}/{product}_{color}.zip",
        product = product,
        color = color_name
    )
}

fn download_and_cache(model: &str, color_code: &str) -> anyhow::Result<PathBuf> {
    let product = normalize_model(model);
    let color_name = color_code_to_name(color_code);
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)?;

    let dest = dir.join(format!("{product}_{color_name}_com_device.png"));
    if dest.exists() {
        return Ok(dest);
    }

    let url = zip_url(model, color_code);
    tracing::info!("downloading device image from {url}");

    let client = reqwest::blocking::Client::builder()
        .user_agent("okhttp/4.9.3")
        .build()?;

    let bytes = client
        .get(&url)
        .header("Accept", "application/zip, */*")
        .send()?
        .error_for_status()?
        .bytes()?;

    let reader = std::io::Cursor::new(&bytes);
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| anyhow::anyhow!("failed to open zip: {e}"))?;

    // Find *_com_device.png inside the ZIP
    let target_suffix = "_com_device.png";
    let mut found = false;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| anyhow::anyhow!("failed to read zip entry: {e}"))?;
        let name = entry.name().to_string();
        if name.ends_with(target_suffix) && !name.starts_with("__MACOSX") {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            std::fs::write(&dest, &buf)?;
            tracing::info!("cached device image to {}", dest.display());
            found = true;
            break;
        }
    }

    if !found {
        // Fallback: try any non-macOS PNG
        for i in 0..archive.len() {
            let mut entry = archive
                .by_index(i)
                .map_err(|e| anyhow::anyhow!("failed to read zip entry: {e}"))?;
            let name = entry.name().to_string();
            if name.ends_with(".png") && !name.starts_with("__MACOSX") && !name.ends_with("._") {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                std::fs::write(&dest, &buf)?;
                tracing::info!("cached device image (fallback) to {}", dest.display());
                found = true;
                break;
            }
        }
    }

    if found {
        Ok(dest)
    } else {
        Err(anyhow::anyhow!(
            "no PNG found in ZIP for {product}/{color_name}"
        ))
    }
}

pub fn get_or_download(model: &str, color_code: &str) -> Option<PathBuf> {
    if let Some(path) = cached_image_path(model, color_code) {
        return Some(path);
    }

    let key = format!("{}:{}", normalize_model(model), color_code);
    if FAILED.lock().unwrap().contains(&key) {
        return None;
    }

    match download_and_cache(model, color_code) {
        Ok(path) => Some(path),
        Err(e) => {
            tracing::warn!("failed to download device image: {e:#}");
            FAILED.lock().unwrap().insert(key);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mapping() {
        assert_eq!(color_code_to_name("1"), "black");
        assert_eq!(color_code_to_name("2"), "white");
        assert_eq!(color_code_to_name("a"), "gray");
    }

    #[test]
    fn test_model_normalization() {
        assert_eq!(normalize_model("SoundcoreA3959"), "a3959");
        assert_eq!(normalize_model("A3939WEU"), "a3939");
        assert_eq!(normalize_model("A3959R50"), "a3959");
        assert_eq!(normalize_model("soundcorea3959"), "a3959");
    }

    #[test]
    fn test_zip_url() {
        let url = zip_url("SoundcoreA3959", "1");
        assert!(url.contains("a3959/black/a3959_black.zip"));
    }
}
