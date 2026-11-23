use crate::ui::scale::px;
use gpui::{
    AnimationExt as _, AnimationPhase, Context, Entity, IntoElement, ParentElement, SharedString,
    SpringAnimation, SpringConfig, Styled, Window, div, prelude::*,
};
use soundcore_protocol::analysis::{
    CapturedFrameDto, ReportShare, SHARE_URL_MAX, analyze, build_share_bundle,
};

use super::common::chevron_left;
use crate::ui::{RootView, data, theme};

const MAX_VISIBLE_FRAMES: usize = 80;
const PROGRESS_WIDTH: f32 = 352.;
const PROGRESS_SPRING: SpringConfig = SpringConfig::new(220.0, 26.0, 1.0);
const CONTROL_SPRING: SpringConfig = SpringConfig::new(280.0, 26.0, 1.0);
const ART_SPRING: SpringConfig = SpringConfig::new(260.0, 24.0, 1.0);

pub fn render(
    root: &mut RootView,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let view = cx.entity();

    let frames: Vec<_> = root
        .capture_frames
        .iter()
        .take(MAX_VISIBLE_FRAMES)
        .cloned()
        .collect();
    let frame_count = root.capture_frames.len();
    let selected_frame = root.protocol_selected.and_then(|i| frames.get(i)).cloned();
    let active_device = root.protocol_active_device.clone();
    let show_details = root.protocol_show_details;
    let progress = root.protocol_capture_progress;
    let encrypting = root.protocol_capture_encrypting;
    let share_error = root.protocol_share_error.clone();
    let share_notice = root.protocol_share_notice.clone();

    let (report_model, report_name) = active_device
        .as_ref()
        .map(|(_, name, model)| (model.clone(), name.clone()))
        .unwrap_or_else(|| ("Unknown".to_string(), "Unknown device".to_string()));
    let progress_name = report_name.clone();

    let analysis = selected_frame
        .as_ref()
        .and_then(|frame| analyze(&frame.hex, &report_model));

    let mut device_rows: Vec<gpui::Stateful<gpui::Div>> = Vec::new();
    let connected: Vec<_> = root.devices.iter().filter(|d| d.connected).collect();
    let selected_connected = root
        .protocol_selected
        .and_then(|i| connected.get(i).copied());
    let active_connected = active_device
        .as_ref()
        .and_then(|(mac, _, _)| connected.iter().find(|d| d.mac_address == *mac).copied());
    let display_device = selected_connected.or(active_connected);
    if connected.is_empty() {
        device_rows.push(hint("protocol-devices-empty", "No connected devices"));
    } else {
        let rows = connected
            .iter()
            .enumerate()
            .map(|(i, device)| {
                let name = data::display_device_name(device);
                let mac = device.mac_address.clone();
                let device_model = device.model.clone();
                let row_label = format!("{} · {}", mac, device_model);
                let is_selected = root.protocol_selected == Some(i);
                let view = view.clone();
                div()
                    .id(SharedString::from(format!("protocol-device-{i}")))
                    .w_full()
                    .px_2()
                    .py(px(6.))
                    .rounded_md()
                    .bg(if is_selected {
                        theme::brand_15()
                    } else {
                        theme::hover_subtle()
                    })
                    .hover(|st| st.bg(theme::hover_medium()))
                    .cursor_pointer()
                    .on_click(move |_, _, cx| {
                        view.update(cx, |root, cx| {
                            root.protocol_selected = Some(i);
                            cx.notify();
                        });
                    })
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .min_w_0()
                            .text_size(px(11.))
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_ellipsis()
                            .child(name),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .text_size(px(10.))
                            .text_color(theme::muted())
                            .text_ellipsis()
                            .child(row_label),
                    )
            })
            .collect::<Vec<_>>();
        device_rows.push(
            div()
                .id("protocol-device-list")
                .w_full()
                .max_h(px(150.))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(rows),
        );
    }

    let mut active_rows: Vec<gpui::Stateful<gpui::Div>> = Vec::new();
    if let Some((_, name, _)) = active_device.as_ref() {
        active_rows.push(
            div()
                .id("protocol-active-head")
                .w_full()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .min_w_0()
                        .text_size(px(12.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_ellipsis()
                        .child(format!("soundcore {} · {} frames", name, frame_count)),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .gap(px(4.))
                        .child(button(
                            &view,
                            "protocol-details-toggle",
                            if show_details {
                                "Hide details"
                            } else {
                                "Details"
                            },
                            |root, cx| {
                                root.protocol_show_details = !root.protocol_show_details;
                                cx.notify();
                            },
                        ))
                        .child(button(
                            &view,
                            "protocol-change-device",
                            "Change device",
                            |root, cx| {
                                crate::stop_capture(crate::state());
                                root.protocol_active_device = None;
                                root.protocol_selected = None;
                                root.protocol_capture_progress = None;
                                root.protocol_capture_encrypting = false;
                                root.protocol_share = None;
                                root.protocol_show_details = false;
                                root.protocol_share_error = None;
                                root.protocol_share_notice = None;
                                cx.notify();
                            },
                        )),
                ),
        );
    }

    let mut capture_rows: Vec<gpui::Stateful<gpui::Div>> = Vec::new();
    capture_rows.push(
        div()
            .id("protocol-capture-meta")
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .child(
                div()
                    .min_w_0()
                    .text_size(px(11.))
                    .text_color(theme::muted())
                    .child(format!("{frame_count} frames")),
            )
            .child(button(&view, "protocol-clear", "Clear", |root, cx| {
                crate::clear_capture(crate::state());
                root.protocol_selected = None;
                root.protocol_capture_encrypting = false;
                root.protocol_share = None;
                root.protocol_share_error = None;
                root.protocol_share_notice = None;
                cx.notify();
            })),
    );

    if frames.is_empty() {
        capture_rows.push(hint("protocol-capture-empty", "No frames captured yet"));
    } else {
        let rows = frames
            .iter()
            .enumerate()
            .map(|(i, frame)| {
                let tag = analyze(&frame.hex, &report_model)
                    .map(|decoded| format!("{:02x}.{:02x}", decoded.group, decoded.command))
                    .unwrap_or_else(|| "--".to_string());
                let is_selected = root.protocol_selected == Some(i);
                let preview = if frame.hex.len() > 32 {
                    format!("{}…", &frame.hex[..32])
                } else {
                    frame.hex.clone()
                };
                let view = view.clone();
                div()
                    .id(SharedString::from(format!("protocol-frame-{i}")))
                    .w_full()
                    .px_2()
                    .py(px(6.))
                    .rounded_md()
                    .bg(if is_selected {
                        theme::brand_15()
                    } else {
                        theme::hover_subtle()
                    })
                    .hover(|st| st.bg(theme::hover_medium()))
                    .cursor_pointer()
                    .on_click(move |_, _, cx| {
                        view.update(cx, |root, cx| {
                            root.protocol_selected = Some(i);
                            cx.notify();
                        });
                    })
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(
                                div()
                                    .min_w_0()
                                    .text_size(px(11.))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_ellipsis()
                                    .child(format!("{} · {}", frame.direction.to_uppercase(), tag)),
                            )
                            .child(
                                div()
                                    .flex_shrink_0()
                                    .text_size(px(10.))
                                    .text_color(theme::muted())
                                    .child(format!("{}ms", frame.timestamp_ms)),
                            ),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .text_size(px(10.))
                            .text_color(theme::muted_dim())
                            .text_ellipsis()
                            .child(preview),
                    )
            })
            .collect::<Vec<_>>();
        capture_rows.push(
            div()
                .id("protocol-frame-list")
                .w_full()
                .max_h(px(170.))
                .overflow_y_scroll()
                .flex()
                .flex_col()
                .gap(px(2.))
                .children(rows),
        );
    }

    let mut analysis_rows: Vec<gpui::Stateful<gpui::Div>> = Vec::new();
    match (selected_frame.as_ref(), analysis.as_ref()) {
        (Some(_), Some(decoded)) => {
            analysis_rows.push(
                div()
                    .id("protocol-analysis-head")
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .child(
                        div()
                            .min_w_0()
                            .text_size(px(12.))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(format!("{:02x}.{:02x}", decoded.group, decoded.command)),
                    )
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_size(px(10.))
                            .text_color(if decoded.checksum_valid {
                                theme::success()
                            } else {
                                theme::error()
                            })
                            .child(if decoded.checksum_valid {
                                "checksum ok"
                            } else {
                                "checksum bad"
                            }),
                    ),
            );
            for (i, (key, value)) in decoded.fields.iter().enumerate() {
                analysis_rows.push(
                    div()
                        .id(SharedString::from(format!("protocol-field-{i}")))
                        .w_full()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .py(px(3.))
                        .border_t_1()
                        .border_color(theme::border_subtle())
                        .child(
                            div()
                                .min_w_0()
                                .text_size(px(11.))
                                .text_color(theme::muted())
                                .text_ellipsis()
                                .child(key.clone()),
                        )
                        .child(
                            div()
                                .min_w_0()
                                .text_right()
                                .text_size(px(11.))
                                .text_ellipsis()
                                .child(value.clone()),
                        ),
                );
            }
            analysis_rows.push(
                div()
                    .id("protocol-analysis-meta")
                    .w_full()
                    .pt_1()
                    .text_size(px(10.))
                    .text_color(theme::muted_dim())
                    .child(format!(
                        "{} bytes · direction {}",
                        decoded.bytes,
                        decoded.direction.to_uppercase()
                    )),
            );
        }
        (Some(_), None) => {
            analysis_rows.push(hint(
                "protocol-analysis-bad",
                "Could not decode the selected frame",
            ));
        }
        (None, _) => {
            analysis_rows.push(hint(
                "protocol-analysis-hint",
                "Select a captured frame to inspect its decoded fields",
            ));
        }
    }

    let copy_model = report_model.clone();
    let copy_frames = frames.clone();
    let copy_name = report_name.clone();
    let issue_model = report_model.clone();
    let issue_frames = frames.clone();
    let issue_name = report_name.clone();
    let email_model = report_model;
    let email_frames = frames.clone();
    let email_name = report_name;
    let mut report_rows = vec![
        div()
            .id("protocol-share-row")
            .w_full()
            .flex()
            .items_center()
            .justify_center()
            .gap_2()
            .child(share_button(
                &view,
                "protocol-copy",
                "Copy",
                move |root, cx| {
                    match cached_share(root, &copy_model, &copy_name, &copy_frames) {
                        Ok(share) => {
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(share.report));
                            root.protocol_share_error = None;
                            root.protocol_share_notice =
                                Some("Full encrypted report copied.".to_string());
                        }
                        Err(err) => {
                            root.protocol_share_notice = None;
                            root.protocol_share_error =
                                Some(format!("Could not encrypt report: {err}"));
                        }
                    }
                    cx.notify();
                },
            ))
            .child(share_button(
                &view,
                "protocol-issue",
                "GitHub issue",
                move |root, cx| {
                    match cached_share(root, &issue_model, &issue_name, &issue_frames) {
                        Ok(share) => {
                            root.protocol_share_error = None;
                            if share.github_url.chars().count() > SHARE_URL_MAX {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                    share.report,
                                ));
                                crate::open_url(share.github_title_url);
                                root.protocol_share_notice = Some(
                                    "Full report copied; paste it into the issue.".to_string(),
                                );
                            } else {
                                crate::open_url(share.github_url);
                                root.protocol_share_notice = None;
                            }
                        }
                        Err(err) => {
                            root.protocol_share_notice = None;
                            root.protocol_share_error =
                                Some(format!("Could not encrypt report: {err}"));
                        }
                    }
                    cx.notify();
                },
            ))
            .child(share_button(
                &view,
                "protocol-email",
                "Email",
                move |root, cx| {
                    match cached_share(root, &email_model, &email_name, &email_frames) {
                        Ok(share) => {
                            root.protocol_share_error = None;
                            if share.email_url.chars().count() > SHARE_URL_MAX {
                                cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                    share.report,
                                ));
                                crate::open_url(share.email_title_url);
                                root.protocol_share_notice = Some(
                                    "Full report copied; paste it into the email.".to_string(),
                                );
                            } else {
                                crate::open_url(share.email_url);
                                root.protocol_share_notice = None;
                            }
                        }
                        Err(err) => {
                            root.protocol_share_notice = None;
                            root.protocol_share_error =
                                Some(format!("Could not encrypt report: {err}"));
                        }
                    }
                    cx.notify();
                },
            )),
    ];
    if let Some(notice) = share_notice {
        report_rows.push(
            div()
                .id("protocol-share-notice")
                .w_full()
                .text_center()
                .text_size(px(10.))
                .text_color(theme::muted())
                .child(notice),
        );
    }
    if let Some(error) = share_error {
        report_rows.push(
            div()
                .id("protocol-share-error")
                .w_full()
                .text_center()
                .text_size(px(10.))
                .text_color(theme::error())
                .child(error),
        );
    }

    let percent = progress.unwrap_or(0);
    let progress_rows: Vec<gpui::Stateful<gpui::Div>> = vec![
        div()
            .id("protocol-progress-head")
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .gap_2()
            .child(
                div()
                    .min_w_0()
                    .text_size(px(12.))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_ellipsis()
                    .child(if encrypting {
                        format!("Encrypting {}", progress_name)
                    } else {
                        format!("Capturing {}", progress_name)
                    }),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(px(13.))
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(theme::foreground())
                    .child(format!("{percent}%")),
            ),
        div()
            .id("protocol-progress-wrap")
            .mt(px(8.))
            .w_full()
            .flex()
            .child(
                div()
                    .id("protocol-progress")
                    .w_full()
                    .h(px(6.))
                    .rounded_full()
                    .bg(theme::toggle_off())
                    .relative()
                    .overflow_hidden()
                    .child(
                        div()
                            .absolute()
                            .left(px(0.))
                            .top(px(0.))
                            .bottom(px(0.))
                            .bg(theme::brand())
                            .rounded_full()
                            .with_spring(
                                "protocol-progress-fill",
                                SpringAnimation::new(PROGRESS_SPRING)
                                    .to(AnimationPhase(percent as f32)),
                                |this, p| {
                                    this.w(px(PROGRESS_WIDTH * p.0.clamp(0.0, 100.0) / 100.0))
                                },
                            ),
                    ),
            ),
    ];
    let capturing = progress.is_some();
    let complete = active_device.is_some() && !capturing;
    let expanded = capturing || complete;
    let bottom_control = div()
        .id("protocol-bottom-control")
        .w_full()
        .rounded_xl()
        .bg(if expanded {
            theme::surface()
        } else {
            theme::brand_40()
        })
        .border_1()
        .border_color(theme::border_subtle())
        .when(!expanded, |el| el.hover(|st| st.bg(theme::brand())))
        .relative()
        .overflow_hidden()
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .with_spring(
                    "protocol-capture-slide",
                    SpringAnimation::new(CONTROL_SPRING).to(AnimationPhase(if expanded {
                        -1.0
                    } else {
                        0.0
                    })),
                    |this, phase| {
                        let t = phase.0.clamp(-1.0, 0.0);
                        this.left(px(360.0 * t)).opacity(1.0 + t)
                    },
                )
                .child(capture_button(
                    &view,
                    root.protocol_selected.is_some() && !expanded,
                )),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .px_3()
                .flex()
                .flex_col()
                .justify_center()
                .with_spring(
                    "protocol-progress-slide",
                    SpringAnimation::new(CONTROL_SPRING).to(AnimationPhase(if capturing {
                        0.0
                    } else {
                        1.0
                    })),
                    |this, phase| {
                        let t = phase.0.clamp(0.0, 1.0);
                        this.left(px(360.0 * t)).opacity(1.0 - t)
                    },
                )
                .children(progress_rows),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .px_3()
                .flex()
                .flex_col()
                .justify_center()
                .with_spring(
                    "protocol-report-slide",
                    SpringAnimation::new(CONTROL_SPRING).to(AnimationPhase(if complete {
                        0.0
                    } else {
                        1.0
                    })),
                    |this, phase| {
                        let t = phase.0.clamp(0.0, 1.0);
                        this.left(px(360.0 * t)).opacity(1.0 - t)
                    },
                )
                .children(report_rows),
        )
        .with_spring(
            "protocol-bottom-height",
            SpringAnimation::new(CONTROL_SPRING).to(AnimationPhase(if expanded {
                70.0
            } else {
                42.0
            })),
            |this, phase| this.h(px(phase.0)),
        );

    let image_element = display_device.map(|device| {
        let spring_key = SharedString::from(format!("protocol-art-{}", device.mac_address));
        let image = root.load_device_image(device);
        if let Some(image) = image {
            div()
                .id("protocol-device-art")
                .w_full()
                .h(px(220.))
                .flex()
                .items_center()
                .justify_center()
                .with_spring(
                    spring_key,
                    SpringAnimation::new(ART_SPRING).to(AnimationPhase(1.0)),
                    |this, phase| this.opacity(phase.0.clamp(0.0, 1.0)),
                )
                .child(
                    gpui::img(image)
                        .size_full()
                        .object_fit(gpui::ObjectFit::Contain),
                )
        } else {
            div()
                .id("protocol-device-art")
                .w_full()
                .h(px(220.))
                .flex()
                .items_center()
                .justify_center()
                .with_spring(
                    spring_key,
                    SpringAnimation::new(ART_SPRING).to(AnimationPhase(1.0)),
                    |this, phase| this.opacity(phase.0.clamp(0.0, 1.0)),
                )
                .child(super::devices::device_art_placeholder(
                    &data::display_device_name(device),
                    theme::brand(),
                ))
        }
    });

    let mut body_sections: Vec<gpui::AnyElement> = Vec::new();
    if capturing {
        body_sections.push(section("DEVICES", device_rows).into_any_element());
        body_sections.push(
            div()
                .id("protocol-art-top-spacer")
                .flex_1()
                .min_h(px(8.))
                .into_any_element(),
        );
        if let Some(image) = image_element {
            body_sections.push(image.into_any_element());
        }
        body_sections.push(
            div()
                .id("protocol-art-bottom-spacer")
                .flex_1()
                .min_h(px(8.))
                .into_any_element(),
        );
        body_sections.push(bottom_control.into_any_element());
    } else if complete {
        body_sections.push(card(active_rows).into_any_element());
        body_sections.push(
            div()
                .id("protocol-art-top-spacer")
                .flex_1()
                .min_h(px(8.))
                .into_any_element(),
        );
        if let Some(image) = image_element {
            body_sections.push(image.into_any_element());
        }
        body_sections.push(
            div()
                .id("protocol-art-bottom-spacer")
                .flex_1()
                .min_h(px(8.))
                .into_any_element(),
        );
        body_sections.push(bottom_control.into_any_element());
        if show_details {
            body_sections.push(section("CAPTURE", capture_rows).into_any_element());
            body_sections.push(section("ANALYSIS", analysis_rows).into_any_element());
        }
    } else {
        body_sections.push(section("DEVICES", device_rows).into_any_element());
        body_sections.push(
            div()
                .id("protocol-art-top-spacer")
                .flex_1()
                .min_h(px(8.))
                .into_any_element(),
        );
        if let Some(image) = image_element {
            body_sections.push(image.into_any_element());
        }
        body_sections.push(
            div()
                .id("protocol-art-bottom-spacer")
                .flex_1()
                .min_h(px(8.))
                .into_any_element(),
        );
        body_sections.push(bottom_control.into_any_element());
    }

    div()
        .id("protocol-lab")
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
                        .child("Protocol Lab"),
                ),
        )
        .child(
            div()
                .id("protocol-lab-body")
                .absolute()
                .top(px(59.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .overflow_y_scroll()
                .p_3()
                .flex()
                .flex_col()
                .gap(px(14.))
                .children(body_sections),
        )
}

fn back_button(view: &Entity<RootView>) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    div()
        .id("protocol-lab-back")
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
                crate::stop_capture(crate::state());
                root.show_protocol_lab = false;
                root.protocol_active_device = None;
                root.protocol_selected = None;
                root.protocol_capture_progress = None;
                root.protocol_capture_encrypting = false;
                root.protocol_share = None;
                root.protocol_show_details = false;
                root.protocol_share_error = None;
                root.protocol_share_notice = None;
                cx.notify();
            });
        })
        .child(chevron_left(14., theme::muted()))
}

