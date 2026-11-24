//! Control screen (connected device + header).

use crate::ui::scale::px;
use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, prelude::*};

use super::common::{
    chevron_right, controls_icon, download_icon, gear_icon, info_icon, sliders_icon,
};
use crate::ui::animation::{morphing_pill, segment_text, toggle_switch};
use crate::ui::{RootView, data, theme};

pub fn render(
    root: &mut RootView,
    d: &crate::DeviceStateDto,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let view = cx.entity();
    let s = root.cached_settings(&d.mac_address);
    let d = d;
    let eq = root.eq_presets.clone();
    let current_effect = data::sound_effect_label(&s, &eq);
    let name = data::display_device_name(d);
    let batteries = data::header_batteries(d);
    let update_info = root.update_info.clone();
    let update_progress = root.update_progress.clone();
    let update_error = root.update_error.clone();
    let version = root.version.clone();

    div()
        .size_full()
        .relative()
        .flex()
        .flex_col()
        .child(header(
            &view,
            d,
            &name,
            &batteries,
            update_info.as_ref(),
            update_progress.as_ref(),
            update_error.as_deref(),
            root,
        ))
        .child(
            div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .px_3()
                .py_3()
                .gap_3()
                .overflow_hidden()
                .when(s.contains_key("ambientSoundMode"), |el| {
                    el.child(sound_mode(&view, d, &s, root))
                })
                .when(
                    s.contains_key("volumeAdjustments") || s.contains_key("spatialAudio"),
                    |el| {
                        let model = d.model.clone();
                        el.child(
                            div()
                                .id("sound-effects-row")
                                .w_full()
                                .rounded_2xl()
                                .bg(theme::surface())
                                .border_1()
                                .border_color(theme::border_subtle())
                                .px_3()
                                .py(px(10.))
                                .flex()
                                .items_center()
                                .justify_between()
                                .hover(|st| st.bg(theme::hover_subtle()))
                                .cursor_pointer()
                                .on_click({
                                    let view = view.clone();
                                    move |_, _window, cx| {
                                        let model = model.clone();
                                        view.update(cx, |root, cx| {
                                            root.eq_presets =
                                                crate::list_eq_presets(crate::state(), &model);
                                            root.show_sound_effects = true;
                                            cx.notify();
                                        });
                                    }
                                })
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(10.))
                                        .child(
                                            div()
                                                .size(px(32.))
                                                .rounded_xl()
                                                .bg(theme::brand_10())
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(sliders_icon(16., theme::brand())),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_baseline()
                                                .gap_2()
                                                .child(
                                                    div()
                                                        .text_size(px(14.))
                                                        .font_weight(gpui::FontWeight::BOLD)
                                                        .text_color(theme::foreground())
                                                        .child("Sound Effects"),
                                                )
                                                .when(!current_effect.is_empty(), |el| {
                                                    el.child(
                                                        div()
                                                            .text_size(px(12.))
                                                            .text_color(theme::muted())
                                                            .child(current_effect.clone()),
                                                    )
                                                }),
                                        ),
                                )
                                .child(chevron_right(16., theme::muted())),
                        )
                    },
                )
                .child(
                    div()
                        .id("device-settings-row")
                        .w_full()
                        .rounded_2xl()
                        .bg(theme::surface())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .px_3()
                        .py(px(10.))
                        .flex()
                        .items_center()
                        .justify_between()
                        .hover(|st| st.bg(theme::hover_subtle()))
                        .cursor_pointer()
                        .on_click({
                            let view = view.clone();
                            move |_, _window, cx| {
                                view.update(cx, |root, cx| {
                                    root.show_device_settings = true;
                                    root.show_sound_effects = false;
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.))
                                .child(
                                    div()
                                        .size(px(32.))
                                        .rounded_xl()
                                        .bg(theme::brand_10())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(gear_icon(16., theme::brand())),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_size(px(14.))
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(theme::foreground())
                                                .child("Device Settings"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(12.))
                                                .text_color(theme::muted())
                                                .child("All settings reported by this device"),
                                        ),
                                ),
                        )
                        .child(chevron_right(16., theme::muted())),
                )
                .child(
                    div()
                        .id("control-settings-row")
                        .w_full()
                        .rounded_2xl()
                        .bg(theme::surface())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .px_3()
                        .py(px(10.))
                        .flex()
                        .items_center()
                        .justify_between()
                        .hover(|st| st.bg(theme::hover_subtle()))
                        .cursor_pointer()
                        .on_click({
                            let view = view.clone();
                            move |_, _, cx| {
                                view.update(cx, |root, cx| {
                                    root.show_control_settings = true;
                                    root.show_device_settings = false;
                                    root.show_sound_effects = false;
                                    cx.notify();
                                });
                            }
                        })
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.))
                                .child(
                                    div()
                                        .size(px(32.))
                                        .rounded_xl()
                                        .bg(theme::brand_10())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(controls_icon(16., theme::brand())),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_size(px(14.))
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(theme::foreground())
                                                .child("Control Settings"),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(12.))
                                                .text_color(theme::muted())
                                                .child("Button actions and gestures"),
                                        ),
                                ),
                        )
                        .child(chevron_right(16., theme::muted())),
                )
                .child(
                    div()
                        .id("quick-toggles")
                        .flex_1()
                        .min_h_0()
                        .overflow_hidden()
                        .child(quick_toggles(&view, &s, d, root)),
                ),
        )
        .child(
            div()
                .pl(px(30.))
                .pr_4()
                .py_2()
                .text_size(px(10.))
                .text_color(theme::muted_dim())
                .child(format!("v{version}")),
        )
}

