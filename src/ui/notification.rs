use gpui::{
    AnimationExt as _, AnimationPhase, Context, IntoElement, ParentElement, Render, SharedString,
    SpringAnimation, SpringConfig, Styled, Window, div, prelude::*,
};

use super::screens::common::sliders_icon;
use super::{load_device_image_url, scale::px, theme};
use crate::PendingNotification;

const ENTRY_SPRING: SpringConfig = SpringConfig::new(180.0, 22.0, 1.0);

pub struct NotificationView {
    notification: PendingNotification,
    entry_id: u64,
}

impl NotificationView {
    pub fn new(notification: PendingNotification) -> Self {
        Self {
            notification,
            entry_id: 0,
        }
    }

    pub fn set_notification(&mut self, notification: PendingNotification) {
        self.notification = notification;
        self.entry_id = self.entry_id.wrapping_add(1);
    }
}

impl Render for NotificationView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let n = &self.notification;
        let connected = n.status == "Connected";
        let accent = if connected {
            theme::success()
        } else {
            theme::muted()
        };
        let image = n.image.as_deref().and_then(load_device_image_url);
        let has_image = image.is_some();
        let battery = n.battery.as_ref().and_then(|b| {
            let mut parts = Vec::new();
            if let Some(pct) = b.combined {
                parts.push(format!("{pct}%"));
            } else {
                if let Some(pct) = b.left {
                    parts.push(format!("L {pct}%"));
                }
                if let Some(pct) = b.right {
                    parts.push(format!("R {pct}%"));
                }
                if let Some(pct) = b.case {
                    parts.push(format!("Case {pct}%"));
                }
            }
            if parts.is_empty() {
                None
            } else {
                Some(parts.join("  ·  "))
            }
        });
        let entry_key = SharedString::from(format!("notification-entry-{}", self.entry_id));

        div()
            .id("notification-root")
            .size_full()
            .p(px(8.))
            .with_spring(
                entry_key,
                SpringAnimation::new(ENTRY_SPRING)
                    .to(AnimationPhase(1.0))
                    .from(AnimationPhase(0.0)),
                move |this, phase| {
                    let p = phase.clamp(0.0..=1.0);
                    this.opacity(p.0).mt(px((1.0 - p.0) * 14.0))
                },
            )
            .child(
                div()
                    .id("notification-card")
                    .w_full()
                    .h_full()
                    .rounded_2xl()
                    .bg(theme::surface())
                    .border_1()
                    .border_color(theme::border_medium())
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_3()
                    .child(
                        div()
                            .size(px(60.))
                            .flex_shrink_0()
                            .rounded_xl()
                            .overflow_hidden()
                            .flex()
                            .items_center()
                            .justify_center()
                            .children(image.map(|img| gpui::img(img).size_full()))
                            .when(!has_image, |el| el.child(sliders_icon(22., theme::brand()))),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(theme::foreground())
                                    .text_ellipsis()
                                    .child(n.name.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(div().size(px(7.)).rounded_full().bg(accent))
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(accent)
                                            .child(n.status.clone()),
                                    ),
                            )
                            .when_some(battery, |el, text| {
                                el.child(
                                    div()
                                        .text_size(px(10.))
                                        .text_color(theme::muted())
                                        .text_ellipsis()
                                        .child(text),
                                )
                            }),
                    ),
            )
    }
}
