use std::sync::{Arc, Barrier};

use crate::core::framework::project::{
    ExportPackagingStrategy, ProjectPluginFeatureSelection, ProjectPluginManifest,
    ProjectPluginSelection,
};
use crate::plugin::{
    CompiledProjectPluginPlan, PluginCatalogGeneration, PluginFeatureBundleManifest,
    PluginFeatureDependency, PluginModuleManifest, PluginPackageManifest, RuntimePluginCatalog,
    RuntimePluginCatalogProjectPlanMetrics, RuntimePluginDescriptor,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

#[path = "runtime_plugin_catalog_features/compiled_selection.rs"]
mod compiled_selection;
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
    let completed = catalog.complete_project_manifest(
        &ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Sound,
                true,
                false,
            )],
        },
        RuntimeTargetMode::ClientRuntime,
    );

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
    let completed = catalog.complete_project_manifest(
        &ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Sound,
                true,
                false,
            )],
        },
        RuntimeTargetMode::ClientRuntime,
    );

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
    let feature = sound_timeline_feature_manifest()
        .with_default_packaging([ExportPackagingStrategy::NativeDynamic]);
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration(), animation_timeline_registration()],
        [
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
                feature,
                Some("sound_timeline_animation_track".to_string()),
            ),
        ],
    );
    let completed = catalog.complete_project_manifest(
        &ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Sound,
                true,
                false,
            )],
        },
        RuntimeTargetMode::ClientRuntime,
    );
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
    let provider = completed
        .selections
        .iter()
        .find(|selection| selection.id == "sound_timeline_animation_track")
        .expect("external feature provider package selection");
    assert!(!provider.enabled);
    assert_eq!(
        provider.packaging,
        ExportPackagingStrategy::NativeDynamic,
        "the provider package selection must preserve the feature declaration packaging"
    );
    assert_eq!(
        provider.runtime_crate.as_deref(),
        Some("zircon_plugin_sound_timeline_animation_runtime")
    );
    assert_eq!(
        provider.editor_crate.as_deref(),
        Some("zircon_plugin_sound_timeline_animation_editor")
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

#[test]
fn runtime_plugin_catalog_reuses_one_frozen_project_plan_per_generation() {
    let mut catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    };

    let completed = catalog.complete_project_manifest(&manifest, RuntimeTargetMode::ClientRuntime);
    let report = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);
    let extensions =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert_eq!(completed.selections.len(), 1);
    assert!(report.blocked_features.is_empty());
    assert!(extensions.is_success());
    assert_eq!(catalog.project_plan_metrics().project_plan_builds, 1);

    assert!(catalog
        .register_reports_batch(
            [animation_timeline_registration()],
            std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
        )
        .is_published());
    let _ = catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    let metrics = catalog.project_plan_cache_metrics();
    assert_eq!(metrics.project_plan_builds, 2);
    assert_eq!(metrics.cache_hits, 2);
    assert_eq!(metrics.cache_misses, 2);
    assert_eq!(metrics.cache_evictions, 0);
    assert_eq!(metrics.cached_plan_count, 1);
}

#[test]
fn runtime_plugin_catalog_project_plan_summary_remains_exhaustively_constructible() {
    let metrics = RuntimePluginCatalogProjectPlanMetrics {
        catalog_generation: PluginCatalogGeneration::INITIAL,
        project_plan_builds: 11,
    };

    assert_eq!(metrics.catalog_generation, PluginCatalogGeneration::INITIAL);
    assert_eq!(metrics.catalog_generation.get(), 1);
    assert_eq!(metrics.project_plan_builds, 11);
}

#[test]
fn runtime_plugin_catalog_exposes_one_compiled_project_plan_artifact() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    };

    let plan: Arc<CompiledProjectPluginPlan> =
        catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);
    let repeated = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);
    let completed = catalog.complete_project_manifest(&manifest, RuntimeTargetMode::ClientRuntime);
    let features = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);
    let extensions =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(Arc::ptr_eq(&plan, &repeated));
    assert_eq!(
        plan.catalog_generation(),
        catalog.project_plan_metrics().catalog_generation
    );
    assert_eq!(plan.target_mode(), RuntimeTargetMode::ClientRuntime);
    assert!(std::ptr::eq(plan.completed_manifest(), completed.as_ref()));
    assert!(std::ptr::eq(
        plan.feature_dependency_report(),
        features.as_ref()
    ));
    assert!(std::ptr::eq(plan.runtime_extensions(), extensions.as_ref()));
    assert_eq!(catalog.project_plan_metrics().project_plan_builds, 1);
}