#[allow(clippy::too_many_arguments)]
fn header(
    view: &Entity<RootView>,
    d: &crate::DeviceStateDto,
    name: &str,
    batteries: &[(String, i32)],
    update_info: Option<&crate::UpdateInfo>,
    update_progress: Option<&crate::UpdateProgress>,
    update_error: Option<&str>,
    root: &RootView,
) -> impl IntoElement {
    let image = root.load_device_image(d);
    let connected = d.connected;

    let art = if let Some(img) = image {
        div()
            .relative()
            .w(px(66.))
            .h(px(58.))
            .flex_shrink_0()
            .rounded_2xl()
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .text_color(theme::brand())
            .child(
                gpui::img(img)
                    .size_full()
                    .object_fit(gpui::ObjectFit::Cover),
            )
    } else {
        div()
            .relative()
            .w(px(66.))
            .h(px(58.))
            .flex_shrink_0()
            .rounded_2xl()
            .bg(theme::surface_elevated())
            .border_1()
            .border_color(theme::border_subtle())
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center()
            .text_color(theme::brand())
            .child(super::devices::device_art_placeholder(name, theme::brand()))
    };

    let view_for_info = view.clone();
    let art = art
        .id("device-image-button")
        .cursor_pointer()
        .hover(|st| st.opacity(0.85))
        .on_click(move |_, _, cx| {
            view_for_info.update(cx, |root, cx| {
                root.show_device_info = true;
                root.show_sound_effects = false;
                root.show_device_settings = false;
                root.show_control_settings = false;
                root.button_dropdown = None;
                cx.notify();
            });
        });

    let show_update_badge = update_info.is_some() && update_progress.is_none();
    let latest = update_info
        .map(|u| u.latest_version.clone())
        .unwrap_or_default();

    div()
        .px_3()
        .pt_3()
        .pb(px(10.))
        .flex()
        .items_start()
        .gap_3()
        .border_b_1()
        .border_color(theme::border_medium())
        .child(art)
        .child({
            // Close lives on the name row so the battery row gets full column
            // width (target: L | R | Case on one line under the name).
            let view_for_close = view.clone();
            div()
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .min_w_0()
                                .flex_1()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_size(px(18.))
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(theme::foreground())
                                        .text_ellipsis()
                                        .whitespace_nowrap()
                                        .child(name.to_string()),
                                )
                                .child(status_dot(connected)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .flex_shrink_0()
                                .when(show_update_badge, |el| {
                                    let view = view.clone();
                                    el.child(
                                        div()
                                            .id("update-badge")
                                            .p_1()
                                            .rounded_md()
                                            .text_color(theme::brand())
                                            .hover(|st| st.bg(theme::hover_medium()))
                                            .cursor_pointer()
                                            .on_click(move |_, _window, cx| {
                                                let _ = &latest;
                                                view.update(cx, |root, cx| {
                                                    root.start_update(cx);
                                                });
                                            })
                                            .child(
                                                div()
                                                    .relative()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(download_icon(16., theme::brand()))
                                                    .child(
                                                        div()
                                                            .absolute()
                                                            .right(px(-2.))
                                                            .top(px(-2.))
                                                            .size(px(6.))
                                                            .rounded_full()
                                                            .bg(theme::brand()),
                                                    ),
                                            ),
                                    )
                                })
                                .child({
                                    let view = view_for_close.clone();
                                    div()
                                        .id("header-about")
                                        .size(px(30.))
                                        .rounded_full()
                                        .bg(theme::hover_subtle())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(theme::foreground())
                                        .cursor_pointer()
                                        .hover(|st| st.bg(theme::hover_medium()))
                                        .on_click(move |_, _window, cx| {
                                            view.update(cx, |root, cx| {
                                                root.show_about = true;
                                                root.show_settings = false;
                                                root.show_sound_effects = false;
                                                cx.notify();
                                            });
                                        })
                                        .child(info_icon(16., theme::foreground()))
                                })
                                .child({
                                    let view = view_for_close;
                                    div()
                                        .id("header-settings")
                                        .size(px(30.))
                                        .rounded_full()
                                        .bg(theme::hover_subtle())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_color(theme::foreground())
                                        .cursor_pointer()
                                        .hover(|st| st.bg(theme::hover_medium()))
                                        .on_click(move |_, _window, cx| {
                                            view.update(cx, |root, cx| {
                                                root.show_settings = true;
                                                root.show_about = false;
                                                root.show_sound_effects = false;
                                                cx.notify();
                                            });
                                        })
                                        .child(gear_icon(16., theme::foreground()))
                                }),
                        ),
                )
                .child(
                    div()
                        .text_size(px(10.5))
                        .mt(px(2.))
                        .text_color(theme::muted())
                        .child(if connected {
                            "Connected · Soundcore"
                        } else {
                            "Disconnected · Soundcore"
                        }),
                )
                .when(!batteries.is_empty(), |el| {
                    // One row (target): L | R | Case — hug content, no wrap.
                    el.child(div().mt(px(8.)).flex().items_center().gap_1().children(
                        batteries.iter().map(|(label, pct)| {
                            battery_pill(*pct, if label.is_empty() { None } else { Some(label) })
                        }),
                    ))
                })
                .when(update_progress.is_some() || update_error.is_some(), |el| {
                    el.child(update_block(update_progress, update_error))
                })
        })
}

