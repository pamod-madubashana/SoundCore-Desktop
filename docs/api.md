---
id: api
title: Architecture
---

# Architecture

SoundCore-Desktop is structured as a desktop tray application with a React frontend and a Rust Tauri backend.

## Core folders

- `ui/` — React + Vite frontend for the tray UI and user interaction.
- `src-tauri/` — Rust backend with Tauri integration, system APIs, and device handling.
- `docs/` — Docusaurus documentation site and markdown content.
- `soundcore-graphify/` — analysis and graph data for the repository.

## Main Rust components

- `src-tauri/src/main.rs` — application launch and Tauri command registration.
- `src-tauri/src/worker.rs` — device connection detection and profile restore logic.
- `src-tauri/src/config.rs` — configuration parsing and app settings.
- `src-tauri/src/autostart.rs` — optional autostart implementation.

## Frontend components

- `ui/src/App.jsx` — main React app entry point.
- `ui/src/main.jsx` — React render logic.
- `ui/src/index.css` — application styling.

## Deployment and packaging

The project is built using Tauri for native packaging, with `npm run tauri build` producing desktop installers.

The docs site is a static Docusaurus site served from `docs/build` and published separately to GitHub Pages.
