use crate::core::ModuleDescriptor;
use crate::plugin::{
    ExportPackagingStrategy, PluginFeatureBundleManifest, PluginFeatureDependency,
    PluginModuleManifest, ProjectPluginFeatureSelection, ProjectPluginManifest,
    ProjectPluginSelection, RuntimeExtensionRegistry, RuntimePluginCatalog,
    RuntimePluginDescriptor, RuntimePluginFeature, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};
use crate::{RuntimePluginId, RuntimeTargetMode};

#[test]
fn runtime_plugin_catalog_merges_available_feature_extensions_after_base_plugins() {
    let feature = SoundTimelineFeaturePlugin;
    let mut catalog = RuntimePluginCatalog::from_descriptors([
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
        .with_optional_feature(feature.manifest()),
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
    catalog.register_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.registry.modules().len(), 1);
    assert_eq!(
        report.registry.modules()[0].name,
        "SoundTimelineAnimationFeatureModule"
    );
}

#[test]
fn runtime_plugin_catalog_reports_duplicate_feature_runtime_registrations() {
    let feature = SoundTimelineFeaturePlugin;
    let mut catalog = RuntimePluginCatalog::from_descriptors([
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
        .with_optional_feature(feature.manifest()),
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
    catalog.register_feature(&feature);
    catalog.register_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(!report.is_success());
    assert!(report.has_fatal_diagnostics());
    assert!(report
        .fatal_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains(
            "duplicate optional feature id sound.timeline_animation_track registered at runtime"
        )));
}

#[test]
fn runtime_plugin_catalog_reports_conflicting_feature_defaults_between_package_and_runtime() {
    let feature = SoundTimelineFeaturePlugin;
    let declared_feature = feature
        .manifest()
        .with_default_packaging([ExportPackagingStrategy::NativeDynamic])
        .enabled_by_default(true);
    let mut catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_capability("runtime.plugin.sound")
    .with_optional_feature(declared_feature)]);
    catalog.register_feature(&feature);

    let report = catalog.feature_dependency_report(
        &ProjectPluginManifest {
            selections: Vec::new(),
        },
        RuntimeTargetMode::ClientRuntime,
    );

    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(
        "optional feature id sound.timeline_animation_track has conflicting package manifest and runtime registration"
    )));
}

#[test]
fn runtime_extension_catalog_treats_blocked_optional_features_as_warnings() {
    let mut catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound")]);
    let feature = SoundTimelineFeaturePlugin;
    catalog.register_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track").enabled(true),
        )],
    };

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(report.is_success(), "{:?}", report.fatal_diagnostics);
    assert!(report.fatal_diagnostics.is_empty());
    assert!(report.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("optional feature sound.timeline_animation_track is blocked")));
    assert!(report.registry.modules().is_empty());
}

#[test]
fn runtime_extension_catalog_treats_blocked_required_features_as_fatal() {
    let mut catalog = RuntimePluginCatalog::from_descriptors([RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound")]);
    let feature = SoundTimelineFeaturePlugin;
    catalog.register_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .enabled(true)
                .required(true),
        )],
    };

    let report =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(!report.is_success());
    assert!(report.has_fatal_diagnostics());
    assert!(report.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("required feature sound.timeline_animation_track is blocked")));
    assert!(report.registry.modules().is_empty());
}

#[test]
fn runtime_module_load_reports_blocked_optional_features_as_warnings() {
    let sound = RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound");
    let feature = SoundTimelineFeaturePlugin;
    let sound_registration = RuntimePluginRegistrationReport::from_plugin(&sound);
    let feature_registration = RuntimePluginFeatureRegistrationReport::from_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track").enabled(true),
        )],
    };

    let report =
        crate::builtin::runtime_modules_for_target_with_plugin_and_feature_registration_reports(
            RuntimeTargetMode::ClientRuntime,
            Some(&manifest),
            [&sound_registration],
            [&feature_registration],
        );

    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert!(report
        .warnings
        .iter()
        .any(|warning| warning
            .contains("optional feature sound.timeline_animation_track is blocked")));
}

#[test]
fn runtime_module_load_reports_blocked_required_features_as_errors() {
    let sound = RuntimePluginDescriptor::new(
        "sound",
        "Sound",
        RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ])
    .with_capability("runtime.plugin.sound");
    let feature = SoundTimelineFeaturePlugin;
    let sound_registration = RuntimePluginRegistrationReport::from_plugin(&sound);
    let feature_registration = RuntimePluginFeatureRegistrationReport::from_feature(&feature);
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .enabled(true)
                .required(true),
        )],
    };

    let report =
        crate::builtin::runtime_modules_for_target_with_plugin_and_feature_registration_reports(
            RuntimeTargetMode::ClientRuntime,
            Some(&manifest),
            [&sound_registration],
            [&feature_registration],
        );

    assert!(report.warnings.is_empty(), "{:?}", report.warnings);
    assert!(report.errors.iter().any(|error| {
        error.contains("required feature sound.timeline_animation_track is blocked")
    }));
}

#[derive(Debug)]
struct SoundTimelineFeaturePlugin;

impl RuntimePluginFeature for SoundTimelineFeaturePlugin {
    fn manifest(&self) -> PluginFeatureBundleManifest {
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
            .with_capabilities(["runtime.feature.sound.timeline_animation_track"]),
        )
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), crate::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(ModuleDescriptor::new(
            "SoundTimelineAnimationFeatureModule",
            "Sound timeline animation track feature",
        ))
    }
}
