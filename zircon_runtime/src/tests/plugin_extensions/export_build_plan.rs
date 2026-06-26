use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::{AssetUri, ProjectManifest};
use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::{
    plugin::ExportBuildPlan, plugin::ExportPackagingStrategy, plugin::ExportPipelineStage,
    plugin::ExportProfile, plugin::ExportTargetPlatform, plugin::ExportValidateReport,
    plugin::LibraryEmbedCompileHostTarget, plugin::ProjectPluginManifest,
    plugin::ProjectPluginSelection, plugin::RuntimePluginCatalog,
};

#[path = "export_build_plan/catalog_projection.rs"]
mod catalog_projection;

#[test]
fn source_template_generates_linked_external_runtime_plugin_registration_calls() {
    let mut manifest = ProjectManifest::new(
        "Plugin Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.asset_manifest = Some("export/assets.json".to_string());
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            true,
        )
        .with_runtime_crate("zircon_plugin_sound_runtime")],
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
    let project_manifest = generated_file(&plan, "assets/zircon-project.toml");

    assert!(plugin_source.contains(
        "pub fn runtime_plugin_registration_providers() -> Vec<ExportRuntimePluginRegistrationProvider>"
    ));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_sound_runtime::plugin_registration)"
    ));
    assert!(!plugin_source.contains("zircon_plugin_sound_runtime::plugin_registration()"));
    assert!(plugin_source
        .contains("pub fn export_runtime_bootstrap_config() -> ExportRuntimeBootstrapConfig"));
    assert!(main_source.contains("zircon_app::bootstrap_export_runtime"));
    assert!(main_source.contains("zircon_plugins::export_runtime_bootstrap_config()"));
    assert!(project_manifest.contains("asset_manifest = \"export/assets.json\""));
    assert!(!main_source.contains("EntryRunner::"));
    assert!(!main_source.contains("zircon_plugins::runtime_plugin_registrations()"));
    assert!(plan
        .runtime_plugin_availability
        .linked
        .iter()
        .any(|entry| entry.id == "sound"));
}

#[test]
fn export_plan_treats_missing_required_profile_providers_as_fatal() {
    let mut manifest = ProjectManifest::new(
        "Missing Required Provider Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            true,
        )],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::SourceTemplate])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();

    assert!(availability_contains(
        &plan.runtime_plugin_availability.externalized_missing,
        "sound"
    ));
    assert!(availability_contains(
        &plan.runtime_plugin_availability.missing_required,
        "sound"
    ));
    assert!(plan.has_fatal_diagnostics());
    assert!(plan.effective_fatal_diagnostics().iter().any(|diagnostic| {
        diagnostic
            .contains("required runtime plugin sound is unavailable for export profile client")
    }));

    let output_root = temp_dir("zircon_missing_required_provider_export");
    let report = plan.materialize(&output_root).unwrap();
    assert!(report.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic
            .contains("required runtime plugin sound is unavailable for export profile client")
    }));
    assert!(!output_root.exists());

    let archive_root = temp_dir("zircon_missing_required_provider_archive");
    let archive_path = archive_root.join("client-export.zip");
    let archive_report = plan
        .materialize_zip_archive(&archive_root, &archive_path)
        .unwrap();
    assert_eq!(
        archive_report.archive_file.as_deref(),
        Some(archive_path.as_path())
    );
    assert!(archive_report.generated_files.is_empty());
    assert!(archive_report.copied_packages.is_empty());
    assert!(archive_report.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic
            .contains("required runtime plugin sound is unavailable for export profile client")
    }));
    assert!(archive_report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("export archive materialization blocked")));
    assert!(!archive_path.exists());

    let _ = std::fs::remove_dir_all(output_root);
    let _ = std::fs::remove_dir_all(archive_root);
}

