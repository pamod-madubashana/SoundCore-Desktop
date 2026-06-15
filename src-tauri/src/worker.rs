//! Background worker: watches every configured device over RFCOMM, applies its profile
//! on connect, publishes a live snapshot of the device's settings for the UI to render,
//! and applies individual changes the user makes in the popup.

use std::{
    collections::HashMap,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Context, anyhow, bail};
use macaddr::MacAddr6;
use openscq30_lib::{
    DeviceModel, OpenSCQ30Session,
    connection::ConnectionStatus,
    device::OpenSCQ30Device,
    settings::{CategoryId, ModifiableSelectCommand, Select, Setting, SettingId, Value},
    storage::PairedDevice,
};
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::{info, warn};

use crate::config::{Config, DeviceConfig, SettingEntry};

/// One category's worth of live settings, in display order.
pub type Snapshot = Vec<(CategoryId, Vec<(SettingId, Setting)>)>;

#[derive(Debug, Clone, Default)]
pub struct DeviceState {
    pub connected: bool,
    pub message: String,
    pub snapshot: Option<Snapshot>,
}

#[derive(Default)]
pub struct SharedState {
    pub devices: Mutex<HashMap<MacAddr6, DeviceState>>,
    pub scan_results: Mutex<Vec<ScanResult>>,
    pub scanning: Mutex<bool>,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub name: String,
    pub mac_address: String,
}

/// Commands the UI sends to the worker.
#[derive(Debug)]
pub enum Command {
    /// Rebuild all per-device tasks from a new config.
    UpdateConfig(Config),
    /// Re-apply the saved profile to a device now.
    ApplyNow(MacAddr6),
    /// Apply a single live setting change (from a popup widget).
    SetSetting { mac: MacAddr6, id: SettingId, value: Value },
    /// Scan for connectable Bluetooth devices (to pick a MAC).
    Scan { model: DeviceModel },
    Quit,
}

/// Commands routed to an individual device task.
#[derive(Debug)]
enum DeviceCommand {
    ApplyNow,
    SetSetting { id: SettingId, value: Value },
}

#[derive(Clone)]
pub struct WorkerHandle {
    pub tx: mpsc::UnboundedSender<Command>,
    pub state: Arc<SharedState>,
}

pub fn spawn(config: Config) -> WorkerHandle {
    let (tx, rx) = mpsc::unbounded_channel();
    let state = Arc::new(SharedState::default());
    let state_clone = state.clone();
    std::thread::Builder::new()
        .name("auto-mode-worker".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("failed to build tokio runtime");
            rt.block_on(run(config, rx, state_clone));
        })
        .expect("failed to spawn worker thread");
    WorkerHandle { tx, state }
}

struct DeviceTasks {
    senders: HashMap<MacAddr6, mpsc::UnboundedSender<DeviceCommand>>,
    handles: Vec<JoinHandle<()>>,
}

impl DeviceTasks {
    fn abort(self) {
        for h in self.handles {
            h.abort();
        }
    }
}

async fn run(
    config: Config,
    mut rx: mpsc::UnboundedReceiver<Command>,
    state: Arc<SharedState>,
) {
    let session = match open_session().await {
        Ok(s) => Arc::new(s),
        Err(err) => {
            warn!("database error: {err:#}");
            return;
        }
    };

    let mut tasks = spawn_devices(&session, &state, &config);

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::UpdateConfig(new_config) => {
                std::mem::replace(&mut tasks, DeviceTasks { senders: HashMap::new(), handles: Vec::new() }).abort();
                state.devices.lock().unwrap().clear();
                tasks = spawn_devices(&session, &state, &new_config);
            }
            Command::ApplyNow(mac) => {
                if let Some(tx) = tasks.senders.get(&mac) {
                    let _ = tx.send(DeviceCommand::ApplyNow);
                }
            }
            Command::SetSetting { mac, id, value } => {
                if let Some(tx) = tasks.senders.get(&mac) {
                    let _ = tx.send(DeviceCommand::SetSetting { id, value });
                }
            }
            Command::Scan { model } => {
                let session = session.clone();
                let state = state.clone();
                tokio::spawn(async move { scan(&session, &state, model).await });
            }
            Command::Quit => {
                tasks.abort();
                return;
            }
        }
    }
}

