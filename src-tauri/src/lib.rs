//! SoundCore-Desktop (Tauri shell). The tray + popup UI is HTML/CSS in `../ui`; this file
//! bridges it to the background worker that talks to devices via OpenSCQ30.

mod autostart;
mod config;
mod worker;

use std::{path::PathBuf, str::FromStr, sync::Mutex};

use config::Config;
use macaddr::MacAddr6;
use openscq30_lib::{
    DeviceModel,
    settings::{Setting, SettingId},
};
use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow, WindowEvent,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use worker::{Command, WorkerHandle};

struct AppState {
    worker: WorkerHandle,
    config: Mutex<Config>,
    config_path: PathBuf,
}

// ---- DTOs sent to the web UI as JSON ----

#[derive(Serialize)]
struct DeviceStateDto {
    name: String,
    mac_address: String,
    model: String,
    poll_seconds: u64,
    apply_delay_seconds: u64,
    connected: bool,
    message: String,
    image: Option<String>,
    categories: Vec<CategoryDto>,
    profile_ids: Vec<String>,
}

#[derive(Serialize)]
struct CategoryDto {
    id: String,
    settings: Vec<SettingDto>,
}

#[derive(Serialize)]
struct SettingDto {
    id: String,
    #[serde(flatten)]
    setting: Setting,
}

#[derive(Serialize)]
struct ScanDto {
    scanning: bool,
    results: Vec<ScanItemDto>,
}

