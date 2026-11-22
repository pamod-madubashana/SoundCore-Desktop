//! Shared spring-based UI transitions for toggle switches.
//!
//! GPUI springs keep their position/velocity keyed by element id, so a retargeted
//! spring animates smoothly from wherever it currently is. `key` must be stable
//! and unique per toggle.

use gpui::{
    AnimationExt as _, AnimationPhase, BoxShadow, Hsla, IntoElement, ParentElement, Pixels,
    SharedString, SpringAnimation, SpringConfig, Styled, div, point, px,
};

use crate::ui::theme;

/// Snappy toggle: slight overshoot, settles in ~200ms.
const TOGGLE_SPRING: SpringConfig = SpringConfig::new(260.0, 26.0, 1.0);
/// Segmented control: smooth, settles in ~200ms.
const SEGMENT_SPRING: SpringConfig = SpringConfig::new(240.0, 25.0, 1.0);

/// Animated on/off switch. `key` must be stable and unique per toggle so GPUI
/// preserves spring state across renders and retargets it when `on` flips.
pub(crate) fn toggle_switch(on: bool, key: impl Into<SharedString>) -> impl IntoElement {
    div()
        .w(px(32.))
        .h(px(18.))
        .rounded_full()
        .flex_shrink_0()
        .relative()
        .with_spring(
            key.into(),
            SpringAnimation::new(TOGGLE_SPRING).to(AnimationPhase(if on { 1.0 } else { 0.0 })),
            |this, phase| {
                let p = phase.clamp(0.0..=1.0);
                let left: Pixels = p.interpolate(px(2.0), px(16.0));
                let track: Hsla = p.interpolate(theme::toggle_off(), theme::brand());
                this.bg(track).child(
                    div()
                        .absolute()
                        .top(px(2.))
                        .left(left)
                        .size(px(14.))
                        .rounded_full()
                        .bg(theme::toggle_thumb()),
                )
            },
        )
}

const OFF_SEGMENT_TEXT: Hsla = gpui::hsla(0.0, 0.0, 1.0, 0.55);

/// Animate only the text color of a segmented button; the pill behind it is
/// what morphs between positions. `key` must be stable and unique.
pub(crate) fn segment_text<E: IntoElement + Styled + 'static>(
    el: E,
    on: bool,
    key: impl Into<SharedString>,
) -> impl IntoElement {
    el.with_spring(
        key.into(),
        SpringAnimation::new(SEGMENT_SPRING).to(AnimationPhase(if on { 1.0 } else { 0.0 })),
        move |this, phase| {
            let p = phase.clamp(0.0..=1.0);
            let text: Hsla = p.interpolate(OFF_SEGMENT_TEXT, theme::brand_foreground());
            this.text_color(text)
        },
    )
}

/// The morphing navigation pill, mirroring the frontend `ActivePill`: a single
/// filled element that slides to the active segment. Its leading spacer and the
/// pill use two springs with different stiffness so the leading edge moves ahead
/// (power3.out) while the trailing edge settles later (power3.inOut), producing
/// the stretch-and-settle rhythm. Render this as an absolute inset-0 layer behind
/// the transparent segment buttons.
pub(crate) fn morphing_pill(
    active_index: usize,
    count: usize,
    key: &'static str,
) -> impl IntoElement {
    const LEAD: SpringConfig = SpringConfig::new(320.0, 28.0, 1.0);
    const TRAIL: SpringConfig = SpringConfig::new(170.0, 22.0, 1.0);
    let target = active_index.min(count.saturating_sub(1)) as f32;
    let trailing_target = (count.saturating_sub(1)) as f32 - target;
    let lead_key = SharedString::from(format!("pill-lead-{key}"));
    let space_key = SharedString::from(format!("pill-space-{key}"));
    let grow_key = SharedString::from(format!("pill-grow-{key}"));

    div()
        .absolute()
        .inset_0()
        .flex()
        .child(div().h_full().with_spring(
            lead_key,
            SpringAnimation::new(LEAD).to(target),
            |this, phase| this.flex_grow(phase.max(0.0)),
        ))
        .child(
            div()
                .h_full()
                .rounded_xl()
                .bg(theme::brand())
                .shadow(vec![BoxShadow {
                    color: gpui::rgb(0x6d6cff).into(),
                    offset: point(px(0.), px(3.)),
                    blur_radius: px(5.),
                    spread_radius: px(0.),
                    inset: false,
                }])
                .with_spring(
                    grow_key,
                    SpringAnimation::new(TRAIL).to(target),
                    move |this, phase| {
                        // Width stretches while in flight, settles to one segment.
                        let stretch = (phase - target).abs().clamp(0.0, 0.5);
                        this.flex_grow(1.0 + stretch)
                    },
                ),
        )
        .child(div().h_full().with_spring(
            space_key,
            SpringAnimation::new(LEAD).to(trailing_target),
            |this, phase| this.flex_grow(phase.max(0.0)),
        ))
}