fn spawn_devices(session: &Arc<OpenSCQ30Session>, state: &Arc<SharedState>, config: &Config) -> DeviceTasks {
    let mut senders = HashMap::new();
    let mut handles = Vec::new();
    for dev in &config.devices {
        let mac = match dev.parse() {
            Ok((mac, _)) => mac,
            Err(err) => {
                warn!("skipping device '{}': {err:#}", dev.label());
                continue;
            }
        };
        let (dtx, drx) = mpsc::unbounded_channel();
        senders.insert(mac, dtx);
        let session = session.clone();
        let state = state.clone();
        let dev = dev.clone();
        handles.push(tokio::spawn(async move { device_loop(session, state, dev, drx).await }));
    }
    DeviceTasks { senders, handles }
}

fn set_device_state(state: &SharedState, mac: MacAddr6, f: impl FnOnce(&mut DeviceState)) {
    let mut map = state.devices.lock().unwrap();
    f(map.entry(mac).or_default());
}

async fn device_loop(
    session: Arc<OpenSCQ30Session>,
    state: Arc<SharedState>,
    dev: DeviceConfig,
    mut rx: mpsc::UnboundedReceiver<DeviceCommand>,
) {
    let (mac, model) = match dev.parse() {
        Ok(v) => v,
        Err(err) => {
            set_device_state(&state, MacAddr6::nil(), |s| s.message = format!("config error: {err:#}"));
            return;
        }
    };

    // OpenSCQ30 only ships Bluetooth backends for Windows and Linux. On any other OS
    // (e.g. macOS) connecting would panic, so degrade gracefully and park the task.
    if openscq30_lib::default_backends().is_none() {
        set_device_state(&state, mac, |s| {
            s.message = "This OS has no Bluetooth backend (OpenSCQ30 supports Windows & Linux)".into();
        });
        while rx.recv().await.is_some() {}
        return;
    }

    // Idempotent mac<->model registration.
    let _ = session
        .pair(PairedDevice { mac_address: mac, model, is_demo: false })
        .await;

    let poll = Duration::from_secs(dev.poll_seconds.max(1));
    let apply_delay = Duration::from_secs(dev.apply_delay_seconds);

    loop {
        set_device_state(&state, mac, |s| {
            s.connected = false;
            s.snapshot = None;
            s.message = format!("waiting for {model}...");
        });

        match timeout(Duration::from_secs(10), session.connect(mac)).await {
            Ok(Ok(device)) => {
                set_device_state(&state, mac, |s| {
                    s.connected = true;
                    s.message = "connected".into();
                });
                if !apply_delay.is_zero() {
                    sleep(apply_delay).await;
                }
                apply_and_report(device.as_ref(), &dev.profile, &state, mac).await;
                publish_snapshot(device.as_ref(), &state, mac);

                if connected_session(device.as_ref(), &dev, &mut rx, &state, mac).await {
                    return; // channel dropped -> task replaced
                }

                set_device_state(&state, mac, |s| {
                    s.connected = false;
                    s.snapshot = None;
                    s.message = "disconnected".into();
                });
            }
            _ => {
                sleep(poll).await;
            }
        }
    }
}

/// Returns true if the device command channel closed (task is being replaced).
async fn connected_session(
    device: &(dyn OpenSCQ30Device + Send + Sync),
    dev: &DeviceConfig,
    rx: &mut mpsc::UnboundedReceiver<DeviceCommand>,
    state: &SharedState,
    mac: MacAddr6,
) -> bool {
    let mut conn = device.connection_status();
    let mut changes = device.watch_for_changes();
    loop {
        tokio::select! {
            cmd = rx.recv() => match cmd {
                Some(DeviceCommand::ApplyNow) => {
                    apply_and_report(device, &dev.profile, state, mac).await;
                    publish_snapshot(device, state, mac);
                }
                Some(DeviceCommand::SetSetting { id, value }) => {
                    if let Err(err) = device.set_setting_values(vec![(id, value)]).await {
                        warn!("set failed: {err}");
                        set_device_state(state, mac, |s| s.message = format!("set failed: {err}"));
                    }
                    publish_snapshot(device, state, mac);
                }
                None => return true,
            },
            changed = changes.changed() => {
                if changed.is_ok() {
                    publish_snapshot(device, state, mac);
                }
            }
            conn_changed = conn.changed() => {
                if conn_changed.is_err() || *conn.borrow_and_update() == ConnectionStatus::Disconnected {
                    return false;
                }
            }
        }
    }
}

