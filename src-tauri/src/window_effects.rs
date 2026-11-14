//! Windows backdrop effects (Acrylic, Mica) and native rounded window corners.
//!
//! All visual effect code is Windows-only. On Linux/macOS this module compiles
//! to no-op `apply_backdrop` / `apply_rounded_corners` so callers don't need
//! conditional compilation.

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

/// Corner radius of the popup window, in logical pixels. Matches `--radius`
/// in `ui/src/styles/theme-windows.css` so the OS-level window shape and the
/// CSS surface line up.
#[cfg(target_os = "windows")]
const CORNER_RADIUS: f64 = 12.0;

/// Round the corners of the actual OS window (not just the CSS surface), so the
/// native window frame, the Acrylic/Mica backdrop and the webview are all
/// clipped to the same rounded shape.
///
/// On Windows 11 this asks DWM for anti-aliased rounded corners. On Windows 10
/// (where DWM rejects that attribute) it falls back to clipping the window with
/// a rounded-rectangle region. On Linux/macOS: no-op.
///
/// Safe to call repeatedly; call it after the window is shown so its size is final.
pub fn apply_rounded_corners(window: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        apply_windows_rounded_corners(window);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = window;
    }
}

#[cfg(target_os = "windows")]
fn apply_windows_rounded_corners(window: &WebviewWindow) {
    use std::ffi::c_void;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DWM_WINDOW_CORNER_PREFERENCE,
        DwmSetWindowAttribute,
    };
    use windows::Win32::Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn};

    // Tauri links its own `windows` version, so rebuild the handle from the raw
    // pointer instead of using its HWND type directly.
    let Ok(raw) = window.hwnd() else {
        return;
    };
    let hwnd = HWND(raw.0);

    // 1) Windows 11: native, anti-aliased rounded corners from DWM.
    let preference: DWM_WINDOW_CORNER_PREFERENCE = DWMWCP_ROUND;
    let dwm_result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const c_void,
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        )
    };
    if dwm_result.is_ok() {
        tracing::debug!("applied native rounded window corners");
        return;
    }

    // 2) Windows 10: DWM doesn't know this attribute, so clip the window to a
    // rounded-rectangle region instead (edges are not anti-aliased).
    let (Ok(size), Ok(scale)) = (window.outer_size(), window.scale_factor()) else {
        return;
    };
    let diameter = (CORNER_RADIUS * 2.0 * scale).round() as i32;
    unsafe {
        // +1: CreateRoundRectRgn's right/bottom edges are exclusive.
        let region = CreateRoundRectRgn(
            0,
            0,
            size.width as i32 + 1,
            size.height as i32 + 1,
            diameter,
            diameter,
        );
        // On success the system owns the region; don't delete it.
        SetWindowRgn(hwnd, Some(region), true);
    }
    tracing::debug!("applied region-based rounded window corners");
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
