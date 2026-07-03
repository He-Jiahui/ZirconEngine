use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::plugin::{
    ExportPackagingStrategy, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginModuleManifest, PluginPackageManifest, ProjectPluginFeatureSelection,
    ProjectPluginManifest, ProjectPluginSelection, RuntimePluginCatalog, RuntimePluginDescriptor,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

#[path = "runtime_plugin_catalog_features/feature_dependency_reports.rs"]
mod feature_dependency_reports;

#[test]
fn runtime_plugin_catalog_completes_owner_feature_selections_as_disabled_by_default() {
    let catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::builder(
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
    .with_optional_feature(sound_timeline_feature_manifest())
    .build()]);
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
    let catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::builder(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_capability("runtime.plugin.sound")
    .with_optional_feature(first)
    .with_optional_feature(second)
    .build()]);
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

fn sound_registration() -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("sound", "Sound")
            .with_supported_targets([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ])
            .with_capability("runtime.plugin.sound")
            .with_runtime_module(
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
        PluginPackageManifest::new("animation", "Animation")
            .with_supported_targets([
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ])
            .with_capability("runtime.plugin.animation")
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    "animation.runtime",
                    "zircon_plugin_animation_runtime",
                )
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
    .with_editor_module(
        PluginModuleManifest::editor(
            "sound.timeline_animation_track.editor",
            "zircon_plugin_sound_timeline_animation_editor",
        )
        .with_capabilities(vec![
            "editor.feature.sound.timeline_animation_track".to_string()
        ]),
    )
}