fn publish_snapshot(device: &(dyn OpenSCQ30Device + Send + Sync), state: &SharedState, mac: MacAddr6) {
    let snapshot: Snapshot = device
        .settings_by_category()
        .into_iter()
        .map(|(cat, settings)| (cat, settings.into_iter().collect()))
        .collect();
    set_device_state(state, mac, |s| s.snapshot = Some(snapshot));
}

async fn apply_and_report(
    device: &(dyn OpenSCQ30Device + Send + Sync),
    entries: &[SettingEntry],
    state: &SharedState,
    mac: MacAddr6,
) {
    match apply_profile(device, entries).await {
        Ok(()) => {
            info!("profile applied to {mac}");
            set_device_state(state, mac, |s| s.message = "profile applied".into());
        }
        Err(err) => {
            warn!("apply failed for {mac}: {err:#}");
            set_device_state(state, mac, |s| s.message = format!("apply failed: {err:#}"));
        }
    }
}

async fn apply_profile(device: &(dyn OpenSCQ30Device + Send + Sync), entries: &[SettingEntry]) -> anyhow::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let mut values = Vec::with_capacity(entries.len());
    for entry in entries {
        let setting_id = SettingId::from_str(&entry.id)
            .map_err(|_| anyhow!("setting id '{}' does not exist", entry.id))?;
        let setting = device
            .setting(&setting_id)
            .ok_or_else(|| anyhow!("{} does not expose '{}' right now", device.model(), entry.id))?;
        let value = parse_value(&setting, &entry.value)
            .with_context(|| format!("invalid value for '{}'", entry.id))?;
        values.push((setting_id, value));
    }
    device.set_setting_values(values).await.context("device rejected settings")?;
    Ok(())
}

async fn scan(session: &OpenSCQ30Session, state: &SharedState, model: DeviceModel) {
    *state.scanning.lock().unwrap() = true;
    let results = match session.list_devices(model).await {
        Ok(devices) => devices
            .into_iter()
            .map(|d| ScanResult {
                name: d.name,
                mac_address: d.mac_address.to_string(),
            })
            .collect(),
        Err(err) => {
            warn!("scan failed: {err}");
            Vec::new()
        }
    };
    *state.scan_results.lock().unwrap() = results;
    *state.scanning.lock().unwrap() = false;
}

/// Converts a setting's current value into the string form stored in a profile.
pub fn current_value_string(setting: &Setting) -> String {
    match setting {
        Setting::Toggle { value } => value.to_string(),
        Setting::I32Range { value, .. } => value.to_string(),
        Setting::Select { value, .. } => value.to_string(),
        Setting::OptionalSelect { value, .. } | Setting::ModifiableSelect { value, .. } => {
            value.clone().map(|v| v.to_string()).unwrap_or_default()
        }
        Setting::MultiSelect { values, .. } => values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(","),
        Setting::Equalizer { value, .. } => value
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(","),
        Setting::Information { value, .. } => value.clone(),
        Setting::ImportString { .. } | Setting::Action => String::new(),
    }
}

