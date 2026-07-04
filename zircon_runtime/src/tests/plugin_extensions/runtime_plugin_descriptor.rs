use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor};
use crate::plugin::{
    ExportPackagingStrategy, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginModuleKind, PluginModuleManifest, RuntimeExtensionRegistry, RuntimePlugin,
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginDescriptorBuilder,
    RuntimePluginRegistrationReport,
};
use crate::scene::SystemStage;

#[test]
fn runtime_plugin_registration_report_rejects_invalid_descriptor_package_ids() {
    let uppercase = RuntimePluginDescriptor::builder(
        "Weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("runtime.plugin.weather")
    .build();
    let registration = RuntimePluginRegistrationReport::from_plugin(&uppercase);

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("package_id `Weather`")
            && diagnostic.contains("lowercase ASCII")));

    let empty_segment = RuntimePluginDescriptor::builder(
        "weather..layer",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("runtime.plugin.weather")
    .build();
    let catalog = RuntimePluginCatalog::from_descriptors([empty_segment]);

    assert!(!catalog.is_success());
    assert!(catalog.diagnostics().iter().any(|diagnostic| diagnostic
        .contains("package_id `weather..layer`")
        && diagnostic.contains("non-empty segments")));
}

#[test]
fn runtime_plugin_registration_report_rejects_invalid_descriptor_display_names() {
    let descriptor = RuntimePluginDescriptor::builder(
        "weather",
        " Weather ",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("runtime.plugin.weather")
    .build();
    let registration = RuntimePluginRegistrationReport::from_plugin(&descriptor);

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("display_name ` Weather `")
            && diagnostic.contains("non-empty and trimmed")));
}

#[test]
fn runtime_plugin_registration_report_rejects_invalid_descriptor_crate_names() {
    let hyphenated = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon-plugin-weather-runtime",
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("runtime.plugin.weather")
    .build();
    let registration = RuntimePluginRegistrationReport::from_plugin(&hyphenated);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("crate_name `zircon-plugin-weather-runtime`")
        && diagnostic.contains("lowercase ASCII")));

    let missing_prefix = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "weather_runtime",
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("runtime.plugin.weather")
    .build();
    let registration = RuntimePluginRegistrationReport::from_plugin(&missing_prefix);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("descriptor crate_name `weather_runtime`")
        && diagnostic.contains("`zircon_plugin_` prefix")));

    let repeated_underscore = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather__runtime",
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("runtime.plugin.weather")
    .build();
    let registration = RuntimePluginRegistrationReport::from_plugin(&repeated_underscore);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("descriptor crate_name `zircon_plugin_weather__runtime`")
        && diagnostic.contains("repeated underscores")));
}

#[test]
fn runtime_plugin_registration_report_rejects_empty_descriptor_default_packaging() {
    let descriptor = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("runtime.plugin.weather")
    .with_default_packaging(Vec::<ExportPackagingStrategy>::new())
    .build();
    let registration = RuntimePluginRegistrationReport::from_plugin(&descriptor);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("descriptor default_packaging")
        && diagnostic.contains("at least one")));
}

#[test]
fn runtime_plugin_registration_report_rejects_duplicate_descriptor_default_packaging() {
    let descriptor = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("runtime.plugin.weather")
    .with_default_packaging([
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::LibraryEmbed,
    ])
    .build();
    let registration = RuntimePluginRegistrationReport::from_plugin(&descriptor);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("descriptor default_packaging strategy LibraryEmbed")
        && diagnostic.contains("unique")));
}

#[test]
fn runtime_plugin_registration_report_rejects_invalid_descriptor_target_modes() {
    let empty = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_capability("runtime.plugin.weather")
    .build();
    let registration = RuntimePluginRegistrationReport::from_plugin(&empty);

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("descriptor target_modes")
            && diagnostic.contains("at least one target mode")));

    let duplicate = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ClientRuntime,
    ])
    .with_capability("runtime.plugin.weather")
    .build();
    let registration = RuntimePluginRegistrationReport::from_plugin(&duplicate);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("descriptor target mode ClientRuntime")
        && diagnostic.contains("unique")));
}

