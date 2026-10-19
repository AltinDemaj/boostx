fn main() {
    tauri_build::build();

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("app.manifest");
        res.compile()
            .expect("Failed to embed requireAdministrator manifest (winres)");
    }
}
