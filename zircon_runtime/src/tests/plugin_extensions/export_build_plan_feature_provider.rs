use crate::asset::{AssetUri, ProjectManifest};
use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};
use crate::{
    core::framework::project::ExportPackagingStrategy, core::framework::project::ExportProfile,
    core::framework::project::ExportTargetPlatform,
    core::framework::project::ProjectPluginFeatureSelection,
    core::framework::project::ProjectPluginManifest,
    core::framework::project::ProjectPluginSelection, core::framework::project::RuntimeProfileId,
    plugin::ExportBuildPlan,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn source_template_catalog_completion_links_active_external_optional_feature_runtime_crates() {
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
            external_feature_provider_selection("sound_timeline_animation_track", true),
        ],
    };
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Client2d,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let root = linked_feature_source_root("implicit-external-provider");
    write_linked_feature_provider(&root, "production");
    let plan = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &manifest,
        "client",
        root.join("zircon_plugins"),
    )
    .unwrap();
    assert!(
        !plan.has_fatal_diagnostics(),
        "{:?}",
        plan.fatal_diagnostics
    );

    let persisted: ExportBuildPlan =
        serde_json::from_str(&serde_json::to_string(&plan).expect("serialize admitted plan"))
            .expect("deserialize admitted plan");
    let error = persisted
        .materialize(root.join("persisted-output"))
        .expect_err("deserialized plans require canonical replanning");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(error.to_string().contains("replan"));
    let mut blocked = plan.clone();
    blocked
        .fatal_diagnostics
        .push("blocked source preview".into());
    let blocked: ExportBuildPlan =
        serde_json::from_str(&serde_json::to_string(&blocked).expect("serialize blocked plan"))
            .expect("deserialize blocked plan");
    let error = blocked
        .preview_materialize(root.join("blocked-preview"))
        .expect_err("deserialized fatal plan cannot preview generated paths");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let main_source = generated_file(&plan, "src/main.rs");
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_sound_timeline_animation_runtime".to_string()));
    assert!(
        generated_feature_dependency_path(cargo_manifest)
            == std::path::absolute(
                root.join("zircon_plugins/sound_timeline_animation_track/runtime")
            )
            .unwrap(),
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
    let _ = fs::remove_dir_all(root);
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
        RuntimeProfileId::Client2d,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];

    let root = linked_feature_source_root("explicit-external-provider");
    write_linked_feature_provider(&root, "production");
    let plan = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &manifest,
        "client",
        root.join("zircon_plugins"),
    )
    .unwrap();
    assert!(
        !plan.has_fatal_diagnostics(),
        "{:?}",
        plan.fatal_diagnostics
    );
    let plugin_source = generated_file(&plan, "src/zircon_plugins.rs");
    let cargo_manifest = generated_file(&plan, "Cargo.toml");

    assert!(plan
        .linked_runtime_crates
        .contains(&"zircon_plugin_sound_timeline_animation_runtime".to_string()));
    assert!(
        generated_feature_dependency_path(cargo_manifest)
            == std::path::absolute(
                root.join("zircon_plugins/sound_timeline_animation_track/runtime")
            )
            .unwrap(),
        "{cargo_manifest}"
    );
    assert!(plugin_source.contains(
        "ExportRuntimePluginFeatureRegistrationProvider::new(zircon_plugin_sound_timeline_animation_runtime::plugin_feature_registration).with_provider_package_id(\"sound_timeline_animation_track\")"
    ));
    assert!(!plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("feature is not declared by the plugin catalog")));
    let _ = fs::remove_dir_all(root);
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
        RuntimeProfileId::Client2d,
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
fn source_template_reports_missing_native_dynamic_feature_provider_as_fatal_when_required() {
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
        RuntimeProfileId::Client2d,
    )
    .with_strategies([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::NativeDynamic,
    ])];

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();

    assert!(plan.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("required feature sound.timeline_animation_track is blocked")
            && diagnostic.contains("missing plugins: sound_timeline_animation_track")
    }));
    assert!(plan.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("required feature sound.timeline_animation_track is blocked")
            && diagnostic.contains("missing plugins: sound_timeline_animation_track")
    }));
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
        RuntimeProfileId::Client2d,
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
        RuntimeProfileId::Client2d,
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

