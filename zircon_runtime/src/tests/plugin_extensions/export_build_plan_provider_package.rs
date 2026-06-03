use crate::asset::{AssetUri, ProjectManifest};
use crate::{
    plugin::ExportBuildPlan, plugin::ExportPackagingStrategy, plugin::ExportProfile,
    plugin::ExportTargetPlatform, plugin::ProjectPluginFeatureSelection,
    plugin::ProjectPluginManifest, plugin::ProjectPluginSelection, RuntimePluginId,
    RuntimeTargetMode,
};

#[test]
fn export_plan_reports_malformed_feature_provider_package_id_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Malformed Feature Provider Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true)
                        .required(true)
                        .with_provider_package_id("1sound__")
                        .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("feature sound.timeline_animation_track provider_package_id `1sound__`")
        && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("feature sound.timeline_animation_track provider_package_id `1sound__`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("feature sound.timeline_animation_track provider_package_id `1sound__`")
        && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("feature sound.timeline_animation_track provider_package_id `1sound__`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(!plugin_source.contains("provider_package_id: Some(\"1sound__\""));
}

#[test]
fn export_plan_reports_malformed_project_feature_ids_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Malformed Project Feature Id Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound..timeline_animation_track")
                        .enabled(true)
                        .required(true)
                        .with_provider_package_id("animation")
                        .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let cargo_manifest = generated_file(&plan, "Cargo.toml");
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin feature id `sound..timeline_animation_track`")
        && diagnostic.contains("must not contain empty namespace segments")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin feature id `sound..timeline_animation_track`")
        && diagnostic.contains("must not contain empty namespace segments")));
    assert!(!plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_sound_timeline_animation_runtime".to_string()));
    assert!(!cargo_manifest.contains("zircon_plugin_sound_timeline_animation_runtime ="));
    assert!(!plugin_source.contains("id: \"sound..timeline_animation_track\""));
    assert!(!plugin_source
        .contains("zircon_plugin_sound_timeline_animation_runtime::plugin_feature_registration()"));
}

#[test]
fn export_plan_reports_project_feature_ids_outside_owner_namespace_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Foreign Project Feature Id Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("animation.timeline_animation_track")
                .enabled(true)
                .required(true)
                .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
        )],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin feature id `animation.timeline_animation_track`")
        && diagnostic.contains("must be prefixed by project plugin `sound`")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin feature id `animation.timeline_animation_track`")
        && diagnostic.contains("must be prefixed by project plugin `sound`")));
    assert!(!plugin_source.contains("id: \"animation.timeline_animation_track\""));
}

#[test]
fn export_plan_reports_duplicate_project_plugin_ids_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Duplicate Project Plugin Id Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, true),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin selection id `sound` is declared more than once")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin selection id `sound` is declared more than once")));
    assert_eq!(plugin_source.matches("id: \"sound\"").count(), 1);
}

#[test]
fn export_plan_reports_duplicate_project_feature_ids_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Duplicate Project Feature Id Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .enabled(true)
                .required(false)
                .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
        )
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .enabled(true)
                .required(true)
                .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
        )],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin feature id `sound.timeline_animation_track`")
        && diagnostic.contains("is declared more than once under project plugin `sound`")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin feature id `sound.timeline_animation_track`")
        && diagnostic.contains("is declared more than once under project plugin `sound`")));
    assert_eq!(
        plugin_source
            .matches("id: \"sound.timeline_animation_track\"")
            .count(),
        1
    );
}

#[test]
fn export_plan_reports_duplicate_project_target_modes_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Duplicate Project Target Modes Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, true)
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::ClientRuntime,
                ])
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true)
                        .required(true)
                        .with_target_modes([
                            RuntimeTargetMode::ClientRuntime,
                            RuntimeTargetMode::ClientRuntime,
                        ])
                        .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin sound target_modes")
        && diagnostic.contains("ClientRuntime")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin sound target_modes")
        && diagnostic.contains("ClientRuntime")));
    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin feature sound.timeline_animation_track target_modes")
        && diagnostic.contains("ClientRuntime")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin feature sound.timeline_animation_track target_modes")
        && diagnostic.contains("ClientRuntime")));
    assert!(!plugin_source
        .contains("RuntimeTargetMode::ClientRuntime, RuntimeTargetMode::ClientRuntime"));
}

