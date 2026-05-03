use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::AssetImporterDescriptor;
use crate::plugin::{
    ComponentTypeDescriptor, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginModuleKind, PluginModuleManifest, PluginPackageKind, PluginPackageManifest,
    ProjectPluginFeatureSelection, RuntimePluginDescriptor, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport, UiComponentDescriptor,
};
use crate::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::ProjectPluginManifest,
    plugin::ProjectPluginSelection, plugin::RuntimePluginCatalog, RuntimePluginId,
    RuntimeTargetMode,
};

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
            "asset://weather/editor/cloud_layer_inspector.ui.toml",
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
        "asset://weather/editor/cloud_layer_inspector.ui.toml"
    );

    let encoded = toml::to_string(&manifest).expect("manifest toml");
    let decoded: PluginPackageManifest = toml::from_str(&encoded).expect("manifest roundtrip");
    assert_eq!(decoded, manifest);
}

#[test]
fn builtin_rendering_catalog_declares_owner_features_and_defaults() {
    assert_eq!(
        RuntimePluginId::parse_key("rendering"),
        Some(RuntimePluginId::Rendering)
    );

    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id == "rendering")
        .expect("rendering catalog entry");
    let manifest = descriptor.package_manifest();

    assert_eq!(descriptor.category, "rendering");
    assert_eq!(manifest.category, "rendering");
    assert_eq!(
        descriptor.target_modes,
        vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ]
    );
    assert_eq!(
        descriptor
            .optional_features
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "rendering.post_process",
            "rendering.ssao",
            "rendering.decals",
            "rendering.reflection_probes",
            "rendering.baked_lighting",
            "rendering.ray_tracing_policy",
            "rendering.shader_graph",
            "rendering.vfx_graph",
        ]
    );
    assert_eq!(
        descriptor
            .optional_features
            .iter()
            .filter(|feature| feature.enabled_by_default)
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "rendering.post_process",
            "rendering.ssao",
            "rendering.reflection_probes",
            "rendering.baked_lighting",
        ]
    );
    let vfx_graph = descriptor
        .optional_features
        .iter()
        .find(|feature| feature.id == "rendering.vfx_graph")
        .expect("vfx graph feature");
    assert!(vfx_graph.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "particles" && dependency.capability == "runtime.plugin.particles"
    }));
    assert!(vfx_graph.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "rendering"
            && dependency.capability == "runtime.feature.rendering.shader_graph"
    }));
}

#[test]
fn rendering_plugin_toml_roundtrips_owner_features_and_modules() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "rendering");
    let encoded = toml::to_string(&manifest).expect("rendering plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("rendering plugin manifest roundtrip");

    assert_eq!(decoded, manifest);
    assert_eq!(manifest.id, "rendering");
    assert_eq!(manifest.category, "rendering");
    assert_eq!(
        manifest.default_packaging,
        vec![
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed
        ]
    );
    assert!(manifest.modules.iter().any(|module| {
        module.kind == PluginModuleKind::Runtime
            && module.crate_name == "zircon_plugin_rendering_runtime"
            && module.target_modes
                == vec![
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ]
            && module
                .capabilities
                .contains(&"runtime.plugin.rendering".to_string())
    }));
    assert!(manifest.modules.iter().any(|module| {
        module.kind == PluginModuleKind::Editor
            && module.crate_name == "zircon_plugin_rendering_editor"
            && module.target_modes == vec![RuntimeTargetMode::EditorHost]
            && module
                .capabilities
                .contains(&"editor.extension.rendering_authoring".to_string())
    }));

    let expected_features = vec![
        "rendering.post_process",
        "rendering.ssao",
        "rendering.decals",
        "rendering.reflection_probes",
        "rendering.baked_lighting",
        "rendering.ray_tracing_policy",
        "rendering.shader_graph",
        "rendering.vfx_graph",
    ];
    let default_enabled = BTreeSet::from([
        "rendering.post_process",
        "rendering.ssao",
        "rendering.reflection_probes",
        "rendering.baked_lighting",
    ]);

    assert_eq!(
        manifest
            .optional_features
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        expected_features
    );
    for feature in &manifest.optional_features {
        let suffix = feature
            .id
            .strip_prefix("rendering.")
            .expect("rendering feature id prefix");
        let runtime_capability = format!("runtime.feature.rendering.{suffix}");
        let runtime_crate = format!("zircon_plugin_rendering_{suffix}_runtime");
        let editor_crate = format!("zircon_plugin_rendering_{suffix}_editor");

        assert_eq!(feature.owner_plugin_id, "rendering");
        assert_eq!(
            feature.enabled_by_default,
            default_enabled.contains(feature.id.as_str())
        );
        assert!(feature.capabilities.contains(&runtime_capability));
        assert!(feature.dependencies.iter().any(|dependency| {
            dependency.plugin_id == "rendering"
                && dependency.capability == "runtime.plugin.rendering"
                && dependency.primary
        }));
        assert!(feature.modules.iter().any(|module| {
            module.kind == PluginModuleKind::Runtime
                && module.crate_name == runtime_crate
                && module.target_modes
                    == vec![
                        RuntimeTargetMode::ClientRuntime,
                        RuntimeTargetMode::EditorHost,
                    ]
                && module.capabilities.contains(&runtime_capability)
        }));
        assert!(feature.modules.iter().any(|module| {
            module.kind == PluginModuleKind::Editor
                && module.crate_name == editor_crate
                && module.target_modes == vec![RuntimeTargetMode::EditorHost]
        }));
    }

    let vfx_graph = manifest
        .optional_features
        .iter()
        .find(|feature| feature.id == "rendering.vfx_graph")
        .expect("vfx graph feature");
    assert!(vfx_graph.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "particles"
            && dependency.capability == "runtime.plugin.particles"
            && !dependency.primary
    }));
    assert!(vfx_graph.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "rendering"
            && dependency.capability == "runtime.feature.rendering.shader_graph"
            && !dependency.primary
    }));
}

