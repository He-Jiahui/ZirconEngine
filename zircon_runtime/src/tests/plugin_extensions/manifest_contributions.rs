use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::AssetImporterDescriptor;
use crate::plugin::{
    ComponentTypeDescriptor, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginModuleKind, PluginModuleManifest, PluginPackageManifest, ProjectPluginFeatureSelection,
    RuntimePluginDescriptor, UiComponentDescriptor,
};
use crate::{
    plugin::ExportPackagingStrategy, plugin::ExportTargetPlatform, plugin::ProjectPluginManifest, plugin::ProjectPluginSelection,
    plugin::RuntimePluginCatalog, RuntimePluginId, RuntimeTargetMode,
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
