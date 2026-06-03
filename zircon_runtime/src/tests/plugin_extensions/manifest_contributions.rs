use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::plugin::{
    CapabilityStatus, PluginFeatureBundleManifest, PluginModuleKind, PluginPackageManifest,
    RuntimePluginDescriptor,
};
use crate::{plugin::ExportPackagingStrategy, RuntimePluginId, RuntimeTargetMode};

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
    let manifest_source = fs::read_to_string(plugins_root.join("rendering").join("plugin.toml"))
        .expect("rendering plugin manifest source");
    let encoded = toml::to_string(&manifest).expect("rendering plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("rendering plugin manifest roundtrip");
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id == RuntimePluginId::Rendering)
        .expect("rendering plugin should be in the runtime catalog");
    let projected_manifest = descriptor.package_manifest();
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.rendering".to_string()];

    assert_eq!(decoded, manifest);
    assert_eq!(manifest.id, "rendering");
    assert!(
        manifest_source.contains(r#"sdk_api_version = "0.1.0""#),
        "rendering plugin should explicitly declare SDK API version"
    );
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "rendering");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Stable);
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.rendering"
            && status.status == CapabilityStatus::Complete
    }));
    assert_eq!(descriptor.category, manifest.category);
    assert_eq!(descriptor.maturity, manifest.maturity);
    assert_eq!(descriptor.target_modes, manifest.supported_targets);
    assert_eq!(descriptor.capabilities, manifest.capabilities);
    assert!(descriptor.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.rendering"
            && status.status == CapabilityStatus::Complete
    }));
    assert_eq!(projected_manifest.category, manifest.category);
    assert_eq!(projected_manifest.maturity, manifest.maturity);
    assert_eq!(
        projected_manifest.supported_targets,
        manifest.supported_targets
    );
    assert_eq!(projected_manifest.capabilities, manifest.capabilities);
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
            && module.target_modes == manifest.supported_targets
            && module.capabilities == manifest.capabilities
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
        let editor_capability = format!("editor.feature.rendering.{suffix}");
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
                && module.capabilities.contains(&editor_capability)
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
fn sound_plugin_manifest_matches_catalog_beta_partial_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "sound");
    let manifest_source = fs::read_to_string(plugins_root.join("sound").join("plugin.toml"))
        .expect("sound plugin manifest source");
    let encoded = toml::to_string(&manifest).expect("sound plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("sound plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("sound plugin should declare a runtime module");
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id == RuntimePluginId::Sound)
        .expect("sound plugin should be in the runtime catalog");
    let projected_manifest = descriptor.package_manifest();
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.sound".to_string()];

    assert_eq!(decoded, manifest);
    assert!(
        manifest_source.contains(r#"sdk_api_version = "0.1.0""#),
        "sound plugin should explicitly declare SDK API version"
    );
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Beta);
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.sound"
            && status.status == CapabilityStatus::Partial
            && status
                .bevy_references
                .contains(&"dev/bevy/crates/bevy_audio/src/lib.rs".to_string())
    }));
    assert_eq!(descriptor.category, manifest.category);
    assert_eq!(descriptor.maturity, manifest.maturity);
    assert_eq!(descriptor.target_modes, manifest.supported_targets);
    assert_eq!(descriptor.capabilities, manifest.capabilities);
    assert!(descriptor.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.sound"
            && status.status == CapabilityStatus::Partial
            && status
                .bevy_references
                .contains(&"dev/bevy/crates/bevy_audio/src/lib.rs".to_string())
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
fn animation_plugin_toml_matches_catalog_beta_partial_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "animation");
    let encoded = toml::to_string(&manifest).expect("animation plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("animation plugin manifest roundtrip");

    assert_eq!(decoded, manifest);
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Beta);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.animation"
            && status.status == CapabilityStatus::Partial
            && status
                .bevy_references
                .contains(&"dev/bevy/crates/bevy_animation/src/lib.rs".to_string())
    }));
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.feature.animation.timeline_event_track"
            && status.status == CapabilityStatus::Partial
    }));

    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id == "animation")
        .expect("animation catalog entry");
    assert_eq!(descriptor.category, "runtime");
    assert_eq!(descriptor.maturity, crate::plugin::PluginMaturity::Beta);
    assert!(descriptor.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.animation"
            && status.status == CapabilityStatus::Partial
            && status
                .bevy_references
                .contains(&"dev/bevy/crates/bevy_animation/src/lib.rs".to_string())
    }));
    assert!(descriptor.capability_statuses.iter().any(|status| {
        status.capability == "runtime.feature.animation.timeline_event_track"
            && status.status == CapabilityStatus::Partial
    }));
}

