//! Window shell: show/hide/position/topmost/rounded-corners/backdrop via
//! raw window handles. GPUI has no public hide/set_position API, so we drop
//! to the platform APIs the same way the old Tauri shell did.

use std::time::Duration;

use gpui::{App, Window, WindowHandle};

/// Main popup size in logical pixels (400x560).
pub const MAIN_W: f32 = 400.0;
pub const MAIN_H: f32 = 560.0;

/// Margin from the right edge of the work area (logical px).
pub const MARGIN_X: f32 = 20.0;
/// Gap above the taskbar reserve (logical px).
pub const MARGIN_Y: f32 = 8.0;

/// Logical corner radius matching UI `rounded_3xl` (1.5rem × rem_base 13.05).
pub const CORNER_RADIUS: f32 = 1.5 * 14.5 * 0.9;

// ---- raw HWND helpers (Windows) ----

#[cfg(windows)]
fn hwnd_of(window: &Window) -> Option<windows::Win32::Foundation::HWND> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    // Inherent Window::window_handle() returns AnyWindowHandle; use the trait method.
    let handle = HasWindowHandle::window_handle(window).ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(h) => {
            Some(windows::Win32::Foundation::HWND(h.hwnd.get() as *mut _))
        }
        _ => None,
    }
}

#[cfg(windows)]
fn set_pos_raw(hwnd: windows::Win32::Foundation::HWND, x: i32, y: i32) {
    use windows::Win32::UI::WindowsAndMessaging::{
        HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
    };
    unsafe {
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER,
        );
    }
}

#[cfg(windows)]
pub fn show(window: &Window) {
    use windows::Win32::UI::WindowsAndMessaging::{SW_SHOWNOACTIVATE, ShowWindow};
    if let Some(hwnd) = hwnd_of(window) {
        unsafe {
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
    }
}

/// Whether the OS window is currently visible (matches Tauri `is_visible`).
#[cfg(windows)]
pub fn is_visible(window: &Window) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::IsWindowVisible;
    match hwnd_of(window) {
        Some(hwnd) => unsafe { IsWindowVisible(hwnd).as_bool() },
        None => false,
    }
}

/// Position the window at the bottom-right of its monitor's work area
/// (taskbar excluded). Returns the chosen (x, y) in physical pixels.
#[cfg(windows)]
pub fn position_bottom_right(window: &Window) -> Option<(i32, i32)> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };

    let hwnd = hwnd_of(window)?;
    let scale = window.scale_factor() as f64;
    let w = (MAIN_W as f64 * scale) as i32;
    let h = (MAIN_H as f64 * scale) as i32;
    let margin_x = (MARGIN_X as f64 * scale) as i32;
    let margin_y = (MARGIN_Y as f64 * scale) as i32;

    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO::default();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut mi).as_bool() {
            let work: RECT = mi.rcWork;
            let x = work.right - w - margin_x;
            let y = work.bottom - h - margin_y;
            let max_x = (work.right - w).max(work.left);
            let max_y = (work.bottom - h).max(work.top);
            let pos = (x.clamp(work.left, max_x), y.clamp(work.top, max_y));
            set_pos_raw(hwnd, pos.0, pos.1);
            return Some(pos);
        }
    }
    None
}

/// Monitor height in physical pixels (for slide-off animation).
#[cfg(windows)]
pub fn monitor_height(window: &Window) -> i32 {
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };

    if let Some(hwnd) = hwnd_of(window) {
        unsafe {
            let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
            let mut mi = MONITORINFO::default();
            mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(monitor, &mut mi).as_bool() {
                return mi.rcMonitor.bottom - mi.rcMonitor.top;
            }
        }
    }
    1080
}

#[cfg(windows)]
pub fn set_pos(window: &Window, x: i32, y: i32) {
    if let Some(hwnd) = hwnd_of(window) {
        set_pos_raw(hwnd, x, y);
    }
}

#[cfg(windows)]
pub fn outer_pos(window: &Window) -> (i32, i32) {
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;
    if let Some(hwnd) = hwnd_of(window) {
        unsafe {
            let mut rect = windows::Win32::Foundation::RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                return (rect.left, rect.top);
            }
        }
    }
    (0, 0)
}