#[test]
fn source_template_links_physics_as_external_runtime_plugin() {
    let mut manifest = ProjectManifest::new(
        "Builtin Domain Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Physics,
            true,
            true,
        )
        .with_runtime_crate("zircon_plugin_physics_runtime")],
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

    assert_eq!(
        plan.linked_runtime_crates,
        vec!["zircon_plugin_physics_runtime".to_string()]
    );
    assert!(plugin_source.contains("id: \"physics\".to_string()"));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_physics_runtime::plugin_registration)"
    ));
    assert!(cargo_manifest.contains("zircon_plugin_physics_runtime"));
}

#[test]
fn source_template_profile_carries_build_validation_plan() {
    let mut manifest = ProjectManifest::new(
        "Source Template Build Validation Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Sound,
            true,
            true,
        )
        .with_runtime_crate("zircon_plugin_sound_runtime")],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let cargo_manifest = generated_file(&plan, "Cargo.toml");
    let source_template_build = plan
        .source_template_build
        .as_ref()
        .expect("SourceTemplate profile should carry a generated project build plan");

    assert!(plan.library_embed_compile_host.is_none());
    assert_eq!(
        plan.linked_runtime_crates,
        vec!["zircon_plugin_sound_runtime".to_string()]
    );
    assert!(cargo_manifest.contains("zircon_plugin_sound_runtime"));
    assert_eq!(source_template_build.manifest_path, "Cargo.toml");
    assert_eq!(
        source_template_build.target_dir,
        "stages/source_template/target"
    );
    assert!(source_template_build
        .command
        .windows(2)
        .any(|window| window == ["--manifest-path", "Cargo.toml"]));

    let report = ExportValidateReport::from_build_plan("zircon-project.toml", None, &plan);
    let source_template_report = report
        .plan_summary
        .expect("validate report should include a plan summary")
        .source_template_build
        .expect("validate report should expose SourceTemplate build plan");
    assert_eq!(
        source_template_report.command,
        source_template_build.command
    );
}

#[test]
fn export_plan_treats_animation_as_external_native_dynamic_package() {
    let mut manifest = ProjectManifest::new(
        "Builtin Native Domain Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Animation,
            true,
            true,
        )
        .with_runtime_crate("zircon_plugin_animation_runtime")
        .with_packaging(ExportPackagingStrategy::NativeDynamic)],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "native-only",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::NativeDynamic])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "native-only").unwrap();

    assert!(plan.linked_runtime_crates.is_empty());
    assert_eq!(plan.native_dynamic_packages, vec!["animation".to_string()]);
    assert!(availability_contains(
        &plan.runtime_plugin_availability.native_dynamic,
        "animation"
    ));
    assert!(generated_file(&plan, "plugins/native_plugins.toml").contains("id = \"animation\""));
    assert!(plan
        .generated_files
        .iter()
        .all(|file| file.path != "Cargo.toml"));
    assert!(plan.diagnostics.is_empty());
}

#[test]
fn source_template_keeps_editor_only_plugins_out_of_runtime_registrations() {
    let mut manifest = ProjectManifest::new(
        "Editor Only Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection {
            id: "runtime_diagnostics".to_string(),
            enabled: true,
            required: false,
            target_modes: vec![RuntimeTargetMode::EditorHost],
            packaging: ExportPackagingStrategy::LibraryEmbed,
            runtime_crate: None,
            editor_crate: Some("zircon_plugin_runtime_diagnostics_editor".to_string()),
            features: Vec::new(),
        }],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "editor",
        RuntimeTargetMode::EditorHost,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "editor").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    assert!(plan.linked_runtime_crates.is_empty());
    assert!(!plugin_source.contains("runtime_diagnostics_runtime::plugin_registration()"));
    assert!(!plugin_source.contains("runtime_diagnostics_editor::plugin_registration()"));
    assert!(!cargo_manifest.contains("zircon_plugin_runtime_diagnostics_editor"));
}

