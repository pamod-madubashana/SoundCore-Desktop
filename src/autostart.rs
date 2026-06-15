//! "Run at startup" via the per-user Windows Run registry key. No admin rights needed.

#[cfg(windows)]
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(windows)]
const VALUE_NAME: &str = "SoundcoreAutoMode";

/// Makes the registry match the desired state, pointing at the current executable.
pub fn reconcile(enabled: bool) {
    let result = if enabled { enable() } else { disable() };
    if let Err(err) = result {
        tracing::warn!("autostart reconcile failed: {err}");
    }
}

fn enable() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;
        let exe = std::env::current_exe()?;
        let command = format!("\"{}\"", exe.display());
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (run, _) = hkcu.create_subkey(RUN_KEY)?;
        run.set_value(VALUE_NAME, &command)?;
    }
    Ok(())
}

fn disable() -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use winreg::RegKey;
        use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(run) = hkcu.open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE) {
            match run.delete_value(VALUE_NAME) {
                Ok(()) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => return Err(err),
            }
        }
    }
    Ok(())
}
