// Hide the console window in release builds (it's a tray app). Keep it in debug for logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! SoundCore-Desktop — lives in the system tray, applies a saved profile to each
//! configured device the instant it connects. Left-click the tray icon for a small config
//! popup; there is no main window. Supports any device OpenSCQ30 supports (Windows & Linux).

mod autostart;
mod config;
mod worker;

use std::{borrow::Cow, path::PathBuf, str::FromStr, time::Duration};

use config::{Config, DeviceConfig, SettingEntry};
use eframe::egui;
use macaddr::MacAddr6;
use openscq30_lib::{
    DeviceModel,
    settings::{Setting, SettingId, Value},
};
use strum::VariantArray;
use tray_icon::{
    TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};
use worker::{Command, WorkerHandle};

const WIN_W: f32 = 420.0;
const WIN_H: f32 = 560.0;

fn main() -> eframe::Result {
    init_logging();

    let config_path = config::resolve_path(std::env::args().nth(1));
    let config = Config::load(&config_path).unwrap_or_else(|err| {
        tracing::warn!("could not load {}: {err:#}; using empty config", config_path.display());
        Config::default()
    });

    // Make the Windows "run at startup" entry match the saved preference.
    autostart::reconcile(config.autostart);

    let worker = worker::spawn(config.clone());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WIN_W, WIN_H])
            .with_decorations(false)
            .with_resizable(false)
            .with_always_on_top()
            .with_taskbar(false)
            .with_transparent(true)
            .with_visible(false),
        ..Default::default()
    };

    eframe::run_native(
        "SoundCore-Desktop",
        options,
        Box::new(move |cc| Ok(Box::new(App::new(cc, config, config_path, worker)))),
    )
}

enum PostAction {
    None,
    Hide,
    Quit,
}

struct App {
    config: Config,
    config_path: PathBuf,
    worker: WorkerHandle,
    selected: usize,

    _tray: TrayIcon,
    menu_settings_id: tray_icon::menu::MenuId,
    menu_apply_id: tray_icon::menu::MenuId,
    menu_quit_id: tray_icon::menu::MenuId,