#[test]
fn rendering_vfx_graph_dependency_report_blocks_without_implicit_feature_enablement() {
    let catalog = RuntimePluginCatalog::builtin();
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Rendering,
            true,
            false,
        )
        .with_feature(ProjectPluginFeatureSelection::new("rendering.vfx_graph").enabled(true))],
    };

    let blocked = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);
    let vfx_block = blocked
        .blocked_features
        .iter()
        .find(|block| block.feature_id == "rendering.vfx_graph")
        .expect("vfx graph should be blocked without particles and shader graph");

    assert!(vfx_block.missing_plugins.contains(&"particles".to_string()));
    assert!(vfx_block
        .missing_capabilities
        .contains(&"runtime.plugin.particles".to_string()));
    assert!(vfx_block
        .missing_capabilities
        .contains(&"runtime.feature.rendering.shader_graph".to_string()));

    let completed = catalog.complete_project_manifest(&manifest);
    let rendering = completed
        .selections
        .iter()
        .find(|selection| selection.id == "rendering")
        .expect("rendering selection should be completed");
    assert!(rendering
        .features
        .iter()
        .any(|feature| feature.id == "rendering.vfx_graph" && feature.enabled));
    assert!(rendering
        .features
        .iter()
        .any(|feature| feature.id == "rendering.shader_graph" && !feature.enabled));
    assert!(completed
        .selections
        .iter()
        .any(|selection| selection.id == "particles" && !selection.enabled));
}

#[test]
fn rendering_vfx_graph_becomes_available_after_explicit_dependencies_are_enabled() {
    let catalog = RuntimePluginCatalog::builtin();
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Rendering, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("rendering.shader_graph").enabled(true),
                )
                .with_feature(
                    ProjectPluginFeatureSelection::new("rendering.vfx_graph").enabled(true),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Particles, true, false),
        ],
    };

    let report = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(report
        .available_features
        .contains(&"rendering.shader_graph".to_string()));
    assert!(report
        .available_features
        .contains(&"rendering.vfx_graph".to_string()));
    assert!(!report
        .blocked_features
        .iter()
        .any(|block| block.feature_id == "rendering.vfx_graph"));
}

