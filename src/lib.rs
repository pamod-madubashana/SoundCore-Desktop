//! SoundCore-Desktop (GPUI shell). The tray + popup UI is Rust in `src/UI`;
//! this file bridges it to the background worker that talks to devices via OpenSCQ30.

mod autostart;
mod config;
mod device_colors;
mod device_images;
mod platform;
mod shell;
mod updater;
mod worker;

#[allow(non_snake_case)]
pub mod UI;
pub use updater::{UpdateInfo, UpdateProgress};

use std::{
    path::PathBuf,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        Mutex, OnceLock,
    },
    time::Duration,
};

use base64::Engine;
use config::Config;
use gpui::{
    App, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, point,
    prelude::*, px, size,
};
use macaddr::MacAddr6;
use openscq30_lib::{
    DeviceModel,
    settings::{Setting, SettingId},
};
use serde::Serialize;
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use worker::{Command, WorkerHandle};

// ---- shared app state ----

pub struct AppState {
    pub worker: WorkerHandle,
    pub config: Mutex<Config>,
    pub config_path: PathBuf,
    pub pending_notification: Mutex<Option<PendingNotification>>,
    pub hide_cancel: AtomicBool,
    /// Latest update progress for the UI to poll (set by `start_update`).
    pub update_progress: Mutex<Option<updater::UpdateProgress>>,
    /// Runtime for async commands (bluetooth status, update check/download).
    pub runtime: tokio::runtime::Runtime,
}

/// Process-wide state, set once in `run()`. Tray callbacks and UI poll loops
/// read this; GPUI contexts are not `Send`, so we never store them here.
static STATE: OnceLock<Arc<AppState>> = OnceLock::new();

pub fn state() -> &'static Arc<AppState> {
    STATE.get().expect("AppState not initialized")
}

/// Main window handle, set after `open_window`. Tray toggle / hide commands
/// drive show/hide through this.
static MAIN_WINDOW: OnceLock<Mutex<Option<gpui::WindowHandle<UI::RootView>>>> = OnceLock::new();

fn main_window() -> &'static Mutex<Option<gpui::WindowHandle<UI::RootView>>> {
    MAIN_WINDOW.get_or_init(|| Mutex::new(None))
}

#[derive(Clone, Serialize)]
pub struct PendingNotification {
    pub name: String,
    pub image: Option<String>,
    pub status: String,
    pub battery: Option<BatteryInfo>,
}

#[derive(Clone, Serialize)]
pub struct BatteryInfo {
    pub left: Option<i32>,
    pub right: Option<i32>,
    pub combined: Option<i32>,
    pub case: Option<i32>,
}

// ---- DTOs sent to the UI ----

#[derive(Clone, Serialize)]
pub struct DeviceStateDto {
    pub name: String,
    pub mac_address: String,
    pub model: String,
    pub poll_seconds: u64,
    pub apply_delay_seconds: u64,
    pub connected: bool,
    pub message: String,
    pub image: Option<String>,
    pub color: Option<String>,
    pub categories: Vec<CategoryDto>,
    pub profile_ids: Vec<String>,
    pub battery_left: Option<i32>,
    pub battery_right: Option<i32>,
    pub battery_combined: Option<i32>,
    pub battery_case: Option<i32>,
}

#[derive(Clone, Serialize)]
pub struct CategoryDto {
    pub id: String,
    pub settings: Vec<SettingDto>,
}

#[derive(Clone, Serialize)]
pub struct SettingDto {
    pub id: String,
    #[serde(flatten)]
    pub setting: Setting,
}

#[derive(Clone, Serialize)]
pub struct ScanDto {
    pub scanning: bool,
    pub results: Vec<ScanItemDto>,
}

#[derive(Clone, Serialize)]
pub struct ScanItemDto {
    pub name: String,
    pub mac_address: String,
}

#[derive(Clone, Serialize)]
pub struct EqPresetDto {
    pub id: i64,
    pub name: String,
    pub bands: String,
    pub model: String,
}

fn build_categories(snapshot: &worker::Snapshot) -> Vec<CategoryDto> {
    snapshot
        .iter()
        .map(|(cat, settings)| CategoryDto {
            id: cat.to_string(),
            settings: settings
                .iter()
                .map(|(id, setting)| SettingDto {
                    id: id.to_string(),
                    setting: setting.clone(),
                })
                .collect(),
        })
        .collect()
}