fn card(rows: Vec<gpui::Stateful<gpui::Div>>) -> gpui::Div {
    div()
        .w_full()
        .rounded_2xl()
        .bg(theme::surface())
        .border_1()
        .border_color(theme::border_subtle())
        .p_3()
        .flex()
        .flex_col()
        .gap(px(6.))
        .children(rows)
}

fn section(title: &str, rows: Vec<gpui::Stateful<gpui::Div>>) -> gpui::Div {
    div()
        .w_full()
        .flex()
        .flex_col()
        .gap(px(6.))
        .child(
            div()
                .px_1()
                .text_size(px(10.5))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(theme::muted())
                .child(title.to_string()),
        )
        .child(card(rows))
}

fn button(
    view: &Entity<RootView>,
    id: &'static str,
    label: &'static str,
    on: impl Fn(&mut RootView, &mut Context<RootView>) + 'static,
) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    div()
        .id(id)
        .flex_shrink_0()
        .px_2()
        .py(px(4.))
        .rounded_md()
        .bg(theme::brand_20())
        .text_color(theme::foreground())
        .text_size(px(11.))
        .font_weight(gpui::FontWeight::MEDIUM)
        .hover(|st| st.bg(theme::brand_20()))
        .cursor_pointer()
        .on_click(move |_, _, cx| {
            view.update(cx, |root, cx| on(root, cx));
        })
        .child(label)
}

