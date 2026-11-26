//! Shared chrome helpers — text/glyph icons (GPUI divs cannot rotate).

use crate::ui::scale::px;
use gpui::{Animation, AnimationExt as _, Hsla, IntoElement, ParentElement, Styled, div};
use std::{sync::OnceLock, time::Duration};

/// Soundcore brand glyph (Lanczos-upscaled tray icon, smooth at display size).
pub(crate) fn soundcore_glyph(size: f32) -> impl IntoElement {
    static CACHE: OnceLock<std::sync::Arc<gpui::Image>> = OnceLock::new();
    let img = CACHE.get_or_init(|| {
        let bytes = include_bytes!("../../../assets/soundcore-glyph.png");
        std::sync::Arc::new(gpui::Image::from_bytes(
            gpui::ImageFormat::Png,
            bytes.to_vec(),
        ))
    });
    gpui::img(img.clone()).size(px(size))
}

/// Close (X) glyph.
pub(crate) fn close_icon(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size * 0.9))
        .child("✕")
}

/// Chevron-right nav indicator.
pub(crate) fn chevron_right(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size * 1.1))
        .child("›")
}

/// Chevron-left (back) glyph.
pub(crate) fn chevron_left(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size * 1.1))
        .child("‹")
}

/// Chevron-down glyph for dropdown rows.
pub(crate) fn chevron_down(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size * 1.2))
        .child("⌄")
}

/// Chevron-up glyph for an open dropdown row.
pub(crate) fn chevron_up(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size * 1.2))
        .child("⌃")
}

/// Section icons extracted from the official Soundcore APK.
pub(crate) fn eq_image_icon(kind: &str, size: f32) -> impl IntoElement {
    static SPATIAL: OnceLock<std::sync::Arc<gpui::Image>> = OnceLock::new();
    static DEFAULT: OnceLock<std::sync::Arc<gpui::Image>> = OnceLock::new();

    let img = match kind {
        "spatial-icon" => SPATIAL.get_or_init(|| {
            std::sync::Arc::new(gpui::Image::from_bytes(
                gpui::ImageFormat::Png,
                include_bytes!("../../../assets/icons/eq_spatial.png").to_vec(),
            ))
        }),
        _ => DEFAULT.get_or_init(|| {
            std::sync::Arc::new(gpui::Image::from_bytes(
                gpui::ImageFormat::Png,
                include_bytes!("../../../assets/icons/eq_default.png").to_vec(),
            ))
        }),
    };
    gpui::img(img.clone()).size(px(size))
}

/// Download arrow (update badge).
pub(crate) fn download_icon(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size))
        .child("↓")
}

/// Sliders / equalizer mark for the Sound Effects row.
pub(crate) fn sliders_icon(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size * 0.95))
        .child("≡")
}

/// 12-dot spinner driven by GPUI's frame scheduler, not `cx.notify()` timers.
pub(crate) fn spinner(color: Hsla) -> impl IntoElement {
    div().relative().size(px(28.)).with_animation(
        "search-spinner",
        Animation::new(Duration::from_millis(900)).repeat_synced(),
        move |this, phase| {
            let frame = (phase * 12.0) as u8 % 12;
            this.children((0..12u8).map(|i| {
                let angle = i as f32 * 30.0_f32;
                let dist = 11.0_f32;
                let rad = angle.to_radians();
                let x = 14.0 + dist * rad.cos() - 3.0;
                let y = 14.0 + dist * rad.sin() - 3.0;
                let age = (frame + 12 - i) % 12;
                let opacity = if age == 0 {
                    1.0
                } else {
                    (1.0_f32 - age as f32 / 12.0).max(0.15_f32)
                };
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .size(px(6.))
                    .rounded_full()
                    .bg(color)
                    .opacity(opacity)
            }))
        },
    )
}

/// Gear glyph (Settings nav). VS15 keeps U+2699 monochrome, not emoji.
pub(crate) fn gear_icon(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size))
        .child("⚙︎")
}

/// Custom EQ glyph matching the official app's `ic_eq_custom.svg`.
pub(crate) fn eq_bars_icon(size: f32, color: Hsla) -> impl IntoElement {
    let s = size / 24.0;
    let line = |x: f32, top: f32, bottom: f32| {
        div()
            .absolute()
            .left(px((x - 0.75) * s))
            .top(px(top * s))
            .w(px(1.5 * s))
            .h(px((bottom - top) * s))
            .rounded_full()
            .bg(color)
    };
    let knob = |x: f32, y: f32| {
        div()
            .absolute()
            .left(px((x - 2.5) * s))
            .top(px((y - 2.5) * s))
            .size(px(5.0 * s))
            .rounded_full()
            .border_1()
            .border_color(color)
    };
    div()
        .size(px(size))
        .relative()
        .child(line(5.5, 8.0, 21.0))
        .child(line(12.0, 3.0, 9.5))
        .child(line(12.0, 14.5, 21.0))
        .child(line(18.5, 3.0, 16.0))
        .child(knob(5.5, 5.5))
        .child(knob(12.0, 12.0))
        .child(knob(18.5, 18.5))
}

/// Mixer glyph for the Control Settings row.
pub(crate) fn controls_icon(size: f32, color: Hsla) -> impl IntoElement {
    let s = size / 24.0;
    let track = |y: f32| {
        div()
            .absolute()
            .left(px(3.0 * s))
            .top(px(y * s))
            .w(px(18.0 * s))
            .h(px(2.0 * s))
            .rounded_full()
            .bg(color)
    };
    let knob = |x: f32, y: f32| {
        div()
            .absolute()
            .left(px((x - 3.0) * s))
            .top(px((y - 3.0) * s))
            .size(px(6.0 * s))
            .rounded_full()
            .bg(color)
    };
    div()
        .size(px(size))
        .relative()
        .child(track(7.0))
        .child(knob(9.0, 8.0))
        .child(track(16.0))
        .child(knob(16.0, 17.0))
}

/// Circled-i glyph (About nav).
pub(crate) fn info_icon(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size))
        .child("ⓘ")
}
