use crate::asset::{AssetUri, ProjectManifest};
use crate::{
    plugin::ExportBuildPlan, plugin::ExportPackagingStrategy, plugin::ExportProfile,
    plugin::ExportTargetPlatform, plugin::ProjectPluginFeatureSelection,
    plugin::ProjectPluginManifest, plugin::ProjectPluginSelection, RuntimePluginId,
    RuntimeTargetMode,
};

#[test]
fn source_template_links_active_optional_feature_runtime_crates() {
    let mut manifest = ProjectManifest::new(
        "Optional Feature Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_runtime_crate("zircon_plugin_sound_runtime")
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true)
                        .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false)
                .with_runtime_crate("zircon_plugin_animation_runtime"),
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
    let main_source = generated_file(&plan, "src/main.rs");
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_sound_timeline_animation_runtime".to_string()));
    assert!(
        cargo_manifest.contains(
            "zircon_plugin_sound_timeline_animation_runtime = { path = \"../../zircon_plugins/sound/features/timeline_animation_track/runtime\" }"
        ),
        "{cargo_manifest}"
    );
    assert!(plugin_source.contains(
        "pub fn runtime_plugin_feature_registration_providers() -> Vec<ExportRuntimePluginFeatureRegistrationProvider>"
    ));
    assert!(plugin_source.contains(
        "ExportRuntimePluginFeatureRegistrationProvider::new(zircon_plugin_sound_timeline_animation_runtime::plugin_feature_registration)"
    ));
    assert!(!plugin_source
        .contains("zircon_plugin_sound_timeline_animation_runtime::plugin_feature_registration()"));
    assert!(plugin_source.contains(
        ".with_runtime_plugin_feature_registration_providers(runtime_plugin_feature_registration_providers())"
    ));
    assert!(main_source.contains("zircon_app::bootstrap_export_runtime"));
    assert!(main_source.contains("zircon_plugins::export_runtime_bootstrap_config()"));
    assert!(!main_source.contains("EntryRunner::"));
    assert!(
        !main_source.contains("zircon_plugins::runtime_plugin_feature_registration_providers()")
    );
}

#[test]
fn source_template_links_external_feature_provider_runtime_crates() {
    let mut manifest = ProjectManifest::new(
        "External Feature Provider Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_runtime_crate("zircon_plugin_sound_runtime")
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true)
                        .with_provider_package_id("sound_timeline_animation_track")
                        .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false)
                .with_runtime_crate("zircon_plugin_animation_runtime"),
            external_feature_provider_selection("sound_timeline_animation_track", true),
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
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_sound_timeline_animation_runtime".to_string()));
    assert!(
        cargo_manifest.contains(
            "zircon_plugin_sound_timeline_animation_runtime = { path = \"../../zircon_plugins/sound_timeline_animation_track/runtime\" }"
        ),
        "{cargo_manifest}"
    );
    assert!(plugin_source.contains(
        "ExportRuntimePluginFeatureRegistrationProvider::new(zircon_plugin_sound_timeline_animation_runtime::plugin_feature_registration).with_provider_package_id(\"sound_timeline_animation_track\")"
    ));
    assert!(!plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("feature is not declared by the plugin catalog")));
}

#[test]
fn native_dynamic_exports_external_feature_provider_package_without_native_owner() {
    let mut manifest = ProjectManifest::new(
        "External Native Feature Provider Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_feature(
                    ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                        .enabled(true)
                        .with_provider_package_id("sound_timeline_animation_track")
                        .with_packaging(ExportPackagingStrategy::NativeDynamic),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
            external_feature_provider_selection("sound_timeline_animation_track", true),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::NativeDynamic,
    ])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let native_manifest = generated_file(&plan, "plugins/native_plugins.toml");

    assert_eq!(
        plan.native_dynamic_packages,
        vec!["sound_timeline_animation_track".to_string()]
    );
    assert!(native_manifest.contains("id = \"sound_timeline_animation_track\""));
    assert!(!plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("owner plugin sound is not NativeDynamic")));
}

#[test]
fn source_template_reports_native_dynamic_feature_without_native_owner_as_fatal_when_required() {
    let mut manifest = ProjectManifest::new(
        "Native Dynamic Feature Export Test",
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
                        .with_packaging(ExportPackagingStrategy::NativeDynamic),
                ),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Animation, true, false),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::NativeDynamic,
    ])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic.contains(
        "optional feature sound.timeline_animation_track uses NativeDynamic packaging but owner plugin sound is not NativeDynamic"
    )));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic.contains(
        "optional feature sound.timeline_animation_track uses NativeDynamic packaging but owner plugin sound is not NativeDynamic"
    )));
}

#[test]
fn source_template_reports_blocked_optional_feature_as_warning_only() {
    let mut manifest = ProjectManifest::new(
        "Blocked Optional Feature Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_runtime_crate("zircon_plugin_sound_runtime")
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .enabled(true)
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

    assert!(!plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_sound_timeline_animation_runtime".to_string()));
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("optional feature sound.timeline_animation_track is blocked")
            && diagnostic.contains("animation")
    }));
    assert!(plan.fatal_diagnostics.is_empty());
}

#[test]
fn source_template_reports_blocked_required_feature_as_fatal_diagnostic() {
    let mut manifest = ProjectManifest::new(
        "Blocked Required Feature Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            false,
        )
        .with_runtime_crate("zircon_plugin_sound_runtime")
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

    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("required feature sound.timeline_animation_track is blocked")
            && diagnostic.contains("animation")
    }));
    assert!(plan.has_fatal_diagnostics());
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("required feature sound.timeline_animation_track is blocked")
            && diagnostic.contains("animation")
    }));
}

fn generated_file<'a>(plan: &'a ExportBuildPlan, path: &str) -> &'a str {
    plan.generated_files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
        .unwrap_or_else(|| panic!("missing generated file {path}"))
}

fn external_feature_provider_selection(package_id: &str, enabled: bool) -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: package_id.to_string(),
        enabled,
        required: false,
        target_modes: vec![
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ],
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: None,
        editor_crate: None,
        features: Vec::new(),
    }
}
