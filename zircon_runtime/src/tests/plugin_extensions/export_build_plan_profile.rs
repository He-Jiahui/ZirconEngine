use crate::asset::{AssetUri, ProjectManifest};
use crate::builtin::RuntimeTargetMode;
use crate::{
    plugin::ExportBuildPlan, plugin::ExportPackagingStrategy, plugin::ExportPlatformHostKind,
    plugin::ExportPlatformResourceStrategy, plugin::ExportProfile, plugin::ExportTargetPlatform,
    plugin::RuntimePluginAvailabilityEntry, plugin::RuntimeProfileId,
};

#[test]
fn export_plan_uses_explicit_runtime_profile_id_before_name_inference() {
    let mut manifest = ProjectManifest::new(
        "Explicit Runtime Profile Export Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_runtime_profile_id(RuntimeProfileId::Client3d)
    .with_strategy(ExportPackagingStrategy::SourceTemplate)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");

    assert!(plugin_source.contains("runtime_profile_id: Some(RuntimeProfileId::Client3d)"));
    assert!(availability_contains(
        &plan.runtime_plugin_availability.externalized_missing,
        "navigation"
    ));
    assert!(!availability_contains(
        &plan.runtime_plugin_availability.externalized_missing,
        "tilemap_2d"
    ));
}

#[test]
fn built_in_default_export_profiles_have_explicit_runtime_profile_ids_and_server_is_headless() {
    let manifest = ProjectManifest::new(
        "Default Profile Runtime Profile Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );

    let client = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let server = ExportBuildPlan::from_project_manifest(&manifest, "server").unwrap();

    assert_eq!(
        client.profile.runtime_profile_id,
        Some(RuntimeProfileId::Client2d)
    );
    assert_eq!(
        server.profile.runtime_profile_id,
        Some(RuntimeProfileId::Server)
    );
    assert_eq!(
        server.profile.target_platform,
        ExportTargetPlatform::Headless
    );
    assert_eq!(
        server.platform_policy.host_kind,
        ExportPlatformHostKind::Headless
    );
    assert_eq!(
        server.platform_policy.resource_strategy,
        ExportPlatformResourceStrategy::FilesystemBundle
    );
}

#[test]
fn export_plan_rejects_runtime_profile_id_target_mode_mismatch() {
    let mut manifest = ProjectManifest::new(
        "Runtime Profile Target Mismatch Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.export_profiles = vec![ExportProfile::new(
        "bad-client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_runtime_profile_id(RuntimeProfileId::Server)
    .with_strategy(ExportPackagingStrategy::SourceTemplate)];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "bad-client").unwrap();

    assert!(plan.has_fatal_diagnostics());
    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains(
            "export profile bad-client selects runtime profile Server with target mode ServerRuntime",
        )
    }));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains(
            "export profile bad-client selects runtime profile Server with target mode ServerRuntime",
        )
    }));
}

#[test]
fn export_plan_reports_duplicate_profile_strategies_and_generates_sanitized_profile() {
    let mut manifest = ProjectManifest::new(
        "Duplicate Export Strategy Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
    ])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let export_profile_source = plugin_source
        .split("pub fn export_profile() -> ExportProfile {")
        .nth(1)
        .and_then(|source| source.split("pub fn project_plugins()").next())
        .expect("generated plugin source must include export_profile before project_plugins");

    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("export profile client strategies")
        && diagnostic.contains("SourceTemplate")));
    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("export profile client strategies")
        && diagnostic.contains("LibraryEmbed")));
    assert!(!plan
        .fatal_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("export profile client strategies")));
    assert_eq!(
        plan.profile.strategies,
        vec![
            ExportPackagingStrategy::SourceTemplate,
            ExportPackagingStrategy::LibraryEmbed,
        ]
    );
    assert_eq!(
        export_profile_source
            .matches("ExportPackagingStrategy::SourceTemplate")
            .count(),
        1
    );
    assert_eq!(
        export_profile_source
            .matches("ExportPackagingStrategy::LibraryEmbed")
            .count(),
        1
    );
}