#[test]
fn rendering_features_are_blocked_on_server_target() {
    let catalog = RuntimePluginCatalog::builtin();
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Rendering,
            true,
            false,
        )],
    };

    let report = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ServerRuntime);
    let blocked = report
        .blocked_features
        .iter()
        .filter(|block| block.owner_plugin_id == "rendering")
        .map(|block| (block.feature_id.as_str(), block.target_unsupported))
        .collect::<Vec<_>>();

    assert!(blocked.contains(&("rendering.post_process", true)));
    assert!(blocked.contains(&("rendering.ssao", true)));
    assert!(blocked.contains(&("rendering.reflection_probes", true)));
    assert!(blocked.contains(&("rendering.baked_lighting", true)));
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
fn project_plugin_manifest_preserves_nested_feature_selections() {
    let selection = ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime")
                .with_editor_crate("zircon_plugin_sound_timeline_animation_editor")
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ]),
        );
    let manifest = ProjectPluginManifest {
        selections: vec![selection],
    };

    let encoded = toml::to_string(&manifest).expect("project manifest toml");
    let decoded: ProjectPluginManifest =
        toml::from_str(&encoded).expect("project manifest roundtrip");

    let sound = decoded
        .selections
        .iter()
        .find(|selection| selection.id == "sound")
        .expect("sound selection");
    assert_eq!(sound.features.len(), 1);
    assert_eq!(sound.features[0].id, "sound.timeline_animation_track");
    assert!(sound.features[0].enabled);
    assert_eq!(
        sound.features[0].runtime_crate.as_deref(),
        Some("zircon_plugin_sound_timeline_animation_runtime")
    );
}

#[test]
fn project_plugin_manifest_preserves_external_feature_provider_selection() {
    let selection = ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .with_provider_package_id("sound_timeline_animation_track")
                .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
        );
    let manifest = ProjectPluginManifest {
        selections: vec![selection],
    };

    let encoded = toml::to_string(&manifest).expect("project manifest toml");
    let decoded: ProjectPluginManifest =
        toml::from_str(&encoded).expect("project manifest roundtrip");
    let sound = decoded
        .selections
        .iter()
        .find(|selection| selection.id == "sound")
        .expect("sound selection");

    assert_eq!(
        sound.features[0].provider_package_id.as_deref(),
        Some("sound_timeline_animation_track")
    );
    assert_eq!(
        sound.features[0].runtime_crate_path("sound"),
        "sound_timeline_animation_track/runtime"
    );
}

#[test]
fn runtime_plugin_descriptor_projects_public_metadata_to_package_manifest() {
    let descriptor = RuntimePluginDescriptor::new(
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
    .with_optional_feature(sound_timeline_feature_manifest());

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
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("runtime module");
    assert_eq!(
        runtime_module.capabilities,
        vec![
            "runtime.plugin.weather".to_string(),
            "runtime.capability.weather.forecast".to_string()
        ]
    );
}

#[test]
fn runtime_plugin_descriptor_projects_default_packaging_to_project_selection() {
    let descriptor = RuntimePluginDescriptor::new(
        "native_weather",
        "Native Weather",
        RuntimePluginId::Particles,
        "zircon_plugin_native_weather_runtime",
    )
    .with_default_packaging([ExportPackagingStrategy::NativeDynamic]);

    let selection = descriptor.project_selection();

    assert_eq!(selection.packaging, ExportPackagingStrategy::NativeDynamic);
}

#[test]
fn runtime_plugin_catalog_completes_owner_feature_selections_as_disabled_by_default() {
    let catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound")
    .with_optional_feature(sound_timeline_feature_manifest())]);
    let completed = catalog.complete_project_manifest(&ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    });

    let sound = completed
        .selections
        .iter()
        .find(|selection| selection.id == "sound")
        .expect("sound selection");
    let feature = sound
        .features
        .iter()
        .find(|feature| feature.id == "sound.timeline_animation_track")
        .expect("feature selection");

    assert!(!feature.enabled);
    assert_eq!(
        feature.runtime_crate.as_deref(),
        Some("zircon_plugin_sound_timeline_animation_runtime")
    );
    assert_eq!(
        feature.editor_crate.as_deref(),
        Some("zircon_plugin_sound_timeline_animation_editor")
    );
}

#[test]
fn runtime_plugin_catalog_completes_owner_feature_selections_in_declaration_order() {
    let first = PluginFeatureBundleManifest::new("sound.first_feature", "First Feature", "sound");
    let second =
        PluginFeatureBundleManifest::new("sound.second_feature", "Second Feature", "sound");
    let catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_optional_feature(first)
    .with_optional_feature(second)]);
    let completed = catalog.complete_project_manifest(&ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    });

    let sound = completed
        .selections
        .iter()
        .find(|selection| selection.id == "sound")
        .expect("sound selection");
    let feature_ids = sound
        .features
        .iter()
        .map(|feature| feature.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        feature_ids,
        vec!["sound.first_feature", "sound.second_feature"]
    );
}

