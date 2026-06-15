# Soundcore Auto-Mode

A small standalone Rust **system-tray app** that **automatically applies a saved profile
to your Soundcore R50i NC the moment it connects to this PC** — gaming mode on, your
ambient sound mode, your EQ — with no phone and no official app involved.

It has **no main window**: it lives in the tray. Left-click the tray icon (or its
"Settings" menu) for a small popup in the bottom-right corner where you can edit the
profile; right-click for Apply now / Quit.

It links the [OpenSCQ30](../OpenSCQ30) library directly and talks to the buds over
**RFCOMM (Bluetooth Classic)**, the same link used for audio. So once the buds are
connected to the PC as a normal Bluetooth audio device, the profile is pushed straight
to them.

## Why

The R50i NC stores almost nothing on-device; everything lives in the phone app, which
keeps resetting. There's no official PC app. Previously the only way to get gaming mode
on the PC was: connect to phone → set gaming mode → reconnect to laptop → play. This
removes that entirely.

## How it works

A background worker thread runs the connect loop; the tray + popup is the UI on the main
thread. They talk over a command channel + shared status.

```
worker loop:
  try to connect over RFCOMM   ──►  fails while buds are away  ──►  retry every few sec
        │ success (= buds just connected)
        ├─ wait apply_delay_seconds
        ├─ push the profile from config.toml in one batched write
        └─ wait until the device disconnects, then go back to polling
           (also re-applies on "Apply now" / after you save changes)
```

A successful RFCOMM connection only happens while the buds are actually connected to the
PC, so "connected successfully" *is* the connect event — no Bluetooth polling hacks
needed.

### Modules

- `config.rs` — the `Config` (device + profile), load/save `config.toml`.
- `worker.rs` — the connect/apply loop and value parsing; driven by `Command`s.
- `main.rs` — the eframe/egui tray app and the bottom-right popup editor.

## Build

Needs the Rust toolchain (stable). From this folder:

```powershell
cargo build --release
```

The binary is produced at `target\release\soundcore-auto-mode.exe`.

## Configure

Easiest: run the app, left-click the tray icon, and use the popup — add a device (＋),
pick the model, **Scan** to fill the MAC from connected Bluetooth devices, then connect
the device and flip the live controls. Tick the **★** next to any setting to re-apply it
on every connect. **Save** writes `config.toml`.

Or edit [`config.toml`](config.toml) directly. One `[[devices]]` block per device, each
with its own `[[devices.profile]]` entries applied in order on connect:

```toml
autostart = true            # run at logon (also toggled from the popup)

[[devices]]
name = "R50i NC"
mac_address = "34:09:C9:B9:EC:30"
model = "SoundcoreA3959"     # see OpenSCQ30 list-models for ids
poll_seconds = 5
apply_delay_seconds = 2

[[devices.profile]]
id = "gamingMode"            # "true" / "false" (low-latency; needs recent firmware)
value = "true"

[[devices.profile]]
id = "ambientSoundMode"      # "Normal" / "Transparency" / "NoiseCanceling"
value = "Normal"
```

Works for **any device OpenSCQ30 supports** — you don't need to memorize setting ids;
connect the device and the real controls appear in the popup. Equalizer is
`volumeAdjustments`, a comma-separated list of bands in tenths of a dB.

## Run

Put `config.toml` next to the exe (or pass a path as the first argument), then:

```powershell
.\target\release\soundcore-auto-mode.exe
```

A tray icon appears (teal square). Left-click it for the config popup; right-click for
Apply now / Quit. The release build has no console window; for logs run the debug build
(`cargo run`) or set `RUST_LOG=debug`.

### Run at startup

On by default — the app registers itself under the per-user `Run` key
(`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, no admin needed) so it starts at
logon. Toggle it any time with the **"Run at startup"** checkbox in the popup; the choice
is saved to `config.toml` and reconciled to the registry on each launch.

## Notes

- Pairing here means OpenSCQ30's own mac↔model association (stored in its sqlite db under
  your config dir); the program sets it up automatically. It's separate from Bluetooth
  pairing, which you do once in Windows settings.
- "does not currently expose setting 'gamingMode'" usually means the buds' firmware is too
  old for that feature.