    visible: bool,
    had_focus: bool,
    needs_position: bool,
    toast: Option<(String, bool)>,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>, config: Config, config_path: PathBuf, worker: WorkerHandle) -> Self {
        let menu = Menu::new();
        let settings_item = MenuItem::new("Settings", true, None);
        let apply_item = MenuItem::new("Apply now (selected)", true, None);
        let quit_item = MenuItem::new("Quit", true, None);
        menu.append(&settings_item).ok();
        menu.append(&apply_item).ok();
        menu.append(&PredefinedMenuItem::separator()).ok();
        menu.append(&quit_item).ok();

        let tray = TrayIconBuilder::new()
            .with_tooltip("SoundCore-Desktop")
            .with_icon(build_icon())
            .with_menu(Box::new(menu))
            // Left-click should open our popup, not the menu. The menu stays on right-click.
            .with_menu_on_left_click(false)
            .build()
            .expect("failed to create tray icon");

        Self {
            config,
            config_path,
            worker,
            selected: 0,
            _tray: tray,
            menu_settings_id: settings_item.id().clone(),
            menu_apply_id: apply_item.id().clone(),
            menu_quit_id: quit_item.id().clone(),
            visible: false,
            had_focus: false,
            needs_position: true,
            toast: None,
        }
    }

    fn show_window(&mut self, ctx: &egui::Context) {
        self.visible = true;
        self.had_focus = false;
        self.needs_position = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    fn hide_window(&mut self, ctx: &egui::Context) {
        self.visible = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }

    fn quit(&mut self, ctx: &egui::Context) {
        let _ = self.worker.tx.send(Command::Quit);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn selected_mac(&self) -> Option<MacAddr6> {
        self.config
            .devices
            .get(self.selected)
            .and_then(|d| MacAddr6::from_str(d.mac_address.trim()).ok())
    }

    fn save_and_apply(&mut self) {
        match self.config.save(&self.config_path) {
            Ok(()) => {
                let _ = self.worker.tx.send(Command::UpdateConfig(self.config.clone()));
                self.toast = Some(("Saved.".to_owned(), false));
            }
            Err(err) => self.toast = Some((format!("Save failed: {err:#}"), true)),
        }
    }

    fn reposition(&mut self, ctx: &egui::Context) {
        if !self.needs_position {
            return;
        }
        if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
            let (margin, taskbar) = (12.0, 48.0);
            let x = (monitor.x - WIN_W - margin).max(0.0);
            let y = (monitor.y - WIN_H - margin - taskbar).max(0.0);
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(x, y)));
            self.needs_position = false;
        }
    }

    fn poll_tray_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.menu_settings_id {
                if self.visible { self.hide_window(ctx); } else { self.show_window(ctx); }
            } else if event.id == self.menu_apply_id {
                if let Some(mac) = self.selected_mac() {
                    let _ = self.worker.tx.send(Command::ApplyNow(mac));
                }
            } else if event.id == self.menu_quit_id {
                self.quit(ctx);
            }
        }
        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button: tray_icon::MouseButton::Left,
                button_state: tray_icon::MouseButtonState::Up,
                ..
            } = event
            {
                if self.visible { self.hide_window(ctx); } else { self.show_window(ctx); }
            }
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _v: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(200));
        self.poll_tray_events(ctx);
        if !self.visible {
            return;
        }
        self.reposition(ctx);

        let focused = ctx.input(|i| i.viewport().focused).unwrap_or(true);
        if focused {
            self.had_focus = true;
        } else if self.had_focus {
            self.hide_window(ctx);
            return;
        }

        // Clamp selection.
        if self.selected >= self.config.devices.len() {
            self.selected = self.config.devices.len().saturating_sub(1);
        }

        let tx = self.worker.tx.clone();
        let mac = self.selected_mac();
        // Read the live state for the selected device.
        let (connected, status_msg, snapshot) = match mac {
            Some(m) => {
                let map = self.worker.state.devices.lock().unwrap();
                map.get(&m)
                    .map(|d| (d.connected, d.message.clone(), d.snapshot.clone()))
                    .unwrap_or((false, "waiting...".into(), None))
            }
            None => (false, "set a MAC and model".into(), None),
        };
        let scan_results = self.worker.state.scan_results.lock().unwrap().clone();
        let scanning = *self.worker.state.scanning.lock().unwrap();

        let mut post = PostAction::None;
        let mut save_now = false;

        egui::CentralPanel::default()
            .frame(egui::Frame::window(&ctx.style()))
            .show(ctx, |ui| {
                // ---- header ----
                ui.horizontal(|ui| {
                    let (color, dot) = if connected {
                        (egui::Color32::from_rgb(76, 175, 80), "●")
                    } else {
                        (egui::Color32::from_rgb(158, 158, 158), "○")
                    };
                    ui.colored_label(color, dot);
                    ui.heading("SoundCore-Desktop");
                });

                // ---- device picker ----
                ui.horizontal(|ui| {
                    let selected_label = self
                        .config
                        .devices
                        .get(self.selected)
                        .map(|d| d.label())
                        .unwrap_or_else(|| "(no devices)".to_owned());
                    egui::ComboBox::from_id_salt("device_picker")
                        .selected_text(selected_label)
                        .show_ui(ui, |ui| {
                            for (i, d) in self.config.devices.iter().enumerate() {
                                ui.selectable_value(&mut self.selected, i, d.label());
                            }
                        });
                    if ui.button("＋").on_hover_text("Add device").clicked() {
                        self.config.devices.push(DeviceConfig::default());
                        self.selected = self.config.devices.len() - 1;
                    }
                    if !self.config.devices.is_empty()
                        && ui.button("🗑").on_hover_text("Remove this device").clicked()
                    {
                        self.config.devices.remove(self.selected);
                        save_now = true;
                    }
                });

                ui.label(egui::RichText::new(&status_msg).small().weak());
                ui.separator();

                if self.config.devices.is_empty() {
                    ui.label("No devices yet. Click ＋ to add one.");
                } else {
                    let sel = self.selected.min(self.config.devices.len() - 1);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        render_device(ui, &mut self.config.devices[sel], mac, connected, &snapshot, &tx, &scan_results, scanning);
                    });
                }

                ui.separator();
                if ui
                    .checkbox(&mut self.config.autostart, "Run at startup")
                    .changed()
                {
                    autostart::reconcile(self.config.autostart);
                    save_now = true;
                }
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        save_now = true;
                    }
                    if let Some(m) = mac {
                        if ui.button("Apply now").clicked() {
                            let _ = tx.send(Command::ApplyNow(m));
                        }
                        if ui.button("Scan").clicked() {
                            if let Ok((_, model)) = self.config.devices[self.selected.min(self.config.devices.len().saturating_sub(1))].parse() {
                                let _ = tx.send(Command::Scan { model });
                            }
                        }
                    }
                    if ui.button("Hide").clicked() {
                        post = PostAction::Hide;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Quit").clicked() {
                            post = PostAction::Quit;
                        }
                    });
                });

                if let Some((msg, is_err)) = &self.toast {
                    let color = if *is_err {
                        egui::Color32::from_rgb(229, 57, 53)
                    } else {
                        egui::Color32::from_rgb(76, 175, 80)
                    };
                    ui.colored_label(color, msg);
                }
            });

        if save_now {
            self.save_and_apply();
        }
        match post {
            PostAction::Hide => self.hide_window(ctx),
            PostAction::Quit => self.quit(ctx),
            PostAction::None => {}
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render_device(
    ui: &mut egui::Ui,
    dev: &mut DeviceConfig,
    mac: Option<MacAddr6>,
    connected: bool,
    snapshot: &Option<worker::Snapshot>,
    tx: &tokio::sync::mpsc::UnboundedSender<Command>,
    scan_results: &[worker::ScanResult],
    scanning: bool,
) {
    egui::Grid::new("device_fields").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
        ui.label("Name");
        ui.add(egui::TextEdit::singleline(&mut dev.name).desired_width(240.0));
        ui.end_row();

        ui.label("MAC");
        ui.add(egui::TextEdit::singleline(&mut dev.mac_address).desired_width(240.0));
        ui.end_row();

        ui.label("Model");
        egui::ComboBox::from_id_salt("model")
            .selected_text(dev.model.clone())
            .width(240.0)
            .show_ui(ui, |ui| {
                for m in DeviceModel::VARIANTS {
                    let s = m.to_string();
                    ui.selectable_value(&mut dev.model, s.clone(), s);
                }
            });
        ui.end_row();

        ui.label("Poll (s)");
        ui.add(egui::DragValue::new(&mut dev.poll_seconds).range(1..=120));
        ui.end_row();

        ui.label("Delay (s)");
        ui.add(egui::DragValue::new(&mut dev.apply_delay_seconds).range(0..=30));
        ui.end_row();
    });

    // Scan results (click to fill MAC).
    if scanning {
        ui.label(egui::RichText::new("scanning...").small().weak());
    } else if !scan_results.is_empty() {
        ui.add_space(4.0);
        ui.label(egui::RichText::new("Connected Bluetooth devices:").small());
        for r in scan_results {
            if ui.small_button(format!("{}  ({})", r.name, r.mac_address)).clicked() {
                dev.mac_address = r.mac_address.clone();
            }
        }
    }

    ui.add_space(8.0);

    // ---- live controls or fallback profile editor ----
    if connected {
        if let Some(snapshot) = snapshot {
            ui.strong("Settings (★ = apply on connect)");
            ui.add_space(2.0);
            for (category, settings) in snapshot {
                egui::CollapsingHeader::new(category.to_string())
                    .default_open(true)
                    .show(ui, |ui| {
                        for (id, setting) in settings {
                            render_setting_row(ui, dev, mac, id, setting, tx);
                        }
                    });
            }
            return;
        }
        ui.label(egui::RichText::new("loading settings...").weak());
        return;
    }

    // Not connected: edit the saved profile as id/value rows.
    ui.strong("Profile (applied on connect)");
    ui.label(egui::RichText::new("Connect the device to edit settings with live controls.").small().weak());
    ui.add_space(2.0);
    let mut remove = None;
    for (i, entry) in dev.profile.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut entry.id).hint_text("settingId").desired_width(150.0));
            ui.add(egui::TextEdit::singleline(&mut entry.value).hint_text("value").desired_width(150.0));
            if ui.small_button("✕").clicked() {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        dev.profile.remove(i);
    }
    if ui.button("+ Add setting").clicked() {
        dev.profile.push(SettingEntry { id: String::new(), value: String::new() });
    }
}

