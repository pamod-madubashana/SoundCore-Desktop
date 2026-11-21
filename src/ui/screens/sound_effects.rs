//! Sound Effects popup — full-screen overlay inside the control window.
//! Sound effects screen (radio-group sections).

use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, prelude::*};
use crate::ui::scale::px;

use super::common::{chevron_left, close_icon, radio_circle, sliders_icon};
use crate::ui::{RootView, data, theme};

pub fn render(
    root: &mut RootView,
    d: &crate::DeviceStateDto,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> gpui::Div {
    let view = cx.entity();
    let s = data::settings_map(d);
    let eq = root.eq_presets.clone();

    if root.show_equalizer {
        return equalizer_screen(&view, d, &s, cx);
    }

    let active = root.se_active();
    let show_spatial = data::has_spatial(&s);
    let show_default = data::has_default(&s);
    let show_custom = data::has_custom(&s);

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
                            root.show_equalizer = false;
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
                    root.show_equalizer = false;
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
                    let name =
                        data::match_custom_preset(&s, &eq).unwrap_or_else(|| "Custom".into());
                    el.child(custom_card(&view, &name, active == data::SECTION_CUSTOM, cx))
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
        "spatial-icon",
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
        "default-icon",
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
                            .grid()
                            .grid_cols(2)
                            .gap_2()
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

/* ── Custom EQ card (header only) ─────────────────────────────────── */

fn custom_card(
    view: &Entity<RootView>,
    name: &str,
    expanded: bool,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    section_shell(
        view,
        "custom",
        "Custom EQ",
        Some(name.to_string()),
        expanded,
        "custom-icon",
        cx,
        {
            let view = view.clone();
            move |root, cx| {
                root.select_section(data::SECTION_CUSTOM, cx);
                let _ = &view;
            }
        },
        None,
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
    icon_id: &'static str,
    cx: &mut Context<RootView>,
    on_toggle: impl Fn(&mut RootView, &mut Context<RootView>) + 'static,
    body: Option<gpui::Div>,
) -> impl IntoElement {
    let _ = cx;
    let view = view.clone();
    let title = title.to_string();
    let subtitle = subtitle;
    let icon = section_icon(icon_id);

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

fn section_icon(kind: &str) -> &'static str {
    match kind {
        "spatial-icon" => "♪",
        "default-icon" => "▦",
        _ => "≡",
    }
}

/* ── Custom EQ second-level screen ────────────────────────────────── */

fn equalizer_screen(
    view: &Entity<RootView>,
    d: &crate::DeviceStateDto,
    s: &data::SettingsMap,
    cx: &mut Context<RootView>,
) -> gpui::Div {
    let _ = d;
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
                .gap(px(10.))
                .border_b_1()
                .border_color(theme::border_medium())
                .child(header_btn(view, "eq-back", cx, |root, cx| {
                    root.show_equalizer = false;
                    cx.notify();
                }, chevron_left(14., theme::muted())))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::foreground())
                        .child("Custom EQ"),
                ),
        )
        // Body fills below header
        .child(
            div()
                .id("eq-body")
                .absolute()
                .top(px(49.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .overflow_y_scroll()
                .p_4()
                .child(equalizer(view, s)),
        )
}

/* ── Read-only equalizer bars (display) ───────────────────────────── */

fn equalizer(view: &Entity<RootView>, s: &data::SettingsMap) -> impl IntoElement {
    let _ = view;
    let Some(eq) = s.get("volumeAdjustments").cloned() else {
        return div();
    };
    let values: Vec<f64> = eq
        .get("value")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_f64()).collect())
        .unwrap_or_default();
    let band_meta = eq
        .get("setting")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let min = band_meta.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let max = band_meta.get("max").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let range = (max - min).abs().max(1e-6);

    div()
        .rounded_2xl()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border_subtle())
        .overflow_hidden()
        .child(
            div()
                .px_3()
                .py_3()
                .flex()
                .items_center()
                .gap_2()
                .border_b_1()
                .border_color(theme::border_subtle())
                .child(sliders_icon(16., theme::brand()))
                .child(
                    div()
                        .text_size(px(14.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::brand())
                        .child("Equalizer"),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(theme::muted())
                        .child("Read-only"),
                ),
        )
        .child(
            div()
                .p_3()
                .flex()
                .items_end()
                .justify_between()
                .gap_1()
                .children(values.iter().enumerate().map(|(i, v)| {
                    let frac = ((v - min) / range).clamp(0.0, 1.0) as f32;
                    let h = 100.0 * frac + 8.0;
                    div()
                        .id(("eq-band", i))
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_1()
                        .child(
                            div()
                                .w(px(14.))
                                .h(px(h))
                                .rounded_sm()
                                .bg(theme::brand())
                                .opacity(0.85),
                        )
                        .child(
                            div()
                                .text_size(px(9.))
                                .text_color(theme::muted())
                                .child(format!("{}", i + 1)),
                        )
                })),
        )
        .child(
            div()
                .px_3()
                .pb_3()
                .text_size(px(11.))
                .text_color(theme::muted())
                .text_center()
                .child("Tune the sound exactly the way you like it."),
        )
}
