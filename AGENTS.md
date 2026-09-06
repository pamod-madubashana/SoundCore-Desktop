# Project Rules

## OpenSCQ30 Submodule — DO NOT EDIT

The `OpenSCQ30/` directory is a **git submodule**. Never modify files inside it.

- Do not edit Rust source files under `OpenSCQ30/lib/`
- Do not edit Cargo.toml files under `OpenSCQ30/`
- If a change to the OpenSCQ30 library is needed, request it upstream or fork the submodule separately

All application-level changes must go in `src-tauri/` or `ui/`.
