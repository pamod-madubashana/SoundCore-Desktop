// Hide the console window in release builds (it's a tray app). Keep it in debug for logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    soundcore_desktop_lib::run();
}