fn generated_feature_dependency_path(cargo_manifest: &str) -> PathBuf {
    let cargo: toml::Value = toml::from_str(cargo_manifest).expect("generated Cargo must parse");
    PathBuf::from(
        cargo["dependencies"]["zircon_plugin_sound_timeline_animation_runtime"]["path"]
            .as_str()
            .expect("linked feature dependency path"),
    )
}

#[test]
fn materialized_linked_feature_cargo_path_is_bound_to_admitted_source_at_any_output_root() {
    let root = linked_feature_source_root("arbitrary-output");
    write_linked_feature_provider(&root, "production");
    let plan = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &linked_feature_test_manifest(),
        "client",
        root.join("zircon_plugins"),
    )
    .expect("source-aware export plan");
    let output = root.join("Builds/zircon/custom-output");
    let materialized = plan.materialize(&output).expect("admitted materialization");
    assert!(materialized.fatal_diagnostics.is_empty());
    let generated_cargo =
        fs::read_to_string(output.join("Cargo.toml")).expect("materialized Cargo manifest");
    let dependency = generated_feature_dependency_path(&generated_cargo);
    assert!(dependency.is_absolute());
    assert!(dependency.starts_with(root.join("zircon_plugins")));
    assert!(!dependency.starts_with(root.join("Builds/zircon_plugins")));
    assert_eq!(
        output.join(&dependency),
        std::path::absolute(root.join("zircon_plugins/sound_timeline_animation_track/runtime"))
            .unwrap()
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn library_embed_only_linked_feature_keeps_its_admitted_source_receipt() {
    let root = linked_feature_source_root("library-embed-only");
    write_linked_feature_provider(&root, "production");
    let mut manifest = linked_feature_test_manifest();
    manifest.export_profiles[0].strategies = vec![ExportPackagingStrategy::LibraryEmbed];
    let plan = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &manifest,
        "client",
        root.join("zircon_plugins"),
    )
    .expect("library embed plan with admitted linked feature");
    assert!(
        !plan.has_fatal_diagnostics(),
        "{:?}",
        plan.fatal_diagnostics
    );
    assert!(plan.generated_files.is_empty());
    assert_eq!(plan.linked_feature_sources.len(), 1);
    let materialized = plan
        .materialize(root.join("library-embed-output"))
        .expect("library embed linked receipt remains valid");
    assert!(materialized.fatal_diagnostics.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn linked_feature_materialization_rejects_stripped_receipts_and_generated_binding_drift() {
    let root = linked_feature_source_root("binding-drift");
    write_linked_feature_provider(&root, "production");
    let mut manifest = linked_feature_test_manifest();
    manifest.export_profiles[0].strategies = vec![ExportPackagingStrategy::SourceTemplate];
    let plan = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &manifest,
        "client",
        root.join("zircon_plugins"),
    )
    .expect("source-aware export plan");
    assert!(
        !plan.has_fatal_diagnostics(),
        "{:?}",
        plan.fatal_diagnostics
    );
    assert!(plan.library_embed_compile_host.is_none());

    let mut stripped = plan.clone();
    stripped.linked_feature_source_count = 0;
    stripped.linked_feature_sources.clear();
    stripped.diagnostics.clear();
    stripped.fatal_diagnostics.clear();
    let stripped: ExportBuildPlan = serde_json::from_str(
        &serde_json::to_string(&stripped).expect("serialize stripped export plan"),
    )
    .expect("deserialize stripped export plan");
    let stripped_output = root.join("stripped-output");
    let error = stripped
        .materialize(&stripped_output)
        .expect_err("generated feature provider must require its admitted receipt");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(!stripped_output.exists());

    let mut wrong_provider = plan.clone();
    let generated = wrong_provider
        .generated_files
        .iter_mut()
        .find(|file| file.path == "src/zircon_plugins.rs")
        .expect("generated provider table");
    generated.contents = generated.contents.replace(
        "\"sound_timeline_animation_track\", \"zircon_plugin_sound_timeline_animation_runtime\"",
        "\"rogue_carrier\", \"zircon_plugin_sound_timeline_animation_runtime\"",
    );
    assert!(generated.contents.contains("\"rogue_carrier\""));
    let error = wrong_provider
        .materialize(root.join("wrong-provider-output"))
        .expect_err("generated provider identity drift must reject");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);

    let mut wrong_cargo = plan.clone();
    let generated = wrong_cargo
        .generated_files
        .iter_mut()
        .find(|file| file.path == "Cargo.toml")
        .expect("generated Cargo manifest");
    generated.contents = generated.contents.replace(
        &root
            .join("zircon_plugins/sound_timeline_animation_track/runtime")
            .to_string_lossy()
            .replace('\\', "/"),
        "../../zircon_plugins/rogue_carrier/runtime",
    );
    assert!(generated
        .contents
        .contains("../../zircon_plugins/rogue_carrier/runtime"));
    let error = wrong_cargo
        .materialize(root.join("wrong-cargo-output"))
        .expect_err("generated Cargo dependency drift must reject");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);

    let mut aliased = plan;
    aliased.linked_feature_source_count = 0;
    aliased.linked_feature_sources.clear();
    aliased.diagnostics.clear();
    aliased.fatal_diagnostics.clear();
    let generated = aliased
        .generated_files
        .iter_mut()
        .find(|file| file.path == "src/zircon_plugins.rs")
        .expect("generated provider table");
    generated.contents = generated
        .contents
        .replacen(
            "use zircon_app::{",
            "use zircon_app::ExportRuntimePluginFeatureRegistrationProvider as FeatureProvider;\nuse zircon_app::{",
            1,
        )
        .replace(
            "ExportRuntimePluginFeatureRegistrationProvider::new(",
            "FeatureProvider::new(",
        );
    assert!(generated.contents.contains("FeatureProvider::new("));
    assert!(!generated
        .contents
        .contains("ExportRuntimePluginFeatureRegistrationProvider::new("));
    let aliased: ExportBuildPlan =
        serde_json::from_str(&serde_json::to_string(&aliased).expect("serialize aliased plan"))
            .expect("deserialize aliased plan");
    write_linked_feature_provider(&root, "test_fixture");
    let error = aliased
        .materialize(root.join("aliased-output"))
        .expect_err("aliased provider cannot bypass source admission after deserialization");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(error.to_string().contains("replan"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn linked_feature_source_role_is_admitted_from_independent_provider_manifest() {
    for (role, eligible) in [
        ("production", true),
        ("developer_tool", true),
        ("sample", false),
        ("test_fixture", false),
    ] {
        let root = linked_feature_source_root(role);
        write_linked_feature_provider(&root, role);
        let plan = ExportBuildPlan::from_project_manifest_with_plugin_root(
            &linked_feature_test_manifest(),
            "client",
            root.join("zircon_plugins"),
        )
        .expect("source-aware export plan");
        let generated = generated_file(&plan, "src/zircon_plugins.rs");
        let role_variant = match role {
            "production" => "Production",
            "developer_tool" => "DeveloperTool",
            "sample" => "Sample",
            "test_fixture" => "TestFixture",
            _ => unreachable!(),
        };

        assert!(generated.contains(&format!(
            ".with_admitted_source_identity(\"sound.timeline_animation_track\", \"sound\", \"sound_timeline_animation_track\", \"zircon_plugin_sound_timeline_animation_runtime\", zircon_runtime::plugin::PluginPackageRole::{role_variant})"
        )));
        assert_eq!(plan.linked_feature_sources.len(), 1);
        assert_eq!(
            plan.linked_feature_sources[0].package_role(),
            match role {
                "production" => crate::plugin::PluginPackageRole::Production,
                "developer_tool" => crate::plugin::PluginPackageRole::DeveloperTool,
                "sample" => crate::plugin::PluginPackageRole::Sample,
                "test_fixture" => crate::plugin::PluginPackageRole::TestFixture,
                _ => unreachable!(),
            }
        );
        assert_eq!(
            plan.fatal_diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.contains("not eligible for product export") }),
            !eligible
        );
        if !eligible {
            let mut stripped = plan.clone();
            stripped.fatal_diagnostics.clear();
            stripped.diagnostics.clear();
            let output = root.join("stripped-diagnostic-export");
            let error = stripped
                .materialize(&output)
                .expect_err("test-only receipt must reject even if diagnostics are removed");
            assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
            assert!(!output.exists());
        }
        let output = root.join("export");
        let materialized = plan
            .materialize(&output)
            .expect("admitted source role should produce a deterministic export outcome");
        assert_eq!(!materialized.fatal_diagnostics.is_empty(), !eligible);
        assert_eq!(output.exists(), eligible);
        let _ = fs::remove_dir_all(root);
    }
}