#[test]
fn runtime_plugin_descriptor_projects_public_metadata_to_package_manifest() {
    let descriptor = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_category("simulation")
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.weather")
    .with_capability("runtime.capability.weather.forecast")
    .with_provided_interface_id("weather.query.v1")
    .with_system_sets(["weather.main", "weather.simulation"])
    .with_system_anchors(["weather.tick"])
    .with_optional_feature(sound_timeline_feature_manifest())
    .build();

    let manifest = descriptor.package_manifest();

    assert_eq!(manifest.category, "simulation");
    assert_eq!(
        manifest.supported_targets,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost
        ]
    );
    assert_eq!(
        manifest.capabilities,
        vec![
            "runtime.plugin.weather".to_string(),
            "runtime.capability.weather.forecast".to_string()
        ]
    );
    assert_eq!(manifest.optional_features.len(), 1);
    assert_eq!(manifest.provides_interfaces.len(), 1);
    assert_eq!(manifest.provides_interfaces[0].id, "weather.query.v1");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("runtime module");
    assert_eq!(runtime_module.name, descriptor.module_descriptor().name);
    assert_eq!(
        runtime_module.capabilities,
        vec![
            "runtime.plugin.weather".to_string(),
            "runtime.capability.weather.forecast".to_string()
        ]
    );
    assert_eq!(
        runtime_module.system_sets,
        vec!["weather.main".to_string(), "weather.simulation".to_string()]
    );
    assert_eq!(
        runtime_module.system_anchors,
        vec!["weather.tick".to_string()]
    );
}

#[test]
fn runtime_plugin_descriptor_builder_matches_fluent_descriptor_projection() {
    let builder: RuntimePluginDescriptorBuilder = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    );
    let built = builder
        .with_category("simulation")
        .with_required_by_default(true)
        .with_enabled_by_default(false)
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.plugin.weather")
        .with_capability("runtime.capability.weather.forecast")
        .with_system_sets(["weather.main", "weather.simulation"])
        .with_system_anchors(["weather.tick"])
        .with_default_packaging([
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
        ])
        .with_optional_feature(sound_timeline_feature_manifest())
        .build();

    let expected = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_category("simulation")
    .with_required_by_default(true)
    .with_enabled_by_default(false)
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.weather")
    .with_capability("runtime.capability.weather.forecast")
    .with_system_sets(["weather.main", "weather.simulation"])
    .with_system_anchors(["weather.tick"])
    .with_default_packaging([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
    ])
    .with_optional_feature(sound_timeline_feature_manifest())
    .build();

    assert_eq!(built, expected);
    let manifest = built.package_manifest();
    assert_eq!(manifest.category, "simulation");
    assert_eq!(
        manifest.supported_targets,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost
        ]
    );
    assert_eq!(manifest.optional_features.len(), 1);
}