// ---- command implementations (plain functions over AppState) ----

fn battery_pct_from_setting(setting: &Setting) -> Option<i32> {
    if let Setting::Information { translated_value, value, .. } = setting {
        if let Some(stripped) = translated_value.trim().strip_suffix('%') {
            if let Ok(n) = stripped.trim().parse::<i32>() {
                return Some(n.min(100));
            }
        }
        if let Some((num, den)) = value.split_once('/') {
            let a: i32 = num.trim().parse().ok()?;
            let b: i32 = den.trim().parse().ok()?;
            if b <= 1 {
                return Some(100);
            }
            return Some((a * 100 / b).min(100));
        }
        if let Ok(n) = value.trim().parse::<i32>() {
            return Some(n.min(100));
        }
    }
    None
}

fn extract_battery_from_snapshot(
    snapshot: &worker::Snapshot,
) -> (Option<i32>, Option<i32>, Option<i32>, Option<i32>) {
    let mut left = None;
    let mut right = None;
    let mut combined = None;
    let mut case = None;
    for (_cat, settings) in snapshot {
        for (id, setting) in settings {
            let id_str = id.to_string();
            if id_str == "batteryLevelLeft" {
                left = battery_pct_from_setting(setting);
            } else if id_str == "batteryLevelRight" {
                right = battery_pct_from_setting(setting);
            } else if id_str == "batteryLevel" {
                combined = battery_pct_from_setting(setting);
            } else if id_str == "caseBatteryLevel" {
                case = battery_pct_from_setting(setting);
            }
        }
    }
    (left, right, combined, case)
}

pub fn get_models() -> Vec<String> {
    use strum::VariantArray;
    DeviceModel::VARIANTS.iter().map(|m| m.to_string()).collect()
}

pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn get_states(state: &AppState) -> Vec<DeviceStateDto> {
    let mut cfg = state.config.lock().unwrap().clone();
    config::sort_devices_by_recency(&mut cfg.devices);
    let map = state.worker.state.devices.lock().unwrap();
    cfg.devices
        .iter()
        .map(|d| {
            let live = MacAddr6::from_str(d.mac_address.trim()).ok().and_then(|m| map.get(&m));
            let (connected, message, categories, battery) = match live {
                Some(s) => {
                    let cats = s.snapshot.as_ref().map(build_categories).unwrap_or_default();
                    let bat = s
                        .snapshot
                        .as_ref()
                        .map(extract_battery_from_snapshot)
                        .unwrap_or((None, None, None, None));
                    (s.connected, s.message.clone(), cats, bat)
                }
                None => (false, String::new(), Vec::new(), (None, None, None, None)),
            };
            DeviceStateDto {
                name: d.name.clone(),
                mac_address: d.mac_address.clone(),
                model: d.model.clone(),
                poll_seconds: d.poll_seconds,
                apply_delay_seconds: d.apply_delay_seconds,
                connected,
                message,
                image: d.image.clone().or_else(|| {
                    let c = d.color.as_deref().unwrap_or("1");
                    if let Some(url) = device_images::bundled_image_url(&d.model, c) {
                        return Some(url);
                    }
                    device_images::get_or_download(&d.model, c).and_then(|p| {
                        let bytes = std::fs::read(&p).ok()?;
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                        Some(format!("data:image/png;base64,{b64}"))
                    })
                }),
                color: d.color.clone(),
                categories,
                profile_ids: d.profile.iter().map(|e| e.id.clone()).collect(),
                battery_left: battery.0,
                battery_right: battery.1,
                battery_combined: battery.2,
                battery_case: battery.3,
            }
        })
        .collect()
}

pub fn save_config(state: &AppState, new_config: Config) -> Result<(), String> {
    new_config.save(&state.config_path).map_err(|e| e.to_string())?;
    autostart::reconcile(new_config.autostart);
    let _ = state.worker.tx.send(Command::UpdateConfig(new_config.clone()));
    *state.config.lock().unwrap() = new_config;
    Ok(())
}

pub fn get_config(state: &AppState) -> Config {
    state.config.lock().unwrap().clone()
}

