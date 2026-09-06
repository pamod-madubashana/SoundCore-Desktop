//! Windows backdrop effects (Mica, Acrylic) using window-vibrancy.
//!
//! All visual effect code is Windows-only. On Linux/macOS this module compiles
//! to a no-op `apply_backdrop` so callers don't need conditional compilation.

use tauri::WebviewWindow;

/// Apply the platform-native translucent backdrop to a window.
///
/// On Windows: tries Mica → Acrylic → opaque (no-op, window stays opaque).
/// On Linux/macOS: no-op — the compositor or existing transparent setup handles it.
pub fn apply_backdrop(window: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        apply_windows_backdrop(window);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
    }
}

#[cfg(target_os = "windows")]
fn apply_windows_backdrop(window: &WebviewWindow) {
    use window_vibrancy::{apply_mica, apply_acrylic};

    // Try Mica first (Windows 11 only). Pass None to match system dark/light preference.
    if apply_mica(window, None).is_ok() {
        tracing::info!("applied Mica backdrop");
        return;
    }

    // Acrylic works on Windows 10 1903+ and Windows 11.
    // Use a dark semi-transparent tint that matches the SoundCore UI.
    if apply_acrylic(window, Some((18, 18, 20, 180))).is_ok() {
        tracing::info!("applied Acrylic backdrop");
        return;
    }

    // Both failed — window stays opaque with its transparent CSS background.
    tracing::info!("backdrop effects unavailable, using opaque surface");
}

/// Remove backdrop effects from a window (useful for cleanup or fallback).
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn clear_backdrop(window: &WebviewWindow) {
    use window_vibrancy::{clear_mica, clear_acrylic};
    let _ = clear_mica(window);
    let _ = clear_acrylic(window);
}
