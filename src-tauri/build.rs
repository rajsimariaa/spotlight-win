fn main() {
    // Embed Windows application manifest to suppress console window
    if cfg!(target_os = "windows") {
        println!("cargo:rerun-if-changed=app.manifest");
    }
    tauri_build::build();
}
