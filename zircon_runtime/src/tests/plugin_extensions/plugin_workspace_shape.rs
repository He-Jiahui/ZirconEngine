use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::plugin::{
    CapabilityStatus, ExportPackagingStrategy, PluginModuleKind, PluginPackageManifest,
    RuntimePluginCatalog, RuntimePluginDescriptor,
};

#[test]
fn builtin_runtime_catalog_entries_have_matching_plugin_manifests_and_workspace_members() {
    let plugins_root = plugins_workspace_root();
    let workspace_members = plugin_workspace_members(&plugins_root);

    for descriptor in RuntimePluginDescriptor::builtin_catalog() {
        let package_id = descriptor.package_id();
        let manifest = read_plugin_manifest(&plugins_root, package_id);
        assert_eq!(manifest.id, package_id);
        assert!(
            workspace_members.contains(&format!("{package_id}/runtime")),
            "runtime catalog entry `{}` is missing its zircon_plugins workspace runtime member",
            package_id
        );
        assert!(
            manifest.modules.iter().any(|module| {
                module.kind == PluginModuleKind::Runtime && module.crate_name == descriptor.crate_name()
            }),
            "runtime catalog entry `{}` is missing matching runtime module crate `{}` in plugin.toml",
            package_id,
            descriptor.crate_name()
        );
    }
}

#[test]
fn builtin_runtime_catalog_optional_features_match_static_plugin_manifests() {
    let plugins_root = plugins_workspace_root();

    for descriptor in RuntimePluginDescriptor::builtin_catalog() {
        let package_id = descriptor.package_id();
        let manifest = read_plugin_manifest(&plugins_root, package_id);

        assert_eq!(
            manifest.optional_features.as_slice(),
            descriptor.optional_features(),
            "runtime catalog entry `{}` optional_features must match zircon_plugins/{}/plugin.toml",
            package_id,
            package_id
        );
    }
}

#[test]
fn advanced_render_plugin_manifests_declare_profile_capabilities() {
    let plugins_root = plugins_workspace_root();
    let catalog = RuntimePluginDescriptor::builtin_catalog();

    for (plugin_id, capability, runtime_id) in [
        (
            "virtual_geometry",
            "runtime.render.advanced.virtual_geometry",
            RuntimePluginId::VirtualGeometry,
        ),
        (
            "hybrid_gi",
            "runtime.render.advanced.hybrid_gi",
            RuntimePluginId::HybridGi,
        ),
    ] {
        let manifest = read_plugin_manifest(&plugins_root, plugin_id);
        let manifest_source = fs::read_to_string(plugins_root.join(plugin_id).join("plugin.toml"))
            .expect("advanced render plugin manifest source");
        let runtime_module = manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Runtime)
            .expect("advanced render plugin should declare a runtime module");
        let descriptor = catalog
            .iter()
            .find(|descriptor| descriptor.runtime_id() == runtime_id)
            .expect("advanced render plugin should be in the runtime catalog");
        let projected_manifest = descriptor.package_manifest();
        let expected_targets = vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ];
        let expected_capabilities = vec![
            format!("runtime.plugin.{plugin_id}"),
            capability.to_string(),
        ];

        assert!(
            manifest_source.contains(r#"sdk_api_version = "0.1.0""#),
            "advanced render plugin `{plugin_id}` should explicitly declare SDK API version"
        );
        assert_eq!(manifest.sdk_api_version, "0.1.0");
        assert_eq!(manifest.category, "rendering");
        assert_eq!(
            manifest.maturity,
            crate::plugin::PluginMaturity::Experimental
        );
        assert_eq!(manifest.supported_targets, expected_targets);
        assert_eq!(manifest.capabilities, expected_capabilities);
        assert!(manifest.capability_statuses.iter().any(|status| {
            status.capability == capability && status.status == CapabilityStatus::Partial
        }));
        assert_eq!(runtime_module.target_modes, manifest.supported_targets);
        assert_eq!(runtime_module.capabilities, manifest.capabilities);
        assert_eq!(descriptor.category(), manifest.category);
        assert_eq!(descriptor.maturity(), manifest.maturity);
        assert_eq!(descriptor.target_modes(), manifest.supported_targets);
        assert_eq!(descriptor.capabilities(), manifest.capabilities);
        assert!(descriptor.capability_statuses().iter().any(|status| {
            status.capability == capability && status.status == CapabilityStatus::Partial
        }));
        assert_eq!(projected_manifest.category, manifest.category);
        assert_eq!(projected_manifest.maturity, manifest.maturity);
        assert_eq!(
            projected_manifest.supported_targets,
            manifest.supported_targets
        );
        assert_eq!(projected_manifest.capabilities, manifest.capabilities);
    }
}

