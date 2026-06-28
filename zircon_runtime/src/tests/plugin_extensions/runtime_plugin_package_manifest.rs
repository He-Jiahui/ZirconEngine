use std::sync::Arc;

use crate::asset::AssetImporterDescriptor;
use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::core::framework::bridge::PluginInterface;
use crate::core::framework::script::{ScriptHostParameterDescriptor, ScriptHostValueKind};
use crate::plugin::{
    CapabilityStatus, CapabilityStatusManifest, ExportPackagingStrategy, ExportTargetPlatform,
    PluginDependencyManifest, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginInterfaceManifest, PluginInterfaceMethodManifest, PluginModuleManifest,
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin,
    RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};

#[path = "runtime_plugin_package_manifest/feature_modules.rs"]
mod feature_modules;

#[test]
fn runtime_plugin_registration_report_rejects_package_manifest_id_mismatch() {
    let plugin = ManifestOverrideRuntimePlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .with_capability("runtime.plugin.weather")
        .build(),
        manifest: PluginPackageManifest::new("storm", "Weather")
            .with_capability("runtime.plugin.storm"),
    };
    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package manifest id `storm`")
            && diagnostic.contains("descriptor package_id `weather`")
    }));

    let catalog = RuntimePluginCatalog::from_registration_reports([registration], []);

    assert!(!catalog.is_success());
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("package manifest id `storm`")
            && diagnostic.contains("descriptor package_id `weather`")
    }));
}

#[test]
fn runtime_plugin_registration_report_rejects_invalid_package_manifest_public_metadata() {
    let mut manifest = PluginPackageManifest::new("weather", " Weather ")
        .with_capability("runtime.plugin.weather");
    manifest.category = " runtime ".to_string();
    manifest.description = " weather ".to_string();
    manifest.version = "1.0".to_string();
    manifest.sdk_api_version = "0.01.0".to_string();
    let plugin = ManifestOverrideRuntimePlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .with_capability("runtime.plugin.weather")
        .build(),
        manifest,
    };
    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("display_name ` Weather `")
            && diagnostic.contains("non-empty and trimmed")));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("category ` runtime `")
            && diagnostic.contains("non-empty and trimmed")));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("description ` weather `")
            && diagnostic.contains("trimmed")));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("version `1.0`")
            && diagnostic.contains("MAJOR.MINOR.PATCH")));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("sdk_api_version `0.01.0`")
            && diagnostic.contains("must not use leading zeroes")));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_bridge_interface_declarations() {
    let mut manifest = PluginPackageManifest::new("weather", "Weather")
        .with_capability("runtime.plugin.weather")
        .with_default_packaging([ExportPackagingStrategy::SourceTemplate])
        .with_provided_interface(PluginInterfaceManifest::new("weather.query.v1"))
        .with_provided_interface(PluginInterfaceManifest::new("weather.query.v1"))
        .with_provided_interface(PluginInterfaceManifest::new(" weather.bad.v1 "))
        .with_dependency(
            PluginDependencyManifest::new("physics", true)
                .with_capability("runtime.plugin.physics")
                .with_interfaces(["physics.query.v1", "physics.query.v1", "physics.Bad.v1"]),
        );
    manifest.description = "weather".to_string();

    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(manifest);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("provided interface `weather.query.v1`")
            && diagnostic.contains("unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("provided interface id ` weather.bad.v1 `")
            && diagnostic.contains("non-empty and trimmed")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("dependency `physics` interface `physics.query.v1`")
            && diagnostic.contains("unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("dependency interface id `physics.Bad.v1`")
            && diagnostic.contains("lowercase ASCII")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_bridge_method_metadata() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_default_packaging([ExportPackagingStrategy::SourceTemplate])
            .with_provided_interface(
                PluginInterfaceManifest::new("weather.query.v1")
                    .with_method(
                        PluginInterfaceMethodManifest::new("SampleTemperature", 0)
                            .with_parameter(ScriptHostParameterDescriptor::new(
                                "Region",
                                ScriptHostValueKind::String,
                            ))
                            .with_required_capability("Runtime.Plugin.Weather.Query"),
                    )
                    .with_method(PluginInterfaceMethodManifest::new("sample_temperature", 0)),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("provided interface method name `SampleTemperature`")
            && diagnostic.contains("lowercase ASCII")
    }));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("method slot 0") && diagnostic.contains("unique")));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("method parameter name `Region`")
            && diagnostic.contains("lowercase ASCII")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("required capability `Runtime.Plugin.Weather.Query`")
            && diagnostic.contains("lowercase ASCII")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_accepts_interface_only_dependency_rows() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_default_packaging([ExportPackagingStrategy::SourceTemplate])
            .with_dependency(
                PluginDependencyManifest::new("physics", true)
                    .with_interfaces(["physics.query.v1", "physics.force.v1"]),
            ),
    );

    assert!(registration.is_success());
    assert!(registration.diagnostics.is_empty());
}