#[test]
fn compiled_project_plan_receipts_partition_target_and_catalog_generation() {
    let mut catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    };

    let client_plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);
    let editor_plan = catalog.compiled_project_plan(&manifest, RuntimeTargetMode::EditorHost);
    let previous_generation = client_plan.catalog_generation();

    assert!(!Arc::ptr_eq(&client_plan, &editor_plan));
    assert_eq!(client_plan.target_mode(), RuntimeTargetMode::ClientRuntime);
    assert_eq!(editor_plan.target_mode(), RuntimeTargetMode::EditorHost);
    assert_eq!(editor_plan.catalog_generation(), previous_generation);

    assert!(catalog
        .register_reports_batch(
            [animation_timeline_registration()],
            std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
        )
        .is_published());
    let current_client_plan =
        catalog.compiled_project_plan(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(!Arc::ptr_eq(&client_plan, &current_client_plan));
    assert_eq!(client_plan.catalog_generation(), previous_generation);
    assert!(current_client_plan.catalog_generation() > previous_generation);
    assert_eq!(
        current_client_plan.target_mode(),
        RuntimeTargetMode::ClientRuntime
    );
}

#[test]
fn runtime_plugin_catalog_reuses_the_same_frozen_extension_snapshot() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    };

    let first = catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);
    let second =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(catalog.project_plan_metrics().project_plan_builds, 1);
}

#[test]
fn runtime_plugin_catalog_initializes_one_plan_for_concurrent_same_key_misses() {
    const READER_COUNT: usize = 8;

    let catalog = Arc::new(RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    ));
    let manifest = Arc::new(ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    });
    let barrier = Arc::new(Barrier::new(READER_COUNT));

    let snapshots = std::thread::scope(|scope| {
        let readers = (0..READER_COUNT)
            .map(|_| {
                let catalog = Arc::clone(&catalog);
                let manifest = Arc::clone(&manifest);
                let barrier = Arc::clone(&barrier);
                scope.spawn(move || {
                    barrier.wait();
                    catalog
                        .runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime)
                })
            })
            .collect::<Vec<_>>();
        readers
            .into_iter()
            .map(|reader| reader.join().expect("project-plan reader should join"))
            .collect::<Vec<_>>()
    });

    assert!(snapshots
        .windows(2)
        .all(|pair| Arc::ptr_eq(&pair[0], &pair[1])));
    assert_eq!(catalog.project_plan_metrics().project_plan_builds, 1);
    assert_eq!(catalog.cached_project_plan_count(), 1);
}

#[test]
fn runtime_plugin_catalog_reuses_completed_report_and_extension_snapshots() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    };

    let first_completed =
        catalog.complete_project_manifest(&manifest, RuntimeTargetMode::EditorHost);
    let second_completed =
        catalog.complete_project_manifest(&manifest, RuntimeTargetMode::EditorHost);
    let first_report = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::EditorHost);
    let second_report = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::EditorHost);
    let first_extensions =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::EditorHost);
    let second_extensions =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::EditorHost);

    assert_eq!(first_completed.selections.len(), 1);
    assert!(Arc::ptr_eq(&first_completed, &second_completed));
    assert!(Arc::ptr_eq(&first_report, &second_report));
    assert!(Arc::ptr_eq(&first_extensions, &second_extensions));
    assert!(first_extensions.is_success());
    assert_eq!(catalog.project_plan_metrics().project_plan_builds, 1);
    assert_eq!(catalog.cached_project_plan_count(), 1);
    let project_source =
        include_str!("../../plugin/runtime_plugin/runtime_plugin_catalog/project.rs");
    let dependency_source =
        include_str!("../../plugin/runtime_plugin/runtime_plugin_catalog/feature_dependencies.rs");
    assert!(!project_source.contains("serde_json::to_string(manifest)"));
    assert!(!dependency_source.contains(".feature_report().clone()"));
}

#[test]
fn runtime_plugin_catalog_reuses_recent_manifests_for_the_same_target() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let first_manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    };
    let second_manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false),
            feature_provider_selection("external", false),
        ],
    };

    let first =
        catalog.runtime_extensions_for_project(&first_manifest, RuntimeTargetMode::ClientRuntime);
    let _ =
        catalog.runtime_extensions_for_project(&second_manifest, RuntimeTargetMode::ClientRuntime);
    let repeated =
        catalog.runtime_extensions_for_project(&first_manifest, RuntimeTargetMode::ClientRuntime);

    assert!(Arc::ptr_eq(&first, &repeated));
    let metrics = catalog.project_plan_cache_metrics();
    assert_eq!(metrics.project_plan_builds, 2);
    assert_eq!(metrics.cache_hits, 1);
    assert_eq!(metrics.cache_misses, 2);
    assert_eq!(metrics.cache_evictions, 0);
    assert_eq!(metrics.cached_plan_count, 2);
    assert!(metrics.total_build_elapsed_ns >= metrics.max_build_elapsed_ns);
    assert_eq!(catalog.cached_project_plan_count(), 2);
}