#[test]
fn navigation_plugin_toml_matches_catalog_beta_partial_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "navigation");
    let encoded = toml::to_string(&manifest).expect("navigation plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("navigation plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("navigation plugin should declare a runtime module");
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.navigation".to_string()];

    assert_eq!(decoded, manifest);
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Beta);
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.navigation"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some(
                    "Gameplay navmesh/pathfinding is optional; UI navigation parity is separate.",
                )
    }));

    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id == "navigation")
        .expect("navigation catalog entry");
    assert_eq!(descriptor.category, "runtime");
    assert_eq!(descriptor.maturity, crate::plugin::PluginMaturity::Beta);
    assert_eq!(descriptor.target_modes, manifest.supported_targets);
    assert_eq!(descriptor.capabilities, manifest.capabilities);
    assert!(descriptor.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.navigation"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some(
                    "Gameplay navmesh/pathfinding is optional; UI navigation parity is separate.",
                )
    }));
}

#[test]
fn particles_plugin_toml_matches_catalog_optional_feature_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "particles");
    let manifest_source = fs::read_to_string(plugins_root.join("particles").join("plugin.toml"))
        .expect("particles plugin manifest source");
    let encoded = toml::to_string(&manifest).expect("particles plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("particles plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("particles plugin should declare a runtime module");
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.particles".to_string()];

    assert_eq!(decoded, manifest);
    assert!(
        manifest_source.contains(r#"sdk_api_version = "0.1.0""#),
        "particles plugin should explicitly declare SDK API version"
    );
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(
        manifest.maturity,
        crate::plugin::PluginMaturity::Experimental
    );
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.particles"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some("Advanced optional VFX capability; not a Bevy default parity blocker.")
    }));
    assert_particles_optional_features(&manifest.optional_features);

    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id == "particles")
        .expect("particles catalog entry");
    assert_eq!(descriptor.category, "runtime");
    assert_eq!(
        descriptor.maturity,
        crate::plugin::PluginMaturity::Experimental
    );
    assert_eq!(descriptor.target_modes, manifest.supported_targets);
    assert_eq!(descriptor.capabilities, manifest.capabilities);
    assert!(descriptor.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.particles"
            && status.status == CapabilityStatus::Partial
            && status.note.as_deref()
                == Some("Advanced optional VFX capability; not a Bevy default parity blocker.")
    }));
    assert_particles_optional_features(&descriptor.optional_features);
}