fn status_dot(connected: bool) -> impl IntoElement {
    let color = if connected {
        theme::success()
    } else {
        theme::toggle_off()
    };
    div()
        .relative()
        .size(px(8.))
        .child(
            div()
                .absolute()
                .left(px(-3.))
                .top(px(-3.))
                .size(px(14.))
                .rounded_full()
                .bg(theme::success_25())
                .when(!connected, |el| el.opacity(0.0)),
        )
        .child(div().absolute().size(px(8.)).rounded_full().bg(color))
}

fn battery_pill(pct: i32, label: Option<&str>) -> impl IntoElement {
    let pct = pct.clamp(0, 100);
    let fill = if pct > 50 {
        theme::success()
    } else if pct > 20 {
        theme::warning()
    } else {
        theme::error()
    };
    let is_case = label == Some("Case");
    div()
        .h(px(18.))
        .px_1()
        .rounded_full()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border_subtle())
        .flex()
        .items_center()
        .gap_1()
        // Fixed label slot so bar + % align across pills.
        .child(
            div()
                .w(if is_case { px(11.) } else { px(7.) })
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .when(!is_case, |el| {
                    el.children(label.map(|l| {
                        div()
                            .text_size(px(9.5))
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(theme::brand_secondary())
                            .child(l.to_string())
                    }))
                })
                .when(is_case, |el| {
                    // Compact case glyph: rounded rect + handle notch.
                    el.child(
                        div()
                            .relative()
                            .w(px(9.))
                            .h(px(8.))
                            .border_1()
                            .border_color(theme::brand_secondary())
                            .rounded(px(2.))
                            .child(
                                div()
                                    .absolute()
                                    .top(px(-3.))
                                    .left(px(2.))
                                    .w(px(4.))
                                    .h(px(3.))
                                    .border_1()
                                    .border_color(theme::brand_secondary())
                                    .rounded_t(px(1.))
                                    .bg(theme::surface()),
                            ),
                    )
                }),
        )
        .child(
            div()
                .w(px(16.))
                .h(px(8.))
                .rounded(px(2.))
                .bg(theme::toggle_off())
                .overflow_hidden()
                .flex_shrink_0()
                .child(
                    div()
                        .h_full()
                        .rounded(px(2.))
                        .bg(fill)
                        .w(gpui::relative(pct as f32 / 100.0)),
                ),
        )
        .child(
            div()
                .text_size(px(10.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme::foreground())
                .child(format!("{pct}%")),
        )
}

