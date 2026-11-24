//! Control Settings screen — button actions and gestures.

use gpui::{
    AnimationExt as _, Context, Entity, IntoElement, ParentElement, SharedString, SpringAnimation,
    SpringConfig, Styled, Window, div, prelude::*,
};
use serde_json::Value;

use super::common::{chevron_down, chevron_left, chevron_up};
use crate::ui::scale::px;
use crate::ui::{RootView, data, theme};

const LEFT: &[&str] = &[
    "leftSinglePress",
    "leftDoublePress",
    "leftTriplePress",
    "leftLongPress",
    "leftSlideUp",
    "leftSlideDown",
];
const RIGHT: &[&str] = &[
    "rightSinglePress",
    "rightDoublePress",
    "rightTriplePress",
    "rightLongPress",
    "rightSlideUp",
    "rightSlideDown",
];
const OTHER: &[&str] = &[
    "doublePress",
    "mfbLongPress",
    "leftDoublePressDuringCall",
    "rightDoublePressDuringCall",
    "leftLongPressDuringCall",
    "rightLongPressDuringCall",
];

pub fn render(
    root: &mut RootView,
    d: &crate::DeviceStateDto,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let view = cx.entity();
    let settings = root.cached_settings(&d.mac_address);
    let s = &settings;
    let mac = d.mac_address.clone();
    let open = root.button_dropdown.clone();

    let mut groups: Vec<gpui::Div> = Vec::new();
    for (title, ids) in [
        ("Left button", LEFT),
        ("Right button", RIGHT),
        ("More actions", OTHER),
    ] {
        let rows: Vec<gpui::Stateful<gpui::Div>> = ids
            .iter()
            .filter_map(|id| {
                button_row(&view, &mac, id, s, open.as_deref() == Some(*id))
            })
            .collect();
        if rows.is_empty() {
            continue;
        }
        groups.push(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    div()
                        .text_size(px(10.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::muted())
                        .child(title),
                )
                .children(rows),
        );
    }
    let has_reset = s.get("resetButtonsToDefault").is_some();
    let empty = groups.is_empty();

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
                        .child("Control Settings"),
                ),
        )
        .child(
            div()
                .id("control-settings-body")
                .absolute()
                .top(px(59.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(72.))
                .overflow_y_scroll()
                .p_3()
                .flex()
                .flex_col()
                .gap(px(12.))
                .when(!empty, |el| el.children(groups))
                .when(empty, |el| {
                    el.child(
                        div()
                            .w_full()
                            .py(px(16.))
                            .text_center()
                            .text_size(px(12.))
                            .text_color(theme::muted())
                            .child("No button settings reported by this device"),
                    )
                }),
        )
        .when(has_reset, |el| {
            el.child(
                div()
                    .id("control-settings-footer")
                    .absolute()
                    .left(px(0.))
                    .right(px(0.))
                    .bottom(px(0.))
                    .h(px(72.))
                    .px_3()
                    .pt_3()
                    .pb_3()
                    .child(reset_row(&view, &mac)),
            )
        })
}

fn back_button(view: &Entity<RootView>) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    div()
        .id("control-settings-back")
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
                root.show_control_settings = false;
                root.button_dropdown = None;
                cx.notify();
            });
        })
        .child(chevron_left(14., theme::muted()))
}

