pub(super) fn dynamic_library_file_name(crate_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        exact_dynamic_library_name("", crate_name, ".dll")
    }
    #[cfg(target_os = "macos")]
    {
        exact_dynamic_library_name("lib", crate_name, ".dylib")
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        exact_dynamic_library_name("lib", crate_name, ".so")
    }
}

fn exact_dynamic_library_name(prefix: &str, crate_name: &str, suffix: &str) -> String {
    let capacity = prefix.len() + crate_name.len() + suffix.len();
    let mut name = String::with_capacity(capacity);
    name.push_str(prefix);
    name.push_str(crate_name);
    name.push_str(suffix);
    name
}

#[cfg(test)]
mod tests {
    use super::exact_dynamic_library_name;

    #[test]
    fn exact_dynamic_library_names_preserve_platform_conventions() {
        let crate_name = "zircon_plugin_weather";
        assert_eq!(
            exact_dynamic_library_name("", crate_name, ".dll"),
            "zircon_plugin_weather.dll"
        );
        assert_eq!(
            exact_dynamic_library_name("lib", crate_name, ".dylib"),
            "libzircon_plugin_weather.dylib"
        );
        assert_eq!(
            exact_dynamic_library_name("lib", crate_name, ".so"),
            "libzircon_plugin_weather.so"
        );
    }
}
