//! "Run at startup", implemented per-OS, all per-user (no admin/root needed):
//! - Windows: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` registry value.
//! - Linux:   `~/.config/autostart/soundcore-desktop.desktop` (XDG autostart).
//! - macOS:   `~/Library/LaunchAgents/com.soundcore.desktop.plist` (LaunchAgent).

const APP_NAME: &str = "SoundCore-Desktop";

/// Makes the OS autostart entry match the desired state, pointing at the current exe.
pub fn reconcile(enabled: bool) {
    let result = if enabled { enable() } else { disable() };
    if let Err(err) = result {
        tracing::warn!("autostart reconcile failed: {err}");
    }
}

#[allow(dead_code)]
fn current_exe() -> std::io::Result<std::path::PathBuf> {
    std::env::current_exe()
}

// ---------------------------------------------------------------------------
// Windows
// ---------------------------------------------------------------------------
#[cfg(windows)]
fn enable() -> std::io::Result<()> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    let command = format!("\"{}\"", current_exe()?.display());
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (run, _) = hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")?;
    run.set_value(APP_NAME, &command)
}

#[cfg(windows)]
fn disable() -> std::io::Result<()> {
    use winreg::RegKey;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run) =
        hkcu.open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_SET_VALUE)
    {
        match run.delete_value(APP_NAME) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Linux (XDG autostart .desktop)
// ---------------------------------------------------------------------------
#[cfg(target_os = "linux")]
fn autostart_path() -> std::io::Result<std::path::PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| std::io::Error::other("no config dir"))?
        .join("autostart");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("soundcore-desktop.desktop"))
}

#[cfg(target_os = "linux")]
fn enable() -> std::io::Result<()> {
    let exe = current_exe()?;
    let contents = format!(
        "[Desktop Entry]\nType=Application\nName={APP_NAME}\nExec=\"{}\"\nX-GNOME-Autostart-enabled=true\nTerminal=false\n",
        exe.display()
    );
    std::fs::write(autostart_path()?, contents)
}

#[cfg(target_os = "linux")]
fn disable() -> std::io::Result<()> {
    remove_if_exists(autostart_path()?)
}

// ---------------------------------------------------------------------------
// macOS (LaunchAgent plist)
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
fn autostart_path() -> std::io::Result<std::path::PathBuf> {
    let dir = dirs::home_dir()
        .ok_or_else(|| std::io::Error::other("no home dir"))?
        .join("Library")
        .join("LaunchAgents");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("com.soundcore.desktop.plist"))
}

#[cfg(target_os = "macos")]
fn enable() -> std::io::Result<()> {
    let exe = current_exe()?;
    let contents = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">\n<dict>\n\
  <key>Label</key><string>com.soundcore.desktop</string>\n\
  <key>ProgramArguments</key><array><string>{}</string></array>\n\
  <key>RunAtLoad</key><true/>\n\
</dict>\n</plist>\n",
        exe.display()
    );
    std::fs::write(autostart_path()?, contents)
}

#[cfg(target_os = "macos")]
fn disable() -> std::io::Result<()> {
    remove_if_exists(autostart_path()?)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn remove_if_exists(path: std::path::PathBuf) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

// ---------------------------------------------------------------------------
// Other platforms: no-op
// ---------------------------------------------------------------------------
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn enable() -> std::io::Result<()> {
    Ok(())
}
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn disable() -> std::io::Result<()> {
    Ok(())
}
