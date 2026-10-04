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