fn update_block(progress: Option<&crate::UpdateProgress>, error: Option<&str>) -> impl IntoElement {
    if let Some(p) = progress {
        if p.phase == "downloading" {
            return div()
                .mt_2()
                .w(px(192.))
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .text_size(px(10.))
                        .text_color(theme::muted_dim())
                        .mb_1()
                        .child("Downloading update...")
                        .child(format!("{}%", p.percent.round() as i32)),
                )
                .child(
                    div()
                        .h(px(4.))
                        .rounded_full()
                        .bg(theme::toggle_off())
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .rounded_full()
                                .bg(theme::brand())
                                .w(px((192.0 * p.percent / 100.0) as f32)),
                        ),
                );
        }
        return div()
            .mt_2()
            .flex()
            .items_center()
            .gap_1()
            .text_size(px(10.))
            .text_color(theme::muted_dim())
            .child(p.message.clone());
    }
    if let Some(e) = error {
        return div()
            .mt_2()
            .text_size(px(10.))
            .text_color(theme::error_copy())
            .text_ellipsis()
            .whitespace_nowrap()
            .child(e.chars().take(60).collect::<String>());
    }
    div()
}

fn sound_mode(
    view: &Entity<RootView>,
    d: &crate::DeviceStateDto,
    s: &data::SettingsMap,
    root: &RootView,
) -> impl IntoElement {
    let mac = d.mac_address.clone();
    let Some(setting) = s.get("ambientSoundMode").cloned() else {
        return div();
    };
    let opts = data::select_entries(s, "ambientSoundMode");
    let polled = setting
        .get("value")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let local = root
        .ambient_local
        .as_ref()
        .filter(|(m, _)| m == &mac)
        .map(|(_, v)| v.clone());
    let active = local.unwrap_or(polled);

    let modes: Vec<(String, String)> = opts;

    let anc_count = modes.len();
    let anc_active_index = modes.iter().position(|(id, _)| id == &active).unwrap_or(0);

    let mut row = div()
        .rounded_2xl()
        .bg(gpui::rgba(0xffffff0f))
        .border_1()
        .border_color(gpui::rgba(0xffffff1f))
        .p(px(10.))
        .flex()
        .flex_col()
        .gap_2()
        .child(
            // ANC track — black 28%, compact height like the target.
            div()
                .h(px(40.))
                .rounded_xl()
                .bg(gpui::rgba(0x00000047))
                .flex()
                .relative()
                .child(morphing_pill(anc_active_index, anc_count, "anc"))
                .children(modes.into_iter().map(|(id, label)| {
                    let on = id == active;
                    let seg_key = format!("seg-mode-{id}");
                    let view = view.clone();
                    let mac = mac.clone();
                    let id_for_click = id.clone();
                    segment_text(
                        div()
                            .id(gpui::SharedString::from(format!("mode-{id}")))
                            .flex_1()
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap(px(6.))
                            .rounded_xl()
                            .text_size(px(11.5))
                            .font_weight(if on {
                                gpui::FontWeight::BOLD
                            } else {
                                gpui::FontWeight::SEMIBOLD
                            })
                            .when(!on, |el| el.hover(|st| st.bg(gpui::rgba(0xffffff14))))
                            .cursor_pointer()
                            .on_click(move |_, _window, cx| {
                                let view = view.clone();
                                let mac = mac.clone();
                                let id = id_for_click.clone();
                                view.update(cx, |root, cx| {
                                    root.ambient_local = Some((mac.clone(), id.clone()));
                                    root.send_setting(&mac, "ambientSoundMode", &id);
                                    let mac2 = mac.clone();
                                    cx.spawn(async move |this, cx| {
                                        cx.background_executor()
                                            .timer(std::time::Duration::from_millis(2000))
                                            .await;
                                        this.update(&mut *cx, |view, cx| {
                                            if view.ambient_local.as_ref().map(|(m, _)| *m == mac2)
                                                == Some(true)
                                            {
                                                view.ambient_local = None;
                                                cx.notify();
                                            }
                                        })
                                        .ok();
                                    })
                                    .detach();
                                    cx.notify();
                                });
                            })
                            .child(label.to_string()),
                        on,
                        seg_key,
                    )
                })),
        );

    let lower = active.to_lowercase();
    let has_transparency = select_sub(s, "transparencyMode");
    let has_nc = select_sub(s, "noiseCancelingMode");
    if (lower.contains("transparen") || (has_transparency.is_some() && has_nc.is_none()))
        && let Some(entries) = has_transparency
    {
        row = row.child(sub_row(entries, s, "transparencyMode", view, d, root));
    }
    if (lower.contains("noise") || lower.contains("nc"))
        && let Some(entries) = has_nc
    {
        row = row.child(sub_row(entries, s, "noiseCancelingMode", view, d, root));
    }
    let nc = s
        .get("noiseCancelingMode")
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();
    let nc_local = root
        .sub_local
        .as_ref()
        .filter(|(m, i, _)| m == &d.mac_address && i == "noiseCancelingMode")
        .map(|(_, _, v)| v.to_lowercase());
    let nc_eff = nc_local.as_deref().unwrap_or(&nc);
    if (nc_eff.contains("multi") || nc_eff.contains("scene"))
        && let Some(scene) = select_sub(s, "multiSceneNoiseCanceling")
    {
        row = row.child(sub_row(scene, s, "multiSceneNoiseCanceling", view, d, root));
    }
    row
}

