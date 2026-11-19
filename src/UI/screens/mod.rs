//! Shared chrome for screens + update prompt (App.tsx `UpdateNotice`).

pub mod about;
pub mod common;
pub mod control;
pub mod devices;
pub mod searching;
pub mod settings;
pub mod sound_effects;

use gpui::{Entity, IntoElement, ParentElement, Styled, div, prelude::*};
use crate::UI::scale::px;

use crate::UI::{RootView, theme};

pub(crate) fn update_notice(
    root: &RootView,
    view: &Entity<RootView>,
) -> impl IntoElement {
    let info = root.update_info.clone();
    let progress = root.update_progress.clone();
    let error = root.update_error.clone();

    if info.is_none() && progress.is_none() && error.is_none() {
        return div();
    }

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .when(info.is_some() && progress.is_none(), |el| {
            let latest = info
                .as_ref()
                .map(|i| i.latest_version.clone())
                .unwrap_or_default();
            let view = view.clone();
            el.child(
                div()
                    .id("update-pill")
                    .px_3()
                    .py(px(6.))
                    .rounded_lg()
                    .bg(theme::brand_10())
                    .text_color(theme::brand())
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(px(11.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .hover(|st| st.bg(theme::brand_20()))
                    .cursor_pointer()
                    .on_click(move |_, _window, cx| {
                        view.update(cx, |root, cx| {
                            root.start_update(cx);
                        });
                    })
                    .child(common::download_icon(14., theme::brand()))
                    .child(format!("Update to v{latest}")),
            )
        })
        .when(
            progress.as_ref().map(|p| p.phase.as_str()) == Some("downloading"),
            |el| {
                let pct = progress.as_ref().map(|p| p.percent).unwrap_or(0.0);
                el.child(
                    div()
                        .w(px(192.))
                        .child(
                            div()
                                .flex()
                                .justify_between()
                                .text_size(px(10.))
                                .text_color(theme::muted_dim())
                                .mb_1()
                                .child("Downloading...")
                                .child(format!("{}%", pct.round() as i32)),
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
                                        .w(px((192.0 * pct / 100.0) as f32)),
                                ),
                        ),
                )
            },
        )
        .when(
            progress
                .as_ref()
                .map(|p| p.phase.as_str())
                .map(|ph| ph != "downloading")
                .unwrap_or(false),
            |el| {
                let msg = progress
                    .as_ref()
                    .map(|p| p.message.clone())
                    .unwrap_or_default();
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(10.))
                        .text_color(theme::muted_dim())
                        .child(msg),
                )
            },
        )
        .when(error.is_some(), |el| {
            let e = error.unwrap_or_default();
            el.child(
                div()
                    .text_size(px(10.))
                    .text_color(theme::error_copy())
                    .max_w(px(240.))
                    .text_ellipsis()
                    .child(e.chars().take(80).collect::<String>()),
            )
        })
}
