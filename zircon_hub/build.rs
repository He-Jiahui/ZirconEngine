fn main() {
    #[cfg(feature = "desktop")]
    {
        let windows_msvc = std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() == Some("windows")
            && std::env::var("CARGO_CFG_TARGET_ENV").ok().as_deref() == Some("msvc");
        if windows_msvc {
            // Share one manifest owner between desktop binaries and the lib test harness.
            let windows = tauri_build::WindowsAttributes::new_without_app_manifest();
            let attributes = tauri_build::Attributes::new().windows_attributes(windows);
            tauri_build::try_build(attributes).expect("failed to run Tauri build script");
            println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
            println!("cargo:rustc-link-arg=/MANIFESTUAC:NO");
            println!("cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'");
        } else {
            tauri_build::build();
        }
    }
}