#[test]
fn texture_plugin_manifest_matches_catalog_stable_complete_metadata() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "texture");
    let manifest_source = fs::read_to_string(plugins_root.join("texture").join("plugin.toml"))
        .expect("texture plugin manifest source");
    let encoded = toml::to_string(&manifest).expect("texture plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("texture plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("texture plugin should declare a runtime module");
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id == RuntimePluginId::Texture)
        .expect("texture plugin should be in the runtime catalog");
    let projected_manifest = descriptor.package_manifest();
    let expected_targets = vec![
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.texture".to_string()];

    assert_eq!(decoded, manifest);
    assert!(
        manifest_source.contains(r#"sdk_api_version = "0.1.0""#),
        "texture plugin should explicitly declare SDK API version"
    );
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Stable);
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.texture" && status.status == CapabilityStatus::Complete
    }));
    assert_eq!(descriptor.category, manifest.category);
    assert_eq!(descriptor.maturity, manifest.maturity);
    assert_eq!(descriptor.target_modes, manifest.supported_targets);
    assert_eq!(descriptor.capabilities, manifest.capabilities);
    assert!(descriptor.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.texture" && status.status == CapabilityStatus::Complete
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
fn runtime_experimental_plugin_toml_matches_catalog_partial_metadata() {
    let plugins_root = plugins_workspace_root();
    for (id, capabilities) in [
        (
            "physics",
            vec![
                "runtime.plugin.physics",
                "runtime.capability.physics.raycast",
            ],
        ),
        (
            "zr_vm_language",
            vec![
                "runtime.plugin.zr_vm_language",
                "runtime.script.backend.zr_vm_project",
            ],
        ),
    ] {
        let manifest = read_plugin_manifest(&plugins_root, id);
        let encoded = toml::to_string(&manifest).expect("runtime plugin manifest toml");
        let decoded: PluginPackageManifest =
            toml::from_str(&encoded).expect("runtime plugin manifest roundtrip");

        assert_eq!(decoded, manifest);
        assert_eq!(manifest.category, "runtime");
        assert_eq!(
            manifest.maturity,
            crate::plugin::PluginMaturity::Experimental
        );
        assert_runtime_partial_capability_statuses(&manifest.capability_statuses, &capabilities);

        let descriptor = RuntimePluginDescriptor::builtin_catalog()
            .into_iter()
            .find(|descriptor| descriptor.package_id == id)
            .expect("runtime plugin catalog entry");
        assert_eq!(descriptor.category, "runtime");
        assert_eq!(
            descriptor.maturity,
            crate::plugin::PluginMaturity::Experimental
        );
        assert_runtime_partial_capability_statuses(&descriptor.capability_statuses, &capabilities);
    }
}

fn assert_runtime_partial_capability_statuses(
    statuses: &[crate::plugin::CapabilityStatusManifest],
    capabilities: &[&str],
) {
    for capability in capabilities {
        assert!(
            statuses.iter().any(|status| {
                status.capability == *capability && status.status == CapabilityStatus::Partial
            }),
            "missing partial capability status for {capability}"
        );
    }
}

fn assert_particles_optional_features(features: &[PluginFeatureBundleManifest]) {
    assert_eq!(
        features
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "particles.physics",
            "particles.animation_control",
            "particles.gpu_simulation",
        ]
    );

    for (feature_id, capability, required_plugin, required_capability) in [
        (
            "particles.physics",
            "runtime.feature.particles.physics",
            "physics",
            "runtime.plugin.physics",
        ),
        (
            "particles.animation_control",
            "runtime.feature.particles.animation_control",
            "animation",
            "runtime.plugin.animation",
        ),
        (
            "particles.gpu_simulation",
            "runtime.feature.particles.gpu_simulation",
            "render_graph",
            "runtime.module.render_graph",
        ),
    ] {
        let feature = features
            .iter()
            .find(|feature| feature.id == feature_id)
            .expect("particles optional feature should be present");

        assert_eq!(feature.owner_plugin_id, "particles");
        assert!(!feature.enabled_by_default);
        assert!(feature.capabilities.contains(&capability.to_string()));
        assert!(feature.dependencies.iter().any(|dependency| {
            dependency.plugin_id == "particles"
                && dependency.capability == "runtime.plugin.particles"
                && dependency.primary
        }));
        assert!(feature.dependencies.iter().any(|dependency| {
            dependency.plugin_id == required_plugin
                && dependency.capability == required_capability
                && !dependency.primary
        }));
    }
}