#[test]
fn source_template_links_runtime_backed_authoring_and_excludes_editor_only_authoring() {
    let mut manifest = ProjectManifest::new(
        "Authoring Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins =
        RuntimePluginCatalog::builtin().complete_project_manifest(&ProjectPluginManifest {
            selections: vec![
                ProjectPluginSelection::runtime_plugin(RuntimePluginId::Terrain, true, false),
                ProjectPluginSelection::runtime_plugin(RuntimePluginId::Tilemap2d, true, false),
                ProjectPluginSelection::runtime_plugin(RuntimePluginId::PrefabTools, true, false),
                ProjectPluginSelection {
                    id: "material_editor".to_string(),
                    enabled: true,
                    required: false,
                    target_modes: vec![RuntimeTargetMode::EditorHost],
                    packaging: ExportPackagingStrategy::LibraryEmbed,
                    runtime_crate: None,
                    editor_crate: Some("zircon_plugin_material_editor_editor".to_string()),
                    features: Vec::new(),
                },
                ProjectPluginSelection {
                    id: "timeline_sequence".to_string(),
                    enabled: true,
                    required: false,
                    target_modes: vec![RuntimeTargetMode::EditorHost],
                    packaging: ExportPackagingStrategy::LibraryEmbed,
                    runtime_crate: None,
                    editor_crate: Some("zircon_plugin_timeline_sequence_editor".to_string()),
                    features: Vec::new(),
                },
                ProjectPluginSelection {
                    id: "animation_graph".to_string(),
                    enabled: true,
                    required: false,
                    target_modes: vec![RuntimeTargetMode::EditorHost],
                    packaging: ExportPackagingStrategy::LibraryEmbed,
                    runtime_crate: None,
                    editor_crate: Some("zircon_plugin_animation_graph_editor".to_string()),
                    features: Vec::new(),
                },
            ],
        });
    manifest.export_profiles = vec![ExportProfile::new(
        "editor",
        RuntimeTargetMode::EditorHost,
        ExportTargetPlatform::Windows,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "editor").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_terrain_runtime".to_string()));
    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_tilemap_2d_runtime".to_string()));
    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_prefab_tools_runtime".to_string()));
    assert!(!plan.linked_runtime_crates.iter().any(|crate_name| {
        crate_name.contains("material_editor")
            || crate_name.contains("timeline_sequence")
            || crate_name.contains("animation_graph")
    }));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_terrain_runtime::plugin_registration)"
    ));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_tilemap_2d_runtime::plugin_registration)"
    ));
    assert!(plugin_source.contains(
        "ExportRuntimePluginRegistrationProvider::new(zircon_plugin_prefab_tools_runtime::plugin_registration)"
    ));
    assert!(!plugin_source.contains("zircon_plugin_material_editor_editor::plugin_registration()"));
    assert!(
        !plugin_source.contains("zircon_plugin_timeline_sequence_editor::plugin_registration()")
    );
    assert!(!plugin_source.contains("zircon_plugin_animation_graph_editor::plugin_registration()"));
    assert!(cargo_manifest.contains("zircon_plugin_terrain_runtime"));
    assert!(cargo_manifest.contains("zircon_plugin_tilemap_2d_runtime"));
    assert!(cargo_manifest.contains("zircon_plugin_prefab_tools_runtime"));
    assert!(!cargo_manifest.contains("zircon_plugin_material_editor_editor"));
    assert!(!cargo_manifest.contains("zircon_plugin_timeline_sequence_editor"));
    assert!(!cargo_manifest.contains("zircon_plugin_animation_graph_editor"));
}

