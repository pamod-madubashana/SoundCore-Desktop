---
id: development
title: Development
---

# Development

These notes cover how to contribute to the SoundCore-Desktop project.

## Project structure

- `ui/` — React + Vite frontend code for the tray UI.
- `src-tauri/` — Rust backend code and Tauri integration.
- `docs/` — documentation site powered by Docusaurus.
- `soundcore-graphify/` — repository analysis and graph data.

## Setup for development

1. Clone the repository:

```bash
git clone https://github.com/pamod-madubashana/SoundCore-Desktop.git
cd SoundCore-Desktop
```

2. Install root and UI dependencies:

```bash
npm install
npm --prefix ui install
```

3. Install docs dependencies if you want to update documentation:

```bash
cd docs
npm install
```

## Working on the frontend

- Edit `ui/src/App.jsx` and `ui/src/main.jsx` for UI changes.
- Update styles in `ui/src/index.css`.

## Working on the backend

- Update Tauri integration in `src-tauri/src/main.rs`.
- Modify device logic in `src-tauri/src/worker.rs`.
- Adjust configuration parsing in `src-tauri/src/config.rs`.

## Running the app during development

```bash
npm run tauri dev
```

## Building for release

```bash
npm run tauri build
```

This generates native installers for supported platforms.

## Updating documentation

1. Add or edit markdown files in `docs/docs/`.
2. Update `docs/sidebars.js` to include new content.
3. Run the docs site locally:

```bash
cd docs
npm run start
```

## Contributing guidelines

- Follow consistent naming and structure for new docs pages.
- Keep the UX simple and focused on restore behavior.
- Test changes on both Windows and Linux where possible.
