//! Background worker: watches every configured device over RFCOMM, fetches its
//! current profile on connect (read-only — never writes), publishes a live
//! snapshot of the device's settings for the UI to render, and applies
//! individual changes the user makes in the popup.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{anyhow, bail};
use macaddr::MacAddr6;
use soundcore_protocol::{
    SoundcoreSession,
    vendor::{
        DeviceModel,
        connection::ConnectionStatus,
        device::OpenSCQ30Device,
        settings::{
            CategoryId, ModifiableSelectCommand, MultiSelectWithRemoveCommand, Select, Setting,
            SettingId, Value,
        },
    },
};
use strum::VariantArray;
use tokio::{
    sync::mpsc,
    task::JoinHandle,
    time::{sleep, timeout},
};
use tracing::{info, warn};

use crate::config::{Config, DeviceConfig, SettingEntry};

const CAPTURE_BUFFER_CAP: usize = 500;

/// One category's worth of live settings, in display order.
pub type Snapshot = Vec<(CategoryId, Vec<(SettingId, Setting)>)>;

#[derive(Debug, Clone, Default)]
pub struct DeviceState {
    pub connected: bool,
    pub message: String,
    pub snapshot: Option<Snapshot>,
    pub capabilities: Option<soundcore_protocol::DeviceCapabilities>,
}

#[derive(Default)]
pub struct SharedState {
    pub devices: Mutex<HashMap<MacAddr6, DeviceState>>,
    pub scan_results: Mutex<Vec<ScanResult>>,
    pub scanning: Mutex<bool>,
    pub capture_frames: Mutex<VecDeque<soundcore_protocol::analysis::CapturedFrameDto>>,
    pub capture_active: Mutex<bool>,
    pub capture_started: Mutex<Option<std::time::Instant>>,
}

/// Best-effort Bluetooth-name -> model guess for zero-config auto-detect.
/// Order matters: more specific patterns must come before less specific ones.
pub fn infer_model(name: &str) -> Option<DeviceModel> {
    let n = name.to_lowercase();

    let mut codes: Vec<(String, DeviceModel)> = DeviceModel::VARIANTS
        .iter()
        .copied()
        .filter(|model| *model != DeviceModel::SoundcoreDevelopment)
        .map(|model| {
            let display = model.to_string();
            let code = display.strip_prefix("Soundcore").unwrap_or(&display);
            (code.to_lowercase(), model)
        })
        .collect();
    codes.sort_unstable_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
    if let Some((_, model)) = codes.iter().find(|(code, _)| n.contains(code.as_str())) {
        return Some(*model);
    }

    const HUMAN_NAMES: &[(&str, DeviceModel)] = &[
        ("q20i", DeviceModel::SoundcoreA3004),
        ("q20", DeviceModel::SoundcoreA3004),
        ("q11i", DeviceModel::SoundcoreA3005),
        ("life q35", DeviceModel::SoundcoreA3027),
        ("q35", DeviceModel::SoundcoreA3027),
        ("life q30", DeviceModel::SoundcoreA3028),
        ("q30", DeviceModel::SoundcoreA3028),
        ("life tune pro", DeviceModel::SoundcoreA3030),
        ("life tune", DeviceModel::SoundcoreA3029),
        ("vortex", DeviceModel::SoundcoreA3031),
        ("life 2 neo", DeviceModel::SoundcoreA3033),
        ("life neo", DeviceModel::SoundcoreA3033),
        ("space one pro", DeviceModel::SoundcoreA3062),
        ("space one", DeviceModel::SoundcoreA3035),
        ("space q45", DeviceModel::SoundcoreA3040),
        ("q45", DeviceModel::SoundcoreA3040),
        ("space 2", DeviceModel::SoundcoreD1402),
        ("space a40", DeviceModel::SoundcoreA3936),
        ("a40", DeviceModel::SoundcoreA3936),
        ("motion+", DeviceModel::SoundcoreA3116),
        ("c30i", DeviceModel::SoundcoreA3330),
        ("aeroclip", DeviceModel::SoundcoreA3388),
        ("v20i", DeviceModel::SoundcoreA3876),
        ("liberty 2 pro+", DeviceModel::SoundcoreA3930),
        ("liberty 2 pro", DeviceModel::SoundcoreA3909),
        ("life dot 2 nc", DeviceModel::SoundcoreA3931),
        ("life dot 2", DeviceModel::SoundcoreA3926),
        ("life note 3s", DeviceModel::SoundcoreA3945),
        ("life note 3", DeviceModel::SoundcoreA3933),
        ("life a2 nc", DeviceModel::SoundcoreA3935),
        ("a2 nc", DeviceModel::SoundcoreA3935),
        ("life p3", DeviceModel::SoundcoreA3939),
        ("life p2 mini", DeviceModel::SoundcoreA3944),
        ("liberty 4 nc", DeviceModel::SoundcoreA3947),
        ("a20i", DeviceModel::SoundcoreA3948),
        ("r50i nc", DeviceModel::SoundcoreA3959),
        ("p30i", DeviceModel::SoundcoreA3959),
        ("r50i", DeviceModel::SoundcoreA3949),
        ("p20i", DeviceModel::SoundcoreA3949),
        ("p25i", DeviceModel::SoundcoreA3949),
        ("liberty air 2 pro", DeviceModel::SoundcoreA3951),
        ("liberty 3 pro", DeviceModel::SoundcoreA3952),
        ("liberty 4 pro", DeviceModel::SoundcoreA3954),
        ("liberty 4 max", DeviceModel::SoundcoreA3954),
        ("liberty 4", DeviceModel::SoundcoreA3953),
        ("p40i", DeviceModel::SoundcoreA3955),
        ("liberty 5 pro max", DeviceModel::SoundcoreD1204),
        ("liberty 5 pro", DeviceModel::SoundcoreD1204),
        ("liberty 5", DeviceModel::SoundcoreA3957),
        ("sport x20", DeviceModel::SoundcoreA3968),
        ("c50i", DeviceModel::SoundcoreD1101),
        ("p31i", DeviceModel::SoundcoreD1202),
        ("r60i nc", DeviceModel::SoundcoreD1202C),
        ("r60i", DeviceModel::SoundcoreD1202),
        ("sleep a30", DeviceModel::SoundcoreD1301),
    ];

    let mut patterns: Vec<(&str, DeviceModel)> = HUMAN_NAMES.to_vec();
    patterns.sort_unstable_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(b.0)));
    patterns
        .into_iter()
        .find_map(|(pattern, model)| n.contains(pattern).then_some(model))
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
    /// Apply a single live setting change (from a popup widget).
    SetSetting {
        mac: MacAddr6,
        id: SettingId,
        value: Value,
    },
    /// Scan for connectable Bluetooth devices (to pick a MAC).
    Scan {
        model: DeviceModel,
    },
    StartCapture,
    StopCapture,
    ClearCapture,
    Pair {
        mac: MacAddr6,
        model: DeviceModel,
    },
    Quit,
}

