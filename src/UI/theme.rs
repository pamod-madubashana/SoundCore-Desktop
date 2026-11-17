//! Color tokens (Fluent dark palette).
//! The popup sits on acrylic; surfaces are white overlays, not solid fills.

use gpui::{Hsla, rgb, rgba};

/// Body / primary text — oklch(0.97 0.003 260)
pub fn foreground() -> Hsla {
    rgb(0xf5f6f8).into()
}

/// Secondary text — oklch(0.72 0.008 260)
pub fn muted() -> Hsla {
    rgb(0xa8abb5).into()
}

/// Muted text at 60% (footer / progress labels)
pub fn muted_dim() -> Hsla {
    rgba(0xa8abb599).into()
}

/// Accent brand — target UI-Target gradient start `#5b8cff`
pub fn brand() -> Hsla {
    rgb(0x5b8cff).into()
}

/// Accent brand end — `#8b5cf6` (gradient stop; solid fills use brand())
pub fn brand_end() -> Hsla {
    rgb(0x8b5cf6).into()
}

/// Secondary accent / battery label — `#9db8ff`
pub fn brand_secondary() -> Hsla {
    rgb(0x9db8ff).into()
}

/// Text on brand fills
pub fn brand_foreground() -> Hsla {
    rgb(0xffffff).into()
}

/// Brand at 10% — icon tile backgrounds
pub fn brand_10() -> Hsla {
    rgba(0x5b8cff1a).into()
}

/// Brand at 15% — MyDevices header tile
pub fn brand_15() -> Hsla {
    rgba(0x5b8cff26).into()
}

/// Brand at 40% — header tile ring
pub fn brand_40() -> Hsla {
    rgba(0x5b8cff66).into()
}

/// Brand at 20% — hover on update pill
pub fn brand_20() -> Hsla {
    rgba(0x5b8cff33).into()
}

/// Success / connected — oklch(0.78 0.16 160)
pub fn success() -> Hsla {
    rgb(0x4ade80).into()
}

/// Success at 25% — status ring pulse
pub fn success_25() -> Hsla {
    rgba(0x4ade8040).into()
}

/// Success at 14% — connected pill bg
pub fn success_14() -> Hsla {
    rgba(0x4ade8024).into()
}

/// Success at 40% — connected pill ring
pub fn success_40() -> Hsla {
    rgba(0x4ade8066).into()
}

/// Bright green pill text
pub fn success_text() -> Hsla {
    rgb(0x86efac).into()
}

/// Error / disconnected
pub fn error() -> Hsla {
    rgb(0xff6b6b).into()
}

pub fn error_25() -> Hsla {
    rgba(0xff6b6b40).into()
}

pub fn error_14() -> Hsla {
    rgba(0xff6b6b24).into()
}

pub fn error_40() -> Hsla {
    rgba(0xff6b6b66).into()
}

pub fn error_text() -> Hsla {
    rgb(0xff9a9a).into()
}

/// Error copy (update failure) — oklch(0.65 0.22 25)
pub fn error_copy() -> Hsla {
    rgb(0xe85d5d).into()
}

/// Amber battery mid — `#fbbf24`
pub fn warning() -> Hsla {
    rgb(0xfbbf24).into()
}

/// Surface — white 10% (cards over acrylic)
pub fn surface() -> Hsla {
    rgba(0xffffff1a).into()
}

/// Surface elevated — white 14%
pub fn surface_elevated() -> Hsla {
    rgba(0xffffff24).into()
}

/// Border subtle — white 12%
pub fn border_subtle() -> Hsla {
    rgba(0xffffff1f).into()
}

/// Border medium — white 16% (header rule)
pub fn border_medium() -> Hsla {
    rgba(0xffffff29).into()
}

/// Border strong — white 22%
pub fn border_strong() -> Hsla {
    rgba(0xffffff38).into()
}

/// Toggle / progress track off — white 14%
pub fn toggle_off() -> Hsla {
    rgba(0xffffff24).into()
}

/// Toggle thumb
pub fn toggle_thumb() -> Hsla {
    rgb(0xffffff).into()
}

/// Hover subtle — white 8%
pub fn hover_subtle() -> Hsla {
    rgba(0xffffff14).into()
}

/// Hover medium — white 12%
pub fn hover_medium() -> Hsla {
    rgba(0xffffff1f).into()
}

/// Overlay behind segmented controls — black 45%
pub fn overlay_bg() -> Hsla {
    rgba(0x00000073).into()
}

/// Popup chrome fill (window is acrylic; keep near-transparent solid)
pub fn popup_bg() -> Hsla {
    rgba(0x121214e6).into()
}

/// Spinner / accent strokes that should read as brand
pub fn spinner() -> Hsla {
    brand()
}