/// Converts a `Value` (as produced by a UI widget) into the string form for a profile.
pub fn value_to_profile_string(value: &Value) -> String {
    match value {
        Value::Bool(b) => b.to_string(),
        Value::U16(n) => n.to_string(),
        Value::I32(n) => n.to_string(),
        Value::OptionalU16(o) => o.map(|x| x.to_string()).unwrap_or_default(),
        Value::U16Vec(v) => v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
        Value::I16Vec(v) => v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
        Value::String(s) => s.to_string(),
        Value::StringVec(v) => v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
        Value::OptionalString(o) => o.clone().map(|s| s.to_string()).unwrap_or_default(),
        Value::ModifiableSelectCommand(_) => String::new(),
    }
}

pub fn parse_value(setting: &Setting, raw: &str) -> anyhow::Result<Value> {
    match setting {
        Setting::Toggle { .. } => Ok(Value::from(bool::from_str(raw)?)),
        Setting::I32Range { setting, .. } => {
            let n = i32::from_str(raw)?;
            if !setting.range.contains(&n) {
                bail!("{n} is out of range {:?}", setting.range);
            }
            if setting.step != 0 && n % setting.step != 0 {
                bail!("{n} does not align with step {}", setting.step);
            }
            Ok(n.into())
        }
        Setting::Select { setting, .. } => Ok(Value::String(one_of(setting, raw)?)),
        Setting::OptionalSelect { setting, .. } => {
            if raw.is_empty() {
                Ok(Value::OptionalString(None))
            } else {
                Ok(Value::OptionalString(Some(one_of(setting, raw)?)))
            }
        }
        Setting::ModifiableSelect { setting, .. } => {
            if let Some(rest) = raw.strip_prefix('+') {
                Ok(Value::ModifiableSelectCommand(ModifiableSelectCommand::Add(rest.to_owned().into())))
            } else if let Some(rest) = raw.strip_prefix('-') {
                Ok(Value::ModifiableSelectCommand(ModifiableSelectCommand::Remove(rest.to_owned().into())))
            } else {
                let name = raw.strip_prefix('\\').unwrap_or(raw);
                Ok(Value::from(one_of(setting, name)?))
            }
        }
        Setting::MultiSelect { setting, .. } => {
            let selections = raw
                .split(',')
                .map(|item| one_of(setting, item.trim()))
                .collect::<anyhow::Result<Vec<_>>>()?;
            Ok(Value::from(selections))
        }
        Setting::Equalizer { setting, .. } => {
            let values = raw
                .split(',')
                .map(|s| i16::from_str(s.trim()))
                .collect::<Result<Vec<_>, _>>()?;
            if values.len() != setting.band_hz.len() {
                bail!("expected {} bands, got {}", setting.band_hz.len(), values.len());
            }
            for v in &values {
                if *v < setting.min || *v > setting.max {
                    bail!("band {v} outside range {}..={}", setting.min, setting.max);
                }
            }
            Ok(Value::I16Vec(values))
        }
        Setting::Information { .. } => bail!("information settings are read-only"),
        Setting::ImportString { .. } => Ok(Value::from(std::borrow::Cow::from(raw.to_owned()))),
        Setting::Action => Ok(Value::Bool(true)),
    }
}

fn one_of(setting: &Select, raw: &str) -> anyhow::Result<std::borrow::Cow<'static, str>> {
    if let Some(exact) = setting.options.iter().find(|o| **o == raw) {
        return Ok(exact.clone());
    }
    let mut matches = setting.options.iter().filter(|o| o.eq_ignore_ascii_case(raw));
    let first = matches.next();
    if let (Some(a), Some(b)) = (first, matches.next()) {
        bail!("'{raw}' is ambiguous: could be {a} or {b}");
    }
    first
        .cloned()
        .ok_or_else(|| anyhow!("'{raw}' is not valid. Expected one of: {:?}", setting.options))
}

async fn open_session() -> anyhow::Result<OpenSCQ30Session> {
    let db_path = match std::env::var_os("OPENSCQ30_DATABASE_PATH") {
        Some(path) => PathBuf::from(path),
        None => dirs::config_dir()
            .ok_or_else(|| anyhow!("failed to find config dir"))?
            .join("openscq30")
            .join("database.sqlite"),
    };
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    OpenSCQ30Session::new(db_path)
        .await
        .map_err(|e| anyhow!("failed to open database: {e}"))
}
