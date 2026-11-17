// Hide the console window in release builds (it's a tray app). Keep it in debug for logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // GPUI enables DirectComposition by default, which sets WS_EX_NOREDIRECTIONBITMAP
    // and prevents DWM acrylic/system-backdrop from rendering. Disable it so the
    // window-vibrancy acrylic backdrop works. Harmless on Linux (env unread there).
    #[cfg(target_os = "windows")]
    // SAFETY: called first thing in main, before any threads spawn.
    unsafe {
        std::env::set_var("GPUI_DISABLE_DIRECT_COMPOSITION", "1");
    }

    soundcore_desktop_lib::run();
}