#[test]
fn linked_feature_export_without_source_root_or_independent_manifest_fails_closed() {
    let manifest = linked_feature_test_manifest();
    let no_root = ExportBuildPlan::from_project_manifest(&manifest, "client")
        .expect("legacy plan must return explicit diagnostics");
    assert!(no_root.has_fatal_diagnostics());
    assert!(no_root.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("linked feature sound.timeline_animation_track")
            && diagnostic.contains("plugin source root")
    }));
    let mut stripped = no_root.clone();
    stripped.fatal_diagnostics.clear();
    stripped.diagnostics.clear();
    let stripped_output = linked_feature_source_root("stripped-no-root-output");
    let error = stripped
        .materialize(&stripped_output)
        .expect_err("a missing linked feature receipt must reject after diagnostics are cleared");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(!stripped_output.exists());

    stripped.linked_feature_source_count = 0;
    let error = stripped
        .materialize(&stripped_output)
        .expect_err("linked feature links must reject a forged zero receipt count");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert!(!stripped_output.exists());

    let root = linked_feature_source_root("missing");
    fs::create_dir_all(root.join("zircon_plugins/sound/features/timeline_animation_track/runtime"))
        .expect("owner-embedded crate fixture");
    fs::create_dir_all(root.join("zircon_plugins/sound_timeline_animation_track"))
        .expect("independent carrier directory fixture");
    let missing_carrier = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &manifest,
        "client",
        root.join("zircon_plugins"),
    )
    .expect("missing carrier must return explicit diagnostics");
    assert!(missing_carrier
        .fatal_diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.contains("sound_timeline_animation_track/plugin.toml") }));
    let output = root.join("export");
    let report = missing_carrier
        .materialize(&output)
        .expect("blocked materialization report");
    assert!(!report.fatal_diagnostics.is_empty());
    assert!(!output.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn linked_feature_source_role_change_blocks_materialization_before_writing() {
    let root = linked_feature_source_root("drift");
    write_linked_feature_provider(&root, "production");
    let plan = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &linked_feature_test_manifest(),
        "client",
        root.join("zircon_plugins"),
    )
    .expect("source-aware export plan");
    assert!(
        !plan.has_fatal_diagnostics(),
        "{:?}",
        plan.fatal_diagnostics
    );
    write_linked_feature_provider(&root, "test_fixture");
    let output = root.join("export");
    let error = plan
        .materialize(&output)
        .expect_err("manifest drift must block export");
    assert!(error.to_string().contains("linked feature source changed"));
    assert!(!output.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn linked_feature_source_rejects_an_oversized_provider_manifest() {
    let root = linked_feature_source_root("oversized-manifest");
    write_linked_feature_provider(&root, "production");
    let manifest_path = root.join("zircon_plugins/sound_timeline_animation_track/plugin.toml");
    fs::OpenOptions::new()
        .write(true)
        .open(&manifest_path)
        .expect("provider manifest fixture")
        .set_len(4 * 1024 * 1024 + 1)
        .expect("oversized provider manifest fixture");
    let plan = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &linked_feature_test_manifest(),
        "client",
        root.join("zircon_plugins"),
    )
    .expect("oversized source must be diagnosed");
    assert!(plan
        .fatal_diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("source manifest size limit")));
    assert!(plan.linked_feature_sources.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn linked_feature_source_rejects_unknown_role_and_mismatched_feature_owner() {
    let root = linked_feature_source_root("invalid-role");
    write_linked_feature_provider(&root, "unknown_carrier_role");
    let unknown_role = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &linked_feature_test_manifest(),
        "client",
        root.join("zircon_plugins"),
    )
    .expect("unknown source role must be diagnosed");
    assert!(unknown_role.has_fatal_diagnostics());
    assert!(unknown_role
        .fatal_diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.contains("unknown_carrier_role") }));
    assert!(unknown_role.linked_feature_sources.is_empty());

    write_linked_feature_provider(&root, "production");
    let manifest_path = root.join("zircon_plugins/sound_timeline_animation_track/plugin.toml");
    let source = fs::read_to_string(&manifest_path).expect("provider manifest fixture");
    fs::write(
        &manifest_path,
        source.replace("package_role = \"production\"\n", ""),
    )
    .expect("missing role fixture");
    let missing_role = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &linked_feature_test_manifest(),
        "client",
        root.join("zircon_plugins"),
    )
    .expect("missing external source role must be diagnosed");
    assert!(missing_role.fatal_diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("sound_timeline_animation_track/plugin.toml")
            && diagnostic.contains("explicit package_role")
    }));
    assert!(missing_role.linked_feature_sources.is_empty());

    fs::write(
        &manifest_path,
        source.replace(
            "owner_plugin_id = \"sound\"",
            "owner_plugin_id = \"animation\"",
        ),
    )
    .expect("mismatched owner fixture");
    let mismatched_owner = ExportBuildPlan::from_project_manifest_with_plugin_root(
        &linked_feature_test_manifest(),
        "client",
        root.join("zircon_plugins"),
    )
    .expect("mismatched owner must be diagnosed");
    assert!(mismatched_owner
        .fatal_diagnostics
        .iter()
        .any(|diagnostic| { diagnostic.contains("owner/provider does not match") }));
    assert!(mismatched_owner.linked_feature_sources.is_empty());
    let _ = fs::remove_dir_all(root);
}

