fn main() {
    tauri_build::build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        #[cfg(windows)]
        {
            let mut res = winres::WindowsResource::new();
            res.set_manifest_file("app.manifest");
            res.compile()
                .expect("Failed to embed requireAdministrator manifest (winres)");
        }
    }
}