pub fn set_setting(
    state: &AppState,
    mac: String,
    id: String,
    raw: String,
) -> Result<(), String> {
    let mac = MacAddr6::from_str(mac.trim()).map_err(|e| e.to_string())?;
    let setting_id = SettingId::from_str(&id).map_err(|_| format!("unknown setting '{id}'"))?;
    let setting = {
        let map = state.worker.state.devices.lock().unwrap();
        map.get(&mac)
            .and_then(|d| d.snapshot.as_ref())
            .and_then(|snap| {
                snap.iter()
                    .flat_map(|(_, list)| list.iter())
                    .find(|(sid, _)| *sid == setting_id)
                    .map(|(_, s)| s.clone())
            })
    }
    .ok_or("setting not currently available")?;
    let value = worker::parse_value(&setting, &raw).map_err(|e| e.to_string())?;
    let _ = state.worker.tx.send(Command::SetSetting {
        mac,
        id: setting_id,
        value,
    });

    {
        let mut cfg = state.config.lock().unwrap();
        if let Some(d) = cfg
            .devices
            .iter_mut()
            .find(|d| MacAddr6::from_str(d.mac_address.trim()).ok() == Some(mac))
        {
            config::upsert_entry(&mut d.profile, id, raw);
            let _ = cfg.save(&state.config_path);
        }
    }
    Ok(())
}

pub fn list_eq_presets(state: &AppState, model: &str) -> Vec<EqPresetDto> {
    let cfg = state.config.lock().unwrap();
    cfg.eq_presets
        .iter()
        .filter(|p| p.model.is_empty() || p.model == model)
        .map(|p| EqPresetDto {
            id: p.id,
            name: p.name.clone(),
            bands: p.bands.clone(),
            model: p.model.clone(),
        })
        .collect()
}

pub fn save_eq_preset(
    state: &AppState,
    name: String,
    bands: String,
    model: String,
) -> Result<EqPresetDto, String> {
    let mut cfg = state.config.lock().unwrap();
    let next_id = cfg.eq_presets.iter().map(|p| p.id).max().unwrap_or(0) + 1;
    let entry = config::EqPresetEntry {
        id: next_id,
        name: name.clone(),
        bands: bands.clone(),
        model: model.clone(),
    };
    cfg.eq_presets.push(entry);
    cfg.save(&state.config_path).map_err(|e| e.to_string())?;
    Ok(EqPresetDto {
        id: next_id,
        name,
        bands,
        model,
    })
}

pub fn rename_eq_preset(state: &AppState, id: i64, name: String) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    let entry = cfg
        .eq_presets
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("preset not found")?;
    entry.name = name;
    cfg.save(&state.config_path).map_err(|e| e.to_string())
}

pub fn delete_eq_preset(state: &AppState, id: i64) -> Result<(), String> {
    let mut cfg = state.config.lock().unwrap();
    cfg.eq_presets.retain(|p| p.id != id);
    cfg.save(&state.config_path).map_err(|e| e.to_string())
}

pub fn scan(state: &AppState, model: String) -> Result<(), String> {
    let model =
        DeviceModel::from_str(model.trim()).map_err(|_| format!("invalid model '{model}'"))?;
    let _ = state.worker.tx.send(Command::Scan { model });
    Ok(())
}

pub fn get_scan(state: &AppState) -> ScanDto {
    let scanning = *state.worker.state.scanning.lock().unwrap();
    let results = state
        .worker
        .state
        .scan_results
        .lock()
        .unwrap()
        .iter()
        .map(|r| ScanItemDto {
            name: r.name.clone(),
            mac_address: r.mac_address.clone(),
        })
        .collect();
    ScanDto { scanning, results }
}

/// OS Bluetooth radio power for the idle screen's footer dot; fail-open true
/// on platforms/errors that can't tell (see `worker::bluetooth_is_on`).
pub fn get_bluetooth_status(state: &AppState) -> bool {
    // Run on the command runtime so `spawn_blocking` inside has a context.
    let (tx, rx) = std::sync::mpsc::channel();
    state.runtime.spawn(async move {
        let ok = worker::bluetooth_is_on().await;
        let _ = tx.send(ok);
    });
    rx.recv_timeout(Duration::from_secs(5)).unwrap_or(true)
}

pub fn open_url(url: String) {
    let _ = std::process::Command::new(match std::env::consts::OS {
        "windows" => "cmd",
        "macos" => "open",
        _ => "xdg-open",
    })
    .args(if std::env::consts::OS == "windows" {
        vec!["/c", "start", "", url.as_str()]
    } else {
        vec![url.as_str()]
    })
    .spawn();
}