/// Commands routed to an individual device task.
#[derive(Debug)]
enum DeviceCommand {
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
    handles: HashMap<MacAddr6, JoinHandle<()>>,
}

impl DeviceTasks {
    fn abort(self) {
        for h in self.handles.into_values() {
            h.abort();
        }
    }
}

async fn run(
    mut config: Config,
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

    let mut capture_rx = soundcore_protocol::vendor::subscribe_capture();
    let mut capture_closed = false;

    loop {
        tokio::select! {
            biased;
            maybe_cmd = rx.recv() => {
                let Some(cmd) = maybe_cmd else { return };
                match cmd {
                    Command::UpdateConfig(new_config) => {
                        let (to_abort, to_spawn) = diff_tasks(&config.devices, &new_config.devices);
                        for mac in to_abort {
                            if let Some(handle) = tasks.handles.remove(&mac) {
                                handle.abort();
                            }
                            tasks.senders.remove(&mac);
                            state.devices.lock().unwrap().remove(&mac);
                        }
                        for idx in to_spawn {
                            spawn_device(&session, &state, &new_config.devices[idx], &mut tasks);
                        }
                        config = new_config;
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
                    Command::StartCapture => {
                        state.capture_frames.lock().unwrap().clear();
                        *state.capture_started.lock().unwrap() = Some(std::time::Instant::now());
                        *state.capture_active.lock().unwrap() = true;
                    }
                    Command::StopCapture => {
                        *state.capture_active.lock().unwrap() = false;
                        *state.capture_started.lock().unwrap() = None;
                    }
                    Command::ClearCapture => {
                        state.capture_frames.lock().unwrap().clear();
                    }
                    Command::Pair { mac, model } => {
                        match session.pair(mac, model).await {
                            Ok(()) => set_device_state(&state, mac, |s| {
                                s.message = format!("paired with {model}");
                            }),
                            Err(err) => set_device_state(&state, mac, |s| {
                                s.message = format!("pair failed: {err}");
                            }),
                        }
                    }
                    Command::Quit => {
                        tasks.abort();
                        return;
                    }
                }
            }
            frames = async {
                loop {
                    match capture_rx.recv().await {
                        Ok(captured) => {
                            let mut batch = vec![captured];
                            while let Ok(more) = capture_rx.try_recv() {
                                batch.push(more);
                            }
                            return Some(batch);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => return None,
                    }
                }
            }, if !capture_closed => {
                match frames {
                    Some(batch) => {
                        for captured in batch {
                            push_capture(&state, captured);
                        }
                    }
                    None => capture_closed = true,
                }
            }
        }
    }
}

fn push_capture(state: &SharedState, captured: soundcore_protocol::vendor::CapturedFrame) {
    if !*state.capture_active.lock().unwrap() {
        return;
    }
    let mut frames = state.capture_frames.lock().unwrap();
    frames.push_back(soundcore_protocol::analysis::CapturedFrameDto {
        direction: captured.direction.to_owned(),
        timestamp_ms: captured.timestamp_ms,
        hex: bytes_to_hex(&captured.bytes),
    });
    while frames.len() > CAPTURE_BUFFER_CAP {
        frames.pop_front();
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(DIGITS[(byte >> 4) as usize] as char);
        out.push(DIGITS[(byte & 0x0f) as usize] as char);
    }
    out
}

/// True for real Soundcore hardware models. Anything else (notably the
/// development/test double) is never listened to: no worker task, no connect
/// attempts, no retry loop.
fn is_supported_model(model: DeviceModel) -> bool {
    !matches!(model, DeviceModel::SoundcoreDevelopment)
}

/// Fingerprint of the device fields that require restarting its task.
/// `name`, `profile`, `image`, and `color` are UI-only and deliberately absent.
type TaskFingerprint = (MacAddr6, DeviceModel, u64, u64);

fn task_fingerprint(dev: &DeviceConfig) -> Option<TaskFingerprint> {
    let (mac, model) = dev.parse().ok()?;
    Some((mac, model, dev.poll_seconds, dev.apply_delay_seconds))
}

/// Pure diff between two device lists: (macs whose running task must be aborted,
/// indices into `new` whose task must be spawned). Entries whose fingerprint is
/// unchanged keep running untouched — a config edit no longer nukes and respawns
/// every device task. Duplicate MACs resolve first-wins on both sides, matching
/// how tasks are keyed.
fn diff_tasks(old: &[DeviceConfig], new: &[DeviceConfig]) -> (Vec<MacAddr6>, Vec<usize>) {
    let mut old_fps: HashMap<MacAddr6, TaskFingerprint> = HashMap::new();
    for dev in old {
        if let Some(fp) = task_fingerprint(dev) {
            old_fps.entry(fp.0).or_insert(fp);
        }
    }
    let mut live: HashSet<MacAddr6> = HashSet::new();
    let mut to_abort = Vec::new();
    let mut to_spawn = Vec::new();
    for (idx, dev) in new.iter().enumerate() {
        let Some(fp) = task_fingerprint(dev) else {
            continue;
        };
        if !live.insert(fp.0) {
            continue;
        }
        match old_fps.get(&fp.0) {
            Some(prev) if *prev == fp => {}
            Some(_) => {
                to_abort.push(fp.0);
                to_spawn.push(idx);
            }
            None => to_spawn.push(idx),
        }
    }
    for mac in old_fps.keys() {
        if !live.contains(mac) {
            to_abort.push(*mac);
        }
    }
    (to_abort, to_spawn)
}

fn spawn_device(
    session: &Arc<SoundcoreSession>,
    state: &Arc<SharedState>,
    dev: &DeviceConfig,
    tasks: &mut DeviceTasks,
) {
    let (mac, model) = match dev.parse() {
        Ok(v) => v,
        Err(err) => {
            // No recognizable Soundcore model (empty/unknown string) or bad
            // MAC: not Soundcore-supported, so skip entirely with a single
            // one-time line. Never WARN, never spawn a retry loop.
            info!("skipping unsupported device '{}': {err:#}", dev.label());
            return;
        }
    };
    if !is_supported_model(model) {
        info!(
            "skipping unsupported device '{}' (model {model}): not a Soundcore-supported device",
            dev.label()
        );
        return;
    }
    if tasks.senders.contains_key(&mac) {
        return; // duplicate MAC: first entry wins
    }
    let (dtx, drx) = mpsc::unbounded_channel();
    tasks.senders.insert(mac, dtx);
    let session = session.clone();
    let state = state.clone();
    let dev = dev.clone();
    let handle = tokio::spawn(async move { device_loop(session, state, dev, drx).await });
    tasks.handles.insert(mac, handle);
}

fn spawn_devices(
    session: &Arc<SoundcoreSession>,
    state: &Arc<SharedState>,
    config: &Config,
) -> DeviceTasks {
    let mut tasks = DeviceTasks {
        senders: HashMap::new(),
        handles: HashMap::new(),
    };
    for dev in &config.devices {
        spawn_device(session, state, dev, &mut tasks);
    }
    tasks
}

fn set_device_state(state: &SharedState, mac: MacAddr6, f: impl FnOnce(&mut DeviceState)) {
    let mut map = state.devices.lock().unwrap();
    f(map.entry(mac).or_default());
}

/// Retry pacing for present-but-unconnected devices: 0 → 5s → 10s → 20s →
/// 40s → 60s → 60s. Never reached while the OS does not report the device
/// connected (that path stays fully passive and just re-polls).
const CONNECT_BACKOFF_BASE: Duration = Duration::from_secs(5);
const CONNECT_BACKOFF_CAP: Duration = Duration::from_secs(60);

/// Doubles `current` up to `cap`, starting from `base` (clamped to `cap`)
/// when zero.
fn next_backoff(current: Duration, base: Duration, cap: Duration) -> Duration {
    if current.is_zero() {
        base.min(cap)
    } else {
        current.saturating_mul(2).min(cap)
    }
}

enum WaitOutcome {
    /// The full wait elapsed with no commands.
    Elapsed,
    /// At least one command arrived and was stashed into `pending`.
    Commands,
    /// The command channel closed — the task is being replaced.
    Closed,
}

/// Waits up to `dur` but returns early when a device command arrives (stashed
/// into `pending` so nothing is lost) or the channel closes. Used for every
/// not-connected wait so queued UI writes interrupt long backoffs promptly.
async fn wait_for_commands(
    rx: &mut mpsc::UnboundedReceiver<DeviceCommand>,
    dur: Duration,
    pending: &mut Vec<DeviceCommand>,
) -> WaitOutcome {
    tokio::select! {
        _ = sleep(dur) => WaitOutcome::Elapsed,
        cmd = rx.recv() => {
            let Some(cmd) = cmd else { return WaitOutcome::Closed };
            pending.push(cmd);
            loop {
                match rx.try_recv() {
                    Ok(cmd) => pending.push(cmd),
                    Err(mpsc::error::TryRecvError::Empty) => break,
                    Err(mpsc::error::TryRecvError::Disconnected) => return WaitOutcome::Closed,
                }
            }
            WaitOutcome::Commands
        }
    }
}

async fn device_loop(
    session: Arc<SoundcoreSession>,
    state: Arc<SharedState>,
    dev: DeviceConfig,
    mut rx: mpsc::UnboundedReceiver<DeviceCommand>,
) {
    let (mac, model) = match dev.parse() {
        Ok(v) => v,
        Err(err) => {
            set_device_state(&state, MacAddr6::nil(), |s| {
                s.message = format!("config error: {err:#}")
            });
            return;
        }
    };

    // OpenSCQ30 only ships Bluetooth backends for Windows and Linux. On any other OS
    // (e.g. macOS) connecting would panic, so degrade gracefully and park the task.
    if soundcore_protocol::vendor::default_backends().is_none() {
        set_device_state(&state, mac, |s| {
            s.message =
                "This OS has no Bluetooth backend (OpenSCQ30 supports Windows & Linux)".into();
        });
        while rx.recv().await.is_some() {}
        return;
    }

    // Idempotent mac<->model registration.
    let _ = session.pair(mac, model).await;

    let poll = Duration::from_secs(dev.poll_seconds.max(1));
    let apply_delay = Duration::from_secs(dev.apply_delay_seconds);
    let mut backoff = Duration::ZERO;
    let mut pending: Vec<DeviceCommand> = Vec::new();

    loop {
        set_device_state(&state, mac, |s| {
            s.connected = false;
            s.snapshot = None;
            s.message = format!("waiting for {model}...");
        });

        // Connectivity is determined from the OS's connected-device list only.
        // Never send SDP/RFCOMM connect attempts just to find out whether the
        // earbuds are there — wait until Windows reports them connected.
        if !is_device_still_connected(mac).await {
            backoff = Duration::ZERO;
            match wait_for_commands(&mut rx, poll, &mut pending).await {
                WaitOutcome::Closed => return,
                WaitOutcome::Commands | WaitOutcome::Elapsed => continue,
            }
        }

        tracing::debug!("attempting connection to {mac} ({model})");
        match timeout(Duration::from_secs(10), session.connect(mac)).await {
            Ok(Ok(connected)) => {
                let device = connected.inner().clone();
                tracing::debug!("connected to {mac} ({model})");
                backoff = Duration::ZERO;
                // Publish snapshot first so battery data is available when frontend detects connect
                publish_snapshot(device.as_ref(), &state, mac);
                set_device_state(&state, mac, |s| {
                    s.connected = true;
                    s.message = "connected".into();
                });
                if !apply_delay.is_zero() {
                    sleep(apply_delay).await;
                }
                // On connect the device is the source of truth: read its current
                // settings into the synced profile instead of pushing ours down.
                // This never writes a packet to the device; live UI changes do.
                let fetched = fetch_profile_from_device(device.as_ref());
                store_fetched_profile(mac, fetched);
                publish_snapshot(device.as_ref(), &state, mac);
                set_device_state(&state, mac, |s| {
                    s.message = "connected (profile read from device)".into();
                });

                if connected_session(device.as_ref(), &mut rx, &state, mac, &mut pending).await {
                    tracing::debug!("connected_session returned for {mac}, task replaced");
                    return; // channel dropped -> task replaced
                }

                tracing::debug!("device {mac} disconnected, restarting connection loop");
                set_device_state(&state, mac, |s| {
                    s.connected = false;
                    s.snapshot = None;
                    s.message = "disconnected".into();
                });
            }
            Ok(Err(err)) => {
                tracing::warn!("connection to {mac} failed: {err:#}");
                backoff = next_backoff(backoff, CONNECT_BACKOFF_BASE, CONNECT_BACKOFF_CAP);
                match wait_for_commands(&mut rx, poll.max(backoff), &mut pending).await {
                    WaitOutcome::Closed => return,
                    WaitOutcome::Commands => backoff = Duration::ZERO,
                    WaitOutcome::Elapsed => {}
                }
            }
            Err(_) => {
                tracing::debug!("connection to {mac} timed out after 10s");
                backoff = next_backoff(backoff, CONNECT_BACKOFF_BASE, CONNECT_BACKOFF_CAP);
                match wait_for_commands(&mut rx, poll.max(backoff), &mut pending).await {
                    WaitOutcome::Closed => return,
                    WaitOutcome::Commands => backoff = Duration::ZERO,
                    WaitOutcome::Elapsed => {}
                }
            }
        }
    }
}

/// Applies one device command. Returns false if the connection must be treated
/// as dead (connection error while writing).
async fn handle_device_command(
    device: &(dyn OpenSCQ30Device + Send + Sync),
    cmd: DeviceCommand,
    state: &SharedState,
    mac: MacAddr6,
) -> bool {
    match cmd {
        DeviceCommand::SetSetting { id, value } => {
            tracing::debug!("SetSetting {id} for {mac}");
            if let Err(err) = device.set_setting_values(vec![(id, value)]).await {
                warn!("set failed: {err}");
                if matches!(
                    err,
                    soundcore_protocol::vendor::device::Error::ConnectionError { .. }
                ) {
                    tracing::debug!(
                        "connection error during set, treating as disconnect for {mac}"
                    );
                    return false;
                }
                set_device_state(state, mac, |s| s.message = format!("set failed: {err}"));
            }
            publish_snapshot(device, state, mac);
            true
        }
    }
}

/// Returns true if the device command channel closed (task is being replaced).
/// Commands stashed in `pending` while disconnected are applied first.
async fn connected_session(
    device: &(dyn OpenSCQ30Device + Send + Sync),
    rx: &mut mpsc::UnboundedReceiver<DeviceCommand>,
    state: &SharedState,
    mac: MacAddr6,
    pending: &mut Vec<DeviceCommand>,
) -> bool {
    let queued = std::mem::take(pending);
    for cmd in queued {
        if !handle_device_command(device, cmd, state, mac).await {
            return false;
        }
    }
    let mut conn = device.connection_status();
    let mut changes = device.watch_for_changes();
    // Periodic presence check: every 5 s verify the device is still in the OS
    // connected-devices list (AQS cache query, no RFCOMM I/O).
    let mut presence_check = tokio::time::interval(Duration::from_secs(5));
    presence_check.tick().await; // consume first immediate tick
    let mut capture_poll = tokio::time::interval(Duration::from_secs(2));
    capture_poll.tick().await;
    tracing::debug!("entered connected_session for {mac}");
    loop {
        tokio::select! {
            cmd = rx.recv() => match cmd {
                Some(cmd) => {
                    if !handle_device_command(device, cmd, state, mac).await {
                        return false;
                    }
                }
                None => {
                    tracing::debug!("command channel closed for {mac}, exiting connected_session");
                    return true;
                }
            },
            changed = changes.changed() => {
                if changed.is_ok() {
                    tracing::trace!("settings changed for {mac}, publishing snapshot");
                    publish_snapshot(device, state, mac);
                }
            }
            conn_changed = conn.changed() => {
                if conn_changed.is_err() || *conn.borrow_and_update() == ConnectionStatus::Disconnected {
                    tracing::debug!("connection_status channel signaled disconnect for {mac}");
                    return false;
                }
            }
            _ = capture_poll.tick() => {
                if *state.capture_active.lock().unwrap() {
                    let _ = device.request_state().await;
                }
            }
            _ = presence_check.tick() => {
                if is_device_still_connected(mac).await {
                    tracing::trace!("presence check: {mac} still connected");
                    publish_snapshot(device, state, mac);
                } else {
                    tracing::warn!("presence check: {mac} no longer in connected devices list");
                    return false;
                }
            }
        }
    }
}

/// Shared BlueZ session for the cheap D-Bus property reads below (presence +
/// radio power), so repeated checks don't open a new connection each time.
#[cfg(target_os = "linux")]
static SESSION: tokio::sync::OnceCell<bluer::Session> = tokio::sync::OnceCell::const_new();

/// Lightweight check: queries the OS Bluetooth cache (AQS filter) to see if a
/// device with the given MAC address is listed as connected.  No RFCOMM I/O is
/// performed — this only reads from the Windows Bluetooth stack's cached
/// device list.  Used both to gate outgoing connect attempts (`device_loop`)
/// and as the in-session presence check (`connected_session`).
#[cfg(target_os = "windows")]
async fn is_device_still_connected(mac: MacAddr6) -> bool {
    tokio::task::spawn_blocking(move || {
        use windows::{
            Devices::{
                Bluetooth::{BluetoothConnectionStatus, BluetoothDevice},
                Enumeration::DeviceInformation,
            },
            core::HSTRING,
        };

        let connected_filter =
            match BluetoothDevice::GetDeviceSelectorFromConnectionStatus(
                BluetoothConnectionStatus::Connected,
            ) {
                Ok(f) => f,
                Err(err) => {
                    tracing::debug!("failed to get BT device selector for {mac}: {err}");
                    return true;
                }
            };
        let mac_hex = hex::encode(mac);
        let mac_filter = format!(
            "System.DeviceInterface.Bluetooth.DeviceAddress:=\"{mac_hex}\"",
        );
        let filter: HSTRING = format!(
            "{connected_filter} AND {mac_filter} AND System.Devices.Aep.IsPresent:=System.StructuredQueryType.Boolean#True"
        ).into();
        tracing::trace!("presence AQS filter for {mac}: {filter}");

        let collection = match DeviceInformation::FindAllAsyncAqsFilter(&filter) {
            Ok(op) => match op.join() {
                Ok(c) => c,
                Err(err) => {
                    tracing::debug!("FindAllAsync join failed for {mac}: {err}");
                    return true;
                }
            },
            Err(err) => {
                tracing::debug!("FindAllAsync failed for {mac}: {err}");
                return true;
            }
        };
        let count = collection.Size().unwrap_or(0);
        tracing::trace!("presence check for {mac}: {count} matching device(s)");
        count > 0
    })
    .await
    .unwrap_or(true)
}

/// Linux: BlueZ connection state via bluer (D-Bus property reads, no RFCOMM
/// I/O), mirroring the multi-adapter lookup in OpenSCQ30's linux backend.
/// Fail-open: any inability to determine presence returns true so the connect
/// attempt (and its backoff) proceeds; only a definitive "connected on no
/// adapter" returns false.
#[cfg(target_os = "linux")]
async fn is_device_still_connected(mac: MacAddr6) -> bool {
    let session = match SESSION.get_or_try_init(|| bluer::Session::new()).await {
        Ok(session) => session,
        Err(err) => {
            tracing::debug!("bluer session unavailable: {err}");
            return true;
        }
    };
    let adapter_names = match session.adapter_names().await {
        Ok(names) => names,
        Err(err) => {
            tracing::debug!("failed to list bluetooth adapters: {err}");
            return true;
        }
    };
    if adapter_names.is_empty() {
        // No adapters: nothing can be connected anywhere.
        return false;
    }
    for name in adapter_names {
        let adapter = match session.adapter(&name) {
            Ok(adapter) => adapter,
            Err(err) => {
                tracing::debug!("bluetooth adapter {name} unavailable: {err}");
                return true;
            }
        };
        match adapter.device(mac.into()) {
            Ok(device) => match device.is_connected().await {
                Ok(true) => return true,
                Ok(false) => {} // not on this adapter; keep checking the others
                Err(err) => {
                    tracing::debug!("is_connected failed for {mac} on {name}: {err}");
                    return true;
                }
            },
            Err(err) => {
                tracing::debug!("device lookup failed for {mac} on {name}: {err}");
                return true;
            }
        }
    }
    false
}

/// Platforms without a presence check (no Windows AQS, no BlueZ): assume
/// present so connect attempts — bounded by backoff — still happen.
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
async fn is_device_still_connected(_mac: MacAddr6) -> bool {
    true
}

/// Whether the OS reports Bluetooth radio power on. Drives the idle screen's
/// footer dot (green "on" / red "Bluetooth off"). Fail-open: any inability to
/// determine radio state (no adapter, null radio, query errors, `Unknown`
/// state) reports `true` so the UI never falsely claims the radio is off;
/// only a definitive `Off`/`Disabled` radio state is false.
#[cfg(target_os = "windows")]
pub async fn bluetooth_is_on() -> bool {
    tokio::task::spawn_blocking(|| {
        use windows::Devices::{Bluetooth::BluetoothAdapter, Radios::RadioState};

        let adapter = match BluetoothAdapter::GetDefaultAsync() {
            Ok(op) => match op.join() {
                Ok(adapter) => adapter,
                Err(err) => {
                    tracing::debug!("no default Bluetooth adapter: {err}");
                    return true;
                }
            },
            Err(err) => {
                tracing::debug!("GetDefaultAsync failed: {err}");
                return true;
            }
        };
        let radio = match adapter.GetRadioAsync() {
            Ok(op) => match op.join() {
                Ok(radio) => radio,
                Err(err) => {
                    tracing::debug!("GetRadioAsync join failed: {err}");
                    return true;
                }
            },
            Err(err) => {
                tracing::debug!("GetRadioAsync failed: {err}");
                return true;
            }
        };
        match radio.State() {
            Ok(state) if state == RadioState::Off || state == RadioState::Disabled => false,
            Ok(_) => true, // On or Unknown — Unknown fails open
            Err(err) => {
                tracing::debug!("radio State() failed: {err}");
                true
            }
        }
    })
    .await
    .unwrap_or(true)
}

/// Linux: any adapter's BlueZ `Powered` property decides the state — one
/// powered adapter means Bluetooth is on. No adapters is a definitive off.
#[cfg(target_os = "linux")]
pub async fn bluetooth_is_on() -> bool {
    let session = match SESSION.get_or_try_init(|| bluer::Session::new()).await {
        Ok(session) => session,
        Err(err) => {
            tracing::debug!("bluer session unavailable: {err}");
            return true;
        }
    };
    let adapter_names = match session.adapter_names().await {
        Ok(names) => names,
        Err(err) => {
            tracing::debug!("failed to list bluetooth adapters: {err}");
            return true;
        }
    };
    if adapter_names.is_empty() {
        return false;
    }
    for name in adapter_names {
        let adapter = match session.adapter(&name) {
            Ok(adapter) => adapter,
            Err(err) => {
                tracing::debug!("bluetooth adapter {name} unavailable: {err}");
                return true;
            }
        };
        match adapter.is_powered().await {
            Ok(true) => return true,
            Ok(false) => {} // powered off here; check the remaining adapters
            Err(err) => {
                tracing::debug!("is_powered failed for {name}: {err}");
                return true;
            }
        }
    }
    false
}

/// Platforms without a radio-state check: assume on (fail-open, same as the
/// presence gate) so the footer never shows a bogus "Bluetooth off".
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub async fn bluetooth_is_on() -> bool {
    true
}

fn publish_snapshot(
    device: &(dyn OpenSCQ30Device + Send + Sync),
    state: &SharedState,
    mac: MacAddr6,
) {
    let snapshot: Snapshot = device
        .settings_by_category()
        .into_iter()
        .map(|(cat, settings)| (cat, settings.into_iter().collect()))
        .collect();
    let capabilities = soundcore_protocol::DeviceCapabilities::from_device(device);
    set_device_state(state, mac, |s| {
        s.snapshot = Some(snapshot);
        s.capabilities = Some(capabilities);
    });
}

/// One-shot actions (power off, ring, reset, export) that must never end up
/// in the stored profile even if the device reports them as writable: they
/// are commands, not state, and a settings record must never describe — let
/// alone round-trip — something that powers off the buds or fires an export.
const FETCH_SKIP: [SettingId; 4] = [
    SettingId::PowerOff,
    SettingId::FindDevice,
    SettingId::ResetButtonsToDefault,
    SettingId::ExportCustomEqualizerProfiles,
];

/// Serializes a setting's current value into the string form the profile
/// format stores. Must round-trip through `parse_value` so a fetched profile
/// stays an exact, lossless mirror of the device. Returns None for value shapes
/// with no faithful string form (MultiSelectWithRemove lists, hue pickers)
/// and for read-only / write-only settings.
fn setting_to_raw(setting: &Setting) -> Option<String> {
    match setting {
        Setting::Toggle { value } => Some(value.to_string()),
        Setting::I32Range { value, .. } => Some(value.to_string()),
        Setting::Select { value, .. } => Some(value.to_string()),
        Setting::OptionalSelect { value, .. } => Some(value.as_deref().unwrap_or("").to_owned()),
        Setting::ModifiableSelect { value, .. } => value.as_ref().map(|v| v.to_string()),
        Setting::MultiSelect { values, .. } => Some(values.join(",")),
        Setting::Equalizer { value, .. } => Some(
            value
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(","),
        ),
        Setting::PresetEqualizerProfileSelect { value, .. } => {
            value.as_ref().map(|v| v.to_string())
        }
        Setting::TimeOfDay {
            minutes_after_midnight,
        } => Some(minutes_after_midnight.to_string()),
        _ => None,
    }
}

/// Reads the device's current settings and rebuilds the profile from them.
/// Purely read-only: `settings_by_category` serves locally cached state — no
/// packet is sent. Only ReadWrite settings are kept, which excludes
/// Information (battery, serial…) and Action/ImportString commands.
fn fetch_profile_from_device(device: &(dyn OpenSCQ30Device + Send + Sync)) -> Vec<SettingEntry> {
    let mut entries = Vec::new();
    for (_category, settings) in device.settings_by_category() {
        for (id, setting) in settings {
            if FETCH_SKIP.contains(&id) {
                continue;
            }
            if !setting.mode().is_writable() || !setting.mode().is_readable() {
                continue;
            }
            if let Some(raw) = setting_to_raw(&setting) {
                entries.push(SettingEntry {
                    id: id.to_string(),
                    value: raw,
                });
            }
        }
    }
    entries
}

/// Persists connect state for a device to config.toml so UI edits and restarts
/// see device reality: stamps `last_connected` on every successful connect and
/// stores the fetched profile so the idle list sorts newest-first.
/// An empty fetch means the device hadn't reported its state yet — the old
/// profile is kept rather than wiped, but the connect is still recorded.
fn store_fetched_profile(mac: MacAddr6, mut profile: Vec<SettingEntry>) {
    if !profile.is_empty() {
        crate::config::prune_profile(&mut profile);
    }
    let count = profile.len();

    let path = crate::config::resolve_path(None);
    let mut cfg = match Config::load(&path) {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!(
                "could not reload {}: {err:#}; connect recency and profile not persisted",
                path.display()
            );
            return;
        }
    };
    let key = mac.to_string();
    let device = cfg.devices.iter_mut().find(|d| {
        d.mac_address.eq_ignore_ascii_case(&key)
            || MacAddr6::from_str(d.mac_address.trim()).is_ok_and(|m| m == mac)
    });
    match device {
        Some(d) => {
            d.last_connected = Some(now_unix());
            if profile.is_empty() {
                warn!("device {mac} reported no settings yet; keeping existing profile");
            } else {
                d.profile = profile;
            }
            if let Err(err) = cfg.save(&path) {
                warn!(
                    "failed to persist connect state to {}: {err:#}",
                    path.display()
                );
            }
            info!("connect recorded for {mac}: {count} settings stored");
        }
        None => tracing::debug!(
            "device {mac} not present in {}; connect recency not persisted",
            path.display()
        ),
    }
}

/// Current unix time in seconds; 0 if the clock is before the epoch.
fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

async fn scan(session: &SoundcoreSession, state: &SharedState, model: DeviceModel) {
    {
        let mut scanning = state.scanning.lock().unwrap();
        if *scanning {
            tracing::debug!("scan already in flight; skipping overlapping scan");
            return;
        }
        *scanning = true;
    }
    let results = match session.inner().list_devices(model).await {
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
                Ok(Value::ModifiableSelectCommand(
                    ModifiableSelectCommand::Add(rest.to_owned().into()),
                ))
            } else if let Some(rest) = raw.strip_prefix('-') {
                Ok(Value::ModifiableSelectCommand(
                    ModifiableSelectCommand::Remove(rest.to_owned().into()),
                ))
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
        Setting::MultiSelectWithRemove { setting, .. } => {
            if let Some(rest) = raw.strip_prefix('-') {
                Ok(Value::MultiSelectWithRemoveCommand(
                    MultiSelectWithRemoveCommand::Remove(rest.to_owned().into()),
                ))
            } else {
                let name = raw.strip_prefix("\\").unwrap_or(raw);
                Ok(one_of(setting, name)?.into())
            }
        }
        Setting::Equalizer { setting, .. } => {
            let values = raw
                .split(',')
                .map(|s| i16::from_str(s.trim()))
                .collect::<Result<Vec<_>, _>>()?;
            if values.len() != setting.band_hz.len() {
                bail!(
                    "expected {} bands, got {}",
                    setting.band_hz.len(),
                    values.len()
                );
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
        Setting::PresetEqualizerProfileSelect { .. } => {
            Ok(Value::String(std::borrow::Cow::from(raw.to_owned())))
        }
        Setting::HueColorPicker { .. } => Ok(Value::String(std::borrow::Cow::from(raw.to_owned()))),
        Setting::TimeOfDay { .. } => {
            let n = i32::from_str(raw.trim())?;
            if !(0..1440).contains(&n) {
                bail!(
                    "{n} is not a valid time of day (expected minutes after midnight in 0..1440)"
                );
            }
            Ok(Value::I32(n))
        }
    }
}

fn one_of(setting: &Select, raw: &str) -> anyhow::Result<std::borrow::Cow<'static, str>> {
    if let Some(exact) = setting.options.iter().find(|o| **o == raw) {
        return Ok(exact.clone());
    }
    let mut matches = setting
        .options
        .iter()
        .filter(|o| o.eq_ignore_ascii_case(raw));
    let first = matches.next();
    if let (Some(a), Some(b)) = (first, matches.next()) {
        bail!("'{raw}' is ambiguous: could be {a} or {b}");
    }
    first.cloned().ok_or_else(|| {
        anyhow!(
            "'{raw}' is not valid. Expected one of: {:?}",
            setting.options
        )
    })
}

async fn open_session() -> anyhow::Result<SoundcoreSession> {
    SoundcoreSession::new()
        .await
        .map_err(|e| anyhow!("failed to open database: {e}"))
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use soundcore_protocol::vendor::settings::{Equalizer, Range};

    use super::*;

    fn select(options: &[&'static str]) -> Select {
        Select {
            options: options.iter().copied().map(Cow::from).collect(),
            localized_options: options.iter().map(|o| o.to_string()).collect(),
        }
    }

    /// A fetched profile is only a faithful mirror if every stored string
    /// round-trips — `setting_to_raw` must invert `parse_value`.
    #[test]
    fn setting_to_raw_round_trips_through_parse_value() {
        let band_hz: Cow<'static, [u16]> = Cow::Borrowed(&[100, 200]);
        let cases: Vec<(Setting, Value)> = vec![
            (Setting::Toggle { value: true }, Value::Bool(true)),
            (Setting::Toggle { value: false }, Value::Bool(false)),
            (
                Setting::I32Range {
                    setting: Range {
                        range: 0..=10,
                        step: 1,
                    },
                    value: 7,
                },
                Value::I32(7),
            ),
            (
                Setting::Select {
                    setting: select(&["Normal", "Transparency"]),
                    value: Cow::Borrowed("Transparency"),
                },
                Value::String(Cow::Borrowed("Transparency")),
            ),
            (
                Setting::OptionalSelect {
                    setting: select(&["a", "b"]),
                    value: Some(Cow::Borrowed("a")),
                },
                Value::OptionalString(Some(Cow::Borrowed("a"))),
            ),
            (
                Setting::OptionalSelect {
                    setting: select(&["a", "b"]),
                    value: None,
                },
                Value::OptionalString(None),
            ),
            (
                Setting::MultiSelect {
                    setting: select(&["x", "y"]),
                    values: vec![Cow::Borrowed("x"), Cow::Borrowed("y")],
                },
                Value::StringVec(vec![Cow::Borrowed("x"), Cow::Borrowed("y")]),
            ),
            (
                Setting::Equalizer {
                    setting: Equalizer {
                        band_hz: band_hz.clone(),
                        fraction_digits: 0,
                        min: -6,
                        max: 6,
                    },
                    read_only: false,
                    value: vec![6, -4],
                },
                Value::I16Vec(vec![6, -4]),
            ),
            (
                Setting::PresetEqualizerProfileSelect {
                    equalizer: Equalizer {
                        band_hz: band_hz.clone(),
                        fraction_digits: 0,
                        min: -6,
                        max: 6,
                    },
                    select: select(&["Flat"]),
                    presets: vec![vec![0, 0]],
                    value: Some(Cow::Borrowed("Flat")),
                },
                Value::String(Cow::Borrowed("Flat")),
            ),
            (
                Setting::TimeOfDay {
                    minutes_after_midnight: 480,
                },
                Value::I32(480),
            ),
        ];

        for (setting, expected) in cases {
            let raw = setting_to_raw(&setting).expect("representable setting must serialize");
            let parsed = parse_value(&setting, &raw)
                .unwrap_or_else(|err| panic!("raw {raw:?} failed to parse: {err}"));
            assert_eq!(parsed, expected, "raw {raw:?}");
        }
    }

    /// Command-shaped and unrepresentable settings must never be captured.
    #[test]
    fn setting_to_raw_skips_unrepresentable_shapes() {
        assert_eq!(
            setting_to_raw(&Setting::Information {
                value: "v".to_owned(),
                translated_value: "v".to_owned()
            }),
            None
        );
        assert_eq!(setting_to_raw(&Setting::Action), None);
        assert_eq!(
            setting_to_raw(&Setting::ImportString {
                confirmation_message: None
            }),
            None
        );
        assert_eq!(
            setting_to_raw(&Setting::ModifiableSelect {
                setting: select(&["a"]),
                value: None
            }),
            None
        );
        assert_eq!(
            setting_to_raw(&Setting::MultiSelectWithRemove {
                setting: select(&["a"]),
                values: vec![Cow::Borrowed("a")]
            }),
            None
        );
        assert_eq!(
            setting_to_raw(&Setting::HueColorPicker { hue: 200.0 }),
            None
        );
    }

    #[test]
    fn fetched_ids_round_trip_through_setting_id_parser() {
        for id in [
            SettingId::AmbientSoundMode,
            SettingId::NoiseCancelingMode,
            SettingId::VolumeAdjustments,
            SettingId::SpatialAudio,
        ] {
            let raw = id.to_string();
            assert_eq!(SettingId::from_str(&raw).ok(), Some(id), "raw {raw:?}");
        }
    }

    #[test]
    fn fetch_skip_ids_parse_from_their_string_form() {
        for id in FETCH_SKIP {
            assert!(SettingId::from_str(&id.to_string()).is_ok());
        }
    }

    fn device_cfg(mac: &str, model: &str, poll_seconds: u64) -> DeviceConfig {
        DeviceConfig {
            mac_address: mac.to_owned(),
            model: model.to_owned(),
            poll_seconds,
            ..DeviceConfig::default()
        }
    }

    #[test]
    fn next_backoff_ramps_from_base_to_cap() {
        let base = Duration::from_secs(5);
        let cap = Duration::from_secs(60);
        let mut backoff = Duration::ZERO;
        let expected = [5u64, 10, 20, 40, 60, 60, 60];
        for want in expected {
            backoff = next_backoff(backoff, base, cap);
            assert_eq!(backoff, Duration::from_secs(want));
        }
    }

    #[test]
    fn next_backoff_clamps_base_and_doubling_to_cap() {
        assert_eq!(
            next_backoff(
                Duration::ZERO,
                Duration::from_secs(30),
                Duration::from_secs(10)
            ),
            Duration::from_secs(10),
            "base must clamp down to cap"
        );
        assert_eq!(
            next_backoff(
                Duration::from_secs(40),
                Duration::from_secs(5),
                Duration::from_secs(45)
            ),
            Duration::from_secs(45),
            "doubling must clamp down to cap"
        );
    }

    #[test]
    fn diff_tasks_unchanged_config_touches_nothing() {
        let old = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5)];
        let new = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5)];
        let (abort, spawn) = diff_tasks(&old, &new);
        assert!(abort.is_empty());
        assert!(spawn.is_empty());
    }

    #[test]
    fn diff_tasks_spawns_only_added_device() {
        let old = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5)];
        let new = [
            device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5),
            device_cfg("00:11:22:33:44:66", "SoundcoreA3947", 5),
        ];
        let (abort, spawn) = diff_tasks(&old, &new);
        assert!(abort.is_empty());
        assert_eq!(spawn, vec![1]);
    }

    #[test]
    fn diff_tasks_aborts_only_removed_device() {
        let old = [
            device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5),
            device_cfg("00:11:22:33:44:66", "SoundcoreA3947", 5),
        ];
        let new = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5)];
        let (abort, spawn) = diff_tasks(&old, &new);
        assert_eq!(abort.len(), 1);
        assert_eq!(abort[0].to_string(), "00:11:22:33:44:66");
        assert!(spawn.is_empty());
    }

    #[test]
    fn diff_tasks_restarts_on_poll_change() {
        let old = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5)];
        let new = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 30)];
        let (abort, spawn) = diff_tasks(&old, &new);
        assert_eq!(abort.len(), 1);
        assert_eq!(spawn, vec![0]);
    }

    #[test]
    fn diff_tasks_restarts_on_model_change() {
        let old = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5)];
        let new = [device_cfg("00:11:22:33:44:55", "SoundcoreA3947", 5)];
        let (abort, spawn) = diff_tasks(&old, &new);
        assert_eq!(abort.len(), 1);
        assert_eq!(spawn, vec![0]);
    }

    #[test]
    fn diff_tasks_ignores_ui_only_fields() {
        let old = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5)];
        let mut new = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5)];
        new[0].name = "Renamed".to_owned();
        new[0].profile = vec![crate::config::SettingEntry {
            id: "gamingMode".into(),
            value: "true".into(),
        }];
        new[0].image = Some("https://example.invalid/img.png".into());
        new[0].color = Some("#123456".into());
        let (abort, spawn) = diff_tasks(&old, &new);
        assert!(abort.is_empty(), "ui-only edits must not abort tasks");
        assert!(spawn.is_empty(), "ui-only edits must not respawn tasks");
    }

    #[test]
    fn diff_tasks_duplicate_mac_is_first_wins() {
        let old = [device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5)];

        // First entry unchanged, second differs: the second is ignored.
        let keep = [
            device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5),
            device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 99),
        ];
        let (abort, spawn) = diff_tasks(&old, &keep);
        assert!(abort.is_empty());
        assert!(spawn.is_empty());

        // First entry changed: restart, second still ignored.
        let restart = [
            device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 30),
            device_cfg("00:11:22:33:44:55", "SoundcoreA3959", 5),
        ];
        let (abort, spawn) = diff_tasks(&old, &restart);
        assert_eq!(abort.len(), 1);
        assert_eq!(spawn, vec![0]);
    }

    #[test]
    fn diff_tasks_skips_unparseable_entries() {
        let old: [DeviceConfig; 0] = [];
        let new = [
            device_cfg("not-a-mac", "SoundcoreA3959", 5),
            device_cfg("00:11:22:33:44:77", "NotASoundcoreModel", 5),
        ];
        let (abort, spawn) = diff_tasks(&old, &new);
        assert!(abort.is_empty());
        assert!(
            spawn.is_empty(),
            "entries with no fingerprint must never spawn"
        );
    }

    #[test]
    fn infer_model_recognizes_liberty_family_and_raw_codes() {
        let cases: Vec<(&str, Option<DeviceModel>)> = vec![
            ("soundcore Liberty 4 Max", Some(DeviceModel::SoundcoreA3954)),
            ("Soundcore Liberty 4", Some(DeviceModel::SoundcoreA3953)),
            ("Soundcore Liberty 4 NC", Some(DeviceModel::SoundcoreA3947)),
            ("Soundcore Liberty 4 Pro", Some(DeviceModel::SoundcoreA3954)),
            ("Soundcore Liberty 5", Some(DeviceModel::SoundcoreA3957)),
            ("soundcore R60i NC", Some(DeviceModel::SoundcoreD1202C)),
            ("A3953 serial 123", Some(DeviceModel::SoundcoreA3953)),
            ("D1202C", Some(DeviceModel::SoundcoreD1202C)),
            ("unknown gadget", None),
        ];
        for (name, expected) in cases {
            assert_eq!(infer_model(name), expected, "name {name:?}");
        }
    }

    #[test]
    fn infer_model_recognizes_every_raw_model_code() {
        for model in DeviceModel::VARIANTS {
            if *model == DeviceModel::SoundcoreDevelopment {
                continue;
            }
            let display = model.to_string();
            let code = display.strip_prefix("Soundcore").unwrap_or(&display);
            let name = format!("Soundcore device {code}");
            assert_eq!(infer_model(&name), Some(*model), "name {name:?}");
        }
    }

    #[test]
    fn infer_model_recognizes_added_human_names() {
        let cases: Vec<(&str, DeviceModel)> = vec![
            ("Soundcore Space 2", DeviceModel::SoundcoreD1402),
            ("Soundcore AeroClip", DeviceModel::SoundcoreA3388),
            ("Soundcore Sport X20", DeviceModel::SoundcoreA3968),
            ("Soundcore Sleep A30", DeviceModel::SoundcoreD1301),
            ("Soundcore P31i", DeviceModel::SoundcoreD1202),
            ("soundcore Liberty 5 Pro Max", DeviceModel::SoundcoreD1204),
            ("Soundcore Liberty 5 Pro", DeviceModel::SoundcoreD1204),
        ];
        for (name, expected) in cases {
            assert_eq!(infer_model(name), Some(expected), "name {name:?}");
        }
    }
}
