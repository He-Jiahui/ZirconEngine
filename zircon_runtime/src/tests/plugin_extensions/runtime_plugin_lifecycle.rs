use std::cell::{Cell, RefCell};

use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::core::{CoreRuntime, InitLevel, ModuleDependencySpec, ModuleDescriptor};
use crate::plugin::{
    CapabilityStatus, CapabilityStatusManifest, CapabilityView, PluginFeatureBundleManifest,
    PluginFeatureDependency, PluginFinishContext, PluginModuleManifest, PluginPackageManifest,
    PluginReadyContext, PluginRuntimeContext, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginCatalog, RuntimePluginDescriptor,
    RuntimePluginFeature, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
use crate::scene::World;

#[path = "runtime_plugin_lifecycle/lifecycle_fixtures.rs"]
mod lifecycle_fixtures;

use lifecycle_fixtures::*;

#[test]
fn runtime_plugin_lifecycle_hard_cuts_to_register_hook() {
    let plugin_trait = include_str!("../../plugin/runtime_plugin/runtime_plugin/plugin.rs");
    let feature_trait = include_str!("../../plugin/runtime_plugin/runtime_plugin/feature.rs");
    let plugin_report = include_str!("../../plugin/runtime_plugin/registration_report/plugin.rs");
    let feature_report =
        include_str!("../../plugin/runtime_plugin/feature_registration_report/feature.rs");

    for source in [plugin_trait, feature_trait, plugin_report, feature_report] {
        assert!(!source.contains("register_runtime_extensions"));
    }
    assert!(plugin_trait.contains("fn register("));
    assert!(feature_trait.contains("fn register("));
    assert!(plugin_report.contains("plugin.register(&mut extensions)"));
    assert!(feature_report.contains("feature.register(&mut extensions)"));
}

#[test]
fn optional_dependency_probe_sees_all_registered_capabilities() {
    let physics_registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("physics", "Physics")
            .with_capability("runtime.capability.physics.raycast")
            .with_capability_status(CapabilityStatusManifest::new(
                "runtime.capability.physics.raycast",
                CapabilityStatus::Complete,
            ))
            .with_runtime_module(
                PluginModuleManifest::runtime("physics.runtime", "zircon_plugin_physics_runtime")
                    .with_capabilities(["runtime.capability.physics.collider_world"]),
            ),
    );
    let sound_feature_registration =
        RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
            PluginFeatureBundleManifest::new("sound.occlusion", "Sound Occlusion", "sound")
                .with_capability("runtime.capability.sound.occlusion")
                .with_runtime_module(
                    PluginModuleManifest::runtime(
                        "sound.occlusion.runtime",
                        "zircon_plugin_sound_occlusion_runtime",
                    )
                    .with_capabilities(["runtime.capability.sound.occlusion.debug"]),
                ),
            Some("sound".to_string()),
        );
    let capability_view = CapabilityView::from_registration_reports(
        [&physics_registration],
        [&sound_feature_registration],
    );

    assert!(capability_view.has("runtime.capability.physics.raycast"));
    assert!(capability_view.has("runtime.capability.physics.collider_world"));
    assert!(capability_view.has("runtime.capability.sound.occlusion"));
    assert!(capability_view.has("runtime.capability.sound.occlusion.debug"));
    assert_eq!(
        capability_view.status("runtime.capability.physics.raycast"),
        Some(CapabilityStatus::Complete)
    );
    assert_eq!(
        capability_view.status("runtime.capability.sound.occlusion"),
        None
    );

    let probe = OptionalDependencyProbe::default();
    let mut registry = RuntimeExtensionRegistry::default();
    let mut context = PluginFinishContext::new(&mut registry, &capability_view);

    probe.finish(&mut context).unwrap();

    assert_eq!(
        probe.result.get(),
        Some(OptionalDependencyProbeResult {
            physics_raycast_available: true,
            physics_status: Some(CapabilityStatus::Complete),
            sound_occlusion_available: true,
        })
    );
}

#[test]
fn feature_register_runs_before_finish() {
    let log = RefCell::new(Vec::new());
    let plugin = LifecycleOrderPlugin::new(&log);
    let feature = LifecycleOrderFeature::new(&log);

    let catalog = RuntimePluginCatalog::from_lifecycle_plugins(
        [&plugin as &dyn RuntimePlugin],
        [&feature as &dyn RuntimePluginFeature],
    );

    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    assert_eq!(
        log.borrow().as_slice(),
        &[
            "plugin.register",
            "feature.register",
            "plugin.finish",
            "feature.finish",
        ]
    );
    assert!(catalog.registrations()[0]
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == "LifecycleFinishModule"));
    assert!(catalog.feature_registrations()[0]
        .extensions
        .modules()
        .iter()
        .any(|module| module.name == "LifecycleFeatureFinishModule"));
}

