//! UI length scale — matches the Tauri/React popup's dual sizing.
//!
//! React applies two factors before anything hits the screen:
//!   1. `html { font-size: 14.5px }` — rem-based Tailwind utilities scale
//!      by 14.5/16. We match this via `Window::set_rem_size(px(13.05))`
//!      (14.5 × 0.9), so gpui's `gap_1()`, `p_3()`, `text_sm()` etc. land
//!      on the same on-screen pixels.
//!   2. `.popup-window { zoom: 0.9 }` — multiplies **every** length by 0.9,
//!      including absolute `px` values. There is no CSS zoom in gpui, so
//!      every absolute `px()` call inside `src/UI` goes through this module
//!      instead of `gpui::px`.
//!
//! Result: `scale::px(18.)` renders at 16.2 logical px — same as React's
//! `text-[18px]` under zoom 0.9. Non-UI code (window bounds, tray, shell)
//! keeps using `gpui::px` unscaled.

/// React `.popup-window { zoom: 0.9 }`.
pub const ZOOM: f32 = 0.9;

/// Scaled pixel: React CSS px → gpui logical px (applies zoom 0.9).
#[inline]
pub fn px(v: f32) -> gpui::Pixels {
    gpui::px(v * ZOOM)
}

/// React rem root (html font-size 14.5px) × zoom 0.9 — pass to
/// `Window::set_rem_size` once at window creation.
#[inline]
pub fn rem_base() -> gpui::Pixels {
    gpui::px(14.5 * ZOOM)
}
