use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{ExportPackagingStrategy, PluginMaturity, PluginModuleKind};

use super::{
    PluginFeatureBundleBuilder, PluginManifestBuilder, PluginModuleBuilder, SDK_API_VERSION,
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
