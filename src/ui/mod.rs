//! GPUI root view — main 400×560 popup.
//!
//! Screen routing:
//!   connected device → Control; devices known → MyDevices; else Searching.

pub mod animation;
pub mod background;
pub mod data;
pub mod notification;
pub mod scale;
pub mod screens;
pub mod theme;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use self::scale::px;
use gpui::{
    Animation, AnimationExt as _, App, Context, IntoElement, ParentElement, Render, Styled, Task,
    Window, div, prelude::*,
};

use crate::{
    DeviceStateDto, EqPresetDto, ScanDto, UpdateInfo, UpdateProgress, get_scan, get_states,
    get_update_progress, get_version, hide_window_now, set_update_progress, show_notification,
    state,
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
    pub show_device_settings: bool,
    /// Device Information screen (opened from the device image).
    pub show_device_info: bool,
    /// Control Settings screen (button actions / gestures).
    pub show_control_settings: bool,
    /// Setting id whose button-config dropdown is expanded.
    pub button_dropdown: Option<String>,
    pub(crate) device_dropdown: Option<String>,
    /// App Settings screen (header gear button).
    pub show_settings: bool,
    /// About screen (header info button).
    pub show_about: bool,
    pub show_protocol_lab: bool,
    pub(crate) capture_frames: Vec<soundcore_protocol::analysis::CapturedFrameDto>,
    pub(crate) protocol_selected: Option<usize>,
    pub(crate) protocol_active_device: Option<(String, String, String)>,
    pub(crate) protocol_capture_progress: Option<u8>,
    pub(crate) protocol_capture_encrypting: bool,
    pub(crate) protocol_share: Option<soundcore_protocol::analysis::ReportShare>,
    pub(crate) unsupported_device: Option<(String, String, String)>,
    pub(crate) protocol_show_details: bool,
    pub(crate) protocol_share_error: Option<String>,
    pub(crate) protocol_share_notice: Option<String>,
    /// Optimistic overrides so taps survive the 900ms poll.
    pub(crate) ambient_local: Option<(String, String)>,
    pub(crate) ambient_local_at: Option<std::time::Instant>,
    pub(crate) sub_local: Option<(String, String, String)>,
    pub(crate) sub_local_at: Option<std::time::Instant>,
    pub(crate) toggle_local: HashMap<String, (bool, std::time::Instant)>,
    /// SE section pending override: (section, Instant), 2000ms grace.
    pub(crate) se_local: Option<(String, std::time::Instant)>,
    /// Spatial mode optimistic: (mac, mode, Instant), 500ms grace.
    pub(crate) spatial_local: Option<(String, String, std::time::Instant)>,
    /// Last built-in preset id seen, so re-selecting Default restores it.
    pub(crate) last_preset_id: Option<String>,
    /// Live curve while editing the custom EQ (kept optimistic across polls).
    pub(crate) eq_draft: Vec<i16>,
    /// Band index currently being dragged in the custom EQ graph.
    pub(crate) eq_drag_band: Option<usize>,
    /// Preset selected by the user, updated optimistically on click.
    pub(crate) eq_active_preset: Option<i64>,
    pub(crate) eq_hover_preset: Option<i64>,
    pub(crate) eq_delete_confirm: Option<(i64, String)>,
    nav_screen: &'static str,
    nav_prev_screen: &'static str,
    nav_id: u64,
    /// +1 push (deeper screen), -1 pop (back), 0 lateral / initial mount.
    nav_direction: i8,
    device_entry_id: u64,
    settings_cache: HashMap<String, data::SettingsMap>,
    frame_analysis_cache:
        HashMap<(String, String), Option<soundcore_protocol::analysis::FrameAnalysis>>,
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
                let capture = crate::get_capture(state());
                let progress = crate::capture_progress(state());
                let encrypt = this
                    .update(&mut *cx, |view, cx| {
                        view.apply_states(next, cx);
                        view.capture_frames = capture;
                        let mut encrypt = None;
                        if view.protocol_capture_progress.is_some() {
                            if view.protocol_capture_encrypting {
                                let current = view.protocol_capture_progress.unwrap_or(0);
                                if current < 90 {
                                    view.protocol_capture_progress = Some((current + 10).min(90));
                                }
                            } else if let Some(percent) = progress {
                                if percent >= 100 {
                                    crate::stop_capture(state());
                                    view.capture_frames = crate::get_capture(state());
                                    view.protocol_capture_progress = Some(0);
                                    view.protocol_capture_encrypting = true;
                                    view.protocol_share = None;
                                    view.protocol_share_error = None;
                                    view.protocol_share_notice = None;
                                    encrypt = view.protocol_active_device.as_ref().map(
                                        |(_, name, model)| {
                                            (
                                                view.capture_frames.clone(),
                                                name.clone(),
                                                model.clone(),
                                            )
                                        },
                                    );
                                    if encrypt.is_none() {
                                        view.protocol_capture_progress = None;
                                        view.protocol_capture_encrypting = false;
                                    }
                                } else {
                                    view.protocol_capture_progress = Some(percent);
                                }
                            } else if view.protocol_capture_progress == Some(100) {
                                view.protocol_capture_progress = None;
                            }
                        }
                        cx.notify();
                        encrypt
                    })
                    .ok()
                    .flatten();

                if let Some((frames, name, model)) = encrypt {
                    let this = this.clone();
                    cx.spawn(async move |cx| {
                        let result = std::thread::spawn(move || {
                            soundcore_protocol::analysis::build_share_bundle(&model, &name, &frames)
                        })
                        .join();
                        this.update(&mut *cx, |view, cx| {
                            match result {
                                Ok(Ok(share)) => {
                                    view.protocol_share = Some(share);
                                    view.protocol_share_error = None;
                                    view.protocol_share_notice = None;
                                }
                                Ok(Err(err)) => {
                                    view.protocol_share = None;
                                    view.protocol_share_error =
                                        Some(format!("Could not encrypt report: {err}"));
                                    view.protocol_share_notice = None;
                                }
                                Err(_) => {
                                    view.protocol_share = None;
                                    view.protocol_share_error = Some(
                                        "Could not encrypt report: worker panicked".to_string(),
                                    );
                                    view.protocol_share_notice = None;
                                }
                            }
                            view.protocol_capture_encrypting = false;
                            view.protocol_capture_progress = Some(100);
                            cx.notify();
                        })
                        .ok();
                    })
                    .detach();
                }
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
                    if view.bt_on != Some(bt) || view.scanning != scan.scanning {
                        view.bt_on = Some(bt);
                        view.scanning = scan.scanning;
                        cx.notify();
                    }
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

        let mut view = Self {
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
            show_device_settings: false,
            show_device_info: false,
            show_control_settings: false,
            button_dropdown: None,
            device_dropdown: None,
            show_settings: false,
            show_about: false,
            show_protocol_lab: false,
            capture_frames: Vec::new(),
            protocol_selected: None,
            protocol_active_device: None,
            protocol_capture_progress: None,
            protocol_capture_encrypting: false,
            protocol_share: None,
            unsupported_device: None,
            protocol_show_details: false,
            protocol_share_error: None,
            protocol_share_notice: None,
            ambient_local: None,
            ambient_local_at: None,
            sub_local: None,
            sub_local_at: None,
            toggle_local: HashMap::new(),
            eq_draft: Vec::new(),
            eq_drag_band: None,
            eq_active_preset: None,
            eq_hover_preset: None,
            eq_delete_confirm: None,
            se_local: None,
            spatial_local: None,
            last_preset_id: None,
            nav_screen: "",
            nav_prev_screen: "",
            nav_id: 0,
            nav_direction: 0,
            device_entry_id: 0,
            settings_cache: HashMap::new(),
            frame_analysis_cache: HashMap::new(),
            _states_task: states_task,
            _bt_task: bt_task,
            _update_task: update_task,
        };

        view.reload_eq_presets();
        view
    }

    fn apply_states(&mut self, next: Vec<DeviceStateDto>, cx: &mut Context<Self>) {
        if let Some(prev) = &self.prev_connected {
            let notify = crate::get_config(crate::state()).notify_connect;
            if notify {
                let next_connected: HashSet<&str> = next
                    .iter()
                    .filter(|d| d.connected)
                    .map(|d| d.mac_address.as_str())
                    .collect();
                for d in next
                    .iter()
                    .filter(|d| d.connected && !prev.contains(&d.mac_address))
                {
                    let _ = show_notification(
                        cx,
                        data::display_device_name(d),
                        d.image.clone(),
                        "Connected".into(),
                        d.battery_left,
                        d.battery_right,
                        d.battery_combined,
                        d.battery_case,
                    );
                }
                for mac in prev
                    .iter()
                    .filter(|mac| !next_connected.contains(mac.as_str()))
                {
                    let old = self.devices.iter().find(|d| &d.mac_address == mac);
                    let name = old
                        .map(data::display_device_name)
                        .unwrap_or_else(|| mac.to_string());
                    let image = old.and_then(|d| d.image.clone());
                    let _ = show_notification(
                        cx,
                        name,
                        image,
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
        self.rebuild_settings_cache();
        if model_changed {
            self.eq_draft.clear();
            self.reload_eq_presets();
        }
        // Drop ambient override once the poll reports the same value or grace expires.
        let ambient_timed_out = self
            .ambient_local_at
            .map(|at| at.elapsed() >= std::time::Duration::from_millis(2000))
            .unwrap_or(false);
        if let Some((mac, val)) = self.ambient_local.clone() {
            let done = self
                .settings_for(&mac)
                .get("ambientSoundMode")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_str())
                == Some(val.as_str());
            if done || ambient_timed_out {
                self.ambient_local = None;
                self.ambient_local_at = None;
            }
        }
        // Remember last built-in preset; clear SE pending once device agrees.
        if let Some(mac) = self.active_device().map(|d| d.mac_address.as_str()) {
            if let Some(id) = data::device_preset_id(&self.settings_for(mac)) {
                self.last_preset_id = Some(id);
            }
        }
        if let Some((section, at)) = self.se_local.clone() {
            let device_section = self
                .active_device()
                .map(|d| data::derive_active_section(&self.settings_for(&d.mac_address)))
                .unwrap_or_default();
            if device_section == section || at.elapsed() >= std::time::Duration::from_millis(2000) {
                self.se_local = None;
            }
        }
        if let Some(at) = self.sub_local_at {
            if at.elapsed() >= std::time::Duration::from_millis(2000) {
                self.sub_local = None;
                self.sub_local_at = None;
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
            .map(|d| data::derive_active_section(&self.settings_for(&d.mac_address)).to_string())
            .unwrap_or_default()
    }

    /// Spatial mode stays on the optimistic choice until the device confirms it.
    pub(crate) fn spatial_mode(&self, mac: &str, polled: Option<String>) -> String {
        if let Some((m, mode, _)) = &self.spatial_local {
            if m == mac {
                return mode.clone();
            }
        }
        polled.unwrap_or_else(|| {
            data::first_option(&self.settings_for(mac), "spatialAudioMode").unwrap_or_default()
        })
    }

    fn sync_eq_draft(&mut self) {
        let Some(mac) = self.active_device().map(|d| d.mac_address.clone()) else {
            return;
        };
        let s = self.settings_for(&mac);
        let Some(vol) = s.get("volumeAdjustments") else {
            return;
        };
        self.eq_draft = vol
            .get("value")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_i64().map(|n| n as i16))
                    .collect()
            })
            .unwrap_or_default();
        self.eq_drag_band = None;
    }

    fn eq_bands_for_apply(&self, mac: &str) -> Vec<String> {
        let polled: Vec<String> = self
            .settings_for(mac)
            .get("volumeAdjustments")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_array())
            .map(|a| a.iter().map(|x| x.to_string()).collect())
            .unwrap_or_default();

        if !self.eq_draft.is_empty() && self.eq_draft.len() == polled.len() {
            return self
                .eq_draft
                .iter()
                .map(|value| value.to_string())
                .collect();
        }

        if let Some(id) = self.eq_active_preset {
            let preset_bands = self
                .eq_presets
                .iter()
                .find(|preset| preset.id == id)
                .map(|preset| preset.bands.as_str())
                .unwrap_or_default();
            let values: Vec<String> = preset_bands
                .split(',')
                .filter(|value| !value.is_empty())
                .map(|value| value.trim().to_string())
                .collect();
            if !values.is_empty() && (polled.is_empty() || values.len() == polled.len()) {
                return values;
            }
        }

        polled
    }

    /// Select an SE section (mirrors React `selectSection`).
    pub(crate) fn select_section(&mut self, section: &str, cx: &mut Context<Self>) {
        let active = self.se_active();
        if active == section {
            if section == data::SECTION_CUSTOM && self.eq_draft.is_empty() {
                self.sync_eq_draft();
            }
            cx.notify();
            return;
        }
        let Some(mac) = self.active_device().map(|d| d.mac_address.clone()) else {
            return;
        };
        let spatial_on = {
            let s = self.settings_for(&mac);
            data::has_spatial(&s) && data::is_spatial_on(&s)
        };

        if section == data::SECTION_SPATIAL {
            self.begin_toggle_override("spatialAudio", true);
            self.send_setting(&mac, "spatialAudio", "true");
        } else {
            if spatial_on {
                self.begin_toggle_override("spatialAudio", false);
                self.send_setting(&mac, "spatialAudio", "false");
            }
            if section == data::SECTION_DEFAULT {
                let id = {
                    let s = self.settings_for(&mac);
                    self.last_preset_id.clone().or_else(|| {
                        data::preset_options(&s)
                            .into_iter()
                            .next()
                            .map(|(id, _)| id)
                    })
                };
                let Some(id) = id else { return };
                self.send_setting(&mac, "presetEqualizerProfile", &id);
            } else if section == data::SECTION_CUSTOM {
                let read_only = self
                    .settings_for(&mac)
                    .get("volumeAdjustments")
                    .and_then(|v| v.get("readOnly"))
                    .and_then(|v| v.as_bool())
                    == Some(true);
                if read_only {
                    return;
                }
                let bands = self.eq_bands_for_apply(&mac);
                if bands.is_empty() {
                    return;
                }
                self.send_setting(&mac, "volumeAdjustments", &bands.join(","));
                let values: Vec<i16> = bands
                    .iter()
                    .filter_map(|value| value.parse().ok())
                    .collect();
                if self.eq_draft.len() != values.len() || values.is_empty() {
                    self.eq_draft = values;
                }
            }
        }
        self.se_local = Some((section.to_string(), std::time::Instant::now()));
        cx.notify();
    }

    /// Set spatialAudioMode with 500ms optimistic grace.
    pub(crate) fn set_spatial_mode(&mut self, mode: &str, cx: &mut Context<Self>) {
        let Some(mac) = self.active_device().map(|d| d.mac_address.clone()) else {
            return;
        };
        if !data::has_option(&self.settings_for(&mac), "spatialAudioMode", mode) {
            return;
        }
        self.send_setting(&mac, "spatialAudioMode", mode);
        self.spatial_local = Some((mac, mode.to_string(), std::time::Instant::now()));
        cx.notify();
    }

    pub(crate) fn set_named_select(
        &mut self,
        mac: &str,
        id: &str,
        value: &str,
        cx: &mut Context<Self>,
    ) {
        if !data::has_option(&self.settings_for(mac), id, value) {
            return;
        }
        self.send_setting(mac, id, value);
        cx.notify();
    }

    fn reload_eq_presets(&mut self) {
        let model = self.active_device().map(|d| d.model.clone());
        self.eq_presets = match model {
            Some(model) => crate::list_eq_presets(state(), &model),
            None => crate::get_config(state())
                .eq_presets
                .into_iter()
                .map(|p| EqPresetDto {
                    id: p.id,
                    name: p.name,
                    bands: p.bands,
                    model: p.model,
                })
                .collect(),
        };
        self.ensure_eq_preset();
        self.sync_eq_selection();
    }

    fn ensure_eq_preset(&mut self) {
        if !self.eq_presets.is_empty() {
            return;
        }
        let settings = self
            .active_device()
            .map(|d| self.settings_for(&d.mac_address))
            .unwrap_or_default();
        let values: Vec<i16> = settings
            .get("volumeAdjustments")
            .and_then(|value| value.get("value"))
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_i64().map(|value| value as i16))
                    .collect()
            })
            .unwrap_or_default();
        let bands = if values.is_empty() {
            vec![0i16; 8]
        } else {
            values
        };
        let bands = bands
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let model = self
            .active_device()
            .map(|device| device.model.clone())
            .unwrap_or_default();
        match crate::save_eq_preset(state(), "Custom 1".to_string(), bands, model) {
            Ok(preset) => {
                self.eq_active_preset = Some(preset.id);
                self.eq_presets.push(preset);
            }
            Err(err) => tracing::warn!("create default eq preset failed: {err}"),
        }
    }

    fn sync_eq_selection(&mut self) {
        if self.eq_presets.is_empty() {
            self.eq_active_preset = None;
            return;
        }
        if self
            .eq_active_preset
            .is_some_and(|id| self.eq_presets.iter().any(|preset| preset.id == id))
        {
            return;
        }
        let settings = self
            .active_device()
            .map(|d| self.settings_for(&d.mac_address))
            .unwrap_or_default();
        let matched = data::match_custom_preset(&settings, &self.eq_presets);
        self.eq_active_preset = matched
            .and_then(|name| self.eq_presets.iter().find(|preset| preset.name == name))
            .or_else(|| self.eq_presets.first())
            .map(|preset| preset.id);
    }

    pub(crate) fn delete_eq_preset_confirmed(
        &mut self,
        id: i64,
        name: &str,
        mac: &str,
        cx: &mut Context<Self>,
    ) {
        self.eq_drag_band = None;
        if self.eq_presets.len() <= 1 {
            self.eq_delete_confirm = None;
            cx.notify();
            return;
        }
        if !mac.is_empty() {
            self.send_setting(mac, "customEqualizerProfile", &format!("-{name}"));
        }
        if let Err(err) = crate::delete_eq_preset(state(), id) {
            tracing::warn!("delete eq preset failed: {err}");
            return;
        }
        self.reload_eq_presets();
        self.eq_hover_preset = None;
        self.eq_delete_confirm = None;
        cx.notify();
    }

    pub fn active_device(&self) -> Option<&DeviceStateDto> {
        self.devices.iter().find(|d| d.connected)
    }

    /// Settings map for a device by MAC (empty if unknown).
    pub fn cached_settings(&self, mac: &str) -> data::SettingsMap {
        self.settings_for(mac)
    }

    fn rebuild_settings_cache(&mut self) {
        self.settings_cache = self
            .devices
            .iter()
            .map(|device| (device.mac_address.clone(), data::settings_map(device)))
            .collect();
    }

    fn settings_for(&self, mac: &str) -> data::SettingsMap {
        self.settings_cache.get(mac).cloned().unwrap_or_default()
    }

    pub(crate) fn frame_analysis(
        &mut self,
        hex: &str,
        model: &str,
    ) -> Option<soundcore_protocol::analysis::FrameAnalysis> {
        let key = (hex.to_string(), model.to_string());
        if let Some(cached) = self.frame_analysis_cache.get(&key) {
            return cached.clone();
        }
        let result = soundcore_protocol::analysis::analyze(hex, model);
        if self.frame_analysis_cache.len() > 512 {
            self.frame_analysis_cache.clear();
        }
        self.frame_analysis_cache.insert(key, result.clone());
        result
    }

    /// Effective toggle value while a device confirmation is pending.
    pub fn effective_toggle(&self, id: &str, polled: bool) -> bool {
        self.toggle_local
            .get(id)
            .map(|(value, _)| *value)
            .unwrap_or(polled)
    }

    pub(crate) fn begin_toggle_override(&mut self, id: impl Into<String>, value: bool) {
        self.toggle_local
            .insert(id.into(), (value, std::time::Instant::now()));
    }

    pub(crate) fn settle_toggle_overrides(&mut self, mac: &str) {
        let settings = self.settings_for(mac);
        self.toggle_local.retain(|id, (expected, started)| {
            let polled = settings
                .get(id)
                .and_then(|value| value.get("value"))
                .and_then(|value| value.as_bool())
                .unwrap_or(!*expected);
            polled != *expected && started.elapsed() < Duration::from_secs(5)
        });
    }

    pub(crate) fn settle_spatial_override(&mut self, mac: &str) {
        let Some((owner, mode, started)) = self.spatial_local.clone() else {
            return;
        };
        if owner != mac {
            return;
        }
        let settings = self.settings_for(mac);
        let polled = settings
            .get("spatialAudioMode")
            .and_then(|value| value.get("value"))
            .and_then(|value| value.as_str());
        if polled == Some(mode.as_str()) || started.elapsed() >= Duration::from_secs(5) {
            self.spatial_local = None;
        }
    }

    pub fn dismiss_update(&mut self, cx: &mut Context<Self>) {
        self.update_info = None;
        self.update_progress = None;
        self.update_error = None;
        cx.notify();
    }

    pub fn start_update(&mut self, cx: &mut Context<Self>) {
        let checking = UpdateProgress {
            phase: "checking".to_string(),
            percent: 0.0,
            message: "Checking for update...".to_string(),
        };
        set_update_progress(state(), Some(checking.clone()));
        self.update_error = None;
        self.update_progress = Some(checking);
        cx.notify();
        cx.spawn(async move |this, cx| {
            let (tx, rx) = tokio::sync::oneshot::channel();
            std::thread::spawn(move || {
                let _ = tx.send(crate::start_update(state()));
            });
            let result = rx
                .await
                .unwrap_or_else(|_| Err("update thread panicked".into()));
            set_update_progress(state(), None);
            this.update(&mut *cx, |view, cx| {
                if let Err(e) = result {
                    view.update_error = Some(e);
                }
                view.update_progress = None;
                cx.notify();
            })
            .ok();
        })
        .detach();
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
        if let Err(err) =
            crate::set_setting(state(), mac.to_string(), id.to_string(), raw.to_string())
        {
            tracing::warn!("set_setting {id} for {mac} failed: {err}");
        }
    }

    pub fn load_device_image(&self, d: &DeviceStateDto) -> Option<std::sync::Arc<gpui::Image>> {
        load_device_image_url(d.image.as_deref()?)
    }
}

