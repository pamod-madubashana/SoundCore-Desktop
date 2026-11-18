//! GPUI root view — main 400×560 popup.
//!
//! Screen routing:
//!   connected device → Control; devices known → MyDevices; else Searching.

pub mod background;
pub mod data;
pub mod scale;
pub mod screens;
pub mod theme;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use gpui::{
    App, Context, IntoElement, ParentElement, Render, Styled, Task, Window, div, prelude::*,
};
use self::scale::px;

use crate::{
    DeviceStateDto, EqPresetDto, ScanDto, UpdateInfo, UpdateProgress, get_scan, get_states,
    get_update_progress, get_version, hide_window_now, show_notification, state,
};

pub struct RootView {
    pub focus_handle: gpui::FocusHandle,
    pub(crate) devices: Vec<DeviceStateDto>,
    prev_connected: Option<HashSet<String>>,
    pub(crate) update_info: Option<UpdateInfo>,
    pub(crate) update_progress: Option<UpdateProgress>,
    pub(crate) update_error: Option<String>,
    /// null until first BT reply (fail open / "on")
    pub(crate) bt_on: Option<bool>,
    pub(crate) scanning: bool,
    pub(crate) version: String,
    pub(crate) eq_presets: Vec<EqPresetDto>,
    pub show_sound_effects: bool,
    /// Second-level Custom EQ editor (React `showEqualizer`).
    pub(crate) show_equalizer: bool,
    /// 0..11 frame for the searching spinner
    pub(crate) spinner_tick: u8,
    /// Optimistic overrides so taps survive the 900ms poll.
    pub(crate) ambient_local: Option<(String, String)>,
    pub(crate) sub_local: Option<(String, String, String)>,
    pub(crate) sub_local_at: Option<std::time::Instant>,
    pub(crate) toggle_local: HashMap<String, bool>,
    /// SE section pending override: (section, Instant), 2000ms grace.
    pub(crate) se_local: Option<(String, std::time::Instant)>,
    /// Spatial mode optimistic: (mac, mode, Instant), 500ms grace.
    pub(crate) spatial_local: Option<(String, String, std::time::Instant)>,
    /// Last built-in preset id seen, so re-selecting Default restores it.
    pub(crate) last_preset_id: Option<String>,
    _states_task: Task<()>,
    _bt_task: Task<()>,
    _update_task: Task<()>,
}