#[test]
fn solari_plugin_manifest_matches_catalog_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "solari");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("solari plugin should declare a runtime module");
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id() == RuntimePluginId::Solari)
        .expect("solari plugin should be in the runtime catalog");
    let projected_manifest = descriptor.package_manifest();
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec![
        "runtime.plugin.solari".to_string(),
        "runtime.render.experimental.solari".to_string(),
    ];

    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "rendering");
    assert_eq!(
        manifest.maturity,
        crate::plugin::PluginMaturity::Experimental
    );
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.solari" && status.status == CapabilityStatus::Partial
    }));
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.render.experimental.solari"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some("Solari realtime raytraced lighting pass executor is not implemented yet")
    }));

    assert_eq!(descriptor.category(), manifest.category);
    assert_eq!(descriptor.maturity(), manifest.maturity);
    assert_eq!(descriptor.target_modes(), manifest.supported_targets);
    assert_eq!(descriptor.capabilities(), manifest.capabilities);
    assert!(descriptor.capability_statuses().iter().any(|status| {
        status.capability == "runtime.plugin.solari" && status.status == CapabilityStatus::Partial
    }));
    assert!(descriptor.capability_statuses().iter().any(|status| {
        status.capability == "runtime.render.experimental.solari"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some("Solari realtime raytraced lighting pass executor is not implemented yet")
    }));
    assert_eq!(projected_manifest.category, manifest.category);
    assert_eq!(projected_manifest.maturity, manifest.maturity);
    assert_eq!(
        projected_manifest.supported_targets,
        manifest.supported_targets
    );
    assert_eq!(projected_manifest.capabilities, manifest.capabilities);
}

#[test]
fn native_dynamic_fixture_manifests_declare_package_level_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "native_dynamic_fixture");
    let native_source_path = plugins_root
        .join("native_dynamic_fixture")
        .join("native")
        .join("src")
        .join("lib.rs");
    let native_source = fs::read_to_string(&native_source_path).unwrap_or_else(|error| {
        panic!("missing native fixture source {native_source_path:?}: {error}")
    });
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec![
        "runtime.plugin.native_dynamic_fixture".to_string(),
        "runtime.asset.importer.native_dynamic_fixture.data_json".to_string(),
        "editor.extension.native_dynamic_fixture".to_string(),
    ];
    let manifest_source_path = plugins_root
        .join("native_dynamic_fixture")
        .join("plugin.toml");
    let manifest_source = fs::read_to_string(&manifest_source_path).unwrap_or_else(|error| {
        panic!("missing native fixture manifest {manifest_source_path:?}: {error}")
    });
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("native fixture should declare a runtime module");
    let editor_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Editor)
        .expect("native fixture should declare an editor module");

    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "sdk");
    assert_eq!(
        manifest.maturity,
        crate::plugin::PluginMaturity::Experimental
    );
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(
        manifest.default_packaging,
        vec![ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(
        runtime_module.capabilities,
        vec![
            "runtime.plugin.native_dynamic_fixture".to_string(),
            "runtime.asset.importer.native_dynamic_fixture.data_json".to_string(),
        ]
    );
    assert_eq!(
        editor_module.target_modes,
        vec![RuntimeTargetMode::EditorHost]
    );
    assert_eq!(
        editor_module.capabilities,
        vec!["editor.extension.native_dynamic_fixture".to_string()]
    );

    for expected_line in [
        r#"sdk_api_version = "0.1.0""#,
        r#"category = "sdk""#,
        r#"maturity = "experimental""#,
        r#"supported_targets = ["client_runtime", "server_runtime", "editor_host"]"#,
        r#""runtime.plugin.native_dynamic_fixture""#,
        r#""runtime.asset.importer.native_dynamic_fixture.data_json""#,
        r#""editor.extension.native_dynamic_fixture""#,
        r#"default_packaging = ["native_dynamic"]"#,
    ] {
        assert!(
            manifest_source.contains(expected_line),
            "native fixture manifest should contain `{expected_line}`"
        );
    }
    assert!(
        native_source.contains(r#"include_str!("../../plugin.toml")"#),
        "native fixture should embed its root plugin.toml manifest"
    );
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
        .map(RuntimePluginDescriptor::package_id)
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
        let manifest_source = fs::read_to_string(plugins_root.join(id).join("plugin.toml"))
            .expect("authoring plugin manifest source");
        let descriptor = runtime_catalog
            .iter()
            .find(|descriptor| descriptor.package_id() == id)
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
        let expected_targets = vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ];
        let expected_capabilities = vec![runtime_capability.to_string()];

        assert_eq!(RuntimePluginId::parse_key(id), Some(runtime_id));
        assert!(
            manifest_source.contains(r#"sdk_api_version = "0.1.0""#),
            "runtime-backed authoring plugin `{id}` should explicitly declare SDK API version"
        );
        assert_eq!(manifest.sdk_api_version, "0.1.0");
        assert_eq!(manifest.category, "authoring");
        assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Beta);
        assert_eq!(manifest.supported_targets, expected_targets);
        assert_eq!(manifest.capabilities, expected_capabilities);
        assert!(manifest.capability_statuses.iter().any(|status| {
            status.capability == runtime_capability && status.status == CapabilityStatus::Partial
        }));
        assert_eq!(descriptor.category(), "authoring");
        assert_eq!(descriptor.maturity(), crate::plugin::PluginMaturity::Beta);
        assert_eq!(descriptor.target_modes(), manifest.supported_targets);
        assert_eq!(descriptor.capabilities(), manifest.capabilities);
        assert!(descriptor.capability_statuses().iter().any(|status| {
            status.capability == runtime_capability && status.status == CapabilityStatus::Partial
        }));
        assert_eq!(descriptor.runtime_id(), runtime_id);
        assert_eq!(descriptor.crate_name(), runtime_crate);
        assert!(descriptor
            .capabilities()
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

        assert_eq!(
            RuntimePluginId::parse_key(id),
            Some(RuntimePluginId::new(id))
        );
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
