//! Windows backdrop effects (Acrylic, Mica).
//!
//! All visual effect code is Windows-only. On Linux/macOS this module compiles
//! to a no-op `apply_backdrop` so callers don't need conditional compilation.

use tauri::WebviewWindow;

/// Apply the platform-native translucent backdrop to a window.
///
/// On Windows: frosted Acrylic (like Quick Settings) first, then Mica, then
/// opaque fallback. Acrylic lets the desktop show through; Mica is subtler and
/// more opaque, so it stays a last resort.
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
    use window_vibrancy::{apply_acrylic, apply_mica};

    // 1) Native Tauri v2 backdrop: frosted Acrylic, like Windows Quick Settings.
    if apply_tauri_acrylic(window).is_ok() {
        tracing::debug!("applied native Acrylic backdrop");
        return;
    }

    // 2) window-vibrancy Acrylic (Windows 10 1903+ and Windows 11).
    // Dark semi-transparent tint matching the SoundCore UI; the low alpha is
    // what lets the desktop show through for the frosted effect.
    if apply_acrylic(window, Some((18, 18, 20, 140))).is_ok() {
        tracing::debug!("applied Acrylic backdrop");
        return;
    }

    // 3) Last resort: Mica (Windows 11 only, more opaque than Acrylic).
    // Pass None to match system dark/light preference.
    if apply_mica(window, None).is_ok() {
        tracing::debug!("applied Mica backdrop");
        return;
    }

    // All failed — window keeps its transparent CSS background, opaque surface.
    tracing::debug!("backdrop effects unavailable, using opaque surface");
}

/// Native Tauri v2 frosted-Acrylic effect (Windows only).
#[cfg(target_os = "windows")]
fn apply_tauri_acrylic(window: &WebviewWindow) -> tauri::Result<()> {
    use tauri::window::{Effect, EffectsBuilder};
    window.set_effects(Some(
        EffectsBuilder::new().effect(Effect::Acrylic).build(),
    ))
}

/// Remove backdrop effects from a window (useful for cleanup or fallback).
#[cfg(target_os = "windows")]
#[allow(dead_code)]
pub fn clear_backdrop(window: &WebviewWindow) {
    use window_vibrancy::{clear_acrylic, clear_mica};
    let _ = window.set_effects(None);
    let _ = clear_acrylic(window);
    let _ = clear_mica(window);
}
