use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let devices_dir = Path::new(&manifest_dir).join("assets/devices");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("embedded_devices.rs");

    let mut entries: Vec<(String, String)> = Vec::new();

    if let Ok(read_dir) = fs::read_dir(&devices_dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "png") {
                let filename = path.file_name().unwrap().to_string_lossy().to_string();
                let abs_path = path.canonicalize().unwrap().to_string_lossy().to_string();
                // Use forward slashes for include_bytes on all platforms
                let abs_path = abs_path.replace('\\', "/");
                entries.push((filename, abs_path));
            }
        }
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut code = String::from(
        "/// Auto-generated embedded device images. DO NOT EDIT.\n\
         /// Returns raw PNG bytes for the given filename, or None.\n\
         pub fn get_embedded_image(filename: &str) -> Option<&'static [u8]> {\n\
         match filename {\n",
    );

    for (filename, abs_path) in &entries {
        code.push_str(&format!(
            "    \"{filename}\" => Some(include_bytes!(\"{abs_path}\")),\n"
        ));
    }

    code.push_str("    _ => None,\n}\n}\n");

    fs::write(&dest, code).expect("failed to write embedded_devices.rs");
    println!("cargo:rerun-if-changed=assets/devices");

    // Windows exe/window icon: PE resource ID 1 (gpui `load_icon` reads this).
    // Under `dx serve` the CLI injects its own winres .lib; linking both emits
    // duplicate RT_ICON entries and fails rust-lld, so yield to dx there
    // (Dioxus.toml [bundle] icon keeps our icon in dx builds).
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-env-changed=DX_RUSTC");
        if std::env::var_os("DX_RUSTC").is_none() {
            let icons_dir = Path::new(&manifest_dir).join("assets/icons");
            let rc = icons_dir.join("app.rc");
            if rc.is_file() {
                println!("cargo:rerun-if-changed={}", rc.display());
                println!("cargo:rerun-if-changed={}", icons_dir.join("icon.ico").display());
                let _ = embed_resource::compile(&rc, embed_resource::ParamsIncludeDirs([icons_dir]));
            }
        }
    }
}
