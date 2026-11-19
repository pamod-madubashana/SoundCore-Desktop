//! App Settings screen — general preferences (header 3-dots → Settings).

use gpui::{Context, Entity, IntoElement, ParentElement, Styled, Window, div, prelude::*};
use crate::UI::scale::px;

use super::common::{chevron_left, chevron_right};
use crate::config::Config;
use crate::UI::{RootView, theme};

pub fn render(
    root: &mut RootView,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let view = cx.entity();
    let _ = root;
    let cfg = crate::get_config(crate::state());

    let general = vec![
        toggle_row(&view, "Start at login", cfg.autostart, |c, v| c.autostart = v),
        toggle_row(
            &view,
            "Connection notifications",
            cfg.notify_connect,
            |c, v| c.notify_connect = v,
        ),
    ];
    let config_path = crate::config::resolve_path(None).display().to_string();
    let system = vec![
        action_row("Bluetooth settings", "ms-settings:bluetooth"),
        action_row("Open config file", &config_path),
    ];

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
                .gap_3()
                .border_b_1()
                .border_color(theme::border_medium())
                .child(header_btn(&view, cx, |root, cx| {
                    root.show_settings = false;
                    cx.notify();
                }))
                .child(
                    div()
                        .text_size(px(15.))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(theme::foreground())
                        .child("Settings"),
                ),
        )
        // Body fills below header
        .child(
            div()
                .id("settings-body")
                .absolute()
                .top(px(49.))
                .left(px(0.))
                .right(px(0.))
                .bottom(px(0.))
                .overflow_y_scroll()
                .p_3()
                .flex()
                .flex_col()
                .gap(px(14.))
                .child(section("GENERAL", general))
                .child(section("SYSTEM", system)),
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
        .id("settings-back")
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
        .child(chevron_left(14., theme::muted()))
}

fn section(title: &str, rows: Vec<impl IntoElement + Styled>) -> gpui::Div {
    div()
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
        .child(
            div()
                .rounded_2xl()
                .bg(theme::surface())
                .border_1()
                .border_color(theme::border_subtle())
                .overflow_hidden()
                .children(rows.into_iter().enumerate().map(|(i, row)| {
                    row.when(i > 0, |el| {
                        el.border_t_1().border_color(theme::border_subtle())
                    })
                })),
        )
}

fn toggle_row(
    view: &Entity<RootView>,
    label: &str,
    on: bool,
    set: fn(&mut Config, bool),
) -> impl IntoElement + Styled {
    let label = label.to_string();
    let view = view.clone();
    div()
        .id(gpui::SharedString::from(format!("toggle-{label}")))
        .w_full()
        .px_3()
        .py(px(10.))
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .hover(|st| st.bg(theme::hover_subtle()))
        .cursor_pointer()
        .on_click(move |_, _window, cx| {
            let next = !on;
            view.update(cx, |_root, cx| {
                let mut cfg = crate::get_config(crate::state());
                set(&mut cfg, next);
                let _ = crate::save_config(crate::state(), cfg);
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
                .child(label),
        )
        .child(switch(on))
}

fn action_row(label: &str, target: &str) -> impl IntoElement + Styled {
    let target = target.to_string();
    div()
        .id(gpui::SharedString::from(format!("action-{label}")))
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
            crate::open_url(target.clone());
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
        .child(chevron_right(16., theme::muted()))
}

fn switch(on: bool) -> gpui::Div {
    div()
        .w(px(32.))
        .h(px(18.))
        .rounded_full()
        .bg(if on { theme::brand() } else { theme::toggle_off() })
        .flex_shrink_0()
        .relative()
        .child(
            div()
                .absolute()
                .top(px(2.))
                .left(if on { px(16.) } else { px(2.) })
                .size(px(14.))
                .rounded_full()
                .bg(theme::toggle_thumb()),
        )
}
