use std::collections::HashSet;

use crate::ui::scale::px;
use gpui::{
    Context, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, prelude::*,
};
use serde_json::Value;

use super::common::chevron_left;
use crate::ui::{RootView, data, theme};

const EXCLUDED_IDS: &[&str] = &[
    "ambientSoundMode",
    "transparencyMode",
    "noiseCancelingMode",
    "multiSceneNoiseCanceling",
    "spatialAudio",
    "spatialAudioMode",
    "spatialAudioMusicMode",
    "volumeAdjustments",
    "presetEqualizerProfile",
    "customEqualizerProfile",
    "batteryLevel",
    "batteryLevelLeft",
    "batteryLevelRight",
    "caseBatteryLevel",
];

pub fn render(
    root: &mut RootView,
    d: &crate::DeviceStateDto,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let view = cx.entity();
    let settings = root.cached_settings(&d.mac_address);
    let s: &data::SettingsMap = &settings;
    let mac = d.mac_address.clone();
    let quick: HashSet<String> = data::quick_toggles(s)
        .into_iter()
        .map(|(id, _)| id)
        .collect();

    let mut groups: Vec<gpui::Div> = Vec::new();
    for category in &d.categories {
        if category.id == "deviceInformation" {
            continue;
        }
        let mut rows: Vec<gpui::Div> = Vec::new();
        for setting in &category.settings {
            if let Some(row) = setting_row(&view, &mac, &setting.id, s, &quick) {
                rows.push(row);
            }
        }
        if rows.is_empty() {
            continue;
        }
        groups.push(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .text_size(px(10.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::muted())
                        .child(data::pretty_setting_id(&category.id)),
                )
                .children(rows),
        );
    }
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
                .flex()
                .items_center()
                .gap_3()
                .border_b_1()
                .border_color(theme::border_medium())
                .child(back_button(&view))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::foreground())
                        .child("Device Settings"),
                ),
        )
        .child(
            div()
                .id("device-settings-body")
                .absolute()
                .top(px(59.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(0.))
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
                            .child("No additional settings"),
                    )
                }),
        )
}

fn back_button(view: &Entity<RootView>) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    div()
        .id("device-settings-back")
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
                root.show_device_settings = false;
                cx.notify();
            });
        })
        .child(chevron_left(14., theme::muted()))
}

fn setting_row(
    view: &Entity<RootView>,
    mac: &str,
    id: &str,
    s: &data::SettingsMap,
    quick: &HashSet<String>,
) -> Option<gpui::Div> {
    if EXCLUDED_IDS.contains(&id) || data::is_button_config(id) {
        return None;
    }
    let value = s.get(id)?;
    let label = data::pretty_setting_id(id);
    let ty = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match ty {
        "toggle" => {
            if quick.contains(id) {
                return None;
            }
            let on = value.get("value").and_then(Value::as_bool).unwrap_or(false);
            Some(toggle_row(view, mac, id, &label, on))
        }
        "i32Range" => Some(range_row(view, mac, id, &label, value)),
        "select" | "optionalSelect" | "modifiableSelect" => {
            select_row(view, mac, id, &label, s, ty == "optionalSelect")
        }
        "multiSelect" | "multiSelectWithRemove" => Some(multi_row(view, mac, id, &label, value, s)),
        "action" => Some(action_row(view, mac, id)),
        "timeOfDay" => Some(time_row(view, mac, id, &label, value)),
        "hueColorPicker" => Some(hue_row(view, mac, id, &label, value)),
        "information" => Some(info_row(&label, value)),
        _ => None,
    }
}

fn label_element(label: &str) -> gpui::Div {
    div()
        .text_size(px(11.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(theme::foreground())
        .child(label.to_string())
}

fn row_shell(label: &str, control: impl IntoElement) -> gpui::Div {
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
        .child(label_element(label))
        .child(control)
}

fn pills_row(label: &str, pills: Vec<gpui::Stateful<gpui::Div>>) -> gpui::Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(6.))
        .px_3()
        .py_2()
        .rounded_xl()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border_subtle())
        .child(label_element(label))
        .child(
            div()
                .w_full()
                .flex()
                .flex_wrap()
                .gap(px(6.))
                .children(pills),
        )
}