#[test]
fn runtime_plugin_catalog_reports_optional_feature_dependency_status() {
    let catalog = RuntimePluginCatalog::from_descriptors([
        RuntimePluginDescriptor::new(
            "sound",
            "Sound",
            RuntimePluginId::Sound,
            "zircon_plugin_sound_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.plugin.sound")
        .with_optional_feature(sound_timeline_feature_manifest()),
        RuntimePluginDescriptor::new(
            "animation",
            "Animation",
            RuntimePluginId::Animation,
            "zircon_plugin_animation_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.plugin.animation")
        .with_capability("runtime.feature.animation.timeline_event_track"),
    ]);
    let mut manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track").enabled(true),
        )],
    };

    let blocked = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);
    assert!(blocked.available_features.is_empty());
    assert_eq!(blocked.blocked_features.len(), 1);
    assert!(blocked.blocked_features[0]
        .missing_plugins
        .contains(&"animation".to_string()));
    assert!(blocked.blocked_features[0]
        .missing_capabilities
        .contains(&"runtime.feature.animation.timeline_event_track".to_string()));

    manifest.set_enabled(ProjectPluginSelection::runtime_plugin(
        RuntimePluginId::Animation,
        true,
        false,
    ));
    let available = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);

    assert_eq!(
        available.available_features,
        vec!["sound.timeline_animation_track".to_string()]
    );
    assert!(available.blocked_features.is_empty());
}

#[test]
fn runtime_plugin_catalog_projects_external_feature_packages_under_owner() {
    let feature = sound_timeline_feature_manifest();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration(), animation_timeline_registration()],
        [
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
                feature,
                Some("sound_timeline_animation_track".to_string()),
            ),
        ],
    );
    let completed = catalog.complete_project_manifest(&ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    });
    let sound = completed
        .selections
        .iter()
        .find(|selection| selection.id == "sound")
        .expect("sound selection");
    let projected = sound
        .features
        .iter()
        .find(|feature| feature.id == "sound.timeline_animation_track")
        .expect("external feature projection");

    assert!(!projected.enabled);
    assert_eq!(
        projected.provider_package_id.as_deref(),
        Some("sound_timeline_animation_track")
    );
    assert_eq!(
        projected.runtime_crate.as_deref(),
        Some("zircon_plugin_sound_timeline_animation_runtime")
    );
}

#[test]
fn runtime_plugin_catalog_gates_external_feature_packages_on_provider_selection() {
    let feature = sound_timeline_feature_manifest();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration(), animation_timeline_registration()],
        [
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
                feature,
                Some("sound_timeline_animation_track".to_string()),
            ),
        ],
    );
    let mut manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };

    let blocked = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);
    assert!(blocked.available_features.is_empty());
    assert_eq!(blocked.blocked_features.len(), 1);
    assert!(blocked.blocked_features[0]
        .missing_plugins
        .contains(&"sound_timeline_animation_track".to_string()));

    manifest.selections.push(feature_provider_selection(
        "sound_timeline_animation_track",
        true,
    ));
    let available = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);

    assert_eq!(
        available.available_features,
        vec!["sound.timeline_animation_track".to_string()]
    );
    assert!(available.blocked_features.is_empty());
}

#[test]
fn runtime_plugin_catalog_merges_runtime_extensions_from_external_feature_provider() {
    let feature = sound_timeline_feature_manifest();
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration(), animation_timeline_registration()],
        [
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
                feature,
                Some("sound_timeline_animation_track".to_string()),
            ),
        ],
    );
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
            feature_provider_selection("sound_timeline_animation_track", true),
        ],
    };

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(report.is_success(), "{:?}", report.fatal_diagnostics);
    assert!(report
        .registry
        .modules()
        .iter()
        .any(|module| module.name == "sound.timeline_animation_track.runtime"));
}