fn share_button(
    view: &Entity<RootView>,
    id: &'static str,
    label: &'static str,
    on: impl Fn(&mut RootView, &mut Context<RootView>) + 'static,
) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    div()
        .id(id)
        .flex_1()
        .py(px(9.))
        .rounded_lg()
        .bg(theme::brand_20())
        .text_color(theme::foreground())
        .text_center()
        .text_size(px(12.))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .hover(|st| st.bg(theme::brand_40()))
        .cursor_pointer()
        .on_click(move |_, _, cx| {
            view.update(cx, |root, cx| on(root, cx));
        })
        .child(label)
}

fn capture_button(view: &Entity<RootView>, enabled: bool) -> gpui::Stateful<gpui::Div> {
    let view = view.clone();
    div()
        .id("protocol-capture-start")
        .w_full()
        .text_center()
        .text_size(px(14.))
        .font_weight(gpui::FontWeight::BOLD)
        .when(enabled, |el| {
            el.text_color(theme::brand_foreground()).cursor_pointer()
        })
        .when(!enabled, |el| el.text_color(theme::muted_dim()))
        .on_click(move |_, _, cx| {
            if !enabled {
                return;
            }
            view.update(cx, |root, cx| {
                let device = root
                    .protocol_selected
                    .and_then(|i| root.devices.iter().filter(|d| d.connected).nth(i))
                    .map(|d| {
                        (
                            d.mac_address.clone(),
                            data::display_device_name(d),
                            d.model.clone(),
                        )
                    });
                if let Some((mac, name, model)) = device {
                    root.protocol_active_device = Some((mac, name.clone(), model));
                    crate::start_capture(crate::state());
                    root.protocol_capture_progress = Some(0);
                    root.protocol_capture_encrypting = false;
                    root.protocol_share = None;
                    root.protocol_selected = None;
                    root.protocol_show_details = false;
                    root.protocol_share_error = None;
                    root.protocol_share_notice = None;
                }
                cx.notify();
            });
        })
        .child("Capture")
}

fn cached_share(
    root: &RootView,
    model: &str,
    name: &str,
    frames: &[CapturedFrameDto],
) -> Result<ReportShare, String> {
    root.protocol_share
        .clone()
        .map(Ok)
        .unwrap_or_else(|| build_share_bundle(model, name, frames).map_err(|err| err.to_string()))
}

fn hint(id: &'static str, text: &'static str) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id)
        .w_full()
        .py(px(10.))
        .text_center()
        .text_size(px(11.))
        .text_color(theme::muted())
        .child(text)
}
