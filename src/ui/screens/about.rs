//! About screen (header 3-dots → About).

use crate::ui::scale::px;
use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, prelude::*};

use super::common::chevron_left;
use crate::ui::{RootView, theme};

const REPO: &str = "https://github.com/pamod-madubashana/SoundCore-Desktop";

pub fn render(root: &mut RootView, _window: &mut Window, cx: &mut Context<RootView>) -> gpui::Div {
    let view = cx.entity();
    let version = root.version.clone();

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
                .justify_center()
                .border_b_1()
                .border_color(theme::border_medium())
                .child(
                    div()
                        .absolute()
                        .left(px(12.))
                        .top(px(0.))
                        .bottom(px(0.))
                        .flex()
                        .items_center()
                        .child(header_btn(&view, cx, |root, cx| {
                            root.show_about = false;
                            cx.notify();
                        })),
                )
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::foreground())
                        .child("About"),
                ),
        )
        // Body fills below header
        .child(
            div()
                .id("about-body")
                .absolute()
                .top(px(59.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .overflow_y_scroll()
                .p_4()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(8.))
                .pt(px(26.))
                .child(
                    div()
                        .size(px(64.))
                        .rounded_2xl()
                        .overflow_hidden()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(project_icon(56.)),
                )
                .child(
                    div()
                        .mt(px(4.))
                        .text_size(px(16.))
                        .font_weight(gpui::FontWeight::BOLD)
                        .text_color(theme::foreground())
                        .child("SoundCore Desktop"),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(theme::muted())
                        .child(format!("v{version}")),
                )
                .child(
                    div()
                        .mt(px(2.))
                        .max_w(px(280.))
                        .text_size(px(12.))
                        .text_color(theme::muted())
                        .text_center()
                        .child("Control your Soundcore headphones straight from the Windows tray."),
                )
                .child(
                    div()
                        .mt(px(12.))
                        .w_full()
                        .rounded_2xl()
                        .bg(theme::surface())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .overflow_hidden()
                        .child(link_row("GitHub repository", REPO.to_string()))
                        .child(
                            div()
                                .border_t_1()
                                .border_color(theme::border_subtle())
                                .child(link_row("Report an issue", format!("{REPO}/issues/new"))),
                        ),
                ),
        )
}

fn header_btn(
    view: &Entity<RootView>,
    cx: &mut Context<RootView>,
    on: impl Fn(&mut RootView, &mut Context<RootView>) + 'static,
) -> impl IntoElement {
    let _ = cx;
    let view = view.clone();
    div()
        .id("about-back")
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
        .child(chevron_left(14., theme::muted()))
}

/// Project app icon (assets/icons/icon.png).
fn project_icon(size: f32) -> impl IntoElement {
    static CACHE: std::sync::OnceLock<std::sync::Arc<gpui::Image>> = std::sync::OnceLock::new();
    let img = CACHE.get_or_init(|| {
        let bytes = include_bytes!("../../../assets/icons/icon.png");
        std::sync::Arc::new(gpui::Image::from_bytes(
            gpui::ImageFormat::Png,
            bytes.to_vec(),
        ))
    });
    gpui::img(img.clone()).size(px(size))
}

fn link_row(label: &str, url: String) -> impl IntoElement {
    div()
        .id(gpui::SharedString::from(format!("link-{label}")))
        .w_full()
        .px_3()
        .py(px(10.))
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .hover(|st| st.bg(theme::hover_subtle()))
        .cursor_pointer()
        .on_click(move |_, _window, _cx| {
            crate::open_url(url.clone());
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(13.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(theme::foreground())
                .child(label.to_string()),
        )
        .child(
            div()
                .text_size(px(14.))
                .text_color(theme::muted())
                .child("↗"),
        )
}