fn button_row(
    view: &Entity<RootView>,
    mac: &str,
    id: &str,
    s: &data::SettingsMap,
    expanded: bool,
) -> Option<gpui::Stateful<gpui::Div>> {
    let value = s.get(id)?;
    let current = value
        .get("value")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let optional = value
        .get("type")
        .and_then(Value::as_str)
        == Some("optionalSelect");
    let mut options: Vec<(String, String)> = data::select_entries(s, id);
    if optional && !options.iter().any(|(oid, _)| oid.is_empty()) {
        options.push((String::new(), "None".into()));
    }
    if options.is_empty() {
        return None;
    }
    let current_label = options
        .iter()
        .find(|(oid, _)| oid == current)
        .map(|(_, label)| label.clone())
        .unwrap_or_else(|| {
            if current.is_empty() {
                "None".into()
            } else {
                current.to_string()
            }
        });

    let view_toggle = view.clone();
    let id_toggle = id.to_string();
    let mut row = div()
        .id(SharedString::from(format!("cs-row-{id}")))
        .w_full()
        .flex()
        .flex_col()
        .rounded_xl()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border_subtle())
        .overflow_hidden()
        .child(
            div()
                .id(SharedString::from(format!("cs-header-{id}")))
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(10.))
                .px_3()
                .py(px(11.))
                .hover(|st| st.bg(theme::hover_subtle()))
                .cursor_pointer()
                .on_click(move |_, _, cx| {
                    let view = view_toggle.clone();
                    let id = id_toggle.clone();
                    view.update(cx, move |root, cx| {
                        root.button_dropdown = if root.button_dropdown.as_deref() == Some(id.as_str())
                        {
                            None
                        } else {
                            Some(id.clone())
                        };
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme::foreground())
                        .child(data::pretty_setting_id(id)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(theme::muted())
                                .child(current_label),
                        )
                        .child(if expanded {
                            chevron_up(12., theme::muted()).into_any_element()
                        } else {
                            chevron_down(12., theme::muted()).into_any_element()
                        }),
                ),
        );
    let row_count = ((options.len() + 1) / 2).max(1) as f32;
    let target = row_count * 36.0 + 16.0;
    let drop_key = SharedString::from(format!("cs-drop-{id}"));
    row = row.child(
        div()
            .w_full()
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(6.))
                    .border_t_1()
                    .border_color(theme::border_subtle())
                    .px_3()
                    .pt_2()
                    .pb_2()
                    .children(options.into_iter().map(|(oid, label)| {
                        option_row(view, mac, id, &oid, label, oid == current)
                    })),
            )
            .with_spring(
                drop_key,
                SpringAnimation::new(SpringConfig::new(300.0, 28.0, 1.0))
                    .to(if expanded { target } else { 0.0 }),
                move |this, h| {
                    let ratio = (h / target).clamp(0.0, 1.0);
                    this.h(px(h)).opacity(ratio)
                },
            ),
    );
    Some(row)
}

fn reset_row(view: &Entity<RootView>, mac: &str) -> gpui::Div {
    let view = view.clone();
    let mac = mac.to_string();
    div().child(
        div()
            .id("cs-reset-buttons")
            .w_full()
            .px_3()
            .py_2()
            .rounded_xl()
            .bg(theme::error_14())
            .border_1()
            .border_color(theme::error_40())
            .text_size(px(11.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(theme::error_text())
            .cursor_pointer()
            .hover(|st| {
                st.bg(theme::error())
                    .border_color(theme::error())
                    .text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
            })
            .on_click(move |_, _, cx| {
                let mac = mac.clone();
                view.update(cx, move |root, cx| {
                    root.send_setting(&mac, "resetButtonsToDefault", "true");
                    cx.notify();
                });
            })
            .child("Reset Buttons To Default"),
    )
}

fn option_row(
    view: &Entity<RootView>,
    mac: &str,
    id: &str,
    oid: &str,
    label: String,
    selected: bool,
) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    let mac = mac.to_string();
    let sid = id.to_string();
    let raw = oid.to_string();
    div()
        .id(SharedString::from(format!("cs-opt-{id}-{oid}")))
        .w(px(150.))
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .py(px(8.))
        .cursor_pointer()
        .hover(|st| st.bg(theme::hover_subtle()))
        .when(selected, |el| {
            el.bg(theme::brand_10()).text_color(theme::brand())
        })
        .on_click(move |_, _, cx| {
            let view = view.clone();
            let mac = mac.clone();
            let sid = sid.clone();
            let raw = raw.clone();
            view.update(cx, move |root, cx| {
                root.send_setting(&mac, &sid, &raw);
                root.button_dropdown = None;
                cx.notify();
            });
        })
        .child(
            div()
                .text_size(px(11.))
                .font_weight(if selected {
                    gpui::FontWeight::SEMIBOLD
                } else {
                    gpui::FontWeight::MEDIUM
                })
                .child(label),
        )
        .when(selected, |el| {
            el.child(
                div()
                    .text_size(px(11.))
                    .text_color(theme::brand())
                    .child("✓"),
            )
        })
}