fn linked_feature_test_manifest() -> ProjectManifest {
    let mut manifest = ProjectManifest::new(
        "Linked Feature Source Admission",
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
                        .required(true)
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
        RuntimeProfileId::Client2d,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)
    .with_strategy(ExportPackagingStrategy::LibraryEmbed)];
    manifest
}

fn linked_feature_source_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon-linked-feature-source-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn write_linked_feature_provider(root: &Path, role: &str) {
    let package = root.join("zircon_plugins/sound_timeline_animation_track");
    let runtime = package.join("runtime");
    fs::create_dir_all(&runtime).expect("provider runtime fixture");
    fs::write(
        package.join("plugin.toml"),
        format!(
            "id = \"sound_timeline_animation_track\"\nversion = \"0.1.0\"\ndisplay_name = \"Sound Timeline Carrier\"\npackage_kind = \"feature_extension\"\npackage_role = \"{role}\"\n\n[[feature_extensions]]\nid = \"sound.timeline_animation_track\"\ndisplay_name = \"Sound Timeline\"\nowner_plugin_id = \"sound\"\nprovider_package_id = \"sound_timeline_animation_track\"\n\n[[feature_extensions.dependencies]]\nplugin_id = \"sound\"\ncapability = \"runtime.plugin.sound\"\nprimary = true\n\n[[feature_extensions.modules]]\nname = \"sound.timeline_animation_track.runtime\"\nkind = \"runtime\"\ncrate_name = \"zircon_plugin_sound_timeline_animation_runtime\"\ntarget_modes = [\"client_runtime\"]\n"
        ),
    )
    .expect("provider manifest fixture");
    fs::write(
        runtime.join("Cargo.toml"),
        "[package]\nname = \"zircon_plugin_sound_timeline_animation_runtime\"\nversion = \"0.1.0\"\n",
    )
    .expect("provider Cargo fixture");
}