#[test]
fn export_plan_reports_empty_profile_strategies_as_fatal() {
    let mut manifest = ProjectManifest::new(
        "Empty Export Strategy Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
    .with_strategies([])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();

    assert!(plan.has_fatal_diagnostics());
    assert!(plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains(
            "export profile client strategies must include at least one packaging strategy"
        )));
    assert!(plan
        .fatal_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains(
            "export profile client strategies must include at least one packaging strategy"
        )));
    assert!(plan.profile.strategies.is_empty());
}

#[test]
fn export_plan_reports_duplicate_profile_names_as_fatal() {
    let mut manifest = ProjectManifest::new(
        "Duplicate Export Profile Name Test",
        AssetUri::parse("res://scenes/main.zscene").unwrap(),
        1,
    );
    manifest.export_profiles = vec![
        ExportProfile::new(
            "client",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate),
        ExportProfile::new(
            "client",
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Headless,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate),
    ];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();

    assert!(plan.has_fatal_diagnostics());
    assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("export profile name \"client\" must be unique")
        && diagnostic.contains("found 2 matching export profiles")));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("export profile name \"client\" must be unique")
        && diagnostic.contains("found 2 matching export profiles")));
    assert_eq!(plan.profile.target_platform, ExportTargetPlatform::Windows);
}

#[test]
fn export_plan_reports_invalid_profile_names_as_fatal() {
    for profile_name in [" client ", "   "] {
        let mut manifest = ProjectManifest::new(
            "Invalid Export Profile Name Test",
            AssetUri::parse("res://scenes/main.zscene").unwrap(),
            1,
        );
        manifest.export_profiles = vec![ExportProfile::new(
            profile_name,
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate)];

        let plan = ExportBuildPlan::from_project_manifest(&manifest, profile_name).unwrap();

        assert!(plan.has_fatal_diagnostics());
        assert!(plan
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("export profile name")
                && diagnostic.contains("must be non-empty and trimmed")));
        assert!(plan
            .fatal_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("export profile name")
                && diagnostic.contains("must be non-empty and trimmed")));
        assert_eq!(plan.profile.name, profile_name);
    }
}

#[test]
fn export_plan_reports_invalid_profile_output_names_and_generates_sanitized_profile() {
    for (raw_output_name, expected_output_name) in
        [(" client-output ", "client-output"), ("   ", "client")]
    {
        let mut manifest = ProjectManifest::new(
            "Invalid Export Output Name Test",
            AssetUri::parse("res://scenes/main.zscene").unwrap(),
            1,
        );
        let mut profile = ExportProfile::new(
            "client",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate);
        profile.output_name = raw_output_name.to_string();
        manifest.export_profiles = vec![profile];

        let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
        let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
        let cargo_manifest = generated_file(&plan, "Cargo.toml");

        assert!(plan.diagnostics.iter().any(|diagnostic| diagnostic
            .contains("export profile client output_name")
            && diagnostic.contains("must be non-empty and trimmed")));
        assert!(!plan
            .fatal_diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("export profile client output_name")));
        assert_eq!(plan.profile.output_name, expected_output_name);
        assert!(plugin_source.contains(&format!(
            "output_name: {expected_output_name:?}.to_string()"
        )));
        assert!(!plugin_source.contains(&format!("output_name: {raw_output_name:?}.to_string()")));
        assert!(
            cargo_manifest.contains(&format!("name = \"zircon_export_{expected_output_name}\""))
        );
    }
}

fn generated_file<'a>(plan: &'a ExportBuildPlan, path: &str) -> &'a str {
    plan.generated_files
        .iter()
        .find(|file| file.path == path)
        .map(|file| file.contents.as_str())
        .unwrap_or_else(|| panic!("missing generated file {path}"))
}

fn availability_contains(entries: &[RuntimePluginAvailabilityEntry], plugin_id: &str) -> bool {
    entries.iter().any(|entry| entry.id == plugin_id)
}