#[test]
fn runtime_plugin_descriptor_projects_embedded_module_descriptor_to_manifest() {
    let descriptor = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_module_descriptor(
        ModuleDescriptor::new("weather.simulation.runtime", "Weather simulation runtime")
            .with_init_level(InitLevel::Scene)
            .with_module_dependency(ModuleDependencySpec::named("SceneModule")),
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("runtime.plugin.weather")
    .build();

    assert_eq!(
        descriptor.module_descriptor().name,
        "weather.simulation.runtime"
    );
    assert_eq!(descriptor.module_descriptor().init_level, InitLevel::Scene);
    assert_eq!(
        descriptor.module_descriptor().module_dependencies,
        vec![ModuleDependencySpec::named("SceneModule")]
    );

    let manifest = descriptor.package_manifest();
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("runtime module");

    assert_eq!(runtime_module.name, "weather.simulation.runtime");
    assert_eq!(runtime_module.crate_name, "zircon_plugin_weather_runtime");
    assert_eq!(
        runtime_module.target_modes,
        vec![RuntimeTargetMode::ClientRuntime]
    );
    assert_eq!(
        runtime_module.capabilities,
        vec!["runtime.plugin.weather".to_string()]
    );
}

#[test]
fn runtime_plugin_registration_report_validates_declared_system_anchors() {
    let missing = WeatherAnchorPlugin::new(WeatherAnchorRegistrationMode::Missing);
    let registration = RuntimePluginRegistrationReport::from_plugin(&missing);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("weather.runtime")
            && diagnostic.contains("system anchor `weather.tick`")
            && diagnostic.contains("did not register")
    }));

    let wrong_owner = WeatherAnchorPlugin::new(WeatherAnchorRegistrationMode::WrongOwner);
    let registration = RuntimePluginRegistrationReport::from_plugin(&wrong_owner);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("weather.runtime")
            && diagnostic.contains("system anchor `weather.tick`")
            && diagnostic.contains("did not register")
    }));

    let registered = WeatherAnchorPlugin::new(WeatherAnchorRegistrationMode::DeclaredOwner);
    let registration = RuntimePluginRegistrationReport::from_plugin(&registered);

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration
        .extensions
        .plugin_systems()
        .any(|(_, system)| system.id == "weather.tick"));
}

#[test]
fn runtime_plugin_descriptor_projects_default_packaging_to_project_selection() {
    let descriptor = RuntimePluginDescriptor::builder(
        "native_weather",
        "Native Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_native_weather_runtime",
    )
    .with_default_packaging([ExportPackagingStrategy::NativeDynamic])
    .build();

    let selection = descriptor.project_selection();

    assert_eq!(selection.packaging, ExportPackagingStrategy::NativeDynamic);
}

fn sound_timeline_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        "sound.timeline_animation_track",
        "Timeline Animation Track",
        "sound",
    )
    .with_dependency(PluginFeatureDependency::primary(
        "sound",
        "runtime.plugin.sound",
    ))
    .with_dependency(PluginFeatureDependency::required(
        "animation",
        "runtime.feature.animation.timeline_event_track",
    ))
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.timeline_animation_track.runtime",
            "zircon_plugin_sound_timeline_animation_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(["runtime.feature.sound.timeline_animation_track"]),
    )
    .with_editor_module(
        PluginModuleManifest::editor(
            "sound.timeline_animation_track.editor",
            "zircon_plugin_sound_timeline_animation_editor",
        )
        .with_capabilities(["editor.feature.sound.timeline_animation_track"]),
    )
}

#[derive(Clone, Debug)]
struct WeatherAnchorPlugin {
    descriptor: RuntimePluginDescriptor,
    registration_mode: WeatherAnchorRegistrationMode,
}

impl WeatherAnchorPlugin {
    fn new(registration_mode: WeatherAnchorRegistrationMode) -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                "weather",
                "Weather",
                RuntimePluginId::Particles,
                "zircon_plugin_weather_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.weather")
            .with_system_sets(["weather.main"])
            .with_system_anchors(["weather.tick"])
            .build(),
            registration_mode,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WeatherAnchorRegistrationMode {
    Missing,
    WrongOwner,
    DeclaredOwner,
}

impl RuntimePlugin for WeatherAnchorPlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), crate::plugin::RuntimeExtensionRegistryError> {
        let owner = match self.registration_mode {
            WeatherAnchorRegistrationMode::Missing => return Ok(()),
            WeatherAnchorRegistrationMode::WrongOwner => {
                registry.intern_plugin_module("weather.tools")?
            }
            WeatherAnchorRegistrationMode::DeclaredOwner => {
                registry.intern_plugin_module("weather.runtime")?
            }
        };
        let set = registry.intern_system_set("weather.main")?;
        registry
            .register_native_system::<(), _>(owner, "weather.tick", SystemStage::Update, |()| {})
            .in_set(set)
            .register()
    }
}
