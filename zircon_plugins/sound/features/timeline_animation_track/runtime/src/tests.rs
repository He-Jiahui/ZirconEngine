use super::*;

#[test]
fn timeline_feature_provider_manifest_matches_sound_owner_contract() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert_eq!(
        report.manifest.display_name,
        "Sound Timeline Animation Track"
    );
    assert_eq!(report.manifest.owner_plugin_id, "sound");
    assert_eq!(
        report.manifest.provider_package_id.as_deref(),
        Some(DIST_PROVIDER_PACKAGE_ID)
    );
    let distribution = report
        .manifest
        .distribution
        .as_ref()
        .expect("timeline feature declares native dist distribution");
    assert_eq!(distribution.forms, vec!["dist"]);
    assert_eq!(
        distribution.default_packaging,
        vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
    );
    assert_eq!(distribution.abi_version, Some(3));
    assert_eq!(distribution.dist_crate, DIST_CRATE_NAME);
    assert_eq!(
        distribution.descriptor_symbol,
        "zircon_native_plugin_descriptor_v3"
    );
    assert_eq!(distribution.runtime_entry, DIST_RUNTIME_ENTRY);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(
        report.manifest.default_packaging,
        vec![
            zircon_runtime::plugin::ExportPackagingStrategy::SourceTemplate,
            zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
            zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic,
        ]
    );
    assert!(report
        .manifest
        .capabilities
        .contains(&RUNTIME_CAPABILITY.to_string()));
    assert!(report.manifest.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "sound"
            && dependency.capability == "runtime.plugin.sound"
            && dependency.primary
    }));
    assert!(report.manifest.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "animation"
            && dependency.capability == "runtime.feature.animation.timeline_event_track"
            && !dependency.primary
    }));
    assert!(report.manifest.modules.iter().any(|module| {
        module.name == "sound.timeline_animation_track.runtime"
            && module.crate_name == "zircon_plugin_sound_timeline_animation_runtime"
            && module
                .capabilities
                .contains(&RUNTIME_CAPABILITY.to_string())
    }));
    assert!(report.manifest.modules.iter().any(|module| {
        module.name == "sound.timeline_animation_track.editor"
            && module.crate_name == "zircon_plugin_sound_timeline_animation_editor"
            && module.capabilities.contains(&EDITOR_CAPABILITY.to_string())
    }));
    assert!(report.manifest.modules.iter().any(|module| {
        module.name == "sound.timeline_animation_track.dist"
            && module.kind == zircon_runtime::plugin::PluginModuleKind::Native
            && module.crate_name == DIST_CRATE_NAME
            && module
                .target_modes
                .contains(&zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime)
            && module
                .target_modes
                .contains(&zircon_runtime::builtin::RuntimeTargetMode::EditorHost)
            && module
                .capabilities
                .contains(&RUNTIME_CAPABILITY.to_string())
    }));
}