fn select_sub(s: &data::SettingsMap, id: &str) -> Option<Vec<(String, String)>> {
    let entries = data::select_entries(s, id);
    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

fn sub_row(
    entries: Vec<(String, String)>,
    s: &data::SettingsMap,
    id: &str,
    view: &Entity<RootView>,
    d: &crate::DeviceStateDto,
    root: &RootView,
) -> impl IntoElement {
    let mac = d.mac_address.clone();
    let current = s
        .get(id)
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let local = root
        .sub_local
        .as_ref()
        .filter(|(m, i, _)| m == &mac && i == id)
        .map(|(_, _, v)| v.clone());
    let active = local.unwrap_or(current);
    let sub_count = entries.len();
    let sub_active_index = entries
        .iter()
        .position(|(eid, _)| eid == &active)
        .unwrap_or(0);
    let pill_key: &'static str = if id == "transparencyMode" {
        "sub-transparency"
    } else {
        "sub-nc"
    };

    div()
        .h(px(36.))
        .rounded_xl()
        .bg(gpui::rgba(0x00000047))
        .flex()
        .relative()
        .child(morphing_pill(sub_active_index, sub_count, pill_key))
        .children(entries.into_iter().map(|(eid, label)| {
            let on = eid == active;
            let seg_key = format!("seg-sub-{id}-{eid}");
            let view = view.clone();
            let mac = mac.clone();
            let sid = id.to_string();
            let eid2 = eid.clone();
            segment_text(
                div()
                    .id(gpui::SharedString::from(format!("sub-{id}-{eid}")))
                    .flex_1()
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(6.))
                    .rounded_xl()
                    .text_size(px(11.5))
                    .font_weight(if on {
                        gpui::FontWeight::BOLD
                    } else {
                        gpui::FontWeight::SEMIBOLD
                    })
                    .when(!on, |el| el.hover(|st| st.bg(gpui::rgba(0xffffff14))))
                    .cursor_pointer()
                    .on_click(move |_, _window, cx| {
                        let view = view.clone();
                        let mac = mac.clone();
                        let sid = sid.clone();
                        let eid = eid2.clone();
                        view.update(cx, |root, cx| {
                            root.sub_local = Some((mac.clone(), sid.clone(), eid.clone()));
                            root.sub_local_at = Some(std::time::Instant::now());
                            root.send_setting(&mac, &sid, &eid);
                            cx.notify();
                        });
                    })
                    .child(label),
                on,
                seg_key,
            )
        }))
}