fn render_setting_row(
    ui: &mut egui::Ui,
    dev: &mut DeviceConfig,
    mac: Option<MacAddr6>,
    id: &SettingId,
    setting: &Setting,
    tx: &tokio::sync::mpsc::UnboundedSender<Command>,
) {
    let id_str = id.to_string();
    ui.horizontal(|ui| {
        // "apply on connect" star
        let mut auto = dev.profile.iter().any(|e| e.id == id_str);
        if ui.checkbox(&mut auto, "").on_hover_text("apply this on connect").changed() {
            if auto {
                dev.profile.push(SettingEntry {
                    id: id_str.clone(),
                    value: worker::current_value_string(setting),
                });
            } else {
                dev.profile.retain(|e| e.id != id_str);
            }
        }
        ui.label(&id_str);

        if let Some(value) = setting_widget(ui, id, setting) {
            if let Some(m) = mac {
                let _ = tx.send(Command::SetSetting { mac: m, id: *id, value: value.clone() });
            }
            // Keep the saved profile in sync if this setting is marked auto.
            if let Some(entry) = dev.profile.iter_mut().find(|e| e.id == id_str) {
                entry.value = worker::value_to_profile_string(&value);
            }
        }
    });
}

/// Renders the proper widget for a setting; returns Some(Value) when the user changes it.
fn setting_widget(ui: &mut egui::Ui, id: &SettingId, setting: &Setting) -> Option<Value> {
    match setting {
        Setting::Toggle { value } => {
            let mut v = *value;
            if ui.checkbox(&mut v, "").changed() {
                Some(Value::from(v))
            } else {
                None
            }
        }
        Setting::I32Range { setting, value } => {
            let mut v = *value;
            let resp = ui.add(egui::Slider::new(&mut v, *setting.range.start()..=*setting.range.end()));
            (resp.drag_stopped() || (resp.changed() && !resp.dragged())).then_some(v.into())
        }
        Setting::Select { setting: select, value } => {
            select_combo(ui, id, &select.options, &select.localized_options, value)
                .map(Value::String)
        }
        Setting::OptionalSelect { setting: select, value }
        | Setting::ModifiableSelect { setting: select, value } => {
            let current = value.clone().unwrap_or(Cow::Borrowed(""));
            let mut result = None;
            let label = localized_for(&current, &select.options, &select.localized_options);
            egui::ComboBox::from_id_salt(("optsel", id))
                .selected_text(label)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(value.is_none(), "(none)").clicked() {
                        result = Some(Value::OptionalString(None));
                    }
                    for (opt, loc) in select.options.iter().zip(select.localized_options.iter()) {
                        if ui.selectable_label(Some(opt) == value.as_ref(), loc).clicked() {
                            result = Some(Value::OptionalString(Some(opt.clone())));
                        }
                    }
                });
            result
        }
        Setting::MultiSelect { setting: select, values } => {
            let mut new_values = values.clone();
            let mut changed = false;
            for (opt, loc) in select.options.iter().zip(select.localized_options.iter()) {
                let mut checked = new_values.contains(opt);
                if ui.checkbox(&mut checked, loc).changed() {
                    if checked {
                        new_values.push(opt.clone());
                    } else {
                        new_values.retain(|v| v != opt);
                    }
                    changed = true;
                }
            }
            changed.then(|| Value::from(new_values))
        }
        Setting::Equalizer { setting: eq, value } => {
            let mut bands = value.clone();
            if bands.len() != eq.band_hz.len() {
                bands.resize(eq.band_hz.len(), 0);
            }
            let mut changed = false;
            ui.vertical(|ui| {
                for (i, hz) in eq.band_hz.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{hz} Hz"));
                        let resp = ui.add(egui::Slider::new(&mut bands[i], eq.min..=eq.max));
                        if resp.drag_stopped() || (resp.changed() && !resp.dragged()) {
                            changed = true;
                        }
                    });
                }
            });
            changed.then_some(Value::I16Vec(bands))
        }
        Setting::Information { value, .. } => {
            ui.label(egui::RichText::new(value).weak());
            None
        }
        Setting::Action => {
            if ui.button("Run").clicked() {
                Some(Value::Bool(true))
            } else {
                None
            }
        }
        Setting::ImportString { .. } => {
            ui.label(egui::RichText::new("(import via config)").weak());
            None
        }
    }
}

