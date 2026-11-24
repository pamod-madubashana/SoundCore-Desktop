//! Device Information screen — read-only hardware details.

use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, prelude::*};
use serde_json::Value;

use super::common::chevron_left;
use crate::ui::scale::px;
use crate::ui::{RootView, data, theme};

pub fn render(
    root: &mut RootView,
    d: &crate::DeviceStateDto,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let view = cx.entity();
    let settings = root.cached_settings(&d.mac_address);
    let s = &settings;

    let mut rows: Vec<gpui::Div> = Vec::new();
    for category in &d.categories {
        if category.id != "deviceInformation" {
            continue;
        }
        for setting in &category.settings {
            let Some(value) = s.get(&setting.id) else {
                continue;
            };
            let text = value
                .get("translatedValue")
                .or_else(|| value.get("value"))
                .map(|item| match item {
                    Value::String(text) => text.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            rows.push(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(10.))
                    .px_3()
                    .py_2()
                    .rounded_xl()
                    .bg(theme::surface())
                    .border_1()
                    .border_color(theme::border_subtle())
                    .child(
                        div()
                            .text_size(px(11.))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(theme::foreground())
                            .child(data::pretty_setting_id(&setting.id)),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(theme::muted())
                            .child(text),
                    ),
            );
        }
    }
    let empty = rows.is_empty();

    div()
        .size_full()
        .relative()
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .top(px(10.))
                .left(px(0.))
                .right(px(0.))
                .h(px(44.))
                .px_3()
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .border_b_1()
                .border_color(theme::border_medium())
                .child(
                    div()
                        .absolute()
                        .left(px(12.))
                        .child(back_button(&view)),
                )
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::foreground())
                        .child("Device Information"),
                ),
        )
        .child(
            div()
                .id("device-info-body")
                .absolute()
                .top(px(59.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .overflow_y_scroll()
                .p_3()
                .flex()
                .flex_col()
                .gap(px(8.))
                .when(!empty, |el| el.children(rows))
                .when(empty, |el| {
                    el.child(
                        div()
                            .w_full()
                            .py(px(16.))
                            .text_center()
                            .text_size(px(12.))
                            .text_color(theme::muted())
                            .child("No device information reported"),
                    )
                }),
        )
}

fn back_button(view: &Entity<RootView>) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    div()
        .id("device-info-back")
        .p_1()
        .rounded_md()
        .text_color(theme::muted())
        .flex()
        .items_center()
        .justify_center()
        .hover(|st| st.text_color(theme::foreground()).bg(theme::hover_medium()))
        .cursor_pointer()
        .on_click(move |_, _, cx| {
            view.update(cx, |root, cx| {
                root.show_device_info = false;
                cx.notify();
            });
        })
        .child(chevron_left(14., theme::muted()))
}