/// Hide the main window with the slide animation. Must run on the GPUI main thread.
pub fn hide_window_now(cx: &mut App) {
    state().hide_cancel.store(true, Ordering::SeqCst);
    let handle = *main_window().lock().unwrap();
    if let Some(h) = handle
        && let Some(snap) = shell::snapshot(h, cx)
    {
        shell::animate(snap, shell::Anim::Hide);
    }
}

/// Show the main window with the slide animation. Must run on the GPUI main thread.
pub fn show_window_now(cx: &mut App) {
    state().hide_cancel.store(true, Ordering::SeqCst);
    let handle = *main_window().lock().unwrap();
    let Some(h) = handle else { return };

    // Position off-screen below + show (one main-thread update).
    // Topmost/bottom-right run once in finish_show after the show
    // animation — doing them here too re-enters GPUI's window RefCell
    // via activate_window().
    let prepared = h.update(cx, |view, window, cx| {
        let (target_x, target_y) = shell::position_bottom_right(window)
            .unwrap_or_else(|| shell::outer_pos(window));
        let start_y = target_y + 80;
        shell::set_pos(window, target_x, start_y);
        shell::show(window);
        view.focus_handle.focus(window, cx);
        // SW_SHOWNOACTIVATE skips WM_ACTIVATE; bring to foreground explicitly
        // so focus-loss auto-hide can fire. Deferred initial_placement is
        // flushed here; finish_show re-asserts bottom-right after.
        window.activate_window();
        shell::set_pos(window, target_x, start_y);
    });
    if prepared.is_err() {
        return;
    }

    // Snapshot for the animation worker (second main-thread hop).
    if let Some(mut snap) = shell::snapshot(h, cx) {
        if let Some((x, _)) = shell::position_bottom_right_from_snap(&snap) {
            snap.x = x;
        }
        let target = snap.target_y();
        snap.y = target + 80;
        shell::animate(snap, shell::Anim::Show);
        shell::finish_show(h, cx);
    }
}

pub fn get_notification(state: &AppState) -> Option<PendingNotification> {
    state.pending_notification.lock().unwrap().clone()
}

pub fn get_update_progress(state: &AppState) -> Option<updater::UpdateProgress> {
    state.update_progress.lock().unwrap().clone()
}

pub fn quit_app(cx: &mut App) {
    let st = state();
    let _ = st.worker.tx.send(Command::Quit);
    cx.quit();
}

// ---- updater ----

pub fn check_update(state: &AppState) -> Result<updater::UpdateInfo, String> {
    state.runtime.block_on(updater::check_for_update())
}

pub fn start_update(state: &AppState) -> Result<(), String> {
    let info = state.runtime.block_on(updater::check_for_update())?;
    let current = semver::Version::parse(
        info.current_version
            .trim()
            .strip_prefix('v')
            .unwrap_or(info.current_version.trim()),
    )
    .map_err(|e| format!("Invalid current version: {e}"))?;
    let latest = semver::Version::parse(
        info.latest_version
            .trim()
            .strip_prefix('v')
            .unwrap_or(info.latest_version.trim()),
    )
    .map_err(|e| format!("Invalid latest version: {e}"))?;
    if latest <= current {
        return Err(format!(
            "Already up to date (current v{current}, latest v{latest}); refusing no-op install"
        ));
    }

    // Progress is written into AppState so the UI can poll it.
    // SAFETY: `state` outlives this synchronous call (process-lifetime STATE
    // or a caller-held Arc).
    let st = state as *const AppState;
    let on_progress = move |p: updater::UpdateProgress| {
        *unsafe { &*st }.update_progress.lock().unwrap() = Some(p);
    };
    let download_path = state
        .runtime
        .block_on(updater::download_update(&info, &on_progress))?;
    updater::install_update(&download_path, &info.latest_version, &on_progress)?;
    Ok(())
}

pub fn is_update_restart(_state: &AppState) -> bool {
    let mut cfg = config::load();
    if cfg.update_restart {
        cfg.update_restart = false;
        let _ = config::save(&cfg);
        true
    } else {
        false
    }
}

// ---- platform commands ----

pub fn get_platform() -> String {
    platform::get_platform()
}

pub fn get_windows_accent() -> Option<String> {
    platform::get_windows_accent()
}