#[test]
fn linked_runtime_plugin_registration_report_rejects_declared_but_unexported_interfaces() {
    let plugin = ManifestOverrideRuntimePlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .with_capability("runtime.plugin.weather")
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .build(),
        manifest: PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_runtime_module(
                PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_runtime")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["runtime.plugin.weather"]),
            )
            .with_provided_interface_id("weather.query.v1"),
    };

    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("declares interface `weather.query.v1`")
            && diagnostic.contains("did not export it")
    }));
}

#[test]
fn linked_runtime_plugin_registration_report_rejects_exported_but_undeclared_interfaces() {
    let plugin = InterfaceExportRuntimePlugin {
        manifest: PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_runtime_module(
                PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_runtime")
                    .with_target_modes([RuntimeTargetMode::ClientRuntime])
                    .with_capabilities(["runtime.plugin.weather"]),
            ),
    };

    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("exported interface `weather.query.v1`")
            && diagnostic.contains("package manifest did not declare it")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_manifest_identity() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("Weather", "Weather").with_capability("runtime.plugin.weather"),
    );

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("id `Weather`")
            && diagnostic.contains("lowercase ASCII")));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_static_package_id_shape_violations() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("1weather__", "Weather")
            .with_capability("runtime.plugin.weather"),
    );

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("id `1weather__`")
            && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("id `1weather__`")
            && diagnostic.contains("must not end with an underscore")));
}

#[test]
fn native_runtime_plugin_registration_report_accepts_dot_namespaced_package_id() {
    let manifest = PluginPackageManifest::new("asset_importer.shader", "Shader Asset Importers")
        .with_capability("runtime.plugin.asset_importer.shader")
        .with_asset_importer(
            AssetImporterDescriptor::new(
                "asset_importer.shader.wgsl",
                "asset_importer.shader",
                crate::asset::AssetKind::Shader,
                1,
            )
            .with_source_extensions(["wgsl"])
            .with_required_capabilities(["runtime.plugin.asset_importer.shader"]),
        );

    assert_eq!(manifest.package_name, "asset_importer_shader");
    assert_eq!(manifest.package_id(), "com.zircon.asset_importer_shader");

    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(manifest);

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert!(registration.diagnostics.is_empty());
}