#[test]
fn runtime_plugin_catalog_rejects_secondary_primary_feature_dependency() {
    let invalid_feature = PluginFeatureBundleManifest::new(
        "sound.invalid_extra_primary",
        "Invalid Extra Primary",
        "sound",
    )
    .with_dependency(PluginFeatureDependency::primary(
        "sound",
        "runtime.plugin.sound",
    ))
    .with_dependency(PluginFeatureDependency::primary(
        "animation",
        "runtime.feature.animation.timeline_event_track",
    ))
    .with_capability("runtime.feature.sound.invalid_extra_primary");
    let catalog = RuntimePluginCatalog::from_descriptors([
        RuntimePluginDescriptor::new(
            "sound",
            "Sound",
            RuntimePluginId::Sound,
            "zircon_plugin_sound_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.plugin.sound")
        .with_optional_feature(invalid_feature),
        RuntimePluginDescriptor::new(
            "animation",
            "Animation",
            RuntimePluginId::Animation,
            "zircon_plugin_animation_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capability("runtime.feature.animation.timeline_event_track"),
    ]);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.invalid_extra_primary").enabled(true),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };

    let blocked = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(blocked.available_features.is_empty());
    assert_eq!(blocked.blocked_features.len(), 1);
    assert!(blocked.blocked_features[0].invalid_owner_dependency);
    assert!(blocked.blocked_features[0]
        .to_diagnostic()
        .contains("not the only primary dependency"));
}

#[test]
fn runtime_plugin_catalog_reports_target_mismatch_for_optional_feature() {
    let server_only_feature =
        PluginFeatureBundleManifest::new("sound.server_only", "Server Only", "sound")
            .with_dependency(PluginFeatureDependency::primary(
                "sound",
                "runtime.plugin.sound",
            ))
            .with_capability("runtime.feature.sound.server_only")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "sound.server_only.runtime",
                    "zircon_plugin_sound_server_only_runtime",
                )
                .with_target_modes([RuntimeTargetMode::ServerRuntime])
                .with_capabilities(["runtime.feature.sound.server_only"]),
            );
    let catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound")
    .with_optional_feature(server_only_feature)]);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(ProjectPluginFeatureSelection::new("sound.server_only").enabled(true))],
    };

    let blocked = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(blocked.available_features.is_empty());
    assert_eq!(blocked.blocked_features.len(), 1);
    assert_eq!(blocked.blocked_features[0].feature_id, "sound.server_only");
    assert!(blocked.blocked_features[0].target_unsupported);
    assert!(blocked.blocked_features[0]
        .to_diagnostic()
        .contains("target mode is not supported"));
}

#[test]
fn runtime_plugin_catalog_reports_feature_capability_cycles() {
    let feature_a =
        PluginFeatureBundleManifest::new("rendering.feature_a", "Feature A", "rendering")
            .with_dependency(PluginFeatureDependency::primary(
                "rendering",
                "runtime.plugin.rendering",
            ))
            .with_dependency(PluginFeatureDependency::required(
                "rendering",
                "runtime.feature.rendering.feature_b",
            ))
            .with_capability("runtime.feature.rendering.feature_a")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "rendering.feature_a.runtime",
                    "zircon_plugin_rendering_feature_a_runtime",
                )
                .with_target_modes([RuntimeTargetMode::EditorHost])
                .with_capabilities(["runtime.feature.rendering.feature_a"]),
            );
    let feature_b =
        PluginFeatureBundleManifest::new("rendering.feature_b", "Feature B", "rendering")
            .with_dependency(PluginFeatureDependency::primary(
                "rendering",
                "runtime.plugin.rendering",
            ))
            .with_dependency(PluginFeatureDependency::required(
                "rendering",
                "runtime.feature.rendering.feature_a",
            ))
            .with_capability("runtime.feature.rendering.feature_b")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "rendering.feature_b.runtime",
                    "zircon_plugin_rendering_feature_b_runtime",
                )
                .with_target_modes([RuntimeTargetMode::EditorHost])
                .with_capabilities(["runtime.feature.rendering.feature_b"]),
            );
    let catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "rendering",
        "Rendering",
        RuntimePluginId::Rendering,
        "zircon_plugin_rendering_runtime",
    )
    .with_target_modes([RuntimeTargetMode::EditorHost])
    .with_capability("runtime.plugin.rendering")
    .with_optional_feature(feature_a)
    .with_optional_feature(feature_b)]);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Rendering,
            true,
            false,
        )
        .with_feature(ProjectPluginFeatureSelection::new("rendering.feature_a").enabled(true))
        .with_feature(ProjectPluginFeatureSelection::new("rendering.feature_b").enabled(true))],
    };

    let blocked = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::EditorHost);

    assert!(blocked.available_features.is_empty());
    assert_eq!(blocked.blocked_features.len(), 2);
    assert!(blocked.blocked_features.iter().all(|feature| feature.cycle));
    assert!(blocked.blocked_features.iter().all(|feature| feature
        .to_diagnostic()
        .contains("feature capability dependencies form a cycle")));
}