fn pill(
    view: &Entity<RootView>,
    mac: &str,
    id: &str,
    suffix: &str,
    label: String,
    raw: String,
    selected: bool,
) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    let mac = mac.to_string();
    let sid = id.to_string();
    div()
        .id(SharedString::from(format!("ds-{id}-{suffix}")))
        .px_3()
        .py_2()
        .rounded_full()
        .border_1()
        .flex_shrink_0()
        .text_size(px(11.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .cursor_pointer()
        .when(selected, |el| {
            el.bg(theme::brand())
                .border_color(theme::brand())
                .text_color(theme::brand_foreground())
        })
        .when(!selected, |el| {
            el.bg(theme::surface_elevated())
                .border_color(theme::border_subtle())
                .text_color(theme::foreground())
                .hover(|st| st.bg(theme::hover_subtle()))
        })
        .on_click(move |_, _, cx| {
            let raw = raw.clone();
            let mac = mac.clone();
            let sid = sid.clone();
            view.update(cx, move |root, cx| {
                root.send_setting(&mac, &sid, &raw);
                cx.notify();
            });
        })
        .child(label)
}

fn toggle_row(view: &Entity<RootView>, mac: &str, id: &str, label: &str, on: bool) -> gpui::Div {
    pills_row(
        label,
        vec![
            pill(view, mac, id, "on", "On".into(), "true".into(), on),
            pill(view, mac, id, "off", "Off".into(), "false".into(), !on),
        ],
    )
}

fn range_meta(value: &Value) -> (i32, i32, i32) {
    let setting = value.get("setting");
    let range = setting
        .and_then(|v| v.get("range"))
        .filter(|v| v.is_object())
        .or(setting)
        .filter(|v| v.is_object());
    let start = range
        .and_then(|v| v.get("start"))
        .and_then(Value::as_i64)
        .unwrap_or(0) as i32;
    let end = range
        .and_then(|v| v.get("end"))
        .and_then(Value::as_i64)
        .unwrap_or(i64::from(start)) as i32;
    let step = range
        .and_then(|v| v.get("step"))
        .and_then(Value::as_i64)
        .unwrap_or(1) as i32;
    if start <= end {
        (start, end, step)
    } else {
        (end, start, step)
    }
}

fn range_row(
    view: &Entity<RootView>,
    mac: &str,
    id: &str,
    label: &str,
    value: &Value,
) -> gpui::Div {
    let (start, end, step) = range_meta(value);
    let lower = i64::from(start);
    let upper = i64::from(end);
    let stride = i64::from(step).abs().max(1);
    let current = value
        .get("value")
        .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|n| n.round() as i64)))
        .unwrap_or(lower)
        .clamp(lower, upper);
    let dec = (current - stride).clamp(lower, upper);
    let inc = (current + stride).clamp(lower, upper);
    row_shell(
        label,
        stepper(
            view,
            mac,
            id,
            current.to_string(),
            dec.to_string(),
            inc.to_string(),
            dec < current,
            inc > current,
        ),
    )
}

#[allow(clippy::too_many_arguments)]
fn stepper(
    view: &Entity<RootView>,
    mac: &str,
    id: &str,
    display: String,
    dec: String,
    inc: String,
    can_dec: bool,
    can_inc: bool,
) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(step_button(view, mac, id, "dec", "−".into(), dec, can_dec))
        .child(
            div()
                .w(px(58.))
                .text_center()
                .text_size(px(12.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme::foreground())
                .child(display),
        )
        .child(step_button(view, mac, id, "inc", "+".into(), inc, can_inc))
}

fn step_button(
    view: &Entity<RootView>,
    mac: &str,
    id: &str,
    suffix: &str,
    glyph: String,
    raw: String,
    enabled: bool,
) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    let mac = mac.to_string();
    let sid = id.to_string();
    div()
        .id(SharedString::from(format!("ds-{id}-{suffix}")))
        .size(px(26.))
        .rounded_lg()
        .bg(theme::surface_elevated())
        .border_1()
        .border_color(theme::border_subtle())
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(13.))
        .text_color(theme::foreground())
        .when(enabled, |el| {
            el.cursor_pointer().hover(|st| st.bg(theme::hover_medium()))
        })
        .when(!enabled, |el| el.opacity(0.4))
        .on_click(move |_, _, cx| {
            if !enabled {
                return;
            }
            let raw = raw.clone();
            let mac = mac.clone();
            let sid = sid.clone();
            view.update(cx, move |root, cx| {
                root.send_setting(&mac, &sid, &raw);
                cx.notify();
            });
        })
        .child(glyph)
}

