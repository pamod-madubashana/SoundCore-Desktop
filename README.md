# [SoundCore-Desktop][repo] – Auto profiles for Soundcore headphones

This is the complete source code and the build instructions for **SoundCore-Desktop**, a
tiny system-tray app that automatically applies your saved profile — gaming mode, ambient
sound mode, equalizer, anything — to a [Soundcore][soundcore] device the moment it connects
to your computer. No phone, no official app. Built on the [OpenSCQ30][openscq30] library and
its reverse-engineered MTProto-style device protocol over Bluetooth RFCOMM.

[![Version](https://img.shields.io/badge/version-0.1.0-blue)][repo]
[![Windows](https://img.shields.io/badge/Windows-supported-46A75A?logo=windows&logoColor=white)][openscq30]
[![Linux](https://img.shields.io/badge/Linux-supported-46A75A?logo=linux&logoColor=white)][openscq30]
[![macOS](https://img.shields.io/badge/macOS-unsupported-lightgrey?logo=apple&logoColor=white)][openscq30]
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-DEA584?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Powered by OpenSCQ30](https://img.shields.io/badge/Powered%20by-OpenSCQ30-46A75A)][openscq30]

---

## Why

Devices like the Soundcore R50i NC store almost nothing on-device — every setting lives in
the phone app, which keeps resetting. There's no official desktop app. Previously the only
way to get gaming mode on the PC was: connect the buds to the phone → set gaming mode →
reconnect to the computer → play. SoundCore-Desktop removes that dance entirely.

## How it works

It lives in the **system tray** — there is no main window. A background worker watches each
configured device; the tray popup is the UI.

```
worker (per device):
  try to connect over RFCOMM   ──►  fails while away  ──►  retry every few seconds
        │ success (= device just connected)
        ├─ wait apply_delay_seconds
        ├─ push the device's profile in one batched write
        └─ stay until disconnect (also re-applies on "Apply now" / when you save)
```

A successful RFCOMM connection only happens while the device is actually connected, so
"connected" *is* the connect event — no polling hacks. When connected, the popup renders
the device's **real settings as live controls** (toggles, dropdowns, sliders, EQ bands)
pulled straight from OpenSCQ30, so it works for **every device OpenSCQ30 supports**.

### Modules

| file | responsibility |
| --- | --- |
| [`src/config.rs`](src/config.rs)   | multi-device config + load/save `config.toml` |
| [`src/worker.rs`](src/worker.rs)   | per-device connect/apply loop, live snapshots, value parsing |
| [`src/autostart.rs`](src/autostart.rs) | run-at-startup (Windows registry / Linux .desktop / macOS LaunchAgent) |
| [`src/main.rs`](src/main.rs)       | eframe/egui tray app + bottom-right popup |

## Platform support

| OS | Status | Notes |
| --- | --- | --- |
| **Windows** | ✅ Supported | RFCOMM via the WinRT Bluetooth APIs |
| **Linux**   | ✅ Supported | RFCOMM via BlueZ/`bluer` (needs the system libraries below) |
| **macOS**   | ⚠️ Builds, no Bluetooth | OpenSCQ30 ships no macOS backend; the app runs and shows a clear "no Bluetooth backend" message instead of connecting |

## Build

Needs the stable Rust toolchain.

**Windows**
```powershell
cargo build --release
# -> target\release\soundcore-desktop.exe
```

**Linux** — install the BlueZ/D-Bus headers and the tray dependencies first:
```bash
# Debian/Ubuntu
sudo apt install libdbus-1-dev pkg-config libgtk-3-dev libxdo-dev libayatana-appindicator3-dev
cargo build --release
# -> target/release/soundcore-desktop
```

**macOS** (builds, but can't connect — see above):
```bash
cargo build --release
```

## Configure

Easiest: run the app, left-click the tray icon, and use the popup — add a device (＋), pick
the model, **Scan** to fill the MAC from connected Bluetooth devices, then connect the
device and flip the live controls. Tick the **★** next to any setting to re-apply it on
every connect. **Save** writes `config.toml`.

Or edit [`config.toml`](config.toml) directly — one `[[devices]]` block per device, each
with its own `[[devices.profile]]` entries applied in order on connect:

```toml
autostart = true             # run at logon (also toggled from the popup)

[[devices]]
name = "R50i NC"
mac_address = "34:09:C9:B9:EC:30"
model = "SoundcoreA3959"      # any model OpenSCQ30 supports
poll_seconds = 5
apply_delay_seconds = 2

[[devices.profile]]
id = "gamingMode"             # "true" / "false" (low-latency; needs recent firmware)
value = "true"

[[devices.profile]]
id = "ambientSoundMode"       # "Normal" / "Transparency" / "NoiseCanceling"
value = "Normal"
```

You don't need to memorize setting ids — connect the device and the real controls appear in
the popup. (The equalizer is `volumeAdjustments`, comma-separated bands in tenths of a dB.)

## Run

Put `config.toml` next to the binary (or pass a path as the first argument) and launch it. A
tray icon appears; left-click it for the config popup, right-click for **Apply now** / **Quit**.
Release builds have no console window — for logs, run the debug build or set `RUST_LOG=debug`.

### Run at startup

On by default. The app registers a per-user autostart entry (no admin/root needed):

- **Windows** — `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- **Linux** — `~/.config/autostart/soundcore-desktop.desktop`
- **macOS** — `~/Library/LaunchAgents/com.soundcore.desktop.plist`

Toggle it any time with the **"Run at startup"** checkbox in the popup; the choice is saved
to `config.toml` and reconciled to the OS on each launch.

## Notes

- "Pairing" here is OpenSCQ30's own mac↔model association (stored in its sqlite db under your
  config dir); the app sets it up automatically. It's separate from Bluetooth pairing, which
  you do once in your OS settings.
- "does not expose 'gamingMode' right now" usually means the device's firmware is too old for
  that feature, or it's temporarily unavailable.

## Credits

Built on [OpenSCQ30][openscq30] by Oppzippy — all device protocol support comes from there.

[repo]: https://github.com/pamod/SoundCore-Desktop
[soundcore]: https://www.soundcore.com
[openscq30]: https://github.com/Oppzippy/OpenSCQ30