pub fn get_platform_info() -> platform::PlatformInfo {
    platform::get_platform_info()
}

// ---- notification (scaffold: store data; window port comes with UI) ----

pub fn show_notification(
    state: &AppState,
    name: String,
    image: Option<String>,
    status: String,
    battery_left: Option<i32>,
    battery_right: Option<i32>,
    battery_combined: Option<i32>,
    battery_case: Option<i32>,
) -> Result<(), String> {
    tracing::debug!("show_notification called: name={name}, status={status}");
    let battery = if battery_left.is_some()
        || battery_right.is_some()
        || battery_combined.is_some()
        || battery_case.is_some()
    {
        Some(BatteryInfo {
            left: battery_left,
            right: battery_right,
            combined: battery_combined,
            case: battery_case,
        })
    } else {
        None
    };
    let mut pending = state.pending_notification.lock().unwrap();
    *pending = Some(PendingNotification {
        name,
        image,
        status,
        battery,
    });
    // TODO(port): open a second GPUI window (360x110, bottom-right, auto-hide 3s).
    Ok(())
}

// ---- tray + window wiring ----

fn load_tray_icon() -> Option<Icon> {
    let bytes = include_bytes!("../assets/tray-icon.png");
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().ok()?;
    let buf_size = reader.output_buffer_size()?;
    let mut buf = vec![0u8; buf_size];
    let info = reader.next_frame(&mut buf).ok()?;
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => {
            let raw = &buf[..info.buffer_size()];
            let mut out = Vec::with_capacity((info.width * info.height * 4) as usize);
            for px in raw.chunks_exact(3) {
                out.extend_from_slice(&[px[0], px[1], px[2], 255]);
            }
            out
        }
        _ => {
            tracing::warn!("tray icon has unsupported color type {:?}", info.color_type);
            return None;
        }
    };
    Icon::from_rgba(rgba, info.width, info.height).ok()
}

fn toggle_window(cx: &mut App) {
    // Cancel any pending focus-loss hide.
    state().hide_cancel.store(true, Ordering::SeqCst);
    let handle = *main_window().lock().unwrap();
    let Some(h) = handle else { return };
    // Visibility, not focus: SW_SHOWNOACTIVATE leaves the window inactive
    // after show, so is_window_active() would stay false (matches Tauri
    // is_visible()).
    let visible = h
        .update(cx, |_, window, _| shell::is_visible(window))
        .unwrap_or(false);
    if visible {
        hide_window_now(cx);
    } else {
        show_window_now(cx);
    }
}

fn setup_tray(cx: &mut App) {
    let Some(icon) = load_tray_icon() else {
        tracing::warn!("failed to load tray icon");
        return;
    };

    let menu = Menu::new();
    let open = MenuItem::with_id("open", "Open", true, None);
    let quit = MenuItem::with_id("quit", "Quit", true, None);
    let open_id = open.id().clone();
    let quit_id = quit.id().clone();
    if let Err(err) = menu.append(&open)
        .and_then(|_| menu.append(&PredefinedMenuItem::separator()))
        .and_then(|_| menu.append(&quit))
    {
        tracing::warn!("failed to build tray menu: {err}");
        return;
    }

    let tray = TrayIconBuilder::new()
        .with_id("main")
        .with_icon(icon)
        .with_tooltip("SoundCore-Desktop")
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build();
    let tray = match tray {
        Ok(tray) => tray,
        Err(err) => {
            tracing::warn!("failed to create tray icon: {err}");
            return;
        }
    };

    // Poll tray + menu events on a background loop; hop to the main thread
    // to act. GPUI's Win32 pump dispatches tray messages onto this thread's
    // queue, so try_recv will see them after a short delay.
    // Keep `tray` alive inside the task (TrayIcon is !Send on Windows).
    let open_id = open_id;
    let quit_id = quit_id;
    cx.spawn(async move |cx| {
        let _keep_alive = &tray;
        loop {
            cx.background_executor().timer(Duration::from_millis(50)).await;
            let mut toggle = false;
            let mut quit = false;
            while let Ok(ev) = tray_icon::TrayIconEvent::receiver().try_recv() {
                if let tray_icon::TrayIconEvent::Click {
                    button: tray_icon::MouseButton::Left,
                    button_state: tray_icon::MouseButtonState::Up,
                    ..
                } = ev
                {
                    toggle = true;
                }
            }
            while let Ok(ev) = MenuEvent::receiver().try_recv() {
                if ev.id == open_id {
                    toggle = true;
                } else if ev.id == quit_id {
                    quit = true;
                }
            }
            if toggle || quit {
                let _ = cx.update(|cx| {
                    if quit {
                        quit_app(cx);
                    } else if toggle {
                        toggle_window(cx);
                    }
                });
            }
        }
    })
    .detach();
}