#[test]
fn runtime_plugin_ready_runs_after_register_before_finish() {
    let log = RefCell::new(Vec::new());
    let plugin = ReadyOrderPlugin::new(&log, true).expect_feature_capability();
    let feature = ReadyOrderFeature::new(&log, true);

    let catalog = RuntimePluginCatalog::from_lifecycle_plugins(
        [&plugin as &dyn RuntimePlugin],
        [&feature as &dyn RuntimePluginFeature],
    );

    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    assert_eq!(
        log.borrow().as_slice(),
        &[
            "plugin.register",
            "feature.register",
            "plugin.ready",
            "feature.ready",
            "plugin.finish",
            "feature.finish",
        ]
    );
}

#[test]
fn runtime_plugin_not_ready_blocks_finish() {
    let log = RefCell::new(Vec::new());
    let plugin = ReadyOrderPlugin::new(&log, false);

    let catalog = RuntimePluginCatalog::from_lifecycle_plugins([&plugin as &dyn RuntimePlugin], []);

    assert!(!catalog.is_success());
    assert!(catalog
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic == "runtime plugin `weather` is not ready"));
    assert_eq!(
        log.borrow().as_slice(),
        &["plugin.register", "plugin.ready"]
    );
}

#[test]
fn runtime_plugin_feature_not_ready_blocks_finish() {
    let log = RefCell::new(Vec::new());
    let plugin = ReadyOrderPlugin::new(&log, true);
    let feature = ReadyOrderFeature::new(&log, false);

    let catalog = RuntimePluginCatalog::from_lifecycle_plugins(
        [&plugin as &dyn RuntimePlugin],
        [&feature as &dyn RuntimePluginFeature],
    );

    assert!(!catalog.is_success());
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic == "runtime plugin feature `sound.occlusion` is not ready"
    }));
    assert_eq!(
        log.borrow().as_slice(),
        &[
            "plugin.register",
            "feature.register",
            "plugin.ready",
            "feature.ready",
        ]
    );
}

#[test]
fn runtime_plugin_activate_uses_descriptor_order_before_feature_activate() {
    let log = RefCell::new(Vec::new());
    let base = OrderedLifecyclePlugin::new(
        "weather_base",
        "Weather Base",
        "zircon_plugin_weather_base_runtime",
        ModuleDescriptor::new("weather.base.runtime", "Weather base runtime")
            .with_init_level(InitLevel::Kernel),
        "base",
        &log,
    );
    let simulation = OrderedLifecyclePlugin::new(
        "weather_simulation",
        "Weather Simulation",
        "zircon_plugin_weather_simulation_runtime",
        ModuleDescriptor::new("weather.simulation.runtime", "Weather simulation runtime")
            .with_init_level(InitLevel::Scene)
            .with_module_dependency(ModuleDependencySpec::named("weather.base.runtime")),
        "simulation",
        &log,
    );
    let feature = LifecycleOrderFeature::new(&log);
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let mut world = World::default();
    let mut context = PluginRuntimeContext::new(&mut world, &core);
    let mut catalog = RuntimePluginCatalog::default();

    catalog
        .activate_lifecycle_plugins(
            [
                &simulation as &dyn RuntimePlugin,
                &base as &dyn RuntimePlugin,
            ],
            [&feature as &dyn RuntimePluginFeature],
            &mut context,
        )
        .unwrap();

    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    assert_eq!(
        log.borrow().as_slice(),
        &["base.activate", "simulation.activate", "feature.activate"]
    );
}

