fn main() {
    let mut attributes = tauri_build::Attributes::new();

    // On Windows, Tauri embeds its app manifest (Common Controls v6) only in
    // the app binary. The test binary also links the native dialog code
    // (TaskDialogIndirect), so without the manifest it fails to start with
    // STATUS_ENTRYPOINT_NOT_FOUND. Embed the same manifest in every target.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
        let manifest = std::env::current_dir()
            .expect("build script has a working directory")
            .join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
    }

    tauri_build::try_build(attributes).expect("tauri build script failed");
}
