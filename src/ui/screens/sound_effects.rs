//! Sound Effects popup — full-screen overlay inside the control window.
//! Sound effects screen (radio-group sections).

use crate::ui::scale::px;
use gpui::{
    AnimationExt as _, AnimationPhase, Context, Entity, IntoElement, ParentElement, SharedString,
    SpringAnimation, SpringConfig, Styled, Window, div, prelude::*,
};

use super::common::{chevron_left, close_icon, eq_bars_icon, eq_image_icon};
use crate::ui::{RootView, data, theme};

const CARD_SPRING: SpringConfig = SpringConfig::new(150.0, 23.0, 1.0);
const SECTION_SPRING: SpringConfig = SpringConfig::new(170.0, 25.0, 1.0);
const BODY_SPRING: SpringConfig = SpringConfig::new(140.0, 22.0, 1.0);

type ExtraSpatialSetting = (String, String, Vec<(String, String)>, String);

fn phase(on: bool) -> AnimationPhase {
    AnimationPhase(if on { 1.0 } else { 0.0 })
}

pub fn render(
    root: &mut RootView,
    d: &crate::DeviceStateDto,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> gpui::Div {
    let view = cx.entity();
    let s = data::settings_map(d);

    let active = root.se_active();
    let show_spatial = data::has_spatial(&s);
    let show_default = data::has_default(&s);
    let show_custom = data::has_writable_custom(&s);

    div()
        .size_full()
        .relative()
        .overflow_hidden()
        // Header pinned to top with breathing room
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
                .justify_between()
                .border_b_1()
                .border_color(theme::border_medium())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(header_btn(
                            &view,
                            "se-back",
                            cx,
                            |root, cx| {
                                root.show_sound_effects = false;
                                cx.notify();
                            },
                            chevron_left(14., theme::muted()),
                        ))
                        .child(
                            div()
                                .text_size(px(15.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme::foreground())
                                .child("Sound Effects"),
                        ),
                )
                .child(header_btn(
                    &view,
                    "se-x",
                    cx,
                    |root, cx| {
                        root.show_sound_effects = false;
                        cx.notify();
                    },
                    close_icon(14., theme::muted()),
                )),
        )
        // Body fills below header
        .child(
            div()
                .id("se-body")
                .absolute()
                .top(px(59.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .overflow_y_scroll()
                .p_3()
                .flex()
                .flex_col()
                .gap(px(10.))
                .when(show_spatial, |el| {
                    el.child(spatial_section(
                        root,
                        &view,
                        d,
                        &s,
                        active == data::SECTION_SPATIAL,
                        cx,
                    ))
                })
                .when(show_default, |el| {
                    el.child(default_card(
                        &view,
                        d,
                        &s,
                        active == data::SECTION_DEFAULT,
                        cx,
                    ))
                })
                .when(show_custom, |el| {
                    el.child(custom_card(
                        root,
                        &view,
                        d,
                        &s,
                        active == data::SECTION_CUSTOM,
                        cx,
                    ))
                }),
        )
}

fn header_btn(
    view: &Entity<RootView>,
    id: &'static str,
    cx: &mut Context<RootView>,
    on: impl Fn(&mut RootView, &mut Context<RootView>) + 'static,
    icon: impl IntoElement + 'static,
) -> impl IntoElement {
    let _ = cx;
    let view = view.clone();
    div()
        .id(id)
        .p_1()
        .rounded_md()
        .text_color(theme::muted())
        .flex()
        .items_center()
        .justify_center()
        .hover(|st| st.text_color(theme::foreground()).bg(theme::hover_medium()))
        .cursor_pointer()
        .on_click(move |_, _window, cx| {
            view.update(cx, |root, cx| on(root, cx));
        })
        .child(icon)
}

fn animated_radio(active: bool, key: &str) -> impl IntoElement {
    let outer_key = SharedString::from(format!("se-radio-{key}"));
    let dot_key = SharedString::from(format!("se-radio-dot-{key}"));

    div()
        .size(px(16.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .border_2()
        .with_spring(
            outer_key,
            SpringAnimation::new(SECTION_SPRING).to(phase(active)),
            move |this, p| {
                let p = p.clamp(0.0..=1.0);
                let border = p.interpolate(theme::border_strong(), theme::brand());
                let bg = p.interpolate(gpui::transparent_black(), theme::brand());
                this.border_color(border).bg(bg)
            },
        )
        .child(div().rounded_full().bg(gpui::rgb(0xffffff)).with_spring(
            dot_key,
            SpringAnimation::new(SECTION_SPRING).to(phase(active)),
            move |this, p| {
                let p = p.clamp(0.0..=1.0);
                let size = p.0 * 8.0;
                this.size(px(size)).opacity(p.0)
            },
        ))
}

/* ── Spatial Audio section ────────────────────────────────────────── */

fn spatial_mode_icon(id: &str) -> &'static str {
    let lower = id.to_ascii_lowercase();
    if lower.contains("music") {
        "♪"
    } else if lower.contains("movie") || lower.contains("cinema") {
        "▶"
    } else if lower.contains("game") {
        "◈"
    } else {
        "◉"
    }
}

fn is_spatial_select(value: &serde_json::Value) -> bool {
    matches!(
        value.get("type").and_then(|v| v.as_str()),
        Some("select" | "optionalSelect" | "modifiableSelect")
    )
}

fn spatial_section(
    root: &RootView,
    view: &Entity<RootView>,
    d: &crate::DeviceStateDto,
    s: &data::SettingsMap,
    expanded: bool,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let mac = d.mac_address.clone();
    let polled = s
        .get("spatialAudioMode")
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .map(|x| x.to_string());
    let current = root.spatial_mode(&mac, polled);

    let entries = data::select_entries(s, "spatialAudioMode");
    let allowed: Vec<String> = entries.iter().map(|(id, _)| id.clone()).collect();

    let main_row = div()
        .w_full()
        .flex()
        .gap(px(6.))
        .children(entries.into_iter().map(|(id, label)| {
            let on = id == current;
            let icon = spatial_mode_icon(&id);
            let spring_key = SharedString::from(format!("spatial-mode-spring-{id}"));
            let view = view.clone();
            let allowed = allowed.clone();
            div()
                .id(SharedString::from(format!("spatial-mode-{id}")))
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .py_3()
                .rounded_xl()
                .text_size(px(11.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .border_2()
                .cursor_pointer()
                .on_click(move |_, _window, cx| {
                    if !allowed.contains(&id) {
                        return;
                    }
                    let view = view.clone();
                    let id = id.clone();
                    view.update(cx, |root, cx| {
                        root.set_spatial_mode(&id, cx);
                    });
                })
                .with_spring(
                    spring_key,
                    SpringAnimation::new(CARD_SPRING).to(phase(on)),
                    move |this, p| {
                        let p = p.clamp(0.0..=1.0);
                        let bg = p.interpolate(theme::surface_elevated(), theme::brand());
                        let border = p.interpolate(theme::border_subtle(), theme::brand_40());
                        let text = p.interpolate(theme::foreground(), theme::brand_foreground());
                        let opacity = p.interpolate(0.7, 1.0);
                        let el = this
                            .bg(bg)
                            .border_color(border)
                            .text_color(text)
                            .opacity(opacity);
                        if on {
                            el
                        } else {
                            el.hover(|st| st.opacity(1.0).bg(theme::hover_subtle()))
                        }
                    },
                )
                .child(div().text_size(px(14.)).child(icon))
                .child(label)
        }));

    let mut extra: Vec<ExtraSpatialSetting> = Vec::new();
    for (id, value) in s.iter() {
        if !id.starts_with("spatial") || id == "spatialAudio" || id == "spatialAudioMode" {
            continue;
        }
        if !is_spatial_select(value) {
            continue;
        }
        let options = data::select_entries(s, id);
        if options.is_empty() {
            continue;
        }
        let selected = value
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let mut label = data::pretty_setting_id(id);
        if let Some(short) = label.strip_prefix("Spatial ") {
            label = short.to_string();
        }
        extra.push((id.clone(), selected, options, label));
    }
    extra.sort_by(|a, b| a.0.cmp(&b.0));
    let has_extra = !extra.is_empty();

    let view_for_extra = view.clone();
    let mac_for_extra = mac.clone();
    let extra_rows = extra
        .into_iter()
        .map(move |(setting_id, selected, options, label)| {
            let view = view_for_extra.clone();
            let mac = mac_for_extra.clone();
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_size(px(10.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::muted())
                        .child(label),
                )
                .child(div().w_full().flex().flex_wrap().gap(px(6.)).children(
                    options.into_iter().map(move |(opt_id, opt_label)| {
                        let on = opt_id == selected;
                        let sid = setting_id.clone();
                        let oid = opt_id.clone();
                        let mac = mac.clone();
                        let view = view.clone();
                        div()
                            .id(SharedString::from(format!("spatial-extra-{sid}-{oid}")))
                            .px_3()
                            .py_2()
                            .rounded_full()
                            .border_1()
                            .flex_shrink_0()
                            .text_size(px(11.))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .cursor_pointer()
                            .when(on, |el| {
                                el.bg(theme::brand())
                                    .border_color(theme::brand())
                                    .text_color(theme::brand_foreground())
                            })
                            .when(!on, |el| {
                                el.bg(theme::surface_elevated())
                                    .border_color(theme::border_subtle())
                                    .text_color(theme::foreground())
                                    .hover(|st| st.bg(theme::hover_subtle()))
                            })
                            .on_click(move |_, _, cx| {
                                let sid = sid.clone();
                                let oid = oid.clone();
                                let mac = mac.clone();
                                view.update(cx, move |root, cx| {
                                    root.set_named_select(&mac, &sid, &oid, cx);
                                });
                            })
                            .child(opt_label)
                    }),
                ))
        });

    let body = div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .px_3()
        .pt_1()
        .pb_3()
        .when(!allowed.is_empty(), |el| el.child(main_row))
        .children(extra_rows);

    section_shell(
        view,
        "spatial",
        "Spatial Audio",
        None,
        expanded,
        eq_image_icon("spatial-icon", 16.),
        cx,
        {
            let view = view.clone();
            move |root, cx| {
                root.select_section(data::SECTION_SPATIAL, cx);
                let _ = &view;
            }
        },
        body,
        if has_extra { 132.0 } else { 80.0 },
    )
}

/* ── Default preset card ──────────────────────────────────────────── */

fn default_card(
    view: &Entity<RootView>,
    _d: &crate::DeviceStateDto,
    s: &data::SettingsMap,
    expanded: bool,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let options = data::preset_options(s);
    let current = data::device_preset_id(s);
    let current_label = current
        .as_deref()
        .and_then(|id| {
            options
                .iter()
                .find(|(oid, _)| oid == id)
                .map(|(_, l)| l.clone())
        })
        .or_else(|| current.clone());

    let body = div()
        .px_3()
        .pt_1()
        .pb_3()
        .flex()
        .flex_col()
        .gap(px(10.))
        .child(
            div()
                .id("preset-grid")
                .grid()
                .grid_cols(2)
                .gap_2()
                .max_h(px(120.))
                .overflow_y_scroll()
                .children(options.into_iter().map(|(id, label)| {
                    let on = current.as_deref() == Some(id.as_str());
                    let spring_key = SharedString::from(format!("preset-chip-spring-{id}"));
                    let id_key = SharedString::from(format!("preset-chip-{id}"));
                    let view = view.clone();
                    let id = id.clone();
                    div()
                        .id(id_key)
                        .w_full()
                        .px_3()
                        .py_2()
                        .rounded_full()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .cursor_pointer()
                        .on_click(move |_, _window, cx| {
                            let view = view.clone();
                            let id = id.clone();
                            view.update(cx, |root, _cx| {
                                if root
                                    .active_device()
                                    .map(|d| data::device_preset_id(&data::settings_map(d)))
                                    .flatten()
                                    .as_deref()
                                    != Some(id.as_str())
                                {
                                    let mac = root
                                        .active_device()
                                        .map(|d| d.mac_address.clone())
                                        .unwrap_or_default();
                                    root.send_setting(&mac, "presetEqualizerProfile", &id);
                                }
                            });
                        })
                        .with_spring(
                            spring_key,
                            SpringAnimation::new(CARD_SPRING).to(phase(on)),
                            move |this, p| {
                                let p = p.clamp(0.0..=1.0);
                                let bg = p.interpolate(theme::surface_elevated(), theme::brand());
                                let text =
                                    p.interpolate(theme::foreground(), theme::brand_foreground());
                                let opacity = p.interpolate(0.8, 1.0);
                                let el = this.bg(bg).text_color(text).opacity(opacity);
                                if on {
                                    el
                                } else {
                                    el.hover(|st| st.opacity(1.0).bg(theme::hover_subtle()))
                                }
                            },
                        )
                        .child(label)
                })),
        );

    section_shell(
        view,
        "default",
        "Default",
        current_label,
        expanded,
        eq_image_icon("default-icon", 16.),
        cx,
        {
            let view = view.clone();
            move |root, cx| {
                root.select_section(data::SECTION_DEFAULT, cx);
                let _ = &view;
            }
        },
        body,
        140.0,
    )
}

/* ── Custom EQ card (graph inline when selected) ──────────────────── */

fn custom_card(
    root: &RootView,
    view: &Entity<RootView>,
    d: &crate::DeviceStateDto,
    s: &data::SettingsMap,
    expanded: bool,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let name = data::match_custom_preset(s, &root.eq_presets).unwrap_or_else(|| "Custom".into());
    let body = eq_graph(root, view, d, s, cx);
    section_shell(
        view,
        "custom",
        "Custom EQ",
        Some(name),
        expanded,
        eq_bars_icon(16., theme::brand()),
        cx,
        {
            let view = view.clone();
            move |root, cx| {
                root.select_section(data::SECTION_CUSTOM, cx);
                let _ = &view;
            }
        },
        body,
        360.0,
    )
}

/* ── Shared section chrome ────────────────────────────────────────── */

#[allow(clippy::too_many_arguments)]
fn section_shell(
    view: &Entity<RootView>,
    key: &'static str,
    title: &str,
    subtitle: Option<String>,
    expanded: bool,
    icon: impl IntoElement,
    cx: &mut Context<RootView>,
    on_toggle: impl Fn(&mut RootView, &mut Context<RootView>) + 'static,
    body: gpui::Div,
    body_height: f32,
) -> impl IntoElement {
    let _ = cx;
    let view = view.clone();
    let title = title.to_string();
    let subtitle = subtitle;
    let card_key = SharedString::from(format!("se-card-spring-{key}"));
    let body_key = SharedString::from(format!("se-body-spring-{key}"));

    div()
        .id(SharedString::from(format!("se-card-{key}")))
        .w_full()
        .flex()
        .flex_col()
        .rounded_2xl()
        .overflow_hidden()
        .with_spring(
            card_key,
            SpringAnimation::new(CARD_SPRING).to(phase(expanded)),
            move |this, p| {
                let p = p.clamp(0.0..=1.0);
                let bg = p.interpolate(theme::surface(), theme::brand_10());
                let border = p.interpolate(theme::border_subtle(), theme::brand_40());
                this.bg(bg).border_1().border_color(border)
            },
        )
        .child(
            div()
                .id(SharedString::from(format!("se-head-{key}")))
                .w_full()
                .h(px(48.))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .hover(|st| st.bg(theme::hover_subtle()))
                .cursor_pointer()
                .on_click(move |_, _window, cx| {
                    view.update(cx, |root, cx| on_toggle(root, cx));
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .size(px(32.))
                                .rounded_xl()
                                .bg(theme::brand_10())
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(theme::brand())
                                .child(icon),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_baseline()
                                .child(
                                    div()
                                        .text_size(px(14.))
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(theme::foreground())
                                        .child(title),
                                )
                                .when_some(subtitle, |el, sub| {
                                    el.child(
                                        div()
                                            .ml_2()
                                            .text_size(px(12.))
                                            .text_color(theme::muted())
                                            .child(sub),
                                    )
                                }),
                        ),
                )
                .child(animated_radio(expanded, key)),
        )
        .child(
            div()
                .w_full()
                .overflow_hidden()
                .with_spring(
                    body_key,
                    SpringAnimation::new(BODY_SPRING).to(phase(expanded)),
                    move |this, p| {
                        let p = p.clamp(0.0..=1.0);
                        this.h(px(body_height * p.0)).opacity(p.0)
                    },
                )
                .child(body),
        )
}

/* -- Custom EQ graph (inline) ---------------------------------------- */
const EQ_GRAPH_H: f32 = 240.0;
const EQ_STEPS: usize = 25;

fn eq_db(v: i16, digits: i32) -> String {
    let scale = 10f32.powi(digits);
    let n = v as f32 / scale;
    let s = if digits > 0 {
        format!("{n:.*}", digits as usize)
    } else {
        format!("{n:.0}")
    };
    if n > 0.0 { format!("+{s}") } else { s }
}

fn eq_hz(hz: u16) -> String {
    if hz >= 1000 {
        let k = hz as f32 / 1000.0;
        if k.fract() == 0.0 {
            format!("{k:.0}k")
        } else {
            format!("{k:.1}k")
        }
    } else {
        hz.to_string()
    }
}

fn eq_value_y(v: i16, min: i16, max: i16) -> f32 {
    let range = (max - min).max(1) as f32;
    ((max as f32 - v as f32) / range * EQ_GRAPH_H).clamp(0.0, EQ_GRAPH_H)
}

fn parse_bands(raw: &str) -> Vec<i16> {
    raw.split(',')
        .filter_map(|value| value.trim().parse().ok())
        .collect()
}

fn eq_export_json(presets: &[crate::EqPresetDto], digits: i32) -> String {
    let scale = 10f64.powi(digits);
    let items: Vec<serde_json::Value> = presets
        .iter()
        .map(|preset| {
            let adjustments: Vec<f64> = preset
                .bands
                .split(',')
                .filter_map(|value| value.trim().parse::<i16>().ok())
                .map(|value| value as f64 / scale)
                .collect();

            serde_json::json!({
                "name": preset.name,
                "volumeAdjustments": adjustments,
            })
        })
        .collect();
    serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
}

fn eq_graph(
    root: &RootView,
    view: &Entity<RootView>,
    _d: &crate::DeviceStateDto,
    s: &data::SettingsMap,
    _cx: &mut Context<RootView>,
) -> gpui::Div {
    let eq = s
        .get("volumeAdjustments")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let meta = eq
        .get("setting")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let bands: Vec<String> = eq
        .get("value")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().map(|x| x.to_string()).collect())
        .unwrap_or_default();
    let values: Vec<i16> = bands.iter().filter_map(|x| x.parse().ok()).collect();
    let draft = if root.eq_draft.len() == values.len() && !values.is_empty() {
        root.eq_draft.clone()
    } else {
        values.clone()
    };
    let hz: Vec<u16> = meta
        .get("bandHz")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_u64().map(|n| n as u16))
                .collect()
        })
        .unwrap_or_default();
    let (min, max) = if meta.is_null() {
        (0i16, 0i16)
    } else {
        let min = meta.get("min").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
        let max = meta.get("max").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
        if min <= max { (min, max) } else { (max, min) }
    };
    let digits = if meta.is_null() {
        0i32
    } else {
        meta.get("fractionDigits")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32
    };
    let span = (max - min) as f32;
    let levels = [
        max,
        (max as f32 - span * 0.25).round() as i16,
        (max as f32 - span * 0.50).round() as i16,
        (max as f32 - span * 0.75).round() as i16,
        min,
    ];
    let step_count = EQ_STEPS.max(2);
    let step_h = EQ_GRAPH_H / step_count as f32;
    let reset_value = 0i16.clamp(min, max);
    let mac = root
        .active_device()
        .map(|dev| dev.mac_address.clone())
        .unwrap_or_default();
    let model = root
        .active_device()
        .map(|dev| dev.model.clone())
        .unwrap_or_default();
    let presets = root.eq_presets.clone();
    let active_preset_id = root.eq_active_preset;
    let drag_band = root.eq_drag_band;

    div()
        .px_3()
        .pt_2()
        .pb_3()
        .flex()
        .flex_col()
        .child(
            div()
                .mb_2()
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_size(px(10.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::muted())
                        .child("EQ CURVE"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .id("eq-import")
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .text_size(px(11.))
                                .text_color(theme::foreground())
                                .hover(|st| st.bg(theme::hover_subtle()))
                                .cursor_pointer()
                                .on_click({
                                    let view = view.clone();
                                    let mac = mac.clone();
                                    let model = model.clone();
                                    move |_, _, cx| {
                                        let Some(text) =
                                            cx.read_from_clipboard().and_then(|item| item.text())
                                        else {
                                            return;
                                        };
                                        if text.trim().is_empty() {
                                            return;
                                        }
                                        view.update(cx, |root, cx| {
                                            root.send_setting(
                                                &mac,
                                                "importCustomEqualizerProfiles",
                                                &text,
                                            );
                                            if let Ok(entries) =
                                                serde_json::from_str::<Vec<serde_json::Value>>(
                                                    &text,
                                                )
                                            {
                                                let scale = 10f64.powi(digits);
                                                for entry in entries {
                                                    let name = entry
                                                        .get("name")
                                                        .and_then(|value| value.as_str())
                                                        .unwrap_or("Imported")
                                                        .to_string();
                                                    let adjustments: Vec<i16> = entry
                                                        .get("volumeAdjustments")
                                                        .and_then(|value| value.as_array())
                                                        .map(|values| {
                                                            values
                                                                .iter()
                                                                .filter_map(|value| value.as_f64())
                                                                .map(|value| {
                                                                    (value * scale).round() as i16
                                                                })
                                                                .collect()
                                                        })
                                                        .unwrap_or_default();
                                                    if adjustments.is_empty() {
                                                        continue;
                                                    }
                                                    let bands = adjustments
                                                        .iter()
                                                        .map(|value| value.to_string())
                                                        .collect::<Vec<_>>()
                                                        .join(",");
                                                    if let Ok(preset) = crate::save_eq_preset(
                                                        crate::state(),
                                                        name,
                                                        bands,
                                                        model.clone(),
                                                    ) {
                                                        root.eq_presets.push(preset);
                                                    }
                                                }
                                            }
                                            cx.notify();
                                        });
                                    }
                                })
                                .child("Import"),
                        )
                        .child(
                            div()
                                .id("eq-export")
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .text_size(px(11.))
                                .text_color(theme::foreground())
                                .hover(|st| st.bg(theme::hover_subtle()))
                                .cursor_pointer()
                                .on_click({
                                    let view = view.clone();
                                    move |_, _, cx| {
                                        view.update(cx, |root, cx| {
                                            let json = eq_export_json(&root.eq_presets, digits);
                                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                                json,
                                            ));
                                        });
                                    }
                                })
                                .child("Export"),
                        )
                        .child(
                            div()
                                .id("eq-reset")
                                .px_2()
                                .py_1()
                                .rounded_md()
                                .text_size(px(11.))
                                .text_color(theme::error())
                                .hover(|st| st.bg(theme::hover_subtle()))
                                .cursor_pointer()
                                .on_click({
                                    let view = view.clone();
                                    let mac = mac.clone();
                                    let count = draft.len();
                                    let reset = reset_value.to_string();
                                    move |_, _, cx| {
                                        view.update(cx, |root, cx| {
                                            root.eq_draft = vec![reset_value; count];
                                            root.eq_active_preset = None;
                                            if !mac.is_empty() {
                                                let raw = vec![reset.as_str(); count].join(",");
                                                root.send_setting(&mac, "volumeAdjustments", &raw);
                                            }
                                            cx.notify();
                                        });
                                    }
                                })
                                .child("Reset"),
                        ),
                ),
        )
        .child(
            div()
                .w_full()
                .flex()
                .child(
                    div()
                        .w(px(30.))
                        .h(px(EQ_GRAPH_H))
                        .flex()
                        .flex_col()
                        .justify_between()
                        .pr_1()
                        .text_size(px(9.))
                        .text_color(theme::foreground())
                        .children(levels.iter().map(|v| eq_db(*v, digits))),
                )
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .h(px(EQ_GRAPH_H + 16.0))
                        .children((0..5).map(|i| {
                            div()
                                .absolute()
                                .left(px(0.))
                                .right(px(0.))
                                .top(px(EQ_GRAPH_H * i as f32 / 4.0))
                                .h(px(1.))
                                .bg(theme::border_subtle())
                        }))
                        .flex()
                        .children(draft.iter().enumerate().map(|(i, v)| {
                            let y = eq_value_y(*v, min, max);
                            let active = drag_band == Some(i);
                            let size = if active { 18.0 } else { 14.0 };

                            div()
                                .relative()
                                .flex_1()
                                .h(px(EQ_GRAPH_H))
                                .on_mouse_up(gpui::MouseButton::Left, {
                                    let view = view.clone();
                                    let mac = mac.clone();
                                    move |_, _, cx| {
                                        view.update(cx, |root, cx| {
                                            if root.eq_drag_band.is_none() {
                                                return;
                                            }
                                            root.eq_drag_band = None;
                                            if !mac.is_empty() {
                                                let raw = root
                                                    .eq_draft
                                                    .iter()
                                                    .map(|value| value.to_string())
                                                    .collect::<Vec<_>>()
                                                    .join(",");
                                                root.send_setting(&mac, "volumeAdjustments", &raw);
                                            }
                                            cx.notify();
                                        });
                                    }
                                })
                                .child(
                                    div()
                                        .absolute()
                                        .left(px(0.))
                                        .right(px(0.))
                                        .top(px(0.))
                                        .bottom(px(0.))
                                        .flex()
                                        .justify_center()
                                        .child(div().w(px(1.)).h_full().bg(theme::border_subtle())),
                                )
                                .children((0..step_count).map(|k| {
                                    let t = k as f32 / (step_count - 1) as f32;
                                    let value = (max as f32 - t * span)
                                        .round()
                                        .clamp(min as f32, max as f32)
                                        as i16;
                                    div()
                                        .id(gpui::SharedString::from(format!("eq-step-{i}-{k}")))
                                        .absolute()
                                        .left(px(0.))
                                        .right(px(0.))
                                        .top(px(k as f32 * step_h))
                                        .h(px(step_h))
                                        .on_mouse_down(gpui::MouseButton::Left, {
                                            let view = view.clone();
                                            move |_, _, cx| {
                                                view.update(cx, |root, cx| {
                                                    if i < root.eq_draft.len() {
                                                        root.eq_draft[i] = value;
                                                    }
                                                    root.eq_drag_band = Some(i);
                                                    root.eq_active_preset = None;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_mouse_move({
                                            let view = view.clone();
                                            move |_, _, cx| {
                                                if view.read(cx).eq_drag_band != Some(i) {
                                                    return;
                                                }
                                                view.update(cx, |root, cx| {
                                                    if i < root.eq_draft.len()
                                                        && root.eq_draft[i] != value
                                                    {
                                                        root.eq_draft[i] = value;
                                                        root.eq_active_preset = None;
                                                        cx.notify();
                                                    }
                                                });
                                            }
                                        })
                                }))
                                .child(
                                    div()
                                        .absolute()
                                        .left(px(0.))
                                        .right(px(0.))
                                        .top(px(y - size * 0.5))
                                        .h(px(size))
                                        .flex()
                                        .justify_center()
                                        .child(
                                            div()
                                                .size(px(size))
                                                .rounded_full()
                                                .bg(if active {
                                                    theme::brand()
                                                } else {
                                                    theme::surface_elevated()
                                                })
                                                .border_2()
                                                .border_color(if active {
                                                    theme::foreground()
                                                } else {
                                                    theme::brand()
                                                }),
                                        ),
                                )
                                .child(
                                    div()
                                        .absolute()
                                        .left(px(0.))
                                        .right(px(0.))
                                        .top(px(EQ_GRAPH_H + 2.))
                                        .text_size(px(9.))
                                        .text_color(theme::foreground())
                                        .text_center()
                                        .child(hz.get(i).copied().map(eq_hz).unwrap_or_default()),
                                )
                        })),
                ),
        )
        .child(
            div()
                .id("eq-presets")
                .mt(px(18.))
                .w_full()
                .h(px(38.))
                .flex()
                .items_center()
                .gap_2()
                .overflow_x_scroll()
                .child(
                    div()
                        .id("eq-new-preset")
                        .size(px(38.))
                        .flex_shrink_0()
                        .rounded_xl()
                        .bg(theme::surface_elevated())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(18.))
                        .text_color(theme::brand())
                        .hover(|st| st.bg(theme::hover_subtle()))
                        .cursor_pointer()
                        .on_click({
                            let view = view.clone();
                            let bands = draft
                                .iter()
                                .map(|value| value.to_string())
                                .collect::<Vec<_>>()
                                .join(",");
                            let model = model.clone();
                            let name = format!("Custom {}", presets.len() + 1);
                            move |_, _, cx| {
                                view.update(cx, |root, cx| {
                                    match crate::save_eq_preset(
                                        crate::state(),
                                        name.clone(),
                                        bands.clone(),
                                        model.clone(),
                                    ) {
                                        Ok(preset) => {
                                            root.eq_active_preset = Some(preset.id);
                                            root.eq_presets.push(preset);
                                        }
                                        Err(err) => tracing::warn!("save eq preset failed: {err}"),
                                    }
                                    cx.notify();
                                });
                            }
                        })
                        .child("+"),
                )
                .children(presets.iter().map(|preset| {
                    let view = view.clone();
                    let mac = mac.clone();
                    let bands = preset.bands.clone();
                    let preset_id = preset.id;
                    let active = active_preset_id == Some(preset.id);
                    let spring_key = SharedString::from(format!("eq-preset-spring-{}", preset.id));
                    div()
                        .id(SharedString::from(format!("eq-preset-{}", preset.id)))
                        .h(px(38.))
                        .px_3()
                        .flex_shrink_0()
                        .rounded_xl()
                        .border_1()
                        .flex()
                        .items_center()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .cursor_pointer()
                        .on_click(move |_, _, cx| {
                            view.update(cx, |root, cx| {
                                let next = parse_bands(&bands);
                                if !next.is_empty() {
                                    root.eq_draft = next;
                                }
                                root.eq_active_preset = Some(preset_id);
                                if !mac.is_empty() {
                                    root.send_setting(&mac, "volumeAdjustments", &bands);
                                }
                                cx.notify();
                            });
                        })
                        .with_spring(
                            spring_key,
                            SpringAnimation::new(CARD_SPRING).to(phase(active)),
                            move |this, p| {
                                let p = p.clamp(0.0..=1.0);
                                let bg = p.interpolate(theme::surface_elevated(), theme::brand());
                                let border = p.interpolate(theme::border_subtle(), theme::brand());
                                let text =
                                    p.interpolate(theme::foreground(), theme::brand_foreground());
                                let el = this.bg(bg).border_color(border).text_color(text);
                                if active {
                                    el
                                } else {
                                    el.hover(|st| st.bg(theme::hover_subtle()))
                                }
                            },
                        )
                        .child(preset.name.clone())
                })),
        )
}
