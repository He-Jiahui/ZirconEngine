use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::{InitLevel, ModuleDependencySpec};
use crate::plugin::{
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest,
    PluginPackageManifest, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

#[test]
fn native_reports_register_manifest_module_descriptor_projection() {
    let package_report = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "weather.base.runtime",
                    "zircon_plugin_weather_runtime",
                )
                .with_description("Weather base native module")
                .with_init_level(InitLevel::Kernel)
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.plugin.weather"]),
            )
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "weather.simulation.runtime",
                    "zircon_plugin_weather_simulation_runtime",
                )
                .with_description("Weather simulation native module")
                .with_init_level(InitLevel::Scene)
                .with_module_dependency(ModuleDependencySpec::named("weather.base.runtime"))
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.plugin.weather"]),
            ),
    );

    assert!(
        package_report.is_success(),
        "{:?}",
        package_report.diagnostics
    );
    let simulation = package_report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == "weather.simulation.runtime")
        .expect("simulation module descriptor registered");
    assert_eq!(simulation.description, "Weather simulation native module");
    assert_eq!(simulation.init_level, InitLevel::Scene);
    assert_eq!(
        simulation.module_dependencies,
        vec![ModuleDependencySpec::named("weather.base.runtime")]
    );

    let feature_report = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        PluginFeatureBundleManifest::new("weather.storms", "Weather Storms", "weather")
            .with_dependency(PluginFeatureDependency::primary(
                "weather",
                "runtime.plugin.weather",
            ))
            .with_capability("runtime.feature.weather.storms")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "weather.storms.runtime",
                    "zircon_plugin_weather_storms_runtime",
                )
                .with_description("Weather storms feature module")
                .with_init_level(InitLevel::Scene)
                .with_module_dependency(ModuleDependencySpec::named("weather.base.runtime"))
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.feature.weather.storms"]),
            ),
        Some("weather".to_string()),
    );

    assert!(
        feature_report.is_success(),
        "{:?}",
        feature_report.diagnostics
    );
    let storms = feature_report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == "weather.storms.runtime")
        .expect("feature module descriptor registered");
    assert_eq!(storms.description, "Weather storms feature module");
    assert_eq!(storms.init_level, InitLevel::Scene);
    assert_eq!(
        storms.module_dependencies,
        vec![ModuleDependencySpec::named("weather.base.runtime")]
    );
}
