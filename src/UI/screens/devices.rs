//! MyDevices screen — `ui/src/App.tsx` `MyDevices`.

use gpui::{Context, IntoElement, ParentElement, Styled, Window, div, prelude::*, rgba};
use crate::UI::scale::px;

use super::common::bluetooth_icon;
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
                ),
        )
        // Divider
        .child(div().h(px(1.)).bg(theme::border_subtle()).w_full())
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
                        .text_size(px(10.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::muted())
                        .child("PAIRED"),
                )
                .children(devices.iter().map(|d| device_card(d))),
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
        // Footer status pill
        .child(
            div()
                .px(px(14.))
                .pb(px(14.))
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
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme::foreground())
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(name),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .mt(px(2.))
                        .text_color(theme::muted())
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .child(kind),
                )
                .child(
                    div()
                        .mt(px(6.))
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
                        ),
                ),
        )
}

fn art_tile(d: &crate::DeviceStateDto, name: String) -> impl IntoElement {
    let image = root_image(d);
    let tile = div()
        .size(px(56.))
        .flex_shrink_0()
        .rounded_2xl()
        .bg(theme::surface_elevated())
        .border_1()
        .border_color(theme::border_subtle())
        .overflow_hidden()
        .p_1_5();

    if let Some(img) = image {
        tile.child(gpui::img(img).size_full().object_fit(gpui::ObjectFit::Cover))
    } else {
        tile
            .flex()
            .items_center()
            .justify_center()
            .child(device_art_placeholder(&name, theme::brand()))
    }
}

fn root_image(d: &crate::DeviceStateDto) -> Option<std::sync::Arc<gpui::Image>> {
    let url = d.image.as_deref()?;
    let b64 = url.strip_prefix("data:image/png;base64,")?;
    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64)
        .ok()?;
    Some(std::sync::Arc::new(gpui::Image::from_bytes(
        gpui::ImageFormat::Png,
        bytes,
    )))
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
        _ => div()
            .relative()
            .size(px(34.))
            .child(
                div()
                    .absolute()
                    .left(px(7.))
                    .top(px(5.))
                    .w(px(20.))
                    .h(px(24.))
                    .rounded_md()
                    .bg(brand),
            )
            .child(
                div()
                    .absolute()
                    .left(px(10.))
                    .top(px(15.))
                    .size(px(14.))
                    .rounded_full()
                    .bg(rgba(0xffffff1a))
                    .border_1()
                    .border_color(rgba(0xffffff33)),
            ),
    }
}
