use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::plugin::{
    PluginOptionManifest, PluginPackageManifest, RuntimeExtensionRegistry,
    RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginCatalog, RuntimePluginDescriptor,
    RuntimePluginRegistrationReport,
};

#[test]
fn runtime_plugin_extension_registry_preserves_enum_option_value_sets() {
    let expected_enum_values = vec![
        "disabled".to_string(),
        "preview".to_string(),
        "balanced".to_string(),
        "cinematic".to_string(),
    ];
    let plugin = EnumOptionRuntimePlugin {
        descriptor: RuntimePluginDescriptor::new(
            "sound",
            "Sound",
            RuntimePluginId::Sound,
            "zircon_plugin_sound_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.sound"),
    };

    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);
    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert_eq!(
        enum_option_values(
            registration.extensions.plugin_options(),
            "sound.ray_tracing_quality"
        ),
        expected_enum_values
    );
    assert!(
        option(registration.extensions.plugin_options(), "sound.enabled")
            .enum_values
            .is_empty(),
        "non-enum plugin options should remain empty after registration report collection"
    );

    let report =
        RuntimePluginCatalog::from_registration_reports([registration], []).runtime_extensions();
    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(
        enum_option_values(
            report.registry.plugin_options(),
            "sound.ray_tracing_quality"
        ),
        expected_enum_values
    );
}

#[test]
fn runtime_plugin_registration_report_validates_shadowed_manifest_plugin_options() {
    let plugin = ShadowedInvalidOptionRuntimePlugin {
        descriptor: RuntimePluginDescriptor::new(
            "sound",
            "Sound",
            RuntimePluginId::Sound,
            "zircon_plugin_sound_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.sound"),
    };

    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(
        !registration.is_success(),
        "shadowed invalid manifest option should remain diagnostic: {:?}",
        registration.diagnostics
    );
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("must declare enum_values")));
    assert_eq!(
        enum_option_values(
            registration.extensions.plugin_options(),
            "sound.ray_tracing_quality"
        ),
        vec!["balanced".to_string()]
    );

    let report =
        RuntimePluginCatalog::from_registration_reports([registration], []).runtime_extensions();
    assert!(report.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("runtime plugin sound diagnostic")
        && diagnostic.contains("must declare enum_values")));
}

#[test]
fn native_runtime_plugin_registration_report_diagnoses_duplicate_manifest_plugin_options() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("sound", "Sound")
            .with_capability("runtime.plugin.sound")
            .with_option(PluginOptionManifest::new(
                "sound.enabled",
                "Enabled",
                "bool",
                "true",
            ))
            .with_option(PluginOptionManifest::new(
                "sound.enabled",
                "Enabled Duplicate",
                "bool",
                "false",
            )),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("option key `sound.enabled`")
        && diagnostic.contains("unique")));
    assert_eq!(registration.extensions.plugin_options().len(), 1);
}

#[test]
fn runtime_extension_registry_rejects_invalid_plugin_option_enum_values() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_plugin_option(PluginOptionManifest::new(
            "sound.ray_tracing_quality",
            "Ray tracing quality",
            "enum",
            "balanced",
        ))
        .unwrap_err();
    assert!(error.to_string().contains("must declare enum_values"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_plugin_option(
            PluginOptionManifest::new(
                "sound.ray_tracing_quality",
                "Ray tracing quality",
                "enum",
                "balanced",
            )
            .with_enum_values(["disabled", "preview", "cinematic"]),
        )
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("must be declared in enum_values"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_plugin_option(
            PluginOptionManifest::new(
                "sound.ray_tracing_quality",
                "Ray tracing quality",
                "enum",
                "balanced",
            )
            .with_enum_values(["disabled", "balanced", "balanced"]),
        )
        .unwrap_err();
    assert!(error.to_string().contains("must be unique"));
}

#[test]
fn runtime_extension_registry_rejects_non_enum_options_with_enum_values() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_plugin_option(
            PluginOptionManifest::new("sound.enabled", "Enabled", "bool", "true")
                .with_enum_values(["true", "false"]),
        )
        .unwrap_err();

    assert!(error.to_string().contains("must not declare enum_values"));
}

#[test]
fn runtime_extension_registry_rejects_unknown_option_value_types() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_plugin_option(PluginOptionManifest::new(
            "sound.ray_tracing_quality",
            "Ray tracing quality",
            "choice",
            "balanced",
        ))
        .unwrap_err();

    assert!(error.to_string().contains("value_type `choice`"));
}

#[test]
fn runtime_extension_registry_rejects_non_namespaced_plugin_option_keys() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_plugin_option(PluginOptionManifest::new(
            "ray_tracing_quality",
            "Ray tracing quality",
            "enum",
            "balanced",
        ))
        .unwrap_err();

    assert!(error.to_string().contains("dot-separated namespace"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_plugin_option(PluginOptionManifest::new(
            "sound.RayTracingQuality",
            "Ray tracing quality",
            "enum",
            "balanced",
        ))
        .unwrap_err();

    assert!(error.to_string().contains("lowercase ASCII"));
}

#[test]
fn runtime_extension_registry_rejects_non_namespaced_plugin_option_required_capabilities() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_plugin_option(
            PluginOptionManifest::new("sound.enabled", "Enabled", "bool", "true")
                .with_required_capability("sound"),
        )
        .unwrap_err();

    assert!(error.to_string().contains("dot-separated namespace"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_plugin_option(
            PluginOptionManifest::new("sound.enabled", "Enabled", "bool", "true")
                .with_required_capability("Runtime.Plugin.Sound"),
        )
        .unwrap_err();

    assert!(error.to_string().contains("lowercase ASCII"));
}

#[derive(Debug)]
struct EnumOptionRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for EnumOptionRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor()
            .package_manifest()
            .with_option(
                PluginOptionManifest::new(
                    "sound.ray_tracing_quality",
                    "Ray tracing quality",
                    "enum",
                    "balanced",
                )
                .with_enum_values(["disabled", "preview", "balanced", "cinematic"]),
            )
            .with_option(PluginOptionManifest::new(
                "sound.enabled",
                "Enabled",
                "bool",
                "true",
            ))
    }
}

#[derive(Debug)]
struct ShadowedInvalidOptionRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for ShadowedInvalidOptionRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor()
            .package_manifest()
            .with_option(PluginOptionManifest::new(
                "sound.ray_tracing_quality",
                "Ray tracing quality",
                "enum",
                "balanced",
            ))
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_plugin_option(
            PluginOptionManifest::new(
                "sound.ray_tracing_quality",
                "Ray tracing quality",
                "enum",
                "balanced",
            )
            .with_enum_values(["balanced"]),
        )
    }
}

fn enum_option_values(options: &[PluginOptionManifest], key: &str) -> Vec<String> {
    option(options, key).enum_values.clone()
}

fn option<'a>(options: &'a [PluginOptionManifest], key: &str) -> &'a PluginOptionManifest {
    options
        .iter()
        .find(|option| option.key.as_str() == key)
        .unwrap_or_else(|| panic!("expected plugin option `{key}` to be registered"))
}
