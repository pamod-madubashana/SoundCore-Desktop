//! Shared chrome helpers — text/glyph icons (GPUI divs cannot rotate).

use gpui::{Hsla, IntoElement, ParentElement, Styled, div, prelude::*};
use std::sync::OnceLock;
use crate::UI::scale::px;

use crate::UI::theme;

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

/// Radio-circle expand indicator (React `RadioCircle`).
pub(crate) fn radio_circle(active: bool) -> impl IntoElement {
    div()
        .size(px(16.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .border_2()
        .border_color(if active {
            theme::brand()
        } else {
            theme::border_strong()
        })
        .when(active, |el| el.bg(theme::brand()))
        .when(active, |el| {
            el.child(
                div()
                    .size(px(8.))
                    .rounded_full()
                    .bg(gpui::rgb(0xffffff)),
            )
        })
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

/// 12-dot spinner; `tick` selects which dots are bright.
pub(crate) fn spinner(tick: u8, color: Hsla) -> impl IntoElement {
    let frame = tick % 12;
    div().relative().size(px(28.)).children((0..12u8).map(|i| {
        let angle = i as f32 * 30.0_f32;
        let dist = 11.0_f32;
        let rad = angle.to_radians();
        let x = 14.0 + dist * rad.cos() - 3.0;
        let y = 14.0 + dist * rad.sin() - 3.0;
        let age = (frame + 12 - i) % 12;
        let opacity = if age == 0 {
            1.0
        } else {
            (1.0 - age as f32 / 12.0).max(0.15)
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
}

/// Horizontal three-dot settings glyph.
pub(crate) fn settings_dots_icon() -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .gap(px(3.4))
        .children((0..3).map(|_| {
            div()
                .size(px(3.6))
                .rounded_full()
                .bg(gpui::rgba(0xffffffcc))
        }))
}