#[test]
fn runtime_plugin_catalog_reports_disabled_feature_provider_as_missing_capability() {
    let feature_a =
        PluginFeatureBundleManifest::new("rendering.feature_a", "Feature A", "rendering")
            .with_dependency(PluginFeatureDependency::primary(
                "rendering",
                "runtime.plugin.rendering",
            ))
            .with_dependency(PluginFeatureDependency::required(
                "rendering",
                "runtime.feature.rendering.feature_b",
            ))
            .with_capability("runtime.feature.rendering.feature_a")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "rendering.feature_a.runtime",
                    "zircon_plugin_rendering_feature_a_runtime",
                )
                .with_target_modes([RuntimeTargetMode::EditorHost])
                .with_capabilities(["runtime.feature.rendering.feature_a"]),
            );
    let feature_b =
        PluginFeatureBundleManifest::new("rendering.feature_b", "Feature B", "rendering")
            .with_dependency(PluginFeatureDependency::primary(
                "rendering",
                "runtime.plugin.rendering",
            ))
            .with_capability("runtime.feature.rendering.feature_b")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "rendering.feature_b.runtime",
                    "zircon_plugin_rendering_feature_b_runtime",
                )
                .with_target_modes([RuntimeTargetMode::EditorHost])
                .with_capabilities(["runtime.feature.rendering.feature_b"]),
            );
    let catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "rendering",
        "Rendering",
        RuntimePluginId::Rendering,
        "zircon_plugin_rendering_runtime",
    )
    .with_target_modes([RuntimeTargetMode::EditorHost])
    .with_capability("runtime.plugin.rendering")
    .with_optional_feature(feature_a)
    .with_optional_feature(feature_b)]);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Rendering,
            true,
            false,
        )
        .with_feature(ProjectPluginFeatureSelection::new("rendering.feature_a").enabled(true))],
    };

    let blocked = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::EditorHost);

    assert!(blocked.available_features.is_empty());
    assert_eq!(blocked.blocked_features.len(), 1);
    assert_eq!(
        blocked.blocked_features[0].feature_id,
        "rendering.feature_a"
    );
    assert!(!blocked.blocked_features[0].cycle);
    assert!(blocked.blocked_features[0]
        .missing_capabilities
        .contains(&"runtime.feature.rendering.feature_b".to_string()));
    assert!(!blocked.blocked_features[0]
        .to_diagnostic()
        .contains("feature capability dependencies form a cycle"));
}

#[test]
fn runtime_plugin_catalog_reports_self_feature_capability_cycle() {
    let feature =
        PluginFeatureBundleManifest::new("rendering.self_cycle", "Self Cycle", "rendering")
            .with_dependency(PluginFeatureDependency::primary(
                "rendering",
                "runtime.plugin.rendering",
            ))
            .with_dependency(PluginFeatureDependency::required(
                "rendering",
                "runtime.feature.rendering.self_cycle",
            ))
            .with_capability("runtime.feature.rendering.self_cycle")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "rendering.self_cycle.runtime",
                    "zircon_plugin_rendering_self_cycle_runtime",
                )
                .with_target_modes([RuntimeTargetMode::EditorHost])
                .with_capabilities(["runtime.feature.rendering.self_cycle"]),
            );
    let catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "rendering",
        "Rendering",
        RuntimePluginId::Rendering,
        "zircon_plugin_rendering_runtime",
    )
    .with_target_modes([RuntimeTargetMode::EditorHost])
    .with_capability("runtime.plugin.rendering")
    .with_optional_feature(feature)]);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Rendering,
            true,
            false,
        )
        .with_feature(ProjectPluginFeatureSelection::new("rendering.self_cycle").enabled(true))],
    };

    let blocked = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::EditorHost);

    assert!(blocked.available_features.is_empty());
    assert_eq!(blocked.blocked_features.len(), 1);
    assert_eq!(
        blocked.blocked_features[0].feature_id,
        "rendering.self_cycle"
    );
    assert!(blocked.blocked_features[0].cycle);
    assert!(blocked.blocked_features[0]
        .to_diagnostic()
        .contains("feature capability dependencies form a cycle"));
}

#[test]
fn builtin_runtime_catalog_entries_have_matching_plugin_manifests_and_workspace_members() {
    let plugins_root = plugins_workspace_root();
    let workspace_members = plugin_workspace_members(&plugins_root);

    for descriptor in RuntimePluginDescriptor::builtin_catalog() {
        let manifest = read_plugin_manifest(&plugins_root, &descriptor.package_id);
        assert_eq!(manifest.id, descriptor.package_id);
        assert!(
            workspace_members.contains(&format!("{}/runtime", descriptor.package_id)),
            "runtime catalog entry `{}` is missing its zircon_plugins workspace runtime member",
            descriptor.package_id
        );
        assert!(
            manifest.modules.iter().any(|module| {
                module.kind == PluginModuleKind::Runtime && module.crate_name == descriptor.crate_name
            }),
            "runtime catalog entry `{}` is missing matching runtime module crate `{}` in plugin.toml",
            descriptor.package_id,
            descriptor.crate_name
        );
    }
}