#[test]
fn runtime_plugin_registration_report_rejects_descriptor_package_id_shape_violations() {
    let plugin = RuntimePluginDescriptor::builder(
        "1weather__",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_capability("runtime.plugin.weather")
    .build();

    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("package_id `1weather__`")
            && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("package_id `1weather__`")
            && diagnostic.contains("must not end with an underscore")));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_coordinates() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_package_identity("com..zircon", "", "Weather"),
    );

    assert!(!registration.is_success());
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("package coordinates")
            && diagnostic.contains("package_prefix, package_company, and package_name")));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package_prefix `com..zircon`")
            && diagnostic.contains("coordinate segments")
    }));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("package_company ``")
            && diagnostic.contains("coordinate segment")));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("package_name `Weather`")
            && diagnostic.contains("coordinate segment")));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_layout_arrays() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::ClientRuntime,
            ])
            .with_supported_platforms([
                ExportTargetPlatform::Windows,
                ExportTargetPlatform::Windows,
            ])
            .with_asset_roots(["assets", "assets", "../shared", "bad\\path", " trailing "])
            .with_content_roots(["content/./bad", "/absolute", "content//bad"]),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("supported_targets target mode ClientRuntime")
            && diagnostic.contains("unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("supported_platforms platform Windows") && diagnostic.contains("unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("asset_roots root `assets`") && diagnostic.contains("unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("asset_roots root `bad\\path`")
            && diagnostic.contains("forward slashes")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("asset_roots root ` trailing `") && diagnostic.contains("trimmed")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("content_roots root `/absolute`") && diagnostic.contains("relative")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("content_roots root `content/./bad`")
            && diagnostic.contains("path segments")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("content_roots root `content//bad`")
            && diagnostic.contains("path segments")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_empty_package_default_packaging() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_default_packaging(Vec::<ExportPackagingStrategy>::new()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package manifest default_packaging")
            && diagnostic.contains("at least one")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_duplicate_package_default_packaging() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_default_packaging([
                ExportPackagingStrategy::NativeDynamic,
                ExportPackagingStrategy::NativeDynamic,
            ]),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package manifest default_packaging strategy NativeDynamic")
            && diagnostic.contains("unique")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_empty_package_capabilities() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather"),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package manifest capabilities") && diagnostic.contains("at least one")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_capabilities() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("Runtime.Plugin.Weather")
            .with_capability("runtime.plugin.weather")
            .with_capability("runtime.plugin.weather"),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package capability `Runtime.Plugin.Weather`")
            && diagnostic.contains("lowercase ASCII")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package manifest capability `runtime.plugin.weather`")
            && diagnostic.contains("unique")
    }));
}

#[test]
fn runtime_plugin_registration_report_rejects_invalid_descriptor_package_capabilities() {
    let plugin = RuntimePluginDescriptor::builder(
        "weather",
        "Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_weather_runtime",
    )
    .with_target_modes([RuntimeTargetMode::ClientRuntime])
    .with_capability("Runtime.Plugin.Weather")
    .with_capability("runtime.plugin.weather")
    .with_capability("runtime.plugin.weather")
    .build();

    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package capability `Runtime.Plugin.Weather`")
            && diagnostic.contains("lowercase ASCII")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("package manifest capability `runtime.plugin.weather`")
            && diagnostic.contains("unique")
    }));

    let catalog = RuntimePluginCatalog::from_registration_reports([registration], []);

    assert!(!catalog.is_success());
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("package manifest capability `runtime.plugin.weather`")
            && diagnostic.contains("unique")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_dependencies() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_dependency(PluginDependencyManifest::new("Weather.Core", true))
            .with_dependency(
                PluginDependencyManifest::new("weather", true)
                    .with_capability("Runtime.Plugin.Weather"),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("dependency id `Weather.Core`")
            && diagnostic.contains("lowercase ASCII")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("dependency `Weather.Core`")
            && diagnostic.contains("declare a capability")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("dependency capability `Runtime.Plugin.Weather`")
            && diagnostic.contains("lowercase ASCII")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_duplicate_package_dependencies() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_dependency(
                PluginDependencyManifest::new("asset", true)
                    .with_capability("runtime.module.asset"),
            )
            .with_dependency(
                PluginDependencyManifest::new("asset", false)
                    .with_capability("runtime.module.asset"),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("dependency `asset` capability `runtime.module.asset`")
            && diagnostic.contains("unique")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_package_asset_importer_metadata() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_asset_importer(
                AssetImporterDescriptor::new(
                    "Weather.Data",
                    "storm",
                    crate::asset::AssetKind::Data,
                    0,
                )
                .with_source_extensions(["weather"])
                .with_required_capabilities([
                    "Runtime.Asset.Importer.Weather",
                    "runtime.asset.importer.weather",
                    "runtime.asset.importer.weather",
                ]),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("asset importer id `Weather.Data`")
            && diagnostic.contains("lowercase ASCII")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("asset importer `Weather.Data` plugin_id `storm`")
            && diagnostic.contains("package id `weather`")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("asset importer `Weather.Data` importer_version")
            && diagnostic.contains("positive")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("asset importer required capability `Runtime.Asset.Importer.Weather`")
            && diagnostic.contains("lowercase ASCII")
    }));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains(
            "asset importer `Weather.Data` required capability `runtime.asset.importer.weather`"
        ) && diagnostic.contains("unique")));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_duplicate_package_asset_importers() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_asset_importer(
                AssetImporterDescriptor::new(
                    "weather.data",
                    "weather",
                    crate::asset::AssetKind::Data,
                    1,
                )
                .with_source_extensions(["weather"]),
            )
            .with_asset_importer(
                AssetImporterDescriptor::new(
                    "weather.data",
                    "weather",
                    crate::asset::AssetKind::Data,
                    1,
                )
                .with_source_extensions(["storm"]),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("asset importer id `weather.data`") && diagnostic.contains("unique")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_capability_status_capabilities() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_capability_status(CapabilityStatusManifest::new(
                "Runtime.Plugin.Weather",
                CapabilityStatus::Partial,
            ))
            .with_capability_status(CapabilityStatusManifest::new(
                "runtime.plugin.storm",
                CapabilityStatus::Stub,
            )),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("capability status capability `Runtime.Plugin.Weather`")
            && diagnostic.contains("lowercase ASCII")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("capability status `runtime.plugin.storm`")
            && diagnostic.contains("same package")
    }));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_duplicate_capability_status_targets() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_supported_targets([RuntimeTargetMode::ClientRuntime])
            .with_capability_status(
                CapabilityStatusManifest::new("runtime.plugin.weather", CapabilityStatus::Partial)
                    .with_target_modes([
                        RuntimeTargetMode::ClientRuntime,
                        RuntimeTargetMode::ClientRuntime,
                        RuntimeTargetMode::EditorHost,
                    ]),
            )
            .with_capability_status(CapabilityStatusManifest::new(
                "runtime.plugin.weather",
                CapabilityStatus::Stub,
            )),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("capability status `runtime.plugin.weather`")
            && diagnostic.contains("must be unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("target mode ClientRuntime") && diagnostic.contains("unique")
    }));
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("target mode EditorHost")
            && diagnostic.contains("supported_targets")));
}

