---
id: troubleshooting
title: Troubleshooting
---

# Troubleshooting

If SoundCore-Desktop does not behave as expected, use these steps to diagnose and resolve common issues.

## The app does not appear in the tray

- Verify that the app is running.
- Check operating system tray settings and hidden icon overflow.
- On Windows, confirm the app was not blocked by antivirus or security software.

## Device connection is not detected

- Ensure Bluetooth is enabled on your computer.
- Confirm the Soundcore device is paired and connected.
- Restart the app after reconnecting the device.

## Settings are not restored

- Verify the selected profile has been saved in the app.
- Reconnect the device and watch for profile restore activity.
- If the app cannot communicate with the device, restart both the app and the device.

## Docs build issues

If Docusaurus docs fail to build:

```bash
cd docs
npm install
npm run build
```

Then inspect the terminal output for broken links or missing configuration.

## Build / packaging issues

- Make sure Rust is installed and up to date.
- Confirm `npm install` completed successfully.
- If the Tauri build fails, run:

```bash
npm run tauri build
```

and review the error messages.

## Getting help

- Open an issue on the project repository: https://github.com/pamod-madubashana/SoundCore-Desktop/issues
- Include your OS, device model, and a short description of the problem.