#[test]
fn runtime_backed_workspace_plugin_manifests_are_present_in_builtin_catalog() {
    let plugins_root = plugins_workspace_root();
    let workspace_members = plugin_workspace_members(&plugins_root);
    let catalog_ids = RuntimePluginCatalog::builtin()
        .package_manifests()
        .into_iter()
        .map(|manifest| manifest.id)
        .collect::<BTreeSet<_>>();

    for manifest_path in plugin_manifest_paths(&plugins_root) {
        let manifest_source = fs::read_to_string(&manifest_path).expect("plugin manifest source");
        let manifest: PluginPackageManifest =
            toml::from_str(&manifest_source).expect("plugin manifest should parse");
        let has_workspace_runtime_member =
            workspace_members.contains(&format!("{}/runtime", manifest.id));
        let declares_runtime_module = manifest
            .modules
            .iter()
            .any(|module| module.kind == PluginModuleKind::Runtime);
        if has_workspace_runtime_member && declares_runtime_module {
            assert!(
                catalog_ids.contains(&manifest.id),
                "runtime-backed plugin `{}` is missing from RuntimePluginDescriptor::builtin_catalog()",
                manifest.id
            );
        }
    }
}

#[test]
fn authoring_plugin_manifests_match_catalog_and_workspace_shape() {
    let plugins_root = plugins_workspace_root();
    let workspace_members = plugin_workspace_members(&plugins_root);
    let runtime_catalog = RuntimePluginDescriptor::builtin_catalog();
    let runtime_catalog_ids = runtime_catalog
        .iter()
        .map(|descriptor| descriptor.package_id.as_str())
        .collect::<BTreeSet<_>>();

    for (id, runtime_id, runtime_crate, runtime_capability, editor_crate, editor_capability) in [
        (
            "terrain",
            RuntimePluginId::Terrain,
            "zircon_plugin_terrain_runtime",
            "runtime.plugin.terrain",
            "zircon_plugin_terrain_editor",
            "editor.extension.terrain_authoring",
        ),
        (
            "tilemap_2d",
            RuntimePluginId::Tilemap2d,
            "zircon_plugin_tilemap_2d_runtime",
            "runtime.plugin.tilemap_2d",
            "zircon_plugin_tilemap_2d_editor",
            "editor.extension.tilemap_2d_authoring",
        ),
        (
            "prefab_tools",
            RuntimePluginId::PrefabTools,
            "zircon_plugin_prefab_tools_runtime",
            "runtime.plugin.prefab_tools",
            "zircon_plugin_prefab_tools_editor",
            "editor.extension.prefab_tools_authoring",
        ),
    ] {
        let manifest = read_plugin_manifest(&plugins_root, id);
        let descriptor = runtime_catalog
            .iter()
            .find(|descriptor| descriptor.package_id == id)
            .expect("runtime-backed authoring plugin should be in runtime catalog");
        let runtime_module = manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Runtime)
            .expect("runtime-backed authoring plugin should declare runtime module");
        let editor_module = manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .expect("runtime-backed authoring plugin should declare editor module");

        assert_eq!(RuntimePluginId::parse_key(id), Some(runtime_id));
        assert_eq!(manifest.category, "authoring");
        assert_eq!(descriptor.category, "authoring");
        assert_eq!(descriptor.runtime_id, runtime_id);
        assert_eq!(descriptor.crate_name, runtime_crate);
        assert!(descriptor
            .capabilities
            .contains(&runtime_capability.to_string()));
        assert!(workspace_members.contains(&format!("{id}/runtime")));
        assert!(workspace_members.contains(&format!("{id}/editor")));
        assert_eq!(runtime_module.crate_name, runtime_crate);
        assert_eq!(
            runtime_module.target_modes,
            vec![
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ]
        );
        assert!(runtime_module
            .capabilities
            .contains(&runtime_capability.to_string()));
        assert_eq!(editor_module.crate_name, editor_crate);
        assert_eq!(
            editor_module.target_modes,
            vec![RuntimeTargetMode::EditorHost]
        );
        assert!(editor_module
            .capabilities
            .contains(&editor_capability.to_string()));
    }

    for (id, editor_crate, editor_capability) in [
        (
            "material_editor",
            "zircon_plugin_material_editor_editor",
            "editor.extension.material_editor_authoring",
        ),
        (
            "timeline_sequence",
            "zircon_plugin_timeline_sequence_editor",
            "editor.extension.timeline_sequence_authoring",
        ),
        (
            "animation_graph",
            "zircon_plugin_animation_graph_editor",
            "editor.extension.animation_graph_authoring",
        ),
    ] {
        let manifest = read_plugin_manifest(&plugins_root, id);
        let editor_module = manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .expect("editor-only authoring plugin should declare editor module");

        assert_eq!(RuntimePluginId::parse_key(id), None);
        assert_eq!(manifest.category, "authoring");
        assert!(!runtime_catalog_ids.contains(id));
        assert!(manifest
            .modules
            .iter()
            .all(|module| module.kind != PluginModuleKind::Runtime));
        assert!(workspace_members.contains(&format!("{id}/editor")));
        assert!(!workspace_members.contains(&format!("{id}/runtime")));
        assert_eq!(editor_module.crate_name, editor_crate);
        assert_eq!(
            editor_module.target_modes,
            vec![RuntimeTargetMode::EditorHost]
        );
        assert!(editor_module
            .capabilities
            .contains(&editor_capability.to_string()));
    }

    let timeline = read_plugin_manifest(&plugins_root, "timeline_sequence");
    assert!(timeline.dependencies.iter().any(|dependency| {
        dependency.id == "animation"
            && dependency.required
            && dependency.capability.as_deref()
                == Some("runtime.feature.animation.timeline_event_track")
    }));
    let animation_graph = read_plugin_manifest(&plugins_root, "animation_graph");
    assert!(animation_graph.dependencies.iter().any(|dependency| {
        dependency.id == "animation"
            && dependency.required
            && dependency.capability.as_deref() == Some("runtime.plugin.animation")
    }));
}

