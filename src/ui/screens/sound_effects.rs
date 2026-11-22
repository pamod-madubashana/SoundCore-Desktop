//! Sound Effects popup — full-screen overlay inside the control window.
//! Sound effects screen (radio-group sections).

use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, prelude::*};
use crate::ui::scale::px;

use super::common::{chevron_left, close_icon, eq_bars_icon, eq_image_icon, radio_circle};
use crate::ui::{RootView, data, theme};

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
        // Header pinned to top
        .child(
            div()
                .absolute()
                .top(px(0.))
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
                        .child(header_btn(&view, "se-back", cx, |root, cx| {
                            root.show_sound_effects = false;
                            cx.notify();
                        }, chevron_left(14., theme::muted())))
                        .child(
                            div()
                                .text_size(px(15.))
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme::foreground())
                                .child("Sound Effects"),
                        ),
                )
                .child(header_btn(&view, "se-x", cx, |root, cx| {
                    root.show_sound_effects = false;
                    cx.notify();
                }, close_icon(14., theme::muted()))),
        )
        // Body fills below header
        .child(
            div()
                .id("se-body")
                .absolute()
                .top(px(49.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .overflow_y_scroll()
                .p_3()
                .flex()
                .flex_col()
                .gap(px(10.))
                .when(show_spatial, |el| {
                    el.child(spatial_section(root, &view, d, &s, active == data::SECTION_SPATIAL, cx))
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
        .hover(|st| {
            st.text_color(theme::foreground())
                .bg(theme::hover_medium())
        })
        .cursor_pointer()
        .on_click(move |_, _window, cx| {
            view.update(cx, |root, cx| on(root, cx));
        })
        .child(icon)
}

/* ── Spatial Audio section ────────────────────────────────────────── */

fn spatial_section(
    root: &RootView,
    view: &Entity<RootView>,
    d: &crate::DeviceStateDto,
    s: &data::SettingsMap,
    expanded: bool,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let mac = d.mac_address.clone();
    let mode_setting = s.get("spatialAudioMode").cloned();
    let polled = mode_setting
        .as_ref()
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .map(|x| x.to_string());
    let current = root.spatial_mode(&mac, polled);

    let opts: Vec<String> = mode_setting
        .as_ref()
        .and_then(|v| v.get("setting"))
        .and_then(|v| v.get("options"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let localized: Vec<String> = mode_setting
        .as_ref()
        .and_then(|v| v.get("setting"))
        .and_then(|v| v.get("localizedOptions"))
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let ids: Vec<String> = if opts.is_empty() {
        vec!["Music".into(), "Movie".into(), "Gaming".into()]
    } else {
        opts
    };

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
        if expanded {
            Some(
                div()
                    .flex()
                    .gap(px(6.))
                    .px_3()
                    .pt_1()
                    .pb_3()
                    .children(ids.iter().enumerate().map(|(i, id)| {
                        let label = match id.as_str() {
                            "Music" => "Music Mode".to_string(),
                            "Movie" => "Movie Mode".to_string(),
                            "Gaming" => "Gaming Mode".to_string(),
                            _ => localized
                                .get(i)
                                .cloned()
                                .filter(|x| !x.is_empty())
                                .unwrap_or_else(|| id.clone()),
                        };
                        let on = *id == current;
                        let icon = match id.as_str() {
                            "Music" => "♪",
                            "Movie" => "▶",
                            "Gaming" => "◈",
                            _ => "♪",
                        };
                        let view = view.clone();
                        let id = id.clone();
                        div()
                            .id(gpui::SharedString::from(format!("spatial-mode-{id}")))
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
                            .when(on, |el| {
                                el.bg(theme::brand())
                                    .text_color(theme::brand_foreground())
                                    .border_2()
                                    .border_color(theme::brand_40())
                            })
                            .when(!on, |el| {
                                el.bg(theme::surface_elevated())
                                    .text_color(theme::foreground())
                                    .opacity(0.7)
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .hover(|st| st.opacity(1.0).bg(theme::hover_subtle()))
                            })
                            .cursor_pointer()
                            .on_click(move |_, _window, cx| {
                                let view = view.clone();
                                let id = id.clone();
                                view.update(cx, |root, cx| {
                                    root.set_spatial_mode(&id, cx);
                                });
                            })
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .text_color(if on {
                                        theme::brand_foreground()
                                    } else {
                                        theme::foreground()
                                    })
                                    .child(icon),
                            )
                            .child(label)
                    })),
            )
        } else {
            None
        },
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
        if expanded {
            Some(
                div()
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
                                let view = view.clone();
                                div()
                                    .id(gpui::SharedString::from(format!("preset-chip-{id}")))
                                    .w_full()
                                    .px_3()
                                    .py_2()
                                    .rounded_full()
                                    .text_size(px(12.))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_ellipsis()
                                    .whitespace_nowrap()
                                    .when(on, |el| {
                                        el.bg(theme::brand())
                                            .text_color(theme::brand_foreground())
                                    })
                                    .when(!on, |el| {
                                        el.bg(theme::surface_elevated())
                                            .text_color(theme::foreground())
                                            .opacity(0.8)
                                            .hover(|st| st.opacity(1.0).bg(theme::hover_subtle()))
                                    })
                                    .cursor_pointer()
                                    .on_click(move |_, _window, cx| {
                                        let view = view.clone();
                                        let id = id.clone();
                                        view.update(cx, |root, _cx| {
                                            if root
                                                .active_device()
                                                .map(|d| {
                                                    data::device_preset_id(
                                                        &data::settings_map(d),
                                                    )
                                                })
                                                .flatten()
                                                .as_deref()
                                                != Some(id.as_str())
                                            {
                                                let mac = root
                                                    .active_device()
                                                    .map(|d| d.mac_address.clone())
                                                    .unwrap_or_default();
                                                root.send_setting(
                                                    &mac,
                                                    "presetEqualizerProfile",
                                                    &id,
                                                );
                                            }
                                        });
                                    })
                                    .child(label)
                            })),
                    ),
            )
        } else {
            None
        },
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
    let body = if expanded {
        Some(eq_graph(root, view, d, s, cx))
    } else {
        None
    };
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
    body: Option<gpui::Div>,
) -> impl IntoElement {
    let _ = cx;
    let view = view.clone();
    let title = title.to_string();
    let subtitle = subtitle;

    div()
        .id(gpui::SharedString::from(format!("se-card-{key}")))
        .w_full()
        .flex()
        .flex_col()
        .rounded_2xl()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border_subtle())
        .overflow_hidden()
        .child(
            div()
                .id(gpui::SharedString::from(format!("se-head-{key}")))
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
                .child(radio_circle(expanded)),
        )
        .children(body)
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
    if n > 0.0 {
        format!("+{s}")
    } else {
        s
    }
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
    let min = meta.get("min").and_then(|v| v.as_i64()).unwrap_or(-60) as i16;
    let max = meta.get("max").and_then(|v| v.as_i64()).unwrap_or(60) as i16;
    let digits = meta
        .get("fractionDigits")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;
    let levels = [max, (max as f32 * 0.5) as i16, 0, (min as f32 * 0.5) as i16, min];
    let step_h = EQ_GRAPH_H / EQ_STEPS as f32;
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
                        .id("eq-reset")
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .text_size(px(11.))
                        .text_color(theme::brand())
                        .hover(|st| st.bg(theme::hover_subtle()))
                        .cursor_pointer()
                        .on_click({
                            let view = view.clone();
                            let mac = mac.clone();
                            let count = draft.len();
                            move |_, _, cx| {
                                view.update(cx, |root, cx| {
                                    root.eq_draft = vec![0; count];
                                    root.eq_active_preset = None;
                                    if !mac.is_empty() {
                                        let raw = vec!["0"; count].join(",");
                                        root.send_setting(&mac, "volumeAdjustments", &raw);
                                    }
                                    cx.notify();
                                });
                            }
                        })
                        .child("Reset"),
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
                        .text_color(theme::muted())
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
                                        .child(
                                            div()
                                                .w(px(1.))
                                                .h_full()
                                                .bg(theme::border_subtle()),
                                        ),
                                )
                                .children((0..EQ_STEPS).map(|k| {
                                    let value = (max as f32
                                        - k as f32 * (max - min) as f32
                                            / (EQ_STEPS - 1) as f32)
                                        .round()
                                        as i16;
                                    div()
                                        .id(gpui::SharedString::from(format!(
                                            "eq-step-{i}-{k}"
                                        )))
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
                                        .text_color(theme::muted())
                                        .text_center()
                                        .child(
                                            hz.get(i)
                                                .copied()
                                                .map(eq_hz)
                                                .unwrap_or_default(),
                                        ),
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
                    div()
                        .id(gpui::SharedString::from(format!(
                            "eq-preset-{}",
                            preset.id
                        )))
                        .h(px(38.))
                        .px_3()
                        .flex_shrink_0()
                        .rounded_xl()
                        .bg(if active { theme::brand() } else { theme::surface_elevated() })
                        .border_1()
                        .border_color(if active {
                            theme::brand()
                        } else {
                            theme::border_subtle()
                        })
                        .flex()
                        .items_center()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(if active {
                            theme::brand_foreground()
                        } else {
                            theme::foreground()
                        })
                        .hover(|st| st.bg(theme::hover_subtle()))
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
                        .child(preset.name.clone())
                })),
        )
}
