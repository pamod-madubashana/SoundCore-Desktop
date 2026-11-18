//! MyDevices screen (saved devices list).

use gpui::{Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, rgba};
use crate::UI::scale::px;

use super::common::{bluetooth_icon, plus_icon, refresh_icon, settings_dots_icon};
use crate::UI::{RootView, data, theme};

pub fn render(
    root: &mut RootView,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let view = cx.entity();
    let devices = root.devices.clone();
    let bt_on = root.bt_on;
    let scanning = root.scanning;

    let off = bt_on == Some(false);
    let status_text = if off {
        "Bluetooth off".to_string()
    } else if scanning {
        "Bluetooth is on · Scanning…".to_string()
    } else {
        "Bluetooth is on · Watching for devices".to_string()
    };

    div()
        .size_full()
        .flex()
        .flex_col()
        .min_h_0()
        // Header
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .px_4()
                .pt_4()
                .pb_3()
                .child(
                    div()
                        .size(px(36.))
                        .flex_shrink_0()
                        .rounded_xl()
                        .bg(theme::brand_15())
                        .border_1()
                        .border_color(theme::brand_40())
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(bluetooth_icon(18.)),
                )
                .child(
                    div()
                        .min_w_0()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_size(px(19.))
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(theme::foreground())
                                .child("My Devices"),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .mt(px(2.))
                                .text_color(theme::muted())
                                .child("Pick a device to connect and control"),
                        ),
                )
                .child(
                    div()
                        .id("settings-dots")
                        .size(px(30.))
                        .rounded_full()
                        .bg(rgba(0xffffff14))
                        .border_1()
                        .border_color(rgba(0xffffff26))
                        .flex()
                        .items_center()
                        .justify_center()
                        .flex_shrink_0()
                        .cursor_pointer()
                        .hover(|s| s.bg(rgba(0xffffff24)))
                        .on_click(move |_, _window, _cx| {
                            crate::open_url("ms-settings:privacy".into());
                        })
                        .child(settings_dots_icon()),
                ),
        )
        // Divider (inset like target: x=30..381 on 411 panel)
        .child(
            div()
                .px_4()
                .child(div().h(px(1.)).bg(theme::border_subtle()).w_full()),
        )
        // List
        .child(
            div()
                .id("device-list")
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .px_4()
                .pt_3()
                .pb_2()
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .text_size(px(10.5))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::muted())
                        .child("PAIRED"),
                )
                .children(devices.iter().map(|d| device_card(d)))
                // Ghost "Add a new device" helper card
                .child(add_device_ghost()),
        )
        // Update notice slot
        .child(
            div()
                .px_4()
                .pb_2()
                .flex()
                .justify_center()
                .child(super::update_notice(root, &view)),
        )
        // Footer status pill + Refresh
        .child(
            div()
                .px_4()
                .pb_4()
                .child(
                    div()
                        .h(px(42.))
                        .rounded_full()
                        .bg(theme::surface())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .px_4()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1_5()
                                .child(status_dot(off))
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(if off {
                                            theme::error_text()
                                        } else {
                                            theme::foreground()
                                        })
                                        .opacity(if off { 1.0 } else { 0.7 })
                                        .child(status_text),
                                ),
                        )
                        .child({
                            let view = view.clone();
                            div()
                                .id("refresh-btn")
                                .flex()
                                .items_center()
                                .gap_1()
                                .cursor_pointer()
                                .text_size(px(11.5))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme::brand_secondary())
                                .hover(|s| s.text_color(theme::brand()))
                                .on_click(move |_, _window, cx| {
                                    let view = view.clone();
                                    view.update(cx, |root, cx| root.refresh(cx));
                                })
                                .child(refresh_icon(12., theme::brand_secondary()))
                                .child("Refresh")
                        }),
                ),
        )
}

fn status_dot(off: bool) -> impl IntoElement {
    let color = if off { theme::error() } else { theme::success() };
    div().relative().size(px(10.)).child(
        div()
            .absolute()
            .size(px(10.))
            .rounded_full()
            .bg(color)
            .opacity(0.25),
    ).child(
        div()
            .absolute()
            .left(px(1.))
            .top(px(1.))
            .size(px(8.))
            .rounded_full()
            .bg(color),
    )
}