fn plugins_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should have a repository parent")
        .join("zircon_plugins")
}

fn plugin_workspace_members(plugins_root: &Path) -> BTreeSet<String> {
    let manifest = fs::read_to_string(plugins_root.join("Cargo.toml"))
        .expect("zircon_plugins workspace manifest");
    let manifest: toml::Value = toml::from_str(&manifest).expect("workspace manifest should parse");
    manifest
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .expect("workspace members should be an array")
        .iter()
        .map(|member| {
            member
                .as_str()
                .expect("workspace member should be a string")
                .replace('\\', "/")
        })
        .collect()
}

fn plugin_manifest_paths(plugins_root: &Path) -> Vec<PathBuf> {
    fs::read_dir(plugins_root)
        .expect("zircon_plugins directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("plugin.toml"))
        .filter(|path| path.exists())
        .collect()
}

fn read_plugin_manifest(plugins_root: &Path, package_id: &str) -> PluginPackageManifest {
    let manifest_path = plugins_root.join(package_id).join("plugin.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("missing plugin manifest {manifest_path:?}: {error}"));
    toml::from_str(&manifest)
        .unwrap_or_else(|error| panic!("invalid plugin manifest {manifest_path:?}: {error}"))
}

fn sound_registration() -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("sound", "Sound").with_runtime_module(
            PluginModuleManifest::runtime("sound.runtime", "zircon_plugin_sound_runtime")
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities(["runtime.plugin.sound"]),
        ),
    )
}

fn animation_timeline_registration() -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("animation", "Animation").with_runtime_module(
            PluginModuleManifest::runtime("animation.runtime", "zircon_plugin_animation_runtime")
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ])
                .with_capabilities([
                    "runtime.plugin.animation",
                    "runtime.feature.animation.timeline_event_track",
                ]),
        ),
    )
}

fn feature_provider_selection(package_id: &str, enabled: bool) -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: package_id.to_string(),
        enabled,
        required: false,
        target_modes: vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: Some(format!("zircon_plugin_{package_id}_runtime")),
        editor_crate: None,
        features: Vec::new(),
    }
}

fn sound_timeline_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        "sound.timeline_animation_track",
        "Sound Timeline Animation Track",
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
    .with_capability("runtime.feature.sound.timeline_animation_track")
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.timeline_animation_track.runtime",
            "zircon_plugin_sound_timeline_animation_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(vec![
            "runtime.feature.sound.timeline_animation_track".to_string()
        ]),
    )
    .with_editor_module(PluginModuleManifest::editor(
        "sound.timeline_animation_track.editor",
        "zircon_plugin_sound_timeline_animation_editor",
    ))
}
