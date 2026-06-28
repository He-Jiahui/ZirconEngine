use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::{
    asset::{AssetImporterDescriptor, AssetKind},
    plugin::{ExportPackagingStrategy, PluginMaturity, PluginModuleKind},
};

use super::{
    importer_runtime_supported_platforms, importer_runtime_supported_targets,
    ImporterRuntimeManifestBuilder, PluginFeatureBundleBuilder, PluginManifestBuilder,
    PluginModuleBuilder, NATIVE_ABI_VERSION_V3, NATIVE_DESCRIPTOR_SYMBOL_V3, SDK_API_VERSION,
};

#[test]
fn manifest_builder_declares_required_sdk_defaults_and_runtime_module() {
    let manifest = PluginManifestBuilder::new("physics", "Physics")
        .with_category("runtime")
        .with_description("Physics runtime plugin")
        .with_maturity(PluginMaturity::Beta)
        .with_supported_targets([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.physics")
        .with_module(
            PluginModuleBuilder::runtime("physics", "zircon_plugin_physics_runtime")
                .with_target_modes([RuntimeTargetMode::ClientRuntime])
                .with_capabilities(["runtime.plugin.physics"])
                .with_system_anchors(["physics.simulation"])
                .build(),
        )
        .build();

    assert_eq!(manifest.sdk_api_version, SDK_API_VERSION);
    assert_eq!(manifest.supported_platforms.len(), 3);
    assert_eq!(
        manifest.default_packaging,
        vec![
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed
        ]
    );
    assert_eq!(manifest.modules.len(), 1);
    assert_eq!(manifest.modules[0].name, "physics.runtime");
    assert_eq!(manifest.modules[0].kind, PluginModuleKind::Runtime);
    assert_eq!(
        manifest.modules[0].system_anchors,
        ["physics.simulation".to_string()]
    );
}

#[test]
fn editor_module_builder_targets_editor_host_by_default() {
    let module =
        PluginModuleBuilder::editor("plugin_sdk_examples", "zircon_plugin_sdk_examples_editor")
            .with_capabilities(["editor.extension.plugin_sdk_examples"])
            .build();

    assert_eq!(module.name, "plugin_sdk_examples.editor");
    assert_eq!(module.kind, PluginModuleKind::Editor);
    assert_eq!(module.target_modes, [RuntimeTargetMode::EditorHost]);
    assert_eq!(
        module.capabilities,
        ["editor.extension.plugin_sdk_examples".to_string()]
    );
}

#[test]
fn feature_bundle_builder_projects_capability_to_feature_and_modules() {
    let feature = PluginFeatureBundleBuilder::new(
        "sound.timeline_animation_track",
        "Sound Timeline Animation Track",
        "sound",
    )
    .with_primary_dependency("sound", "runtime.plugin.sound")
    .with_required_dependency(
        "animation",
        "runtime.feature.animation.timeline_event_track",
    )
    .with_runtime_capability_module(
        "runtime.feature.sound.timeline_animation_track",
        "sound.timeline_animation_track.runtime",
        "zircon_plugin_sound_timeline_animation_runtime",
        [
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
    )
    .with_editor_capability_module(
        "editor.feature.sound.timeline_animation_track",
        "sound.timeline_animation_track.editor",
        "zircon_plugin_sound_timeline_animation_editor",
    )
    .enabled_by_default(true)
    .build();

    assert_eq!(
        feature.capabilities,
        [
            "runtime.feature.sound.timeline_animation_track".to_string(),
            "editor.feature.sound.timeline_animation_track".to_string(),
        ]
    );
    assert_eq!(feature.dependencies.len(), 2);
    assert!(feature.dependencies[0].primary);
    assert!(!feature.dependencies[1].primary);
    assert_eq!(feature.modules.len(), 2);
    assert_eq!(feature.modules[0].kind, PluginModuleKind::Runtime);
    assert_eq!(
        feature.modules[0].capabilities,
        ["runtime.feature.sound.timeline_animation_track".to_string()]
    );
    assert_eq!(feature.modules[1].kind, PluginModuleKind::Editor);
    assert_eq!(
        feature.modules[1].target_modes,
        [RuntimeTargetMode::EditorHost]
    );
    assert_eq!(
        feature.modules[1].capabilities,
        ["editor.feature.sound.timeline_animation_track".to_string()]
    );
    assert!(feature.enabled_by_default);
}

#[test]
fn importer_runtime_manifest_builder_projects_dist_and_importer_manifest() {
    let descriptor = zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        "gltf_importer",
        "glTF Importer",
        zircon_runtime::builtin::RuntimePluginId::GltfImporter,
        "zircon_plugin_gltf_importer_runtime",
    )
    .with_category("asset_importer")
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.gltf_importer")
    .with_capability("runtime.asset.importer.gltf")
    .build();
    let importer =
        AssetImporterDescriptor::new("gltf_importer.gltf", "gltf_importer", AssetKind::Model, 1)
            .with_source_extensions(["gltf", "glb"])
            .with_required_capabilities(["runtime.asset.importer.gltf"]);

    let builder = ImporterRuntimeManifestBuilder::new(
        "gltf_importer.runtime",
        "zircon_plugin_gltf_importer_runtime",
        "gltf_importer.dist",
        "zircon_plugin_gltf_importer_dist",
        "zircon_plugin_gltf_importer_runtime_entry_v3",
    )
    .with_capabilities([
        "runtime.plugin.gltf_importer",
        "runtime.asset.importer.gltf",
    ])
    .with_asset_importers([importer.clone()]);

    let runtime_module = builder.runtime_module_manifest();
    assert_eq!(runtime_module.name, "gltf_importer.runtime");
    assert_eq!(
        runtime_module.target_modes,
        [
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost
        ]
    );
    assert_eq!(
        runtime_module.capabilities,
        [
            "runtime.plugin.gltf_importer".to_string(),
            "runtime.asset.importer.gltf".to_string()
        ]
    );

    let manifest = builder.build_package_manifest(&descriptor);
    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));
    assert!(manifest
        .modules
        .iter()
        .any(|module| module.name == "gltf_importer.dist"
            && module.kind == PluginModuleKind::Native
            && module.crate_name == "zircon_plugin_gltf_importer_dist"
            && module.target_modes
                == [
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost
                ]));
    assert_eq!(manifest.asset_importers, [importer]);
    let distribution = manifest
        .distribution
        .as_ref()
        .expect("importer manifests declare native dist distribution");
    assert_eq!(distribution.forms, ["dist".to_string()]);
    assert_eq!(
        distribution.default_packaging,
        [ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, "zircon_plugin_gltf_importer_dist");
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(
        distribution.runtime_entry,
        "zircon_plugin_gltf_importer_runtime_entry_v3"
    );
}

#[test]
fn importer_runtime_manifest_builder_keeps_targets_platforms_modules_and_distribution_in_parity() {
    let descriptor = zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        "gltf_importer",
        "glTF Importer",
        zircon_runtime::builtin::RuntimePluginId::GltfImporter,
        "zircon_plugin_gltf_importer_runtime",
    )
    .with_category("asset_importer")
    .with_target_modes(importer_runtime_supported_targets())
    .with_capability("runtime.plugin.gltf_importer")
    .with_capability("runtime.asset.importer.gltf")
    .build();
    let importer =
        AssetImporterDescriptor::new("gltf_importer.gltf", "gltf_importer", AssetKind::Model, 1)
            .with_source_extensions(["gltf", "glb"])
            .with_required_capabilities(["runtime.asset.importer.gltf"]);

    let builder = ImporterRuntimeManifestBuilder::new(
        "gltf_importer.runtime",
        "zircon_plugin_gltf_importer_runtime",
        "gltf_importer.dist",
        "zircon_plugin_gltf_importer_dist",
        "zircon_plugin_gltf_importer_runtime_entry_v3",
    )
    .with_capabilities([
        "runtime.plugin.gltf_importer",
        "runtime.asset.importer.gltf",
    ])
    .with_asset_importers([importer.clone()]);

    let expected_targets = importer_runtime_supported_targets();
    let expected_platforms = importer_runtime_supported_platforms();
    let runtime_module = builder.runtime_module_manifest();
    let dist_module = builder.dist_module_manifest();
    let manifest = builder.build_package_manifest(&descriptor);

    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.supported_platforms, expected_platforms);
    assert_eq!(runtime_module.target_modes, expected_targets);
    assert_eq!(dist_module.target_modes, expected_targets);
    assert!(manifest.modules.iter().any(|module| {
        module.name == runtime_module.name
            && module.kind == runtime_module.kind
            && module.crate_name == runtime_module.crate_name
            && module.target_modes == runtime_module.target_modes
            && module.capabilities == runtime_module.capabilities
    }));
    assert!(manifest.modules.contains(&dist_module));
    assert_eq!(manifest.asset_importers, [importer]);
    assert!(manifest
        .default_packaging
        .contains(&ExportPackagingStrategy::NativeDynamic));

    let distribution = manifest
        .distribution
        .as_ref()
        .expect("importer runtime manifests keep native distribution parity");
    assert_eq!(
        distribution.default_packaging,
        [ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(NATIVE_ABI_VERSION_V3));
    assert_eq!(distribution.descriptor_symbol, NATIVE_DESCRIPTOR_SYMBOL_V3);
    assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
    assert_eq!(distribution.dist_crate, dist_module.crate_name);
    assert_eq!(
        distribution.runtime_entry,
        "zircon_plugin_gltf_importer_runtime_entry_v3"
    );
}