#[test]
fn profile_with_features_compiles_to_build_plan() {
    let source = r#"
name = "Profile Feature Export"
default_scene = "res://scenes/main.zscene"
schema_version = 1

[export_profiles.windows-release]
platform = "windows-x86_64"
path = "library_embed"
mode = "release"
plugins = ["net", "sound"]
features = { net = ["http", "websocket"] }
asset_filter = "shipping"
"#;

    let manifest: ProjectManifest = toml::from_str(source).unwrap();
    let plan = ExportBuildPlan::from_project_manifest(&manifest, "windows-release").unwrap();

    assert_eq!(plan.profile.name, "windows-release");
    assert_eq!(plan.profile.target_mode, RuntimeTargetMode::ClientRuntime);
    assert_eq!(plan.profile.target_platform, ExportTargetPlatform::Windows);
    assert_eq!(
        plan.profile.strategies,
        vec![ExportPackagingStrategy::LibraryEmbed]
    );
    assert_eq!(plan.profile.selected_plugins, ["net", "sound"]);
    assert_eq!(
        plan.profile
            .features
            .get("net")
            .expect("net feature profile"),
        &vec!["http".to_string(), "websocket".to_string()]
    );
    assert_eq!(plan.profile.asset_filter.as_deref(), Some("shipping"));
    assert_eq!(plan.enabled_runtime_plugins.len(), 2);
    assert!(plan.enabled_runtime_plugins.contains(&"net".to_string()));
    assert!(plan.enabled_runtime_plugins.contains(&"sound".to_string()));
    for crate_name in [
        "zircon_plugin_net_runtime",
        "zircon_plugin_sound_runtime",
        "zircon_plugin_net_http_runtime",
        "zircon_plugin_net_websocket_runtime",
    ] {
        assert!(
            plan.linked_runtime_crates.contains(&crate_name.to_string()),
            "{crate_name} should be linked by the export profile"
        );
    }
    for crate_name in [
        "zircon_plugin_net_rpc_runtime",
        "zircon_plugin_net_content_download_runtime",
        "zircon_plugin_sound_ray_traced_convolution_runtime",
    ] {
        assert!(
            !plan.linked_runtime_crates.contains(&crate_name.to_string()),
            "{crate_name} should be trimmed by the export profile"
        );
    }
    assert!(plan.generated_files.is_empty());
    assert!(plan.diagnostics.is_empty(), "{:?}", plan.diagnostics);
    assert!(
        plan.effective_fatal_diagnostics().is_empty(),
        "{:?}",
        plan.effective_fatal_diagnostics()
    );
}

#[test]
fn invalid_plugin_combination_rejected_with_diagnostic() {
    let mut manifest = ProjectManifest::new(
        "Invalid Profile Plugin Combination",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, true)
                .with_runtime_crate("zircon_plugin_sound_runtime"),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Net, true, false)
                .with_runtime_crate("zircon_plugin_net_runtime"),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "net-only",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::LibraryEmbed])
    .with_selected_plugins(["net".to_string()])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "net-only").unwrap();

    assert!(plan.has_fatal_diagnostics());
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("export profile net-only excludes required plugin sound")
    }));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("export profile net-only excludes required plugin sound")
    }));
    assert!(!plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_sound_runtime".to_string()));
    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_net_runtime".to_string()));
}

#[test]
fn validate_report_summarizes_profile_plan_and_fatal_state() {
    let mut manifest = ProjectManifest::new(
        "Validate Report Export",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Net, true, false)
                .with_runtime_crate("zircon_plugin_net_runtime"),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "net-library",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([ExportPackagingStrategy::LibraryEmbed])
    .with_selected_plugins(["net".to_string()])
    .with_feature_selection("net", ["http".to_string()])
    .with_asset_filter("shipping")];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "net-library").unwrap();
    let report = ExportValidateReport::from_build_plan(
        "zircon-project.toml",
        Some("E:/zircon-export/stages/validate".to_string()),
        &plan,
    );
    let profile = report
        .profile_summary
        .as_ref()
        .expect("validate report should summarize the selected profile");
    let plan_summary = report
        .plan_summary
        .as_ref()
        .expect("validate report should summarize the build plan");

    assert_eq!(report.stage, ExportPipelineStage::Validate);
    assert_eq!(report.profile, "net-library");
    assert!(report.profile_found);
    assert!(!report.fatal);
    assert_eq!(profile.asset_filter.as_deref(), Some("shipping"));
    assert_eq!(profile.selected_plugins, ["net"]);
    assert_eq!(
        profile.features.get("net").expect("net feature profile"),
        &vec!["http".to_string()]
    );
    assert_eq!(plan_summary.enabled_runtime_plugins, ["net"]);
    assert!(plan_summary
        .linked_runtime_crates
        .contains(&"zircon_plugin_net_runtime".to_string()));
    assert!(plan_summary
        .linked_runtime_crates
        .contains(&"zircon_plugin_net_http_runtime".to_string()));
}