pub(crate) fn load_device_image_url(url: &str) -> Option<std::sync::Arc<gpui::Image>> {
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
    let bytes = downscale_png(bytes, 120);
    let img = std::sync::Arc::new(gpui::Image::from_bytes(gpui::ImageFormat::Png, bytes));
    device_image_cache()
        .lock()
        .unwrap()
        .insert(key, img.clone());
    Some(img)
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
    if img.write_to(&mut cursor, image::ImageFormat::Png).is_err() {
        return bytes;
    }
    out
}

const NAV_DURATION: Duration = Duration::from_millis(320);
const NAV_SETTLE: Duration = Duration::from_millis(360);

fn ease_out_cubic(phase: f32) -> f32 {
    1.0 - (1.0 - phase).powi(3)
}

impl RootView {
    /// Replay the saved-device entrance when the window becomes visible.
    pub fn begin_device_entrance(&mut self) {
        self.device_entry_id = self.device_entry_id.wrapping_add(1);
    }

    fn update_nav_animation(&mut self, key: &'static str, cx: &mut Context<Self>) {
        if self.nav_screen == key {
            return;
        }
        self.nav_direction = match screen_depth(key).cmp(&screen_depth(self.nav_screen)) {
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
        };
        self.nav_prev_screen = self.nav_screen;
        self.nav_screen = key;
        self.nav_id = self.nav_id.wrapping_add(1);
        if key == "devices" {
            self.device_entry_id = self.device_entry_id.wrapping_add(1);
        }
        let nav_id = self.nav_id;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(NAV_SETTLE).await;
            this.update(&mut *cx, |view, cx| {
                if view.nav_id == nav_id {
                    view.nav_prev_screen = "";
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    /// Render a screen by key. Each layer is wrapped in its own element id so
    /// GPUI scopes the screens' static ids to that layer during a transition.
    fn render_screen(
        &mut self,
        key: &str,
        active: Option<&DeviceStateDto>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        Some(match key {
            "control" => screens::control::render(self, active?, window, cx).into_any_element(),
            "devices" => screens::devices::render(self, window, cx).into_any_element(),
            "searching" => screens::searching::render(self, window, cx).into_any_element(),
            "sound-effects" => {
                screens::sound_effects::render(self, active?, window, cx).into_any_element()
            }
            "device-settings" => {
                screens::device_settings::render(self, active?, window, cx).into_any_element()
            }
            "device-info" => {
                screens::device_info::render(self, active?, window, cx).into_any_element()
            }
            "control-settings" => {
                screens::control_settings::render(self, active?, window, cx).into_any_element()
            }
            "protocol-lab" => screens::protocol_lab::render(self, window, cx).into_any_element(),
            "settings" => screens::settings::render(self, window, cx).into_any_element(),
            "about" => screens::about::render(self, window, cx).into_any_element(),
            _ => return None,
        })
    }
}

/// Navigation hierarchy depth, used to decide push (deeper) vs pop (shallower).
fn screen_depth(key: &str) -> u8 {
    match key {
        "sound-effects" | "device-settings" | "control-settings" | "device-info" | "settings"
        | "about" => 1,
        "protocol-lab" => 2,
        _ => 0,
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.eq_presets.is_empty() {
            self.reload_eq_presets();
        } else {
            self.sync_eq_selection();
        }

        let connected_device = self.active_device().cloned();
        let confirm_view = cx.entity();
        let eq_delete_confirm = self.eq_delete_confirm.clone();
        let confirm_mac = connected_device
            .as_ref()
            .map(|d| d.mac_address.clone())
            .unwrap_or_default();
        self.unsupported_device = connected_device
            .as_ref()
            .filter(|d| d.categories.is_empty())
            .map(|d| {
                (
                    d.mac_address.clone(),
                    data::display_device_name(d),
                    d.model.clone(),
                )
            });
        let active = connected_device.filter(|d| !d.categories.is_empty());
        if let Some(device) = active.as_ref() {
            self.settle_toggle_overrides(&device.mac_address);
            self.settle_spatial_override(&device.mac_address);
        }
        // Render-time exclusivity: About > Settings > Protocol Lab > Device Settings > Sound Effects.
        let show_about = self.show_about;
        if self.show_settings && !show_about {
            self.show_protocol_lab = false;
        }
        let show_settings = self.show_settings && !show_about;
        let show_protocol_lab = self.show_protocol_lab && !show_settings && !show_about;
        let show_device = self.show_device_settings
            && active.is_some()
            && !show_settings
            && !show_protocol_lab
            && !show_about;
        let show_se = self.show_sound_effects
            && active.is_some()
            && !show_settings
            && !show_about
            && !show_protocol_lab
            && !show_device;
        let show_control = self.show_control_settings
            && active.is_some()
            && !show_settings
            && !show_protocol_lab
            && !show_about
            && !show_device
            && !show_se;
        let show_info = self.show_device_info
            && active.is_some()
            && !show_settings
            && !show_protocol_lab
            && !show_about
            && !show_device
            && !show_se
            && !show_control;

        let nav_key = if show_about {
            "about"
        } else if show_settings {
            "settings"
        } else if show_protocol_lab {
            "protocol-lab"
        } else if show_device {
            "device-settings"
        } else if show_se {
            "sound-effects"
        } else if show_control {
            "control-settings"
        } else if show_info {
            "device-info"
        } else if active.is_some() {
            "control"
        } else if self.devices.is_empty() {
            "searching"
        } else {
            "devices"
        };
        self.update_nav_animation(nav_key, cx);
        let nav_id = self.nav_id;
        let nav_dir = self.nav_direction;
        let current_key = self.nav_screen;
        let prev_key = self.nav_prev_screen;
        let width = window.viewport_size().width.as_f32();

        let incoming = self.render_screen(current_key, active.as_ref(), window, cx);
        let outgoing = if nav_dir != 0 && !prev_key.is_empty() {
            self.render_screen(prev_key, active.as_ref(), window, cx)
        } else {
            None
        };

        let mut layers: Vec<gpui::AnyElement> = Vec::new();
        if let Some(outgoing_el) = outgoing {
            let outer_animation = Animation::new(NAV_DURATION).with_easing(ease_out_cubic);
            let inner_animation = Animation::new(NAV_DURATION).with_easing(ease_out_cubic);
            layers.push(
                div()
                    .id(("nav-outgoing", nav_id))
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .w_full()
                    .h_full()
                    .overflow_hidden()
                    .with_animation(
                        ("nav-outgoing-spring", nav_id),
                        outer_animation,
                        move |this, phase| {
                            let p = phase.clamp(0.0, 1.0);
                            let (left, clip) = if nav_dir > 0 {
                                (0.0, (1.0 - p) * width)
                            } else {
                                (p * width, (1.0 - p) * width)
                            };
                            this.left(gpui::px(left)).w(gpui::px(clip + 1.0))
                        },
                    )
                    .child(
                        div()
                            .relative()
                            .w(gpui::px(width))
                            .h_full()
                            .with_animation(
                                ("nav-outgoing-inner", nav_id),
                                inner_animation,
                                move |this, phase| {
                                    let p = phase.clamp(0.0, 1.0);
                                    let offset = if nav_dir > 0 { -p * width * 0.3 } else { 0.0 };
                                    this.left(gpui::px(offset))
                                },
                            )
                            .child(outgoing_el),
                    )
                    .into_any_element(),
            );
        }
        if let Some(incoming_el) = incoming {
            let outer_animation = Animation::new(NAV_DURATION).with_easing(ease_out_cubic);
            let inner_animation = Animation::new(NAV_DURATION).with_easing(ease_out_cubic);
            layers.push(
                div()
                    .id(("nav-incoming", nav_id))
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .w_full()
                    .h_full()
                    .overflow_hidden()
                    .with_animation(
                        ("nav-incoming-spring", nav_id),
                        outer_animation,
                        move |this, phase| {
                            let p = phase.clamp(0.0, 1.0);
                            let (left, clip, opacity) = if nav_dir > 0 {
                                ((1.0 - p) * width, p * width, 1.0)
                            } else if nav_dir < 0 {
                                (0.0, p * width, 1.0)
                            } else {
                                (0.0, width, p)
                            };
                            this.left(gpui::px(left))
                                .w(gpui::px(clip + 1.0))
                                .opacity(opacity)
                        },
                    )
                    .child(
                        div()
                            .relative()
                            .w(gpui::px(width))
                            .h_full()
                            .with_animation(
                                ("nav-incoming-inner", nav_id),
                                inner_animation,
                                move |this, phase| {
                                    let p = phase.clamp(0.0, 1.0);
                                    let offset = if nav_dir > 0 {
                                        0.0
                                    } else if nav_dir < 0 {
                                        -(1.0 - p) * width * 0.3
                                    } else {
                                        (1.0 - p) * 12.0
                                    };
                                    this.left(gpui::px(offset))
                                },
                            )
                            .child(incoming_el),
                    )
                    .into_any_element(),
            );
        }

        // Root IS the glass: baked PNG (gradient+glows+sheen+stroke) masked
        // to rx=28. HWND stays a full transparent host; alpha defines shape.
        div()
            .id("app-root")
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
                    .overflow_hidden()
                    .children(layers),
            )
            .when_some(eq_delete_confirm, |el, (preset_id, preset_name)| {
                let backdrop_view = confirm_view.clone();
                let dialog_view = confirm_view.clone();
                let cancel_view = confirm_view.clone();
                let confirm_view = confirm_view.clone();
                let confirm_mac = confirm_mac.clone();
                let name = preset_name.clone();
                let message = format!("“{preset_name}” will be removed. This cannot be undone.");
                el.child(
                    div()
                        .id("eq-delete-confirm-backdrop")
                        .absolute()
                        .top(px(0.))
                        .left(px(0.))
                        .right(px(0.))
                        .bottom(px(0.))
                        .bg(gpui::rgba(0x000000cc))
                        .flex()
                        .items_center()
                        .justify_center()
                        .on_click(move |_, _, cx| {
                            backdrop_view.update(cx, |root, cx| {
                                root.eq_drag_band = None;
                                root.eq_delete_confirm = None;
                                cx.notify();
                            });
                        })
                        .child(
                            div()
                                .id("eq-delete-confirm-dialog")
                                .w(px(272.))
                                .flex()
                                .flex_col()
                                .gap_3()
                                .rounded_2xl()
                                .bg(theme::popup_bg())
                                .border_1()
                                .border_color(theme::border_medium())
                                .px_4()
                                .py_4()
                                .on_mouse_up(gpui::MouseButton::Left, {
                                    let view = dialog_view.clone();
                                    move |_, _, cx| {
                                        view.update(cx, |root, cx| {
                                            root.eq_drag_band = None;
                                            cx.notify();
                                        });
                                    }
                                })
                                .on_click(|_, _, cx| cx.stop_propagation())
                                .child(
                                    div()
                                        .text_size(px(14.))
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(theme::foreground())
                                        .child("Delete preset?"),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(theme::muted())
                                        .child(message),
                                )
                                .child(
                                    div()
                                        .w_full()
                                        .mt(px(2.))
                                        .flex()
                                        .items_center()
                                        .justify_end()
                                        .gap_2()
                                        .child(
                                            div()
                                                .id("eq-delete-cancel")
                                                .px_3()
                                                .py(px(7.))
                                                .rounded_lg()
                                                .bg(theme::surface())
                                                .border_1()
                                                .border_color(theme::border_subtle())
                                                .text_size(px(11.))
                                                .text_color(theme::foreground())
                                                .cursor_pointer()
                                                .hover(|style| style.bg(theme::hover_medium()))
                                                .on_click(move |_, _, cx| {
                                                    cancel_view.update(cx, |root, cx| {
                                                        root.eq_drag_band = None;
                                                        root.eq_delete_confirm = None;
                                                        cx.notify();
                                                    });
                                                })
                                                .child("No"),
                                        )
                                        .child(
                                            div()
                                                .id("eq-delete-confirm")
                                                .px_3()
                                                .py(px(7.))
                                                .rounded_lg()
                                                .bg(theme::error())
                                                .text_size(px(11.))
                                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                                .text_color(theme::brand_foreground())
                                                .cursor_pointer()
                                                .hover(|style| style.opacity(0.85))
                                                .on_click(move |_, _, cx| {
                                                    confirm_view.update(cx, |root, cx| {
                                                        root.delete_eq_preset_confirmed(
                                                            preset_id,
                                                            &name,
                                                            &confirm_mac,
                                                            cx,
                                                        );
                                                    });
                                                })
                                                .child("Yes, Delete"),
                                        ),
                                ),
                        ),
                )
            })
    }
}
