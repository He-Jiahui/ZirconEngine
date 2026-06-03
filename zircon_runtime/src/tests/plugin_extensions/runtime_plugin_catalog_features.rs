use crate::plugin::{
    ExportPackagingStrategy, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginModuleManifest, PluginPackageManifest, ProjectPluginFeatureSelection,
    ProjectPluginManifest, ProjectPluginSelection, RuntimePluginCatalog, RuntimePluginDescriptor,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
use crate::{RuntimePluginId, RuntimeTargetMode};

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
    .with_capability("runtime.plugin.sound")
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
