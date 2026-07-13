use super::*;

#[test]
fn profile_with_features_compiles_to_build_plan() {
    let source = r#"
name = "Profile Feature Export"
default_scene = "res://scenes/main.zscene"
schema_version = 1

[export_profiles.windows-release]
platform = "windows-x86_64"
output_name = "windows-release"
runtime_profile_id = "minimal"
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
    .with_runtime_profile_id(crate::core::framework::project::RuntimeProfileId::Minimal)
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

    assert_eq!(report.stage, ExportStage::Validate);
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
    .with_build_mode(crate::core::framework::project::ExportBuildMode::Release)
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
