---
id: api
title: Architecture
---

# Architecture

SoundCore-Desktop is a native desktop tray application built with GPUI (Rust) and the OpenSCQ30 device library.

## Core folders

- `src/` — application entry, worker, config, and window shell.
- `src/UI/` — GPUI screens, theme, and data helpers.
- `assets/` — icons and embedded device images.
- `docs/` — documentation site and markdown content.
- `OpenSCQ30/` — git submodule with the device protocol library.

## Main Rust components

- `src/main.rs` — process entry.
- `src/lib.rs` — app bootstrap, tray wiring, window show/hide.
- `src/worker.rs` — device connection detection and profile restore logic.
- `src/config.rs` — configuration parsing and app settings.
- `src/shell.rs` — Windows shell: position, acrylic, rounded corners.
- `src/autostart.rs` — optional autostart implementation.

## UI components

- `src/UI/mod.rs` — root view and screen routing.
- `src/UI/screens/` — control, devices, searching, sound effects screens.
- `src/UI/theme.rs` — palette tokens.

## Deployment and packaging

Release binaries are built with `cargo build --release`. GitHub Actions (`release.yml`) builds Windows and Linux artifacts.