#[test]
fn editor_only_plugin_tomls_declare_package_level_targets_and_capabilities() {
    let plugins_root = plugins_workspace_root();

    for (id, category, editor_crate, capabilities) in [
        (
            "material_editor",
            "authoring",
            "zircon_plugin_material_editor_editor",
            vec!["editor.extension.material_editor_authoring"],
        ),
        (
            "timeline_sequence",
            "authoring",
            "zircon_plugin_timeline_sequence_editor",
            vec!["editor.extension.timeline_sequence_authoring"],
        ),
        (
            "animation_graph",
            "authoring",
            "zircon_plugin_animation_graph_editor",
            vec!["editor.extension.animation_graph_authoring"],
        ),
        (
            "runtime_diagnostics",
            "diagnostics",
            "zircon_plugin_runtime_diagnostics_editor",
            vec!["editor.extension.runtime_diagnostics"],
        ),
        (
            "ui_asset_authoring",
            "authoring",
            "zircon_plugin_ui_asset_authoring_editor",
            vec!["editor.extension.ui_asset_authoring"],
        ),
        (
            "native_window_hosting",
            "platform",
            "zircon_plugin_native_window_hosting_editor",
            vec!["editor.extension.native_window_hosting"],
        ),
        (
            "editor_build_export_desktop",
            "platform",
            "zircon_plugin_editor_build_export_desktop_editor",
            vec![
                "editor.extension.build_export_desktop",
                "editor.extension.build_export_desktop.diagnostics",
                "editor.extension.build_export_desktop.native_dynamic_report",
            ],
        ),
        (
            "plugin_sdk_examples",
            "sdk",
            "zircon_plugin_sdk_examples_editor",
            vec![
                "editor.extension.plugin_sdk_examples",
                "editor.extension.plugin_sdk_examples.window",
                "editor.extension.plugin_sdk_examples.asset_fixture",
            ],
        ),
    ] {
        let manifest = read_plugin_manifest(&plugins_root, id);
        let editor_module = manifest
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .expect("editor-only plugin should declare an editor module");
        let capabilities = capabilities
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>();

        assert_eq!(manifest.category, category);
        assert_eq!(
            manifest.supported_targets,
            vec![RuntimeTargetMode::EditorHost],
            "editor-only plugin `{id}` should publish editor_host as package metadata"
        );
        assert_eq!(
            manifest.capabilities, capabilities,
            "editor-only plugin `{id}` should publish editor capability as package metadata"
        );
        assert_eq!(editor_module.crate_name, editor_crate);
        assert_eq!(editor_module.capabilities, manifest.capabilities);
    }
}

#[test]
fn low_overlap_editor_only_plugin_tomls_declare_explicit_experimental_maturity() {
    let plugins_root = plugins_workspace_root();

    for id in [
        "runtime_diagnostics",
        "native_window_hosting",
        "editor_build_export_desktop",
        "plugin_sdk_examples",
    ] {
        let manifest_source = fs::read_to_string(plugins_root.join(id).join("plugin.toml"))
            .expect("editor-only plugin manifest source");
        let manifest = read_plugin_manifest(&plugins_root, id);

        assert!(
            manifest_source.contains(r#"maturity = "experimental""#),
            "editor-only plugin `{id}` should explicitly declare experimental maturity"
        );
        assert_eq!(
            manifest.maturity,
            crate::plugin::PluginMaturity::Experimental
        );
    }
}

#[test]
fn editor_authoring_plugin_tomls_declare_explicit_experimental_maturity() {
    let plugins_root = plugins_workspace_root();

    for id in [
        "material_editor",
        "timeline_sequence",
        "animation_graph",
        "ui_asset_authoring",
    ] {
        let manifest_source = fs::read_to_string(plugins_root.join(id).join("plugin.toml"))
            .expect("editor-only authoring plugin manifest source");
        let manifest = read_plugin_manifest(&plugins_root, id);

        assert!(
            manifest_source.contains(r#"maturity = "experimental""#),
            "editor-only authoring plugin `{id}` should explicitly declare experimental maturity"
        );
        assert_eq!(
            manifest.maturity,
            crate::plugin::PluginMaturity::Experimental
        );
    }
}

#[test]
fn net_plugin_toml_declares_content_download_http_dependency() {
    let plugins_root = plugins_workspace_root();
    let manifest = read_plugin_manifest(&plugins_root, "net");
    let encoded = toml::to_string(&manifest).expect("net plugin manifest toml");
    let decoded: PluginPackageManifest =
        toml::from_str(&encoded).expect("net plugin manifest roundtrip");
    let runtime_module = manifest
        .modules
        .iter()
        .find(|module| module.kind == PluginModuleKind::Runtime)
        .expect("net plugin should declare a runtime module");
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.runtime_id == RuntimePluginId::Net)
        .expect("net plugin should be in the runtime catalog");
    let expected_targets = vec![
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ];
    let expected_capabilities = vec!["runtime.plugin.net".to_string()];

    assert_eq!(decoded, manifest);
    assert_eq!(manifest.sdk_api_version, "0.1.0");
    assert_eq!(manifest.category, "runtime");
    assert_eq!(manifest.maturity, crate::plugin::PluginMaturity::Beta);
    assert_eq!(manifest.supported_targets, expected_targets);
    assert_eq!(manifest.capabilities, expected_capabilities);
    assert_eq!(runtime_module.target_modes, manifest.supported_targets);
    assert_eq!(runtime_module.capabilities, manifest.capabilities);
    assert_eq!(descriptor.target_modes, manifest.supported_targets);
    assert_eq!(descriptor.capabilities, manifest.capabilities);
    assert!(manifest.capability_statuses.iter().any(|status| {
        status.capability == "runtime.plugin.net" && status.status == CapabilityStatus::Partial
    }));
    let content_download = manifest
        .optional_features
        .iter()
        .find(|feature| feature.id == "net.content_download")
        .expect("content download optional feature");

    assert!(content_download.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "net"
            && dependency.capability == "runtime.plugin.net"
            && dependency.primary
    }));
    assert!(content_download.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "net"
            && dependency.capability == "runtime.feature.net.http"
            && !dependency.primary
    }));
}

