use std::fs;
use std::path::Path;

fn main() {
    tauri_build::build();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let devices_dir = Path::new(&manifest_dir).join("../ui/public/devices");
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
    println!("cargo:rerun-if-changed=../ui/public/devices");
}