#[test]
fn feature_matrix_links_selected_plugins_only() {
    let mut manifest = ProjectManifest::new(
        "Library Embed Feature Matrix",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.plugins = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Rendering, true, true)
                .with_runtime_crate("zircon_plugin_rendering_runtime"),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
                .with_runtime_crate("zircon_plugin_sound_runtime"),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Net, true, false)
                .with_runtime_crate("zircon_plugin_net_runtime"),
            ProjectPluginSelection::runtime_plugin(RuntimePluginId::Physics, true, false)
                .with_runtime_crate("zircon_plugin_physics_runtime"),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "windows-release",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_build_mode(crate::plugin::ExportBuildMode::Release)
    .with_strategies([ExportPackagingStrategy::LibraryEmbed])
    .with_selected_plugins([
        "rendering".to_string(),
        "net".to_string(),
        "sound".to_string(),
    ])
    .with_feature_selection("net", ["http".to_string(), "websocket".to_string()])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "windows-release").unwrap();
    let compile_host = plan
        .library_embed_compile_host
        .as_ref()
        .expect("LibraryEmbed profile should produce a CompileHost plan");

    assert_eq!(compile_host.package, "zircon_app");
    assert_eq!(compile_host.binary, "zircon_runtime");
    assert_eq!(compile_host.cargo_profile, "release");
    assert!(compile_host.release);
    assert_eq!(compile_host.app_features, ["target-client"]);
    assert_eq!(compile_host.runtime_features, ["target-client"]);
    assert!(compile_host.command.contains(&"--release".to_string()));
    assert!(compile_host.command.contains(&"--features".to_string()));
    assert!(compile_host.command.contains(&"target-client".to_string()));
    for plugin_id in ["rendering", "net", "sound"] {
        assert!(
            compile_host
                .expected_runtime_plugins
                .contains(&plugin_id.to_string()),
            "{plugin_id} should be enabled for LibraryEmbed"
        );
    }
    assert!(!compile_host
        .expected_runtime_plugins
        .contains(&"physics".to_string()));

    for crate_name in [
        "zircon_plugin_rendering_runtime",
        "zircon_plugin_sound_runtime",
        "zircon_plugin_net_runtime",
        "zircon_plugin_net_http_runtime",
        "zircon_plugin_net_websocket_runtime",
    ] {
        assert!(
            compile_host
                .linked_runtime_crates
                .iter()
                .any(|linked_crate| linked_crate.crate_name == crate_name),
            "{crate_name} should be linked by CompileHost"
        );
    }
    for crate_name in [
        "zircon_plugin_physics_runtime",
        "zircon_plugin_net_rpc_runtime",
        "zircon_plugin_net_content_download_runtime",
    ] {
        assert!(
            !compile_host
                .linked_runtime_crates
                .iter()
                .any(|linked_crate| linked_crate.crate_name == crate_name),
            "{crate_name} should be trimmed by CompileHost"
        );
    }
    assert!(compile_host
        .linked_runtime_crates
        .iter()
        .any(|linked_crate| {
            linked_crate.crate_name == "zircon_plugin_net_http_runtime"
                && linked_crate.registration_kind == LibraryEmbedCompileHostTarget::RuntimeFeature
        }));
    assert!(
        plan.effective_fatal_diagnostics().is_empty(),
        "{:?}",
        plan.effective_fatal_diagnostics()
    );
}

fn generated_file<'a>(plan: &'a ExportBuildPlan, path: &str) -> &'a str {
    plan.generated_files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
        .unwrap_or_else(|| panic!("missing generated file {path}"))
}

fn availability_contains(
    entries: &[crate::plugin::RuntimePluginAvailabilityEntry],
    plugin_id: &str,
) -> bool {
    entries.iter().any(|entry| entry.id == plugin_id)
}

fn temp_dir(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{stamp}"))
}