#[test]
fn runtime_plugin_catalog_bounds_frozen_plans_per_target() {
    const EXPECTED_RECENT_CLIENT_PLANS: usize = 4;

    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let base_manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    };
    let _ = catalog.runtime_extensions_for_project(&base_manifest, RuntimeTargetMode::EditorHost);
    let client_manifests = (0..8)
        .map(|index| ProjectPluginManifest {
            selections: vec![
                ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false),
                feature_provider_selection(&format!("external_{index}"), false),
            ],
        })
        .collect::<Vec<_>>();
    let mut latest = None;

    for manifest in &client_manifests {
        latest = Some(
            catalog.runtime_extensions_for_project(manifest, RuntimeTargetMode::ClientRuntime),
        );
        assert!(catalog.cached_project_plan_count() <= 1 + EXPECTED_RECENT_CLIENT_PLANS);
    }

    let builds_before_hit = catalog.project_plan_metrics().project_plan_builds;
    let repeated = catalog.runtime_extensions_for_project(
        client_manifests.last().expect("latest client manifest"),
        RuntimeTargetMode::ClientRuntime,
    );

    assert!(Arc::ptr_eq(
        latest.as_ref().expect("latest client snapshot"),
        &repeated
    ));
    assert_eq!(
        catalog.cached_project_plan_count(),
        1 + EXPECTED_RECENT_CLIENT_PLANS
    );
    assert_eq!(
        catalog.project_plan_metrics().project_plan_builds,
        builds_before_hit
    );
    let metrics = catalog.project_plan_cache_metrics();
    assert_eq!(metrics.cache_hits, 1);
    assert_eq!(metrics.cache_misses, 9);
    assert_eq!(metrics.cache_evictions, 4);
    assert_eq!(metrics.cached_plan_count, 5);
}

#[test]
fn runtime_plugin_catalog_evicts_the_least_recent_completed_plan() {
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifests = (0..5)
        .map(|index| ProjectPluginManifest {
            selections: vec![
                ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false),
                feature_provider_selection(&format!("external_{index}"), false),
            ],
        })
        .collect::<Vec<_>>();

    let first_snapshots = manifests[..4]
        .iter()
        .map(|manifest| {
            catalog.runtime_extensions_for_project(manifest, RuntimeTargetMode::ClientRuntime)
        })
        .collect::<Vec<_>>();
    let touched_first =
        catalog.runtime_extensions_for_project(&manifests[0], RuntimeTargetMode::ClientRuntime);
    let _ = catalog.runtime_extensions_for_project(
        manifests.last().expect("fifth manifest"),
        RuntimeTargetMode::ClientRuntime,
    );
    let repeated_first =
        catalog.runtime_extensions_for_project(&manifests[0], RuntimeTargetMode::ClientRuntime);
    let rebuilt_second =
        catalog.runtime_extensions_for_project(&manifests[1], RuntimeTargetMode::ClientRuntime);

    assert!(Arc::ptr_eq(&first_snapshots[0], &touched_first));
    assert!(Arc::ptr_eq(&first_snapshots[0], &repeated_first));
    assert!(!Arc::ptr_eq(&first_snapshots[1], &rebuilt_second));
    let metrics = catalog.project_plan_cache_metrics();
    assert_eq!(metrics.project_plan_builds, 6);
    assert_eq!(metrics.cache_hits, 2);
    assert_eq!(metrics.cache_misses, 6);
    assert_eq!(metrics.cache_evictions, 2);
    assert_eq!(metrics.cached_plan_count, 4);
    assert_eq!(catalog.cached_project_plan_count(), 4);
}

#[test]
fn runtime_plugin_catalog_keeps_in_flight_extension_snapshots_alive_across_generation_publish() {
    let mut catalog = RuntimePluginCatalog::from_registration_reports(
        [sound_registration()],
        std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
    );
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )],
    };
    let previous =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(catalog
        .register_reports_batch(
            [animation_timeline_registration()],
            std::iter::empty::<RuntimePluginFeatureRegistrationReport>(),
        )
        .is_published());
    let current =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(previous.is_success());
    assert!(current.is_success());
    assert!(!Arc::ptr_eq(&previous, &current));
    assert_eq!(catalog.project_plan_metrics().project_plan_builds, 2);
}

#[test]
fn runtime_plugin_catalog_frozen_plan_cache_is_send_and_sync() {
    fn assert_send_and_sync<T: Send + Sync>() {}

    assert_send_and_sync::<RuntimePluginCatalog>();
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
