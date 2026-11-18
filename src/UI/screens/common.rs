//! Shared chrome helpers — text/glyph icons (GPUI divs cannot rotate).

use gpui::{Hsla, IntoElement, ParentElement, Styled, div, prelude::*};
use crate::UI::scale::px;

use crate::UI::theme;

/// Bluetooth glyph — path stroke matching `devices_window.svg`.
pub(crate) fn bluetooth_icon(size: f32) -> impl IntoElement {
    // Design box 36×36; path from target: M12 12 L24 24 L18 30 L18 6 L24 12 L12 24
    let k = size / 36.0;
    let pts = [
        (12.0, 12.0),
        (24.0, 24.0),
        (18.0, 30.0),
        (18.0, 6.0),
        (24.0, 12.0),
        (12.0, 24.0),
    ];
    let stroke = (2.0 * k).max(1.2);
    let color = theme::brand_secondary();

    div().size(px(size)).child(gpui::canvas(
        move |_bounds, _window, _cx| (),
        move |bounds, _, window, _cx| {
            let mut b = gpui::PathBuilder::stroke(px(stroke));
            let mut first = true;
            for (x, y) in pts {
                let p = gpui::point(
                    bounds.origin.x + px(x * k),
                    bounds.origin.y + px(y * k),
                );
                if first {
                    b.move_to(p);
                    first = false;
                } else {
                    b.line_to(p);
                }
            }
            if let Ok(path) = b.build() {
                window.paint_path(path, color);
            }
        },
    ))
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

/// Ear / volume-mode glyphs for the segmented sound-mode control.
pub(crate) fn mode_icon(kind: &str, size: f32, color: Hsla) -> impl IntoElement {
    let glyph = match kind {
        "noise" => "◉",
        "transparen" => "◎",
        _ => "○",
    };
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size * 0.95))
        .child(glyph)
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

/// Plus glyph for the add-device ghost card.
pub(crate) fn plus_icon(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size * 1.05))
        .font_weight(gpui::FontWeight::BOLD)
        .child("+")
}

/// Refresh glyph for the footer link.
pub(crate) fn refresh_icon(size: f32, color: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .text_color(color)
        .text_size(px(size * 0.95))
        .child("↻")
}