fn select_row(
    view: &Entity<RootView>,
    mac: &str,
    id: &str,
    label: &str,
    s: &data::SettingsMap,
    optional: bool,
) -> Option<gpui::Div> {
    let current = s
        .get(id)
        .and_then(|v| v.get("value"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let mut pills: Vec<gpui::Stateful<gpui::Div>> = data::select_entries(s, id)
        .into_iter()
        .map(|(opt_id, opt_label)| {
            let selected = opt_id == current;
            pill(view, mac, id, &opt_id, opt_label, opt_id.clone(), selected)
        })
        .collect();
    if optional {
        pills.push(pill(
            view,
            mac,
            id,
            "none",
            "None".into(),
            String::new(),
            current.is_empty(),
        ));
    }
    if pills.is_empty() {
        return None;
    }
    Some(pills_row(label, pills))
}

fn multi_row(
    view: &Entity<RootView>,
    mac: &str,
    id: &str,
    label: &str,
    value: &Value,
    s: &data::SettingsMap,
) -> gpui::Div {
    let current: Vec<String> = value
        .get("values")
        .or_else(|| value.get("value"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| match item {
                    Value::String(text) => Some(text.clone()),
                    Value::Null => None,
                    other => Some(other.to_string()),
                })
                .filter(|text| !text.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let pills: Vec<gpui::Stateful<gpui::Div>> = data::select_entries(s, id)
        .into_iter()
        .map(|(opt_id, opt_label)| {
            let selected = current.iter().any(|item| item == &opt_id);
            let mut next = current.clone();
            if selected {
                next.retain(|item| item != &opt_id);
            } else {
                next.push(opt_id.clone());
            }
            pill(view, mac, id, &opt_id, opt_label, next.join(","), selected)
        })
        .collect();
    pills_row(label, pills)
}

fn action_row(view: &Entity<RootView>, mac: &str, id: &str) -> gpui::Div {
    let view = view.clone();
    let mac = mac.to_string();
    let sid = id.to_string();
    let label = data::pretty_setting_id(id);
    div().child(
        div()
            .id(SharedString::from(format!("ds-{id}-action")))
            .w_full()
            .px_3()
            .py_2()
            .rounded_xl()
            .bg(theme::brand_10())
            .border_1()
            .border_color(theme::brand_40())
            .text_size(px(11.))
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(theme::brand())
            .cursor_pointer()
            .hover(|st| st.bg(theme::brand_20()))
            .on_click(move |_, _, cx| {
                let mac = mac.clone();
                let sid = sid.clone();
                view.update(cx, move |root, cx| {
                    root.send_setting(&mac, &sid, "true");
                    cx.notify();
                });
            })
            .child(label),
    )
}

fn time_row(view: &Entity<RootView>, mac: &str, id: &str, label: &str, value: &Value) -> gpui::Div {
    let minutes = value
        .get("minutesAfterMidnight")
        .or_else(|| value.get("value"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .clamp(0, 1439) as i32;
    let dec = (minutes - 15).clamp(0, 1439);
    let inc = (minutes + 15).clamp(0, 1439);
    let display = format!("{:02}:{:02}", minutes / 60, minutes % 60);
    row_shell(
        label,
        stepper(
            view,
            mac,
            id,
            display,
            dec.to_string(),
            inc.to_string(),
            dec < minutes,
            inc > minutes,
        ),
    )
}

fn hue_row(view: &Entity<RootView>, mac: &str, id: &str, label: &str, value: &Value) -> gpui::Div {
    let hue = value
        .get("hue")
        .or_else(|| value.get("value"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .round() as i32;
    let hue = hue.clamp(0, 360);
    let dec = (hue - 15).clamp(0, 360);
    let inc = (hue + 15).clamp(0, 360);
    row_shell(
        label,
        stepper(
            view,
            mac,
            id,
            format!("{hue}°"),
            dec.to_string(),
            inc.to_string(),
            dec < hue,
            inc > hue,
        ),
    )
}

fn info_row(label: &str, value: &Value) -> gpui::Div {
    let text = value
        .get("translatedValue")
        .or_else(|| value.get("value"))
        .map(|item| match item {
            Value::String(text) => text.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        })
        .unwrap_or_default();
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
        .child(label_element(label))
        .child(
            div()
                .text_size(px(11.))
                .text_color(theme::muted())
                .child(text),
        )
}