/// Rounded corners: DWM preference + GDI region from the real client rect.
/// DWM alone does not clip the acrylic composition, so unpainted corners
/// show the transparent clear-color as white. Region must be applied before
/// acrylic and use GetClientRect (not MAIN_W×scale) or edges tear.
#[cfg(windows)]
pub fn apply_rounded_corners(window: &Window) {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Dwm::{
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
    };
    use windows::Win32::Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn};
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    let Some(hwnd) = hwnd_of(window) else { return };

    unsafe {
        let pref = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &pref as *const _ as *const _,
            std::mem::size_of_val(&pref) as u32,
        );

        let mut rc = RECT::default();
        if GetClientRect(hwnd, &mut rc).is_ok() {
            let w = rc.right - rc.left;
            let h = rc.bottom - rc.top;
            if w > 0 && h > 0 {
                let scale = window.scale_factor() as f64;
                // Region radius ≥ paint radius so the clip ⊆ painted area
                // (no clear-color gap). CreateRoundRectRgn ellipse = 2×r.
                let r = ((CORNER_RADIUS as f64 * scale) as i32 + 2).max(2);
                let region = CreateRoundRectRgn(0, 0, w + 1, h + 1, r * 2, r * 2);
                let _ = SetWindowRgn(hwnd, Some(region), true);
            }
        }
    }
    tracing::debug!("rounded corners applied (dwm + client region)");
}

/// Dark acrylic tint matching the old window_effects::apply_backdrop.
/// Call only while the window is visible — DWM ignores backdrop on hidden windows.
#[cfg(windows)]
pub fn apply_backdrop(window: &Window) {
    if !is_visible(window) {
        tracing::debug!("skip acrylic: window not visible yet");
        return;
    }
    match window_vibrancy::apply_acrylic(window, Some((18, 18, 20, 140))) {
        Ok(()) => tracing::debug!("acrylic backdrop applied"),
        Err(err) => tracing::warn!("acrylic backdrop failed: {err}"),
    }
}

#[cfg(windows)]
pub fn apply_topmost(window: &Window) {
    use windows::Win32::UI::WindowsAndMessaging::{
        HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos,
    };
    if let Some(hwnd) = hwnd_of(window) {
        unsafe {
            // SWP_NOMOVE is required: without it x=0,y=0 relocates the window.
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOSIZE | SWP_NOMOVE | SWP_NOACTIVATE | SWP_NOZORDER,
            );
        }
    }
}

// ---- Linux stubs (Wayland has no public hide/position in gpui 0.2.2) ----

#[cfg(not(windows))]
pub fn show(_window: &Window) {}

#[cfg(not(windows))]
pub fn is_visible(_window: &Window) -> bool {
    false
}

#[cfg(not(windows))]
pub fn position_bottom_right(_window: &Window) -> Option<(i32, i32)> {
    None
}

#[cfg(not(windows))]
pub fn monitor_height(_window: &Window) -> i32 {
    1080
}

#[cfg(not(windows))]
pub fn set_pos(_window: &Window, _x: i32, _y: i32) {}

#[cfg(not(windows))]
pub fn outer_pos(_window: &Window) -> (i32, i32) {
    (0, 0)
}

#[cfg(not(windows))]
pub fn apply_rounded_corners(_window: &Window) {}

#[cfg(not(windows))]
pub fn apply_backdrop(_window: &Window) {}

#[cfg(not(windows))]
pub fn apply_topmost(_window: &Window) {}

// ---- high-level animated show/hide ----

/// What the animation worker needs, snapshotted on the main thread.
#[derive(Clone, Copy)]
pub struct Snap {
    pub x: i32,
    pub y: i32,
    pub hwnd: isize,
    pub scale: f32,
    pub mon_h: i32,
}

/// Bottom-right X for a snapshotted HWND (physical px), without moving it.
#[cfg(windows)]
pub fn position_bottom_right_from_snap(snap: &Snap) -> Option<(i32, i32)> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    if snap.hwnd == 0 {
        return None;
    }
    unsafe {
        let hwnd = windows::Win32::Foundation::HWND(snap.hwnd as *mut _);
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO::default();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(monitor, &mut mi).as_bool() {
            let work: RECT = mi.rcWork;
            let w = (MAIN_W as f64 * snap.scale as f64) as i32;
            let h = (MAIN_H as f64 * snap.scale as f64) as i32;
            let margin_x = (MARGIN_X as f64 * snap.scale as f64) as i32;
            let margin_y = (MARGIN_Y as f64 * snap.scale as f64) as i32;
            let x = work.right - w - margin_x;
            let y = work.bottom - h - margin_y;
            let max_x = (work.right - w).max(work.left);
            let max_y = (work.bottom - h).max(work.top);
            return Some((x.clamp(work.left, max_x), y.clamp(work.top, max_y)));
        }
    }
    None
}

#[cfg(not(windows))]
pub fn position_bottom_right_from_snap(_snap: &Snap) -> Option<(i32, i32)> {
    None
}

