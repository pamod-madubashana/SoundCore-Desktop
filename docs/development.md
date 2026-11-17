---
id: development
title: Development
---

# Development

These notes cover how to contribute to the SoundCore-Desktop project.

## Project structure

- `src/` — GPUI tray app (UI + backend).
- `src/UI/` — GPUI screens, theme, and data helpers.
- `assets/` — icons and embedded device images.
- `docs/` — documentation site.
- `OpenSCQ30/` — git submodule (do not edit).

## Setup for development

1. Clone the repository:

```bash
git clone --recurse-submodules https://github.com/pamod-madubashana/SoundCore-Desktop.git
cd SoundCore-Desktop
```

2. Documentation pages are plain markdown under `docs/` (no npm build for the app).

## Working on the UI

- Edit files under `src/UI/` (screens, theme, data).
- Design references live in `UI-Target/*.svg`.

## Working on the backend

- Device/worker logic: `src/worker.rs`.
- Configuration: `src/config.rs`.
- Window shell (position, acrylic, corners): `src/shell.rs`.

## Running the app during development

```bash
cargo run
```

## Building for release

```bash
cargo build --release
```

## Updating documentation

1. Add or edit markdown files under `docs/`.
2. Marketing/docs site source lives in `.site/` (separate npm project for GitHub Pages).

## Contributing guidelines

- Follow consistent naming and structure for new docs pages.
- Keep the UX simple and focused on restore behavior.
- Test changes on both Windows and Linux where possible.