fn quick_toggles(
    view: &Entity<RootView>,
    s: &data::SettingsMap,
    d: &crate::DeviceStateDto,
    root: &RootView,
) -> impl IntoElement {
    let toggles = data::quick_toggles(s);
    if toggles.is_empty() {
        return div();
    }
    let mac = d.mac_address.clone();
    let optimistic = root.toggle_local.clone();
    div()
        .rounded_xl()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border_subtle())
        .children(toggles.into_iter().enumerate().map(|(i, (id, polled))| {
            let on = optimistic.get(&id).copied().unwrap_or(polled);
            let view = view.clone();
            let mac = mac.clone();
            let id2 = id.clone();
            div()
                .id(gpui::SharedString::from(format!("toggle-{id}")))
                .w_full()
                .flex()
                .items_center()
                .gap_3()
                .px_3()
                .py(px(9.))
                .hover(|st| st.bg(theme::hover_subtle()))
                .when(i > 0, |el| {
                    el.border_t_1().border_color(theme::border_subtle())
                })
                .cursor_pointer()
                .on_click(move |_, _window, cx| {
                    let view = view.clone();
                    let mac = mac.clone();
                    let id = id2.clone();
                    let next = !on;
                    view.update(cx, |root, cx| {
                        root.toggle_local.insert(id.clone(), next);
                        root.send_setting(&mac, &id, if next { "true" } else { "false" });
                        cx.spawn(async move |this, cx| {
                            cx.background_executor()
                                .timer(std::time::Duration::from_millis(500))
                                .await;
                            this.update(&mut *cx, |view, cx| {
                                view.toggle_local.remove(&id);
                                cx.notify();
                            })
                            .ok();
                        })
                        .detach();
                        cx.notify();
                    });
                })
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .text_size(px(13.))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(theme::foreground())
                        .child(data::pretty_setting_id(&id)),
                )
                .child(toggle_switch(on, format!("quick-{id}")))
        }))
}