fn open_main_window(cx: &mut App) -> gpui::Result<gpui::WindowHandle<UI::RootView>> {
    // Bottom-right placement: open hidden at a computed position, show later.
    let display = cx
        .primary_display()
        .or_else(|| cx.displays().into_iter().next());
    let (origin, display_id) = match display {
        Some(d) => {
            let b = d.bounds();
            let w = px(shell::MAIN_W);
            let h = px(shell::MAIN_H);
            let margin_x = px(shell::MARGIN_X);
            let margin_y = px(shell::MARGIN_Y + 48.0); // gap above taskbar reserve
            let o = point(
                (b.origin.x + b.size.width - w - margin_x).max(b.origin.x),
                (b.origin.y + b.size.height - h - margin_y).max(b.origin.y),
            );
            (o, Some(d.id()))
        }
        None => (point(px(100.), px(100.)), None),
    };

    let bounds = gpui::Bounds::new(origin, size(px(shell::MAIN_W), px(shell::MAIN_H)));
    let handle = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: None,
            focus: false,
            show: false, // hidden until tray/UI shows it
            kind: WindowKind::PopUp, // skip-taskbar + no decorations on Windows
            is_movable: false,
            is_resizable: false,
            is_minimizable: false,
            display_id,
            window_background: WindowBackgroundAppearance::Transparent,
            app_id: Some("soundcore-desktop".into()),
            window_min_size: Some(size(px(shell::MAIN_W), px(shell::MAIN_H))),
            ..Default::default()
        },
        |window, cx| {
            // Match React: html 14.5px rem root * popup zoom 0.9.
            window.set_rem_size(UI::scale::rem_base());
            // No DWM border/shadow/corner — PNG mask owns the rounded shape.
            shell::remove_dwm_border(window);
            let view = cx.new(|cx| UI::RootView::new(cx));
            // Focus-loss auto-hide (matches Tauri Focused(false) + 100ms delay).
            let focus_handle = view.read(cx).focus_handle.clone();
            focus_handle.focus(window, cx);
            let _ = window.on_focus_out(&focus_handle, cx, |_, window, cx| {
                state().hide_cancel.store(false, Ordering::SeqCst);
                // The window argument lets us skip a redundant hide when this
                // focus-out was itself caused by hiding (tray-toggle path).
                let visible = shell::is_visible(window);
                cx.spawn(async move |cx| {
                    cx.background_executor().timer(Duration::from_millis(100)).await;
                    let _ = cx.update(|cx| {
                        if visible && !state().hide_cancel.load(Ordering::SeqCst) {
                            hide_window_now(cx);
                        }
                    });
                })
                .detach();
            });
            view
        },
    )?;

    // Window exists but is still hidden. Topmost is safe early;
    // finish_show re-asserts it after the show animation.
    let _ = handle.update(cx, |_, window, _| {
        shell::apply_topmost(window);
    });

    *main_window().lock().unwrap() = Some(handle);
    Ok(handle)
}