#[test]
fn native_runtime_plugin_registration_report_rejects_invalid_capability_status_bevy_metadata() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_capability_status(
                CapabilityStatusManifest::new("runtime.plugin.weather", CapabilityStatus::Partial)
                    .with_bevy_reference("../bevy/crates/bevy_app/src/plugin.rs")
                    .with_bevy_reference("../bevy/crates/bevy_app/src/plugin.rs")
                    .with_note(" partial parity "),
            ),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("bevy reference `../bevy/crates/bevy_app/src/plugin.rs`")
            && diagnostic.contains("dev/bevy")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("bevy reference `../bevy/crates/bevy_app/src/plugin.rs`")
            && diagnostic.contains("unique")
    }));
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("capability status note ` partial parity `")
            && diagnostic.contains("non-empty and trimmed")
    }));
}

#[derive(Debug)]
struct ManifestOverrideRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
    manifest: PluginPackageManifest,
}

impl RuntimePlugin for ManifestOverrideRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        self.manifest.clone()
    }
}

#[derive(Debug)]
struct InterfaceExportRuntimePlugin {
    manifest: PluginPackageManifest,
}

impl RuntimePlugin for InterfaceExportRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        static DESCRIPTOR: std::sync::OnceLock<RuntimePluginDescriptor> =
            std::sync::OnceLock::new();
        DESCRIPTOR.get_or_init(|| {
            RuntimePluginDescriptor::builder(
                "weather",
                "Weather",
                RuntimePluginId::Particles,
                "zircon_plugin_weather_runtime",
            )
            .with_capability("runtime.plugin.weather")
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .build()
        })
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        self.manifest.clone()
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        let owner = registry.intern_plugin_module("weather.runtime")?;
        registry
            .export_interface::<dyn WeatherQueryInterface>(owner, Arc::new(WeatherQueryProvider))
    }
}

trait WeatherQueryInterface: Send + Sync {
    fn sample_temperature(&self) -> i32;
}

impl PluginInterface for dyn WeatherQueryInterface {
    const INTERFACE_ID: &'static str = "weather.query.v1";
}

#[derive(Debug)]
struct WeatherQueryProvider;

impl WeatherQueryInterface for WeatherQueryProvider {
    fn sample_temperature(&self) -> i32 {
        21
    }
}
