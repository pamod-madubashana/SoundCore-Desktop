---
id: installation
title: Installation
---

# Installation

## Prerequisites

- Node.js 18+
- Rust toolchain
- npm installed
- Windows 10/11 or Linux

## Install the repository

```bash
git clone https://github.com/pamod-madubashana/SoundCore-Desktop.git
cd SoundCore-Desktop
npm install
npm --prefix ui install
```

## Run the desktop app in development

```bash
npm run tauri dev
```

This starts the Tauri development environment and opens the app in the system tray.

## Build for production

```bash
npm run tauri build
```

The release output is produced by Tauri and can be packaged for Windows or Linux.

## Run the docs locally

```bash
cd docs
npm install
npm run start
```

Then open the local development URL shown in the terminal.