/// Snapshot platform state for an animation. Must run on the GPUI main thread.
pub fn snapshot<V: gpui::Render + 'static>(
    handle: WindowHandle<V>,
    cx: &mut App,
) -> Option<Snap> {
    handle
        .update(cx, |_, window, _| {
            #[cfg(windows)]
            {
                let hwnd = hwnd_of(window).map(|h| h.0 as isize).unwrap_or(0);
                if hwnd == 0 {
                    return None;
                }
                Some(Snap {
                    x: outer_pos(window).0,
                    y: outer_pos(window).1,
                    hwnd,
                    scale: window.scale_factor(),
                    mon_h: monitor_height(window),
                })
            }
            #[cfg(not(windows))]
            {
                let _ = window;
                None
            }
        })
        .ok()
        .flatten()
}

#[derive(Clone, Copy)]
pub enum Anim {
    Show,
    Hide,
}

/// Run the show/hide animation off the main thread from a snapshot.
/// After Show completes, call [`finish_show`] on the main thread.
pub fn animate(snap: Snap, mode: Anim) {
    let _ = std::thread::Builder::new()
        .name("shell-window".into())
        .spawn(move || match mode {
            Anim::Hide => snap.hide_blocking(),
            Anim::Show => snap.show_blocking(),
        });
}

/// Post-show polish on the main thread. Region first (clips clear-color),
/// then acrylic, then topmost/position — each once, to avoid RefCell re-entry.
pub fn finish_show<V: gpui::Render + 'static>(handle: WindowHandle<V>, cx: &mut App) {
    let _ = handle.update(cx, |_, window, _| {
        apply_rounded_corners(window);
        apply_backdrop(window);
        apply_topmost(window);
        position_bottom_right(window);
    });
}

// ---- Snap platform ops ----

#[cfg(windows)]
impl Snap {
    fn set_pos(&self, x: i32, y: i32) {
        use windows::Win32::Foundation::HWND;
        if self.hwnd == 0 {
            return;
        }
        set_pos_raw(HWND(self.hwnd as *mut _), x, y);
    }

    fn show(&self) {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{SW_SHOWNOACTIVATE, ShowWindow};
        if self.hwnd == 0 {
            return;
        }
        unsafe {
            let _ = ShowWindow(HWND(self.hwnd as *mut _), SW_SHOWNOACTIVATE);
        }
    }

    fn hide(&self) {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, ShowWindow};
        if self.hwnd == 0 {
            return;
        }
        unsafe {
            let _ = ShowWindow(HWND(self.hwnd as *mut _), SW_HIDE);
        }
    }

    fn slide(&self, from_y: i32, to_y: i32, duration_ms: u64) {
        let steps = 12;
        let step_delay = duration_ms / steps.max(1);
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let t = 1.0 - (1.0 - t).powi(3);
            let y = from_y as f64 + (to_y as f64 - from_y as f64) * t;
            self.set_pos(self.x, y as i32);
            std::thread::sleep(Duration::from_millis(step_delay));
        }
    }

    fn hide_blocking(self) {
        self.slide(self.y, self.mon_h + 100, 350);
        self.hide();
    }

    pub fn target_y(&self) -> i32 {
        use windows::Win32::Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        };
        if self.hwnd != 0 {
            unsafe {
                let hwnd = windows::Win32::Foundation::HWND(self.hwnd as *mut _);
                let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                let mut mi = MONITORINFO::default();
                mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                if GetMonitorInfoW(monitor, &mut mi).as_bool() {
                    let work = mi.rcWork;
                    let h = (MAIN_H as f64 * self.scale as f64) as i32;
                    let margin_y = (MARGIN_Y as f64 * self.scale as f64) as i32;
                    let y = work.bottom - h - margin_y;
                    return y.clamp(work.top, (work.bottom - h).max(work.top));
                }
            }
        }
        self.mon_h - (MAIN_H * self.scale) as i32 - 48
    }

    /// Position below target, show, slide up. Backdrop is applied by
    /// `finish_show` on the main thread after this returns.
    fn show_blocking(self) {
        let target_y = self.target_y();
        let start_y = target_y + 80;
        self.set_pos(self.x, start_y);
        self.show();
        self.slide(start_y, target_y, 200);
    }
}

#[cfg(not(windows))]
impl Snap {
    pub fn target_y(&self) -> i32 {
        self.mon_h - (MAIN_H * self.scale) as i32 - 48
    }

    fn hide_blocking(self) {}
    fn show_blocking(self) {}
}