#[test]
fn export_plan_reports_malformed_project_plugin_ids_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Malformed Project Plugin Id Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection {
            id: "1weather__".to_string(),
            enabled: true,
            required: true,
            target_modes: vec![RuntimeTargetMode::ClientRuntime],
            packaging: ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: Some("zircon_plugin_weather_runtime".to_string()),
            editor_crate: None,
            features: Vec::new(),
        }],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin selection id `1weather__`")
        && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin selection id `1weather__`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin selection id `1weather__`")
        && diagnostic.contains("start with a lowercase ASCII letter")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin selection id `1weather__`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(!plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_weather_runtime".to_string()));
    assert!(!plugin_source.contains("id: \"1weather__\""));
}

#[test]
fn export_plan_reports_malformed_project_plugin_runtime_crate_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Malformed Project Plugin Runtime Crate Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection {
            id: "weather".to_string(),
            enabled: true,
            required: true,
            target_modes: vec![RuntimeTargetMode::ClientRuntime],
            packaging: ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: Some("zircon_plugin_weather__runtime".to_string()),
            editor_crate: None,
            features: Vec::new(),
        }],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let cargo_manifest = generated_file(&plan, "Cargo.toml");
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin weather runtime_crate `zircon_plugin_weather__runtime`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin weather runtime_crate `zircon_plugin_weather__runtime`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(!plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_weather__runtime".to_string()));
    assert!(!cargo_manifest.contains("zircon_plugin_weather__runtime ="));
    assert!(!plugin_source.contains("runtime_crate: Some(\"zircon_plugin_weather__runtime\""));
    assert!(!plugin_source.contains("zircon_plugin_weather__runtime::plugin_registration()"));
}

#[test]
fn export_plan_reports_malformed_project_feature_runtime_crate_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Malformed Project Feature Runtime Crate Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true)
                        .required(true)
                        .with_runtime_crate("zircon_plugin_sound__timeline_runtime"),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let cargo_manifest = generated_file(&plan, "Cargo.toml");
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains(
            "project plugin feature sound.timeline_animation_track runtime_crate `zircon_plugin_sound__timeline_runtime`",
        ) && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains(
            "project plugin feature sound.timeline_animation_track runtime_crate `zircon_plugin_sound__timeline_runtime`",
        ) && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(!plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_sound__timeline_runtime".to_string()));
    assert!(!cargo_manifest.contains("zircon_plugin_sound__timeline_runtime ="));
    assert!(
        !plugin_source.contains("runtime_crate: Some(\"zircon_plugin_sound__timeline_runtime\"")
    );
    assert!(!plugin_source
        .contains("zircon_plugin_sound__timeline_runtime::plugin_feature_registration()"));
}

#[test]
fn export_plan_reports_malformed_project_plugin_editor_crate_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Malformed Project Plugin Editor Crate Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection {
            id: "weather".to_string(),
            enabled: true,
            required: true,
            target_modes: vec![RuntimeTargetMode::ClientRuntime],
            packaging: ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: Some("zircon_plugin_weather_runtime".to_string()),
            editor_crate: Some("zircon_plugin_weather__editor".to_string()),
            features: Vec::new(),
        }],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin weather editor_crate `zircon_plugin_weather__editor`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("project plugin weather editor_crate `zircon_plugin_weather__editor`")
        && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(!plugin_source.contains("editor_crate: Some(\"zircon_plugin_weather__editor\""));
}

#[test]
fn export_plan_reports_malformed_project_feature_editor_crate_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Malformed Project Feature Editor Crate Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true)
                        .required(true)
                        .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime")
                        .with_editor_crate("zircon_plugin_sound__timeline_editor"),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains(
            "project plugin feature sound.timeline_animation_track editor_crate `zircon_plugin_sound__timeline_editor`",
        ) && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains(
            "project plugin feature sound.timeline_animation_track editor_crate `zircon_plugin_sound__timeline_editor`",
        ) && diagnostic.contains("not end with an underscore or contain repeated underscores")));
    assert!(!plugin_source.contains("editor_crate: Some(\"zircon_plugin_sound__timeline_editor\""));
}

fn generated_file<'a>(plan: &'a ExportBuildPlan, path: &str) -> &'a str {
    plan.generated_files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
        .unwrap_or_else(|| panic!("missing generated file {path}"))
}