fn add_device_ghost() -> impl IntoElement {
    div()
        .id("add-device-ghost")
        .w_full()
        .h(px(64.))
        .rounded(px(18.))
        .border_1()
        .border_dashed()
        .border_color(rgba(0xffffff2e))
        .flex()
        .items_center()
        .gap(px(12.))
        .px(px(16.))
        .cursor_pointer()
        .hover(|s| s.bg(theme::hover_subtle()))
        .on_click(move |_, _window, _cx| {
            crate::open_url("ms-settings:bluetooth".into());
        })
        .child(
            div()
                .size(px(28.))
                .rounded_full()
                .bg(rgba(0xffffff14))
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0()
                .child(plus_icon(14., rgba(0xffffffcc).into())),
        )
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_col()
                .gap(px(2.))
                .child(
                    div()
                        .text_size(px(12.5))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgba(0xffffffe6))
                        .child("Add a new device"),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .text_color(theme::muted())
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child("Put your device in pairing mode to find it"),
                ),
        )
}

fn device_card(d: &crate::DeviceStateDto) -> impl IntoElement {
    let name = if d.name.trim().is_empty() {
        d.model.clone()
    } else {
        d.name.clone()
    };
    let kind = data::device_kind_label(&name);
    let connected = d.connected;
    let mac = d.mac_address.clone();

    let (status_label, pill_bg, pill_ring, pill_text, dot) = if connected {
        (
            "Connected",
            theme::success_14(),
            theme::success_40(),
            theme::success_text(),
            theme::success(),
        )
    } else {
        (
            "Not Connected",
            theme::error_14(),
            theme::error_40(),
            theme::error_text(),
            theme::error(),
        )
    };

    // Status pill: direct child of items_start column so it hugs content.
    let status_pill = div()
        .h(px(20.))
        .px(px(10.))
        .rounded_full()
        .bg(pill_bg)
        .border_1()
        .border_color(pill_ring)
        .flex()
        .items_center()
        .gap(px(6.))
        .child(div().size(px(6.)).rounded_full().bg(dot))
        .child(
            div()
                .text_size(px(10.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(pill_text)
                .child(status_label),
        );

    div()
        .id(gpui::SharedString::from(format!("device-card-{mac}")))
        .w_full()
        .rounded(px(20.))
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border_subtle())
        .p(px(13.))
        .flex()
        .items_center()
        .gap(px(13.))
        .hover(|s| s.bg(theme::hover_subtle()))
        .cursor_pointer()
        .on_click(move |_, _window, _cx| {
            // React cards are display-only (connect happens via backend scan).
        })
        .child(art_tile(d, name.clone()))
        .child(
            div()
                .min_w_0()
                .flex_1()
                .overflow_hidden()
                .flex()
                .flex_col()
                // Children align left (hug) instead of stretch — pill must not fill.
                .items_start()
                .child(
                    div()
                        .w_full()
                        .text_size(px(15.))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme::foreground())
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(name),
                )
                .child(
                    div()
                        .w_full()
                        .text_size(px(11.))
                        .mt(px(2.))
                        .text_color(theme::muted())
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(kind),
                )
                .child(div().mt(px(6.)).child(status_pill)),
        )
        // Connect button (gradient + glow) when not connected
        .when(!connected, |el| {
            el.child(
                div()
                    .id(gpui::SharedString::from(format!("connect-{mac}")))
                    .h(px(30.))
                    .px(px(14.))
                    .rounded_full()
                    .bg(gpui::linear_gradient(
                        90.0,
                        gpui::linear_color_stop(theme::brand(), 0.0),
                        gpui::linear_color_stop(theme::brand_end(), 1.0),
                    ))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .text_size(px(12.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme::brand_foreground())
                    .shadow(vec![gpui::BoxShadow {
                        color: gpui::rgba(0x6d6cff8c).into(),
                        offset: gpui::point(px(0.), px(4.)),
                        blur_radius: px(6.),
                        spread_radius: px(0.),
                        inset: false,
                    }])
                    .cursor_pointer()
                    .on_click(move |_, _window, _cx| {
                        crate::open_url("ms-settings:bluetooth".into());
                    })
                    .child("Connect"),
            )
        })
}

fn art_tile(_d: &crate::DeviceStateDto, name: String) -> impl IntoElement {
    // Target mock uses a fixed 56×56 line-art tile (iconBg gradient), not photos.
    // min_w_0/min_h_0: flex automatic minimum size would otherwise let the
    // child expand the tile past size(56).
    div()
        .size(px(56.))
        .min_w_0()
        .min_h_0()
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .rounded_2xl()
        .bg(gpui::linear_gradient(
            135.0,
            gpui::linear_color_stop(gpui::rgb(0x2a3354), 0.0),
            gpui::linear_color_stop(gpui::rgb(0x1a2038), 1.0),
        ))
        .border_1()
        .border_color(rgba(0xffffff26))
        .overflow_hidden()
        .child(device_art_placeholder(&name, gpui::rgb(0xc9d6ff).into()))
}

/// Simplified DeviceArt fallback (earbuds / speaker / over-ear blocks).
pub(crate) fn device_art_placeholder(name: &str, brand: gpui::Hsla) -> impl IntoElement {
    let kind = crate::UI::data::device_art_category(name);

    match kind {
        "speaker" => div()
            .relative()
            .size(px(34.))
            .child(
                div()
                    .absolute()
                    .left(px(7.))
                    .top(px(3.))
                    .w(px(20.))
                    .h(px(28.))
                    .rounded_lg()
                    .bg(brand),
            )
            .child(
                div()
                    .absolute()
                    .left(px(10.))
                    .top(px(16.))
                    .size(px(14.))
                    .rounded_full()
                    .bg(rgba(0xffffff1a))
                    .border_1()
                    .border_color(rgba(0xffffff33)),
            ),
        "overear" => div()
            .relative()
            .size(px(34.))
            .child(
                div()
                    .absolute()
                    .left(px(6.))
                    .top(px(4.))
                    .w(px(22.))
                    .h(px(16.))
                    .border_t_3()
                    .border_l_3()
                    .border_r_3()
                    .border_color(brand)
                    .rounded_t_full(),
            )
            .child(
                div()
                    .absolute()
                    .left(px(3.))
                    .top(px(14.))
                    .w(px(7.))
                    .h(px(12.))
                    .rounded_md()
                    .bg(brand),
            )
            .child(
                div()
                    .absolute()
                    .right(px(3.))
                    .top(px(14.))
                    .w(px(7.))
                    .h(px(12.))
                    .rounded_md()
                    .bg(brand),
            ),
        _ => {
            // Earbuds: line-art bud + sound waves (devices_window.svg).
            div()
                .relative()
                .size(px(34.))
                .child(earbud_line_art(brand))
        }
    }
}

/// Single earbud outline + waves, centered in a 34×34 box.
fn earbud_line_art(stroke: gpui::Hsla) -> impl IntoElement {
    // Design centered at (0,0), scaled into 34×34.
    let k = 34.0 / 40.0;
    let lw = 2.2 * k;
    div().size(px(34.)).child(gpui::canvas(
        move |_bounds, _window, _cx| (),
        move |bounds, _, window, _cx| {
            let cx = bounds.origin.x + px(17.);
            let cy = bounds.origin.y + px(17.);
            let map = |x: f32, y: f32| gpui::point(cx + px(x * k), cy + px(y * k));

            // Bud body: M-7 -10 a7 7 0 1 1 7 7 v9 a3 3 0 0 1 -6 0
            let mut body = gpui::PathBuilder::stroke(px(lw));
            body.move_to(map(-7., -10.));
            body.relative_arc_to(
                gpui::point(px(7.), px(7.)),
                px(0.),
                true,
                true,
                gpui::point(px(7.), px(7.)),
            );
            body.line_to(map(0., 6.));
            body.relative_arc_to(
                gpui::point(px(3.), px(3.)),
                px(0.),
                false,
                true,
                gpui::point(px(-6.), px(0.)),
            );
            if let Ok(path) = body.build() {
                window.paint_path(path, stroke);
            }

            // Waves: M10 -4 a5 5 0 0 1 0 8 / M14 -8 a10 10 0 0 1 0 16
            for (rad, y0, y1, alpha) in [(5.0f32, -4.0f32, 4.0f32, 0.6f32), (10.0, -8.0, 8.0, 0.35)] {
                let mut w = gpui::PathBuilder::stroke(px(lw));
                w.move_to(map(10. + (rad - 5.), y0));
                w.relative_arc_to(
                    gpui::point(px(rad), px(rad)),
                    px(0.),
                    false,
                    true,
                    gpui::point(px(0.), px(y1 - y0)),
                );
                if let Ok(path) = w.build() {
                    window.paint_path(path, stroke.opacity(alpha));
                }
            }

            // Dot at bud tip: filled circle approx via short cross is wrong —
            // use a tiny filled path (polygon diamond) for the r=1 marker.
            let mut dot = gpui::PathBuilder::fill();
            let d = 1.0 * k;
            let c = map(-7., -10.);
            let pts = [
                gpui::point(c.x + px(d), c.y),
                gpui::point(c.x, c.y + px(d)),
                gpui::point(c.x - px(d), c.y),
                gpui::point(c.x, c.y - px(d)),
            ];
            dot.add_polygon(&pts, true);
            if let Ok(path) = dot.build() {
                window.paint_path(path, stroke);
            }
        },
    ))
}