#[test]
fn runtime_plugin_deactivate_uses_reverse_descriptor_order_after_features() {
    let log = RefCell::new(Vec::new());
    let base = OrderedLifecyclePlugin::new(
        "weather_base",
        "Weather Base",
        "zircon_plugin_weather_base_runtime",
        ModuleDescriptor::new("weather.base.runtime", "Weather base runtime")
            .with_init_level(InitLevel::Kernel),
        "base",
        &log,
    );
    let simulation = OrderedLifecyclePlugin::new(
        "weather_simulation",
        "Weather Simulation",
        "zircon_plugin_weather_simulation_runtime",
        ModuleDescriptor::new("weather.simulation.runtime", "Weather simulation runtime")
            .with_init_level(InitLevel::Scene)
            .with_module_dependency(ModuleDependencySpec::named("weather.base.runtime")),
        "simulation",
        &log,
    );
    let feature = LifecycleOrderFeature::new(&log);
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let mut world = World::default();
    let mut context = PluginRuntimeContext::new(&mut world, &core);
    let mut catalog = RuntimePluginCatalog::default();

    catalog.deactivate_lifecycle_plugins(
        [
            &simulation as &dyn RuntimePlugin,
            &base as &dyn RuntimePlugin,
        ],
        [&feature as &dyn RuntimePluginFeature],
        &mut context,
    );

    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    assert_eq!(
        log.borrow().as_slice(),
        &[
            "feature.deactivate",
            "simulation.deactivate",
            "base.deactivate",
        ]
    );
}

#[test]
fn runtime_plugin_activate_failure_records_catalog_diagnostic() {
    let log = RefCell::new(Vec::new());
    let plugin = OrderedLifecyclePlugin::new(
        "weather_base",
        "Weather Base",
        "zircon_plugin_weather_base_runtime",
        ModuleDescriptor::new("weather.base.runtime", "Weather base runtime")
            .with_init_level(InitLevel::Kernel),
        "base",
        &log,
    )
    .with_activate_error("base activation failed");
    let runtime = CoreRuntime::new();
    let core = runtime.handle();
    let mut world = World::default();
    let mut context = PluginRuntimeContext::new(&mut world, &core);
    let mut catalog = RuntimePluginCatalog::default();

    let error = catalog
        .activate_lifecycle_plugins(
            [&plugin as &dyn RuntimePlugin],
            std::iter::empty::<&dyn RuntimePluginFeature>(),
            &mut context,
        )
        .unwrap_err();

    assert_eq!(
        error,
        RuntimeExtensionRegistryError::InvalidPluginModule("base activation failed".to_string())
    );
    assert!(catalog
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic == "invalid plugin module: base activation failed"));
    assert_eq!(log.borrow().as_slice(), &["base.activate"]);
}

#[test]
fn runtime_plugin_lifecycle_uses_module_descriptor_order() {
    let log = RefCell::new(Vec::new());
    let foundation = OrderedLifecyclePlugin::new(
        "weather_base",
        "Weather Base",
        "zircon_plugin_weather_base_runtime",
        ModuleDescriptor::new("weather.base.runtime", "Weather base runtime")
            .with_init_level(InitLevel::Kernel),
        "base",
        &log,
    );
    let simulation = OrderedLifecyclePlugin::new(
        "weather_simulation",
        "Weather Simulation",
        "zircon_plugin_weather_simulation_runtime",
        ModuleDescriptor::new("weather.simulation.runtime", "Weather simulation runtime")
            .with_init_level(InitLevel::Scene)
            .with_module_dependency(ModuleDependencySpec::named("weather.base.runtime")),
        "simulation",
        &log,
    );

    let catalog = RuntimePluginCatalog::from_lifecycle_plugins(
        [
            &simulation as &dyn RuntimePlugin,
            &foundation as &dyn RuntimePlugin,
        ],
        [],
    );

    assert!(catalog.is_success(), "{:?}", catalog.diagnostics());
    assert_eq!(
        log.borrow().as_slice(),
        &[
            "base.register",
            "simulation.register",
            "base.finish",
            "simulation.finish",
        ]
    );
    assert_eq!(
        catalog.registrations()[0].package_manifest.id,
        "weather_base"
    );
    assert_eq!(
        catalog.registrations()[1].package_manifest.id,
        "weather_simulation"
    );
}

#[test]
fn native_reports_register_manifest_module_descriptor_projection() {
    let package_report = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "weather.base.runtime",
                    "zircon_plugin_weather_runtime",
                )
                .with_description("Weather base native module")
                .with_init_level(InitLevel::Kernel)
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
            .with_capability("runtime.feature.weather.storms")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "weather.storms.runtime",
                    "zircon_plugin_weather_storms_runtime",
                )
                .with_description("Weather storms feature module")
                .with_init_level(InitLevel::Scene)
                .with_module_dependency(ModuleDependencySpec::named("weather.base.runtime"))
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
