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

Edit [`config.toml`](config.toml). It's already filled in for the R50i NC
(`34:09:C9:B9:EC:30`, model `SoundcoreA3959`). The `[[settings]]` entries are applied in
order on every connect:

| setting id          | example                 | meaning                                   |
| ------------------- | ----------------------- | ----------------------------------------- |
| `gamingMode`        | `"true"` / `"false"`    | low-latency mode (needs recent firmware)  |
| `ambientSoundMode`  | `"Normal"` / `"Transparency"` / `"NoiseCanceling"` |                    |
| `volumeAdjustments` | `"0,0,0,0,0,0,0,0"`     | 8-band EQ, tenths of a dB (`-40` = -4.0)  |

To discover every setting your firmware exposes, just run the program once with the buds
connected — it logs each setting it applies, and rejects unknown ids with a clear message.

## Run

Put `config.toml` next to the exe (or pass a path as the first argument), then:

```powershell
.\target\release\soundcore-auto-mode.exe
```

A tray icon appears (teal square). Left-click it for the config popup; right-click for
Apply now / Quit. The release build has no console window; for logs run the debug build
(`cargo run`) or set `RUST_LOG=debug`.

### Start automatically at logon

Register it as a hidden scheduled task (one line, no script files):

```powershell
$exe = "$PWD\target\release\soundcore-auto-mode.exe"
$action  = New-ScheduledTaskAction -Execute $exe -WorkingDirectory (Split-Path $exe)
$trigger = New-ScheduledTaskTrigger -AtLogOn
Register-ScheduledTask -TaskName SoundcoreAutoMode -Action $action -Trigger $trigger -Force
```

Remove it with `Unregister-ScheduledTask -TaskName SoundcoreAutoMode -Confirm:$false`.

## Notes

- Pairing here means OpenSCQ30's own mac↔model association (stored in its sqlite db under
  your config dir); the program sets it up automatically. It's separate from Bluetooth
  pairing, which you do once in Windows settings.
- "does not currently expose setting 'gamingMode'" usually means the buds' firmware is too
  old for that feature.
