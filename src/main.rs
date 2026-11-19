// Hide the console window in release builds (it's a tray app). Keep it in debug for logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Do not set GPUI_DISABLE_DIRECT_COMPOSITION. DirectComposition is required
    // for DXGI_ALPHA_MODE_PREMULTIPLIED — without it the host swapchain ignores
    // alpha and the GPU-masked rounded corners paint as opaque black.
    // (Acrylic is gone; nothing here needs the non-composition path.)
    soundcore_desktop_lib::run();
}