fn select_combo(
    ui: &mut egui::Ui,
    id: &SettingId,
    options: &[Cow<'static, str>],
    localized: &[String],
    value: &Cow<'static, str>,
) -> Option<Cow<'static, str>> {
    let mut result = None;
    egui::ComboBox::from_id_salt(("sel", id))
        .selected_text(localized_for(value, options, localized))
        .show_ui(ui, |ui| {
            for (opt, loc) in options.iter().zip(localized.iter()) {
                if ui.selectable_label(opt == value, loc).clicked() {
                    result = Some(opt.clone());
                }
            }
        });
    result
}

fn localized_for(value: &str, options: &[Cow<'static, str>], localized: &[String]) -> String {
    options
        .iter()
        .position(|o| o == value)
        .and_then(|i| localized.get(i))
        .cloned()
        .unwrap_or_else(|| value.to_owned())
}

/// A simple 32x32 RGBA tray icon: a filled rounded teal square.
fn build_icon() -> tray_icon::Icon {
    const SIZE: u32 = 32;
    let mut rgba = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = ((y * SIZE + x) * 4) as usize;
            let (cx, cy) = (x as f32 - 15.5, y as f32 - 15.5);
            let edge = cx.abs().max(cy.abs());
            if edge <= 14.0 || (cx.hypot(cy) <= 15.5 && edge <= 15.0) {
                rgba[idx] = 0x14;
                rgba[idx + 1] = 0xB8;
                rgba[idx + 2] = 0xA6;
                rgba[idx + 3] = 0xFF;
            }
        }
    }
    tray_icon::Icon::from_rgba(rgba, SIZE, SIZE).expect("failed to build icon")
}

fn init_logging() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).with_target(false).init();
}
