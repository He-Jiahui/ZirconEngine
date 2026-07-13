use crate::core::framework::project::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};
use crate::plugin::{PluginModuleKind, RuntimePluginCatalog, RuntimePluginDescriptor};
use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

#[test]
fn builtin_net_content_download_dependency_report_blocks_without_http_feature() {
    let catalog = RuntimePluginCatalog::builtin();
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Net, true, false).with_feature(
                ProjectPluginFeatureSelection::new("net.content_download").enabled(true),
            ),
        ],
    };

    let blocked = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);
    let content_download_block = blocked
        .blocked_features
        .iter()
        .find(|block| block.feature_id == "net.content_download")
        .expect("content download should be blocked without net.http");

    assert!(content_download_block
        .missing_capabilities
        .contains(&"runtime.feature.net.http".to_string()));

    let completed = catalog.complete_project_manifest(&manifest);
    let net = completed
        .selections
        .iter()
        .find(|selection| selection.id == "net")
        .expect("net selection should be completed");
    assert!(net
        .features
        .iter()
        .any(|feature| feature.id == "net.content_download" && feature.enabled));
    assert!(net
        .features
        .iter()
        .any(|feature| feature.id == "net.http" && !feature.enabled));
}

#[test]
fn builtin_rendering_optional_features_declare_editor_capabilities() {
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id() == "rendering")
        .expect("rendering catalog entry");

    for suffix in [
        "post_process",
        "ssao",
        "contact_shadow",
        "decals",
        "reflection_probes",
        "baked_lighting",
        "ray_tracing_policy",
        "shader_graph",
        "vfx_graph",
    ] {
        let feature_id = format!("rendering.{suffix}");
        let editor_module_name = format!("{feature_id}.editor");
        let editor_capability = format!("editor.feature.rendering.{suffix}");
        let feature = descriptor
            .optional_features()
            .iter()
            .find(|feature| feature.id == feature_id)
            .expect("rendering optional feature should be present in the built-in catalog");
        let editor_module = feature
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .expect("rendering optional feature should declare an editor module");

        assert_eq!(editor_module.name, editor_module_name);
        assert!(
            editor_module.capabilities.contains(&editor_capability),
            "{feature_id} editor module should project {editor_capability}"
        );
    }
}

#[test]
fn builtin_sound_optional_features_declare_editor_capabilities() {
    let descriptor = RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .find(|descriptor| descriptor.package_id() == "sound")
        .expect("sound catalog entry");

    for (feature_id, editor_module_name, editor_capability) in [
        (
            "sound.timeline_animation_track",
            "sound.timeline_animation_track.editor",
            "editor.feature.sound.timeline_animation_track",
        ),
        (
            "sound.ray_traced_convolution_reverb",
            "sound.ray_traced_convolution_reverb.editor",
            "editor.feature.sound.ray_traced_convolution_reverb",
        ),
    ] {
        let feature = descriptor
            .optional_features()
            .iter()
            .find(|feature| feature.id == feature_id)
            .expect("sound optional feature should be present in the built-in catalog");
        let editor_module = feature
            .modules
            .iter()
            .find(|module| module.kind == PluginModuleKind::Editor)
            .expect("sound optional feature should declare an editor module");

        assert_eq!(editor_module.name, editor_module_name);
        assert!(
            editor_module
                .capabilities
                .contains(&editor_capability.to_string()),
            "{feature_id} editor module should project {editor_capability}"
        );
    }
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
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("rendering.contact_shadow").enabled(true),
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
    assert!(blocked.contains(&("rendering.contact_shadow", true)));
    assert!(blocked.contains(&("rendering.reflection_probes", true)));
    assert!(blocked.contains(&("rendering.baked_lighting", true)));
}
