pub(super) fn dynamic_library_file_name(crate_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("{crate_name}.dll")
    }
    #[cfg(target_os = "macos")]
    {
        format!("lib{crate_name}.dylib")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        format!("lib{crate_name}.so")
    }
}