impl RootView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        // 900ms device-state poll (cheap mutex reads).
        let states_task: Task<()> = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(900))
                    .await;
                let next = get_states(state());
                this.update(&mut *cx, |view, cx| {
                    view.apply_states(next, cx);
                })
                .ok();
            }
        });

        // 2000ms BT + scan poll — get_bluetooth_status blocks up to 5s, so hop
        // to a dedicated thread and only touch the view with the reply.
        let bt_task: Task<()> = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(2000))
                    .await;
                let (bt, scan) = std::thread::spawn(|| {
                    let st = state();
                    let bt = crate::get_bluetooth_status(st);
                    let scan = get_scan(st);
                    (bt, scan)
                })
                .join()
                .unwrap_or((
                    true,
                    ScanDto {
                        scanning: false,
                        results: vec![],
                    },
                ));
                this.update(&mut *cx, |view, cx| {
                    view.bt_on = Some(bt);
                    view.scanning = scan.scanning;
                    cx.notify();
                })
                .ok();
            }
        });

        // One-shot update check on mount (blocks; run off-main).
        let update_task: Task<()> = cx.spawn(async move |this, cx| {
            let info = std::thread::spawn(|| match crate::check_update(state()) {
                Ok(info) => Some(Ok(info)),
                Err(e) => {
                    if e.contains("up to date") || e.contains("No update available") {
                        None
                    } else {
                        Some(Err(e))
                    }
                }
            })
            .join()
            .unwrap_or(None);
            match info {
                Some(Ok(info)) => {
                    this.update(&mut *cx, |view, cx| {
                        view.update_info = Some(info);
                        cx.notify();
                    })
                    .ok();
                }
                Some(Err(e)) => {
                    this.update(&mut *cx, |view, cx| {
                        view.update_error = Some(e);
                        cx.notify();
                    })
                    .ok();
                }
                None => {}
            }
            // Progress is polled lightly while an update may be in flight.
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;
                let prog = get_update_progress(state());
                let has_state = this
                    .update(&mut *cx, |view, cx| {
                        if view.update_progress != prog {
                            view.update_progress = prog.clone();
                            cx.notify();
                        }
                        view.update_progress.is_some() || view.update_info.is_some()
                    })
                    .unwrap_or(false);
                if !has_state {
                    // Stay quiet until an update is actually offered.
                    this.update(&mut *cx, |view, cx| {
                        if view.update_progress.is_some() {
                            view.update_progress = None;
                            cx.notify();
                        }
                    })
                    .ok();
                }
            }
        });

        let version = get_version();

        Self {
            focus_handle,
            devices: Vec::new(),
            prev_connected: None,
            update_info: None,
            update_progress: None,
            update_error: None,
            bt_on: None,
            scanning: false,
            version,
            eq_presets: Vec::new(),
        show_sound_effects: false,
        show_equalizer: false,
        spinner_tick: 0,
        ambient_local: None,
        sub_local: None,
        sub_local_at: None,
        toggle_local: HashMap::new(),
        se_local: None,
        spatial_local: None,
        last_preset_id: None,
            _states_task: states_task,
            _bt_task: bt_task,
            _update_task: update_task,
        }
    }

    fn apply_states(&mut self, next: Vec<DeviceStateDto>, cx: &mut Context<Self>) {
        if let Some(prev) = &self.prev_connected {
            for d in &next {
                let was = prev.contains(&d.mac_address);
                let name = data::display_device_name(d);
                if d.connected && !was {
                    let _ = show_notification(
                        state(),
                        name,
                        d.image.clone(),
                        "Connected".into(),
                        d.battery_left,
                        d.battery_right,
                        d.battery_combined,
                        d.battery_case,
                    );
                } else if !d.connected && was {
                    let _ = show_notification(
                        state(),
                        name,
                        d.image.clone(),
                        "Disconnected".into(),
                        None,
                        None,
                        None,
                        None,
                    );
                }
            }
        }
        let now: HashSet<String> = next
            .iter()
            .filter(|d| d.connected)
            .map(|d| d.mac_address.clone())
            .collect();
        self.prev_connected = Some(now);

        let model_changed = self
            .devices
            .iter()
            .find(|d| d.connected)
            .map(|d| d.model.clone())
            != next.iter().find(|d| d.connected).map(|d| d.model.clone());
        self.devices = next;
        if model_changed {
            self.reload_eq_presets();
        }
        // Drop ambient override once the poll reports the same value.
        if let Some((mac, val)) = self.ambient_local.clone() {
            let done = self
                .settings_for(&mac)
                .get("ambientSoundMode")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_str())
                == Some(val.as_str());
            if done {
                self.ambient_local = None;
            }
        }
        // Remember last built-in preset; clear SE pending once device agrees.
        if let Some(id) = self
            .devices
            .iter()
            .find(|d| d.connected)
            .and_then(|d| data::device_preset_id(&data::settings_map(d)))
        {
            self.last_preset_id = Some(id);
        }
        if let Some((section, at)) = self.se_local.clone() {
            let device_section = self
                .active_device()
                .map(|d| data::derive_active_section(&data::settings_map(d)))
                .unwrap_or_default();
            if device_section == section
                || at.elapsed() >= std::time::Duration::from_millis(2000)
            {
                self.se_local = None;
            }
        }
        if let Some(at) = self.sub_local_at {
            if at.elapsed() >= std::time::Duration::from_millis(2000) {
                self.sub_local = None;
                self.sub_local_at = None;
            }
        }
        if let Some((_, _, at)) = self.spatial_local.clone() {
            if at.elapsed() >= std::time::Duration::from_millis(500) {
                self.spatial_local = None;
            }
        }
        cx.notify();
    }

    /// SE active section: optimistic pending wins for 2000ms.
    pub(crate) fn se_active(&self) -> String {
        if let Some((section, at)) = &self.se_local {
            if at.elapsed() < std::time::Duration::from_millis(2000) {
                return section.clone();
            }
        }
        self.active_device()
            .map(|d| data::derive_active_section(&data::settings_map(d)).to_string())
            .unwrap_or_default()
    }

    /// Spatial mode with 500ms optimistic grace for this MAC.
    pub(crate) fn spatial_mode(&self, mac: &str, polled: Option<String>) -> String {
        if let Some((m, mode, at)) = &self.spatial_local {
            if m == mac && at.elapsed() < std::time::Duration::from_millis(500) {
                return mode.clone();
            }
        }
        polled.unwrap_or_else(|| "Music".into())
    }

    /// Select an SE section (mirrors React `selectSection`).
    pub(crate) fn select_section(&mut self, section: &str, cx: &mut Context<Self>) {
        let active = self.se_active();
        if active == section {
            if section == data::SECTION_CUSTOM {
                self.show_equalizer = true;
                cx.notify();
            }
            return;
        }
        let Some(mac) = self.active_device().map(|d| d.mac_address.clone()) else {
            return;
        };
        if section == data::SECTION_SPATIAL {
            self.send_setting(&mac, "spatialAudio", "true");
        } else if section == data::SECTION_DEFAULT {
            let s = self.settings_for(&mac);
            let id = self
                .last_preset_id
                .clone()
                .or_else(|| data::preset_options(&s).into_iter().next().map(|(id, _)| id));
            let Some(id) = id else { return };
            self.send_setting(&mac, "presetEqualizerProfile", &id);
        } else if section == data::SECTION_CUSTOM {
            let s = self.settings_for(&mac);
            let Some(vol) = s.get("volumeAdjustments") else {
                return;
            };
            if vol.get("readOnly").and_then(|v| v.as_bool()) == Some(true) {
                return;
            }
            let bands: Vec<String> = vol
                .get("value")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().map(|x| x.to_string()).collect())
                .unwrap_or_default();
            if bands.is_empty() {
                return;
            }
            self.send_setting(&mac, "volumeAdjustments", &bands.join(","));
        }
        self.se_local = Some((section.to_string(), std::time::Instant::now()));
        cx.notify();
    }

    /// Set spatialAudioMode with 500ms optimistic grace.
    pub(crate) fn set_spatial_mode(&mut self, mode: &str, cx: &mut Context<Self>) {
        let Some(mac) = self.active_device().map(|d| d.mac_address.clone()) else {
            return;
        };
        self.send_setting(&mac, "spatialAudioMode", mode);
        self.spatial_local = Some((mac, mode.to_string(), std::time::Instant::now()));
        cx.notify();
    }

    fn reload_eq_presets(&mut self) {
        let model = self
            .devices
            .iter()
            .find(|d| d.connected)
            .map(|d| d.model.clone());
        let Some(model) = model else {
            self.eq_presets = Vec::new();
            return;
        };
        self.eq_presets = crate::list_eq_presets(state(), &model);
    }

    pub fn active_device(&self) -> Option<&DeviceStateDto> {
        self.devices.iter().find(|d| d.connected)
    }

    /// Settings map for a device by MAC (empty if unknown).
    pub fn cached_settings(&self, mac: &str) -> data::SettingsMap {
        self.settings_for(mac)
    }

    fn settings_for(&self, mac: &str) -> data::SettingsMap {
        self.devices
            .iter()
            .find(|d| d.mac_address == mac)
            .map(data::settings_map)
            .unwrap_or_default()
    }

    /// Effective toggle value (optimistic override wins for 500ms).
    pub fn effective_toggle(&self, id: &str, polled: bool) -> bool {
        self.toggle_local.get(id).copied().unwrap_or(polled)
    }

    pub fn dismiss_update(&mut self, cx: &mut Context<Self>) {
        self.update_info = None;
        self.update_progress = None;
        self.update_error = None;
        cx.notify();
    }

    pub fn start_update(&mut self, cx: &mut Context<Self>) {
        self.update_error = None;
        let result = std::thread::spawn(|| crate::start_update(state()))
            .join()
            .unwrap_or_else(|_| Err("update thread panicked".into()));
        if let Err(e) = result {
            self.update_error = Some(e.chars().take(80).collect());
            self.update_progress = None;
            cx.notify();
        }
    }

    pub fn hide(&self, cx: &mut App) {
        hide_window_now(cx);
    }

    /// Force a state + Bluetooth refresh (footer Refresh button).
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        let next = get_states(state());
        self.apply_states(next, cx);
        cx.spawn(async move |this, cx| {
            let (bt, scan) = std::thread::spawn(|| {
                let st = state();
                let bt = crate::get_bluetooth_status(st);
                let scan = get_scan(st);
                (bt, scan)
            })
            .join()
            .unwrap_or((
                true,
                ScanDto {
                    scanning: false,
                    results: vec![],
                },
            ));
            this.update(&mut *cx, |view, cx| {
                view.bt_on = Some(bt);
                view.scanning = scan.scanning;
                if !scan.scanning
                    && let Some(model) = view.devices.first().map(|d| d.model.clone())
                {
                    let _ = crate::scan(state(), model);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    pub fn send_setting(&self, mac: &str, id: &str, raw: &str) {
        let _ = crate::set_setting(state(), mac.to_string(), id.to_string(), raw.to_string());
    }

    pub fn load_device_image(&self, d: &DeviceStateDto) -> Option<std::sync::Arc<gpui::Image>> {
        let url = d.image.as_deref()?;
        let key = url.to_owned();
        {
            let cache = device_image_cache().lock().unwrap();
            if let Some(hit) = cache.get(&key) {
                return Some(hit.clone());
            }
        }
        let b64 = url.strip_prefix("data:image/png;base64,")?;
        use base64::Engine as _;
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
        // CPU-downscale near display size (68px tile / 66px control header ×
        // zoom 0.9 × ~2 DPI ≈ 120). DX samples bilinear without mips, so a
        // large minification aliases (R60i speculars looked pixelated).
        let bytes = downscale_png(bytes, 120);
        let img = std::sync::Arc::new(gpui::Image::from_bytes(
            gpui::ImageFormat::Png,
            bytes,
        ));
        device_image_cache()
            .lock()
            .unwrap()
            .insert(key, img.clone());
        Some(img)
    }
}

/// Keyed by the source data-URL; holds the downscaled `gpui::Image`.
fn device_image_cache()
-> &'static std::sync::Mutex<std::collections::HashMap<String, std::sync::Arc<gpui::Image>>> {
    static CACHE: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, std::sync::Arc<gpui::Image>>>,
    > = std::sync::OnceLock::new();
    CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

/// Lanczos downscale a PNG; returns original bytes if already small or on error.
fn downscale_png(bytes: Vec<u8>, max: u32) -> Vec<u8> {
    use image::GenericImageView as _;
    let Ok(img) = image::load_from_memory(&bytes) else {
        return bytes;
    };
    let (w, h) = img.dimensions();
    if w <= max && h <= max {
        return bytes;
    }
    let img = img.resize(max, max, image::imageops::FilterType::Lanczos3);
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    if img
        .write_to(&mut cursor, image::ImageFormat::Png)
        .is_err()
    {
        return bytes;
    }
    out
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Spinner only ticks while Searching is visible.
        if self.devices.is_empty() {
            let tick = self.spinner_tick;
            let spinner_task: Task<()> = cx.spawn(async move |this, cx| {
                cx.background_executor().timer(Duration::from_millis(80)).await;
                this.update(&mut *cx, |view, cx| {
                    if view.devices.is_empty() {
                        view.spinner_tick = tick.wrapping_add(1);
                        cx.notify();
                    }
                })
                .ok();
            });
            spinner_task.detach();
        }

        let active = self.active_device().cloned();
        let show_se = self.show_sound_effects;

        // Root IS the glass: baked PNG (gradient+glows+sheen+stroke) masked
        // to rx=28. HWND stays a full transparent host; alpha defines shape.
        div()
            .track_focus(&self.focus_handle)
            .size_full()
            .relative()
            .flex()
            .flex_col()
            .overflow_hidden()
            .text_color(theme::foreground())
            .text_size(px(14.))
            .font_family("Segoe UI")
            // Dark diagonal base + orange/blue radial glows + panel sheen,
            // transparent outside rx=28 (matches *_window.svg).
            .child(
                div()
                    .absolute()
                    .left(px(0.))
                    .top(px(0.))
                    .right(px(0.))
                    .bottom(px(0.))
                    .child(
                        gpui::img(background::image().clone())
                            .size_full()
                            .object_fit(gpui::ObjectFit::Fill),
                    ),
            )
            .child(
                div()
                    .size_full()
                    .relative()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .when(active.is_some() && !show_se, |el| {
                        el.child(screens::control::render(
                            self,
                            active.as_ref().unwrap(),
                            window,
                            cx,
                        ))
                    })
                    .when(active.is_none() && !self.devices.is_empty(), |el| {
                        el.child(screens::devices::render(self, window, cx))
                    })
                    .when(active.is_none() && self.devices.is_empty(), |el| {
                        el.child(screens::searching::render(self, window, cx))
                    })
                    .when(active.is_some() && show_se, |el| {
                        el.child(screens::sound_effects::render(
                            self,
                            active.as_ref().unwrap(),
                            window,
                            cx,
                        ))
                    }),
            )
    }
}
