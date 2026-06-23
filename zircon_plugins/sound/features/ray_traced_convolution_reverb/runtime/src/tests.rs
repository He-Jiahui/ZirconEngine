use super::*;

#[test]
fn ray_traced_feature_provider_manifest_matches_sound_owner_contract() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, FEATURE_ID);
    assert_eq!(
        report.manifest.display_name,
        "Ray Traced Convolution Reverb"
    );
    assert_eq!(report.manifest.owner_plugin_id, "sound");
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(
        report.manifest.default_packaging,
        vec![
            zircon_runtime::plugin::ExportPackagingStrategy::SourceTemplate,
            zircon_runtime::plugin::ExportPackagingStrategy::LibraryEmbed,
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
        dependency.plugin_id == "physics"
            && dependency.capability == "runtime.plugin.physics"
            && !dependency.primary
    }));
    assert!(report.manifest.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "physics"
            && dependency.capability == "runtime.capability.physics.raycast"
            && !dependency.primary
    }));
    assert!(report.manifest.modules.iter().any(|module| {
        module.name == "sound.ray_traced_convolution_reverb.runtime"
            && module.crate_name == "zircon_plugin_sound_ray_traced_convolution_runtime"
            && module
                .capabilities
                .contains(&RUNTIME_CAPABILITY.to_string())
    }));
    assert!(report.manifest.modules.iter().any(|module| {
        module.name == "sound.ray_traced_convolution_reverb.editor"
            && module.crate_name == "zircon_plugin_sound_ray_traced_convolution_editor"
            && module.capabilities.contains(&EDITOR_CAPABILITY.to_string())
    }));
}
