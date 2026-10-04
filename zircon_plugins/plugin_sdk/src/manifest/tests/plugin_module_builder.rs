use super::*;

#[test]
fn exact_module_metadata_preserves_builtin_names_and_descriptions() {
    let modules = [
        PluginModuleBuilder::runtime("weather", "weather_runtime").build(),
        PluginModuleBuilder::editor("weather", "weather_editor").build(),
        PluginModuleBuilder::native("weather", "weather_native").build(),
        PluginModuleBuilder::vm("weather", "weather_vm").build(),
    ];
    let expected_names = [
        "weather.runtime",
        "weather.editor",
        "weather.native",
        "weather.vm",
    ];

    for (module, expected_name) in modules.iter().zip(expected_names) {
        assert_eq!(module.name, expected_name);
        assert_eq!(module.description, format!("Plugin module {expected_name}"));
    }
}