#[derive(Serialize)]
struct ScanItemDto {
    name: String,
    mac_address: String,
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

// ---- commands ----

#[tauri::command]
fn get_models() -> Vec<String> {
    use strum::VariantArray;
    DeviceModel::VARIANTS.iter().map(|m| m.to_string()).collect()
}

#[tauri::command]
fn get_states(state: tauri::State<AppState>) -> Vec<DeviceStateDto> {
    let cfg = state.config.lock().unwrap().clone();
    let map = state.worker.state.devices.lock().unwrap();
    cfg.devices
        .iter()
        .map(|d| {
            let live = MacAddr6::from_str(d.mac_address.trim()).ok().and_then(|m| map.get(&m));
            let (connected, message, categories) = match live {
                Some(s) => (
                    s.connected,
                    s.message.clone(),
                    s.snapshot.as_ref().map(build_categories).unwrap_or_default(),
                ),
                None => (false, String::new(), Vec::new()),
            };
            DeviceStateDto {
                name: d.name.clone(),
                mac_address: d.mac_address.clone(),
                model: d.model.clone(),
                poll_seconds: d.poll_seconds,
                apply_delay_seconds: d.apply_delay_seconds,
                connected,
                message,
                image: d.image.clone(),
                categories,
                profile_ids: d.profile.iter().map(|e| e.id.clone()).collect(),
            }
        })
        .collect()
}

#[tauri::command]
fn save_config(new_config: Config, state: tauri::State<AppState>) -> Result<(), String> {
    new_config.save(&state.config_path).map_err(|e| e.to_string())?;
    autostart::reconcile(new_config.autostart);
    let _ = state.worker.tx.send(Command::UpdateConfig(new_config.clone()));
    *state.config.lock().unwrap() = new_config;
    Ok(())
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn apply_now(mac: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mac = MacAddr6::from_str(mac.trim()).map_err(|e| e.to_string())?;
    let _ = state.worker.tx.send(Command::ApplyNow(mac));
    Ok(())
}

#[tauri::command]
fn set_setting(mac: String, id: String, raw: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mac = MacAddr6::from_str(mac.trim()).map_err(|e| e.to_string())?;
    let setting_id = SettingId::from_str(&id).map_err(|_| format!("unknown setting '{id}'"))?;
    // Look up the live setting to know its type, then parse the raw string into a Value.
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
    let _ = state.worker.tx.send(Command::SetSetting { mac, id: setting_id, value });

    // Persist so it re-applies on the next connect (no worker restart, no reconnect churn).
    worker::upsert_profile_entry(&state.worker.state, mac, id.clone(), raw.clone());
    {
        let mut cfg = state.config.lock().unwrap();
        if let Some(d) = cfg
            .devices
            .iter_mut()
            .find(|d| MacAddr6::from_str(d.mac_address.trim()).ok() == Some(mac))
        {
            match d.profile.iter_mut().find(|e| e.id == id) {
                Some(e) => e.value = raw.clone(),
                None => d.profile.push(config::SettingEntry { id: id.clone(), value: raw.clone() }),
            }
            let _ = cfg.save(&state.config_path);
        }
    }
    Ok(())
}

#[tauri::command]
fn scan(model: String, state: tauri::State<AppState>) -> Result<(), String> {
    let model = DeviceModel::from_str(model.trim()).map_err(|_| format!("invalid model '{model}'"))?;
    let _ = state.worker.tx.send(Command::Scan { model });
    Ok(())
}

#[tauri::command]
fn get_scan(state: tauri::State<AppState>) -> ScanDto {
    let scanning = *state.worker.state.scanning.lock().unwrap();
    let results = state
        .worker
        .state
        .scan_results
        .lock()
        .unwrap()
        .iter()
        .map(|r| ScanItemDto { name: r.name.clone(), mac_address: r.mac_address.clone() })
        .collect();
    ScanDto { scanning, results }
}

#[tauri::command]
fn hide_window(window: WebviewWindow) {
    let _ = window.hide();
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let _ = state.worker.tx.send(Command::Quit);
    }
    app.exit(0);
}

// ---- window helpers ----

fn position_bottom_right(window: &WebviewWindow) {
    if let Ok(Some(monitor)) = window.current_monitor() {
        let msize = monitor.size();
        let mpos = monitor.position();
        let scale = monitor.scale_factor();
        let wsize = window.outer_size().unwrap_or(tauri::PhysicalSize::new(440, 620));
        let margin = (12.0 * scale) as i32;
        let taskbar = (48.0 * scale) as i32;
        let x = mpos.x + msize.width as i32 - wsize.width as i32 - margin;
        let y = mpos.y + msize.height as i32 - wsize.height as i32 - margin - taskbar;
        let _ = window.set_position(PhysicalPosition::new(x.max(0), y.max(0)));
    }
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            position_bottom_right(&window);
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

pub fn run() {
    init_logging();

    let config_path = config::resolve_path(None);
    let config = Config::load(&config_path).unwrap_or_else(|err| {
        tracing::warn!("could not load {}: {err:#}; using empty config", config_path.display());
        Config::default()
    });
    autostart::reconcile(config.autostart);
    let worker = worker::spawn(config.clone());

    tauri::Builder::default()
        .manage(AppState { worker, config: Mutex::new(config), config_path })
        .invoke_handler(tauri::generate_handler![
            get_models,
            get_states,
            get_config,
            save_config,
            apply_now,
            set_setting,
            scan,
            get_scan,
            hide_window,
            quit_app
        ])
        .on_window_event(|window, event| {
            // Auto-hide the popup when it loses focus, like a tray flyout.
            if let WindowEvent::Focused(false) = event {
                let _ = window.hide();
            }
        })
        .setup(|app| {
            // Auto-detect: periodically scan connected Bluetooth devices and add any
            // recognized Soundcore device to the config (zero manual setup).
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use strum::VariantArray;
                let any_model = DeviceModel::VARIANTS[0];
                loop {
                    {
                        let state = handle.state::<AppState>();
                        let _ = state.worker.tx.send(Command::Scan { model: any_model });
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
                    {
                        let state = handle.state::<AppState>();
                        let results = state.worker.state.scan_results.lock().unwrap().clone();
                        let mut cfg = state.config.lock().unwrap().clone();
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
                                    });
                                    changed = true;
                                }
                            }
                        }
                        if changed {
                            let _ = cfg.save(&state.config_path);
                            *state.config.lock().unwrap() = cfg.clone();
                            let _ = state.worker.tx.send(Command::UpdateConfig(cfg));
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                }
            });

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?,
                    &MenuItem::with_id(app, "apply", "Apply now", true, None::<&str>)?,
                    &PredefinedMenuItem::separator(app)?,
                    &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
                ],
            )?;

            TrayIconBuilder::with_id("main")
                .icon(tauri::include_image!("icons/128x128.png"))
                .tooltip("SoundCore-Desktop")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => toggle_window(app),
                    "apply" => {
                        let _ = app.emit("tray-apply", ());
                    }
                    "quit" => {
                        if let Some(state) = app.try_state::<AppState>() {
                            let _ = state.worker.tx.send(Command::Quit);
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running SoundCore-Desktop");
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter).with_target(false).try_init();
}