fn spawn_auto_detect(cx: &App) {
    let st = state().clone();
    // Dedicated thread + current-thread runtime: the loop is sleep + mutex
    // work; no need for the worker's multi-thread runtime.
    let _ = cx;
    std::thread::Builder::new()
        .name("auto-detect".into())
        .spawn(move || {
            use strum::VariantArray;
            let any_model = DeviceModel::VARIANTS[0];
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .expect("auto-detect runtime");
            rt.block_on(async move {
                loop {
                    let scan_sent = {
                        if *st.worker.state.scanning.lock().unwrap() {
                            false
                        } else {
                            let _ = st.worker.tx.send(Command::Scan { model: any_model });
                            true
                        }
                    };
                    if scan_sent {
                        tokio::time::sleep(Duration::from_millis(2500)).await;
                        let results = st.worker.state.scan_results.lock().unwrap().clone();
                        let mut cfg = st.config.lock().unwrap().clone();
                        let mut changed = false;
                        for r in results {
                            if let Some(model) = worker::infer_model(&r.name) {
                                let known = cfg
                                    .devices
                                    .iter()
                                    .any(|d| d.mac_address.eq_ignore_ascii_case(&r.mac_address));
                                if !known {
                                    cfg.devices.push(config::DeviceConfig {
                                        name: r.name.clone(),
                                        mac_address: r.mac_address.clone(),
                                        model: model.to_string(),
                                        poll_seconds: 5,
                                        apply_delay_seconds: 2,
                                        profile: Vec::new(),
                                        image: None,
                                        color: None,
                                        last_connected: None,
                                    });
                                    changed = true;
                                }
                            }
                        }
                        if changed {
                            let _ = cfg.save(&st.config_path);
                            *st.config.lock().unwrap() = cfg.clone();
                            let _ = st.worker.tx.send(Command::UpdateConfig(cfg));
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            });
        })
        .ok();
}

pub fn run() {
    init_logging();

    let config_path = config::resolve_path(None);
    let config = Config::load(&config_path).unwrap_or_else(|err| {
        tracing::warn!(
            "could not load {}: {err:#}; using empty config",
            config_path.display()
        );
        Config::default()
    });
    autostart::reconcile(config.autostart);
    let worker = worker::spawn(config.clone());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("command runtime");

    let state = Arc::new(AppState {
        worker,
        config: Mutex::new(config),
        config_path,
        pending_notification: Mutex::new(None),
        hide_cancel: AtomicBool::new(false),
        update_progress: Mutex::new(None),
        runtime,
    });
    let _ = STATE.set(state);

    updater::cleanup_old_exe();

    gpui_platform::application().run(|cx: &mut App| {
        spawn_auto_detect(cx);
        setup_tray(cx);
        if let Err(err) = open_main_window(cx) {
            tracing::error!("failed to open main window: {err:#}");
        }
        // SOUNDCORE_AUTOSHOW=1: show immediately (screenshot/dev loop).
        // Keeps focus so focus-loss auto-hide won't race the capture.
        if std::env::var("SOUNDCORE_AUTOSHOW").is_ok() {
            cx.spawn(async move |cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(300))
                    .await;
                let _ = cx.update(|cx| {
                    crate::state().hide_cancel.store(true, Ordering::SeqCst);
                    show_window_now(cx);
                });
            })
            .detach();
        } else {
            // Start hidden: GPUI show:false already; nothing to activate.
            cx.activate(false);
        }
    });
}

// ---- logging (ported from Tauri shell) ----

struct CappedLog {
    path: PathBuf,
    max_bytes: u64,
    file: Mutex<std::fs::File>,
}

impl CappedLog {
    fn open(path: PathBuf, max_bytes: u64) -> std::io::Result<Self> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        Ok(Self {
            path,
            max_bytes,
            file: Mutex::new(file),
        })
    }

    fn write(&self, buf: &[u8]) -> std::io::Result<()> {
        use std::io::Write as _;
        let mut file = self.file.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let over_cap =
            file.metadata()?.len().saturating_add(buf.len() as u64) > self.max_bytes;
        if over_cap {
            let _ = std::fs::rename(&self.path, self.path.with_extension("log.1"));
            *file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
        }
        file.write_all(buf)?;
        file.flush()
    }
}

struct CappedWriter<'a>(&'a CappedLog);

impl std::io::Write for CappedWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CappedLog {
    type Writer = CappedWriter<'a>;

    fn make_writer(&'a self) -> Self::Writer {
        CappedWriter(self)
    }
}

fn init_logging() {
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,soundcore_desktop_lib=debug,openscq30_lib=trace")
    });

    let stdout = fmt::layer().with_target(false).with_filter(filter.clone());

    let file_layer = crate::config::resolve_path(None)
        .parent()
        .map(|dir| dir.join("soundcore.log"))
        .and_then(|path| match CappedLog::open(path.clone(), 5 * 1024 * 1024) {
            Ok(log) => Some(
                fmt::layer()
                    .with_writer(log)
                    .with_target(false)
                    .with_filter(filter.clone()),
            ),
            Err(err) => {
                eprintln!("cannot open {} for logging: {err}", path.display());
                None
            }
        });

    let _ = tracing_subscriber::registry()
        .with(stdout)
        .with(file_layer)
        .try_init();
}
