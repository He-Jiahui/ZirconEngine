use crate::asset::AssetImporterDescriptor;
use crate::plugin::{
    ComponentTypeDescriptor, ExportPackagingStrategy, ExportTargetPlatform,
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleKind, PluginModuleManifest,
    PluginPackageKind, PluginPackageManifest, UiComponentDescriptor,
};
use crate::RuntimeTargetMode;

#[test]
fn plugin_package_manifest_declares_runtime_and_editor_contributions() {
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_category("environment")
        .with_runtime_crate("zircon_plugin_weather_runtime")
        .with_editor_crate("zircon_plugin_weather_editor")
        .with_component(ComponentTypeDescriptor::new(
            "weather.Component.CloudLayer",
            "weather",
            "Cloud Layer",
        ))
        .with_ui_component(UiComponentDescriptor::new(
            "weather.Ui.CloudLayerInspector",
            "weather",
            "asset://weather/editor/cloud_layer_inspector.zui",
        ));

    assert_eq!(manifest.components.len(), 1);
    assert_eq!(manifest.category, "environment");
    assert_eq!(
        manifest.components[0].type_id,
        "weather.Component.CloudLayer"
    );
    assert_eq!(manifest.ui_components.len(), 1);
    assert_eq!(
        manifest.ui_components[0].ui_document,
        "asset://weather/editor/cloud_layer_inspector.zui"
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_declares_public_package_metadata() {
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_sdk_api_version("0.2.0")
        .with_category("simulation")
        .with_supported_targets([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_supported_platforms([ExportTargetPlatform::Windows, ExportTargetPlatform::Linux])
        .with_capabilities([
            "runtime.plugin.weather",
            "runtime.capability.weather.forecast",
        ])
        .with_asset_root("assets")
        .with_content_root("content")
        .with_runtime_module(
            PluginModuleManifest::runtime("weather.runtime", "zircon_plugin_weather_runtime")
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(["runtime.plugin.weather"]),
        )
        .with_native_module(
            PluginModuleManifest::native("weather.native", "zircon_plugin_weather_native")
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.native.weather"]),
        )
        .with_vm_module(
            PluginModuleManifest::vm("weather.vm", "weather_vm_module")
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.vm.weather"]),
        );

    assert_eq!(manifest.sdk_api_version, "0.2.0");
    assert_eq!(manifest.category, "simulation");
    assert_eq!(
        manifest.supported_targets,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost
        ]
    );
    assert_eq!(
        manifest.supported_platforms,
        vec![ExportTargetPlatform::Windows, ExportTargetPlatform::Linux]
    );
    assert_eq!(
        manifest.capabilities,
        vec![
            "runtime.plugin.weather".to_string(),
            "runtime.capability.weather.forecast".to_string()
        ]
    );
    assert_eq!(manifest.asset_roots, vec!["assets".to_string()]);
    assert_eq!(manifest.package_id(), "com.zircon.weather");
    assert_eq!(
        manifest.asset_roots_or_default(),
        vec!["assets".to_string()]
    );
    assert_eq!(manifest.content_roots, vec!["content".to_string()]);
    assert!(manifest
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Native));
    assert!(manifest
        .modules
        .iter()
        .any(|module| module.kind == PluginModuleKind::Vm));

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    assert!(encoded.contains("sdk_api_version = \"0.2.0\""));
    assert!(encoded.contains("kind = \"native\""));
    assert!(encoded.contains("kind = \"vm\""));

    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_declares_asset_importer_descriptors() {
    let importer =
        AssetImporterDescriptor::new("weather.data", "weather", crate::asset::AssetKind::Data, 3)
            .with_source_extensions(["weather"])
            .with_full_suffixes([".weather.toml"])
            .with_required_capabilities(["runtime.asset.importer.data"]);
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_runtime_crate("zircon_plugin_weather_runtime")
        .with_asset_importer(importer.clone());

    assert_eq!(manifest.asset_importers, vec![importer]);

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_overrides_default_packaging() {
    let manifest = PluginPackageManifest::new("weather", "Weather")
        .with_default_packaging([ExportPackagingStrategy::NativeDynamic]);

    assert_eq!(
        manifest.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded.default_packaging, manifest.default_packaging);
}

#[test]
fn plugin_package_manifest_declares_optional_feature_bundles() {
    let feature = sound_timeline_feature_manifest();
    let manifest = PluginPackageManifest::new("sound", "Sound")
        .with_runtime_crate("zircon_plugin_sound_runtime")
        .with_optional_feature(feature.clone());

    assert_eq!(manifest.optional_features, vec![feature]);
    assert!(!manifest.optional_features[0].enabled_by_default);
    assert_eq!(manifest.optional_features[0].owner_plugin_id, "sound");
    assert!(manifest.optional_features[0]
        .dependencies
        .iter()
        .any(|dependency| dependency.plugin_id == "sound" && dependency.primary));
    assert!(manifest.optional_features[0]
        .modules
        .iter()
        .any(|module| module.crate_name == "zircon_plugin_sound_timeline_animation_runtime"));

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn plugin_package_manifest_declares_feature_extension_packages() {
    let feature = sound_timeline_feature_manifest();
    let manifest = PluginPackageManifest::new(
        "sound_timeline_animation_track",
        "Sound Timeline Animation Track Provider",
    )
    .as_feature_extension()
    .with_feature_extension(feature.clone());

    assert_eq!(manifest.package_kind, PluginPackageKind::FeatureExtension);
    assert!(manifest.optional_features.is_empty());
    assert_eq!(manifest.feature_extensions, vec![feature]);

    let encoded = toml::to_string(&manifest).expect("feature extension manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("feature extension manifest roundtrip");
    assert_eq!(decoded, manifest);
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
