---
id: installation
title: Installation
---

# Installation

## Prerequisites

- Rust toolchain
- Git (with submodule support)
- Windows 10/11 or Linux

## Install the repository

```bash
git clone --recurse-submodules https://github.com/pamod-madubashana/SoundCore-Desktop.git
cd SoundCore-Desktop
```

## Run the desktop app in development

```bash
cargo run
```

This starts the tray app and opens the control popup.

## Build for production

```bash
cargo build --release
```

The release binary is produced under `target/release/` for Windows or Linux.
