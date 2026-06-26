use super::*;

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_optional_features() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("sound", "Sound")
            .with_capability("runtime.plugin.sound")
            .with_optional_feature(PluginFeatureBundleManifest::new(
                "soundtimeline",
                " Sound Timeline ",
                "Sound",
            )),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("feature id `soundtimeline`")
            && diagnostic.contains("dot-separated namespace")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("display_name ` Sound Timeline `")
            && diagnostic.contains("non-empty and trimmed")
    }));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("owner_plugin_id `Sound`")
            && diagnostic.contains("lowercase ASCII")));
    assert!(registration.diagnostics.iter().any(
        |diagnostic| diagnostic.contains("dependencies") && diagnostic.contains("at least one")
    ));
    assert!(registration.diagnostics.iter().any(
        |diagnostic| diagnostic.contains("capabilities") && diagnostic.contains("at least one")
    ));
}

#[test]
fn native_registration_rejects_duplicate_package_optional_feature_providers() {
    let feature = valid_sound_timeline_feature();
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("sound", "Sound")
            .with_capability("runtime.plugin.sound")
            .with_supported_targets([RuntimeTargetMode::EditorHost])
            .with_optional_feature(feature.clone())
            .with_optional_feature(feature),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("optional feature `sound.timeline` provider `sound`")
            && diagnostic.contains("unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("optional feature `sound.timeline` module `sound.timeline.runtime`")
            && diagnostic.contains("target mode ClientRuntime")
            && diagnostic.contains("supported_targets")
    }));
}

#[test]
fn native_registration_rejects_duplicate_package_feature_extension_providers() {
    let feature = valid_sound_timeline_feature();
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("sound_timeline_provider", "Sound Timeline Provider")
            .as_feature_extension()
            .with_capability("runtime.plugin.sound_timeline_provider")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_feature_extension(feature.clone())
            .with_feature_extension(feature),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("feature extension `sound.timeline` provider `sound_timeline_provider`")
            && diagnostic.contains("unique")
    }));
}

#[test]
fn native_registration_rejects_standard_package_feature_extensions() {
    let feature = valid_sound_timeline_feature();
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("sound_timeline_provider", "Sound Timeline Provider")
            .with_capability("runtime.plugin.sound_timeline_provider")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_feature_extension(feature),
    );

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("Standard package_kind")
            && diagnostic.contains("feature_extensions")));
}

#[test]
fn native_registration_rejects_empty_feature_extension_packages() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("sound_timeline_provider", "Sound Timeline Provider")
            .as_feature_extension()
            .with_capability("runtime.plugin.sound_timeline_provider")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime]),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package_kind FeatureExtension")
            && diagnostic.contains("at least one feature_extension")
    }));
}

#[test]
fn native_registration_rejects_feature_extension_packages_with_optional_features() {
    let feature = valid_sound_timeline_feature();
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("sound_timeline_provider", "Sound Timeline Provider")
            .as_feature_extension()
            .with_capability("runtime.plugin.sound_timeline_provider")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_optional_feature(feature),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package_kind FeatureExtension")
            && diagnostic.contains("must not declare optional_features")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_module_identities() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_runtime_module(
                PluginModuleManifest::runtime("storm.runtime", "zircon-plugin-weather")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["runtime.plugin.weather"]),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module name `storm.runtime`")
            && diagnostic.contains("package id `weather`")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module crate_name `zircon-plugin-weather`")
            && diagnostic.contains("zircon_plugin_")
    }));

    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_runtime_module(
                PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather__runtime")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["runtime.plugin.weather"]),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module crate_name `zircon_plugin_weather__runtime`")
            && diagnostic.contains("repeated underscores")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_module_capabilities() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_runtime_module(
                PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_runtime")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["editor.plugin.weather", "editor.plugin.weather"]),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("runtime module `weather.runtime` capability `editor.plugin.weather`")
            && diagnostic.contains("runtime.")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module `weather.runtime` capability `editor.plugin.weather`")
            && diagnostic.contains("unique")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_module_system_contracts() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_runtime_module(
                PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_runtime")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["runtime.plugin.weather"])
                    .with_system_sets(["weather.main", "storm.main", "weather.main"])
                    .with_system_anchors(["weather.tick", "tick", "weather.tick"]),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module `weather.runtime` system_set `storm.main`")
            && diagnostic.contains("prefixed by package id `weather`")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module `weather.runtime` system_set `weather.main`")
            && diagnostic.contains("unique")
    }));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("system_anchor `tick`")
            && diagnostic.contains("at least two dot-separated namespace segments")));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module `weather.runtime` system_anchor `weather.tick`")
            && diagnostic.contains("unique")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_module_target_modes() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::EditorHost])
            .with_runtime_module(
                PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_runtime")
                    .with_capabilities(["runtime.plugin.weather"]),
            )
            .with_editor_module(
                PluginModuleManifest::editor("weather.editor", "zircon_plugin_weather_editor")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["editor.plugin.weather"]),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module `weather.runtime` target_modes")
            && diagnostic.contains("at least one")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("editor module `weather.editor` target mode ClientRuntime")
            && diagnostic.contains("EditorHost")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module `weather.editor` target mode ClientRuntime")
            && diagnostic.contains("supported_targets")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_duplicate_package_module_names() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_runtime_module(
                PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_runtime")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["runtime.plugin.weather"]),
            )
            .with_runtime_module(
                PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_debug")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["runtime.plugin.weather.debug"]),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("module name `weather.runtime`") && diagnostic.contains("unique")
    }));
}

fn valid_sound_timeline_feature() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new("sound.timeline", "Sound Timeline", "sound")
        .with_dependency(PluginFeatureDependency::primary(
            "sound",
            "runtime.plugin.sound",
        ))
        .with_capability("runtime.feature.sound.timeline")
        .with_runtime_module(
            PluginModuleManifest::runtime(
                "sound.timeline.runtime",
                "zircon_plugin_sound_timeline_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capabilities(["runtime.feature.sound.timeline"]),
        )
}