#[test]
fn builtin_net_catalog_declares_layered_optional_features() {
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id == "net")
        .expect("net catalog entry");

    assert_eq!(
        descriptor
            .optional_features
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "net.http",
            "net.websocket",
            "net.rpc",
            "net.replication",
            "net.reliable_udp",
            "net.content_download",
        ]
    );

    for (feature_id, capability, runtime_crate, target_modes) in [
        (
            "net.http",
            "runtime.feature.net.http",
            "zircon_plugin_net_http_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.websocket",
            "runtime.feature.net.websocket",
            "zircon_plugin_net_websocket_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.rpc",
            "runtime.feature.net.rpc",
            "zircon_plugin_net_rpc_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.replication",
            "runtime.feature.net.replication",
            "zircon_plugin_net_replication_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.reliable_udp",
            "runtime.feature.net.reliable_udp",
            "zircon_plugin_net_reliable_udp_runtime",
            vec![
                RuntimeTargetMode::ServerRuntime,
                RuntimeTargetMode::ClientRuntime,
            ],
        ),
        (
            "net.content_download",
            "runtime.feature.net.cdn_download",
            "zircon_plugin_net_content_download_runtime",
            vec![RuntimeTargetMode::ClientRuntime],
        ),
    ] {
        let feature = descriptor
            .optional_features
            .iter()
            .find(|feature| feature.id == feature_id)
            .expect("net optional feature should be present in the built-in catalog");

        assert!(feature.capabilities.contains(&capability.to_string()));
        assert!(feature.dependencies.iter().any(|dependency| {
            dependency.plugin_id == "net"
                && dependency.capability == "runtime.plugin.net"
                && dependency.primary
        }));
        assert!(feature.modules.iter().any(|module| {
            module.kind == PluginModuleKind::Runtime
                && module.name == format!("{feature_id}.runtime")
                && module.crate_name == runtime_crate
                && module.target_modes == target_modes
                && module.capabilities.contains(&capability.to_string())
        }));
    }

    let content_download = descriptor
        .optional_features
        .iter()
        .find(|feature| feature.id == "net.content_download")
        .expect("content download feature");
    assert!(content_download.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "net"
            && dependency.capability == "runtime.feature.net.http"
            && !dependency.primary
    }));
}

fn plugins_workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime crate should have a repository parent")
        .join("zircon_plugins")
}

fn read_plugin_manifest(plugins_root: &Path, package_id: &str) -> PluginPackageManifest {
    let manifest_path = plugins_root.join(package_id).join("plugin.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("missing plugin manifest {manifest_path:?}: {error}"));
    toml::from_str(&manifest)
        .unwrap_or_else(|error| panic!("invalid plugin manifest {manifest_path:?}: {error}"))
}
