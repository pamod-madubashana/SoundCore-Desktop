<div align='center'>
<h1>SoundCore-Desktop</h1>

<img src='src-tauri/icons/icon.png' width=200>
<p>SoundCore-Desktop is a lightweight cross-platform tray app that restores your Soundcore device profile automatically when the device connects, so your preferred gaming mode, ANC, and EQ settings land without the phone app.</p>


  <a href="https://github.com/pamod-madubashana/SoundCore-Desktop"><img src="https://img.shields.io/badge/version-0.1.0-blue" alt="Version"></a>
  <a href="https://github.com/pamod-madubashana/SoundCore-Desktop"><img src="https://img.shields.io/badge/Windows-supported-46A75A?logo=windows&logoColor=white" alt="Windows"></a>
  <a href="https://github.com/pamod-madubashana/SoundCore-Desktop"><img src="https://img.shields.io/badge/Linux-supported-46A75A?logo=linux&logoColor=white" alt="Linux"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Built%20with-Rust-DEA584?logo=rust&logoColor=white" alt="Built with Rust"></a>
  <a href="https://github.com/pamod-madubashana/SoundCore-Desktop"><img src="https://img.shields.io/badge/Powered%20by-OpenSCQ30-46A75A" alt="Powered by OpenSCQ30"></a>
</p>
</div>
</div>
<br>
<div align="center">

## Downloads

| Platform | Download Link |
|----------|--------------|
| Windows (Setup) | [SoundCore-Desktop-x64-setup.exe](https://github.com/pamod-madubashana/SoundCore-Desktop/releases/latest/download/SoundCore-Desktop-x64-setup.exe) |
| Windows (MSI) | [SoundCore-Desktop-x64_en-US.msi](https://github.com/pamod-madubashana/SoundCore-Desktop/releases/latest/download/SoundCore-Desktop-x64_en-US.msi) |
| Windows (Portable) | [SoundCore-Desktop.exe](https://github.com/pamod-madubashana/SoundCore-Desktop/releases/latest/download/SoundCore-Desktop.exe) |
| Linux (DEB) | [SoundCore-Desktop-amd64.deb](https://github.com/pamod-madubashana/SoundCore-Desktop/releases/latest/download/SoundCore-Desktop-amd64.deb) |
| Linux (RPM) | [SoundCore-Desktop-x86_64.rpm](https://github.com/pamod-madubashana/SoundCore-Desktop/releases/latest/download/SoundCore-Desktop-x86_64.rpm) |

</div>

## Tech Stack

- Frontend: React + Vite
- Styling: Tailwind CSS
- Desktop runtime: Tauri (Rust)
- Packaging: npm + Cargo

## Why This Project Exists

- Problem: Soundcore device settings are often locked behind the mobile app, so desktop users lose access to gaming mode, ANC, or EQ profiles when the device reconnects.
- Goal: Keep Soundcore device preferences consistent by applying the saved profile automatically from a native desktop tray utility.
- Outcome: A small tray app that restores your profile at connect, offers quick controls, and removes the need to use the official phone app.

## Project Structure

```
ui/
├── src/
│   ├── App.jsx
│   ├── main.jsx
│   ├── index.css
│   └── headphones.jpg
src-tauri/
├── src/
│   ├── lib.rs
│   └── main.rs
├── Cargo.toml
├── tauri.conf.json
└── icons/
    ├── icon.png
    ├── icon.ico
    ├── 32x32.png
    └── ...
```

## Key Features

- Runs in the system tray with a compact control panel
- Automatically applies your saved Soundcore profile when a device connects
- Supports quick toggles for gaming mode, ANC, and volume presets
- Build targets for Windows and Linux
- Minimal distraction and no phone app required once configured

## Getting Started

### Prerequisites

- Node.js 18+
- Rust (latest stable)
- Windows 10/11 or Linux

### Installation

```bash
# Clone the repository
git clone https://github.com/pamod-madubashana/SoundCore-Desktop.git

# Navigate to project directory
cd SoundCore-Desktop

# Install root and UI dependencies
npm install
npm --prefix ui install

# Run in development mode
npm run tauri dev
```

### Building

```bash
# Build production bundles
npm run tauri build
```

## Usage

1. Launch SoundCore-Desktop; it will appear in your system tray
2. Click the tray icon to open the control panel
3. Add your Soundcore device and configure the profile settings
4. Use the app popup to apply your profile or save it permanently
5. The app restores your settings the next time the device connects

## Keyboard Shortcuts

- **Left-click tray icon**: Show/hide control panel
- **Right-click tray icon**: Open context menu (Show/Hide, Apply now, Quit)
