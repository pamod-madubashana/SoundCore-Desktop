//! Platform detection and Windows accent color for theme selection.

use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct PlatformInfo {
    pub os: String,
    pub accent_color: Option<String>,
    pub accent_light2: Option<String>,
}

/// Returns the current platform identifier.
#[tauri::command]
pub fn get_platform() -> String {
    std::env::consts::OS.to_string()
}

/// Returns the Windows system accent color as a hex string (e.g. "#0078D4"),
/// or None on non-Windows / if the registry read fails.
#[tauri::command]
pub fn get_windows_accent() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        read_windows_accent().ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn read_windows_accent() -> Result<String, Box<dyn std::error::Error>> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let dwm = hkcu.open_subkey("Software\\Microsoft\\Windows\\DWM")?;

    // AccentColor is stored as 0xAABBGGRR (ABGR) — convert to #RRGGBB
    let val: u32 = dwm.get_value("AccentColor")?;
    let r = (val & 0xFF) as u8;
    let g = ((val >> 8) & 0xFF) as u8;
    let b = ((val >> 16) & 0xFF) as u8;

    Ok(format!("#{:02X}{:02X}{:02X}", r, g, b))
}

/// Try to get the AccentLight2 variant from AccentPalette registry blob.
/// AccentPalette contains 8 RGBA entries (32 bytes) representing tonal variants.
#[cfg(target_os = "windows")]
fn read_accent_palette_light2() -> Option<String> {
    use windows::Win32::System::Registry::*;
    use windows::core::PCWSTR;

    unsafe {
        let mut key = HKEY::default();
        let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\DWM\0".encode_utf16().collect();
        let err = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            Some(0),
            KEY_READ,
            &mut key,
        );
        if err.is_err() {
            return None;
        }

        let mut buf = vec![0u8; 256];
        let mut buf_len = buf.len() as u32;
        let mut reg_type = REG_BINARY;
        let value_name: Vec<u16> = "AccentPalette\0".encode_utf16().collect();

        let err = RegQueryValueExW(
            key,
            PCWSTR(value_name.as_ptr()),
            None,
            Some(&mut reg_type),
            Some(buf.as_mut_ptr()),
            Some(&mut buf_len),
        );

        let _ = RegCloseKey(key);

        if err.is_err() {
            return None;
        }

        buf.truncate(buf_len as usize);
        let palette = &buf;

        tracing::info!("AccentPalette raw ({} bytes): {:?}", palette.len(), palette);

        for i in 0..8.min(palette.len() / 4) {
            let o = i * 4;
            tracing::info!(
                "  AccentPalette[{}]: R={:02X} G={:02X} B={:02X} A={:02X}",
                i, palette[o], palette[o + 1], palette[o + 2], palette[o + 3]
            );
        }

        if palette.len() >= 12 {
            let offset = 8;
            let r = palette[offset];
            let g = palette[offset + 1];
            let b = palette[offset + 2];
            tracing::info!("AccentPalette Light2 (offset 8): #{:02X}{:02X}{:02X}", r, g, b);
            return Some(format!("#{:02X}{:02X}{:02X}", r, g, b));
        }
    }
    None
}

/// Returns combined platform info in a single call (avoids two round-trips).
#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    let accent = get_windows_accent();
    let accent_light2 = {
        #[cfg(target_os = "windows")]
        { read_accent_palette_light2() }
        #[cfg(not(target_os = "windows"))]
        { None }
    };

    tracing::info!("platform info: accent={:?}, accent_light2={:?}", accent, accent_light2);

    PlatformInfo {
        os: std::env::consts::OS.to_string(),
        accent_color: accent,
        accent_light2,
    }
}
