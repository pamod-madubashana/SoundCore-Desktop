//! Searching screen (device discovery).

use crate::ui::scale::px;
use gpui::{Context, IntoElement, ParentElement, Styled, Window, div};

use super::common::spinner;
use crate::ui::{RootView, theme};

pub fn render(
    root: &mut RootView,
    _window: &mut Window,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let tick = root.spinner_tick;
    let view = cx.entity();

    div()
        .size_full()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3()
        .text_color(theme::muted())
        .child(spinner(tick, theme::brand()))
        .child(
            div()
                .text_size(px(13.))
                .child("Searching for a Soundcore device…"),
        )
        .child(
            div()
                .text_size(px(11.))
                .opacity(0.7)
                .child("Connect it via Bluetooth and it'll appear here."),
        )
        .child(super::update_notice(root, &view))
}
