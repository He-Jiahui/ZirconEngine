use std::fs;
use std::path::PathBuf;

use crate::asset::project::{
    ProjectManifest, ProjectManifestError, ProjectPaths, ProjectScriptManifest,
};
use crate::asset::AssetUri;
use crate::core::resource::io::AtomicWriteFault;
use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};
use crate::{
    core::framework::project::ExportBuildMode, core::framework::project::ExportPackagingStrategy,
    core::framework::project::ExportProfile, core::framework::project::ExportTargetPlatform,
    core::framework::project::ProjectPluginSelection, core::framework::project::RuntimeProfileId,
    plugin::ExportBuildPlan,
};

use super::unique_temp_project_root;
use zircon_runtime_interface::project::{
    ProjectManifestSummary, ProjectManifestSummaryError, RelPath,
};

#[test]
fn project_manifest_roundtrip_preserves_default_scene_and_paths() {
    let root = unique_temp_project_root("manifest");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let manifest = ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        3,
    );
    manifest.save(paths.manifest_path()).unwrap();

    let loaded = ProjectManifest::load(paths.manifest_path()).unwrap();

    assert_eq!(loaded, manifest);
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .is_dir());
    assert!(paths.asset_artifact_root().is_dir());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manifest_roundtrip_preserves_declared_ui_roots() {
    let root = unique_temp_project_root("manifest_ui_roots");
    let path = root.join("zircon-project.toml");
    let mut manifest = ProjectManifest::new(
        "Ui Roots",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );
    manifest.ui_roots = vec![
        AssetUri::parse("res://ui/hud.zui").unwrap(),
        AssetUri::parse("res://ui/menu.zui").unwrap(),
    ];

    manifest.save(&path).unwrap();

    assert_eq!(ProjectManifest::load(&path).unwrap(), manifest);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manifest_rejects_non_project_or_duplicate_ui_roots() {
    let non_project = ProjectManifest::from_toml_str(
        r#"
name = "Invalid UI Root"
format_version = 3
project_guid = "d6a3cc3c-2900-4bff-a465-e932cb574e9d"
default_scene = "res://scenes/main.scene.toml"
library_version = 1
ui_roots = ["builtin://ui/hud.zui"]
"#,
    )
    .unwrap_err();
    assert!(matches!(
        non_project,
        ProjectManifestError::InvalidUiRootScheme { .. }
    ));

    let duplicate = ProjectManifest::from_toml_str(
        r#"
name = "Duplicate UI Root"
format_version = 3
project_guid = "0b38357e-6229-4ed9-ae0d-92af48d34b5e"
default_scene = "res://scenes/main.scene.toml"
library_version = 1
ui_roots = ["res://ui/hud.zui", "res://ui/hud.zui"]
"#,
    )
    .unwrap_err();
    assert!(matches!(
        duplicate,
        ProjectManifestError::DuplicateUiRoot { .. }
    ));

    let labelled = ProjectManifest::from_toml_str(
        r#"
name = "Labelled UI Root"
format_version = 3
project_guid = "9e4c2f39-1780-4d14-80b8-1864e37a3a7f"
default_scene = "res://scenes/main.scene.toml"
library_version = 1
ui_roots = ["res://ui/hud.zui#Hud"]
"#,
    )
    .unwrap_err();
    assert!(matches!(
        labelled,
        ProjectManifestError::LabelledUiRoot { .. }
    ));
}

#[test]
fn project_manifest_refuses_legacy_formats_until_explicit_migration() {
    for (fixture, expected_source_format_version) in [("v1", 1), ("v2", 2)] {
        let error =
            ProjectManifest::load_with_report(shared_manifest_fixture(fixture)).unwrap_err();
        let ProjectManifestError::MigrationRequired {
            source_format_version,
        } = error
        else {
            panic!("{fixture} must not enter runtime loading before migration: {error}");
        };

        assert_eq!(source_format_version, expected_source_format_version);
    }
}

#[test]
fn project_manifest_rejects_future_format_versions() {
    let error = ProjectManifest::load(shared_manifest_fixture("future")).unwrap_err();

    assert!(matches!(
        error,
        ProjectManifestError::Summary(ProjectManifestSummaryError::FutureVersion {
            found: 4,
            supported: 3
        })
    ));
}

#[test]
fn project_manifest_rejects_unsafe_empty_and_duplicate_asset_roots() {
    let unsafe_root = ProjectManifest::from_toml_str(
        r#"
name = "Unsafe Roots"
format_version = 3
project_guid = "ba3ee75e-1b75-42d8-8cbe-d3b75b365817"
default_scene = "res://scenes/main.scene.toml"
asset_roots = ["../outside"]
library_version = 1
"#,
    )
    .unwrap_err();
    assert!(matches!(unsafe_root, ProjectManifestError::Decode { .. }));

    let empty_roots = ProjectManifest::from_toml_str(
        r#"
name = "Empty Roots"
format_version = 3
project_guid = "a7ac61cc-53bb-4ea7-a737-222a1b93eac9"
default_scene = "res://scenes/main.scene.toml"
asset_roots = []
library_version = 1
"#,
    )
    .unwrap_err();
    assert!(matches!(empty_roots, ProjectManifestError::EmptyAssetRoots));

    let duplicate_roots = ProjectManifest::from_toml_str(
        r#"
name = "Duplicate Roots"
format_version = 3
project_guid = "878abc6c-b808-4a94-8f67-10653733e213"
default_scene = "res://scenes/main.scene.toml"
asset_roots = ["assets", "assets/"]
library_version = 1
"#,
    )
    .unwrap_err();
    assert!(matches!(
        duplicate_roots,
        ProjectManifestError::DuplicateAssetRoot { .. }
    ));
}

#[test]
fn project_manifest_save_is_stable_and_writes_only_v3() {
    let root = unique_temp_project_root("manifest_stable_v3");
    let path = root.join("zircon-project.toml");
    let mut manifest = ProjectManifest::new(
        "Stable Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        4,
    );
    manifest.engine_version_req = Some("^0.1".to_string());
    manifest.asset_roots = vec![
        RelPath::parse("game-assets").unwrap(),
        RelPath::parse("shared-assets").unwrap(),
    ];
    manifest.settings = Some(RelPath::parse("config/project-settings.toml").unwrap());

    manifest.save(&path).unwrap();
    let first = fs::read_to_string(&path).unwrap();
    let loaded = ProjectManifest::load_with_report(&path).unwrap();
    loaded.value.save(&path).unwrap();
    let second = fs::read_to_string(&path).unwrap();

    assert!(first.contains("format_version = 3"));
    assert!(first.contains("project_guid = \""));
    assert!(first.contains("asset_roots = [\"game-assets\", \"shared-assets\"]"));
    assert_eq!(first, second);
    assert_eq!(loaded.migrated_from, None);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manifest_atomic_save_faults_keep_the_previous_manifest_readable() {
    let root = unique_temp_project_root("manifest_atomic_faults");
    let path = root.join("zircon-project.toml");
    let original = ProjectManifest::new(
        "Original Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        4,
    );
    original.save(&path).unwrap();
    let original_bytes = fs::read(&path).unwrap();

    let replacement = ProjectManifest::new(
        "Replacement Sandbox",
        AssetUri::parse("res://scenes/replacement.scene.toml").unwrap(),
        5,
    );
    for fault in [
        AtomicWriteFault::Write,
        AtomicWriteFault::Sync,
        AtomicWriteFault::Replace,
    ] {
        let error = replacement
            .save_with_atomic_fault(&path, fault)
            .unwrap_err();

        assert!(matches!(error, ProjectManifestError::Write { .. }));
        assert_eq!(fs::read(&path).unwrap(), original_bytes);
        assert_eq!(ProjectManifest::load(&path).unwrap(), original);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
    }

    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn project_manifest_atomic_save_restores_after_windows_replace_failure() {
    let root = unique_temp_project_root("manifest_windows_replace_recovery");
    let path = root.join("zircon-project.toml");
    let original = ProjectManifest::new(
        "Original Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        4,
    );
    original.save(&path).unwrap();

    let replacement = ProjectManifest::new(
        "Replacement Sandbox",
        AssetUri::parse("res://scenes/replacement.scene.toml").unwrap(),
        5,
    );
    let error = replacement
        .save_with_atomic_fault(&path, AtomicWriteFault::ReplaceAfterBackup)
        .unwrap_err();

    assert!(matches!(error, ProjectManifestError::Write { .. }));
    assert_eq!(ProjectManifest::load(&path).unwrap(), original);
    assert_eq!(fs::read_dir(&root).unwrap().count(), 1);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn runtime_projection_matches_interface_summary_for_the_same_manifest_text() {
    let source = fs::read_to_string(shared_manifest_fixture("v3")).unwrap();

    let runtime = ProjectManifest::from_toml_str(&source).unwrap();
    let interface = ProjectManifestSummary::parse_toml_str(&source).unwrap();

    assert_eq!(runtime.value.summary(), interface.value);
    assert_eq!(runtime.migrated_from, interface.migrated_from);
}

#[test]
fn runtime_rejects_shared_invalid_engine_version_requirement_with_typed_source() {
    let error = ProjectManifest::load(shared_manifest_fixture("invalid")).unwrap_err();
    assert!(matches!(
        error,
        ProjectManifestError::Summary(
            ProjectManifestSummaryError::InvalidEngineVersionReq { value, .. }
        ) if value == "not a semver requirement"
    ));
}

fn shared_manifest_fixture(version: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("serialization")
        .join("project-manifest")
        .join(version)
        .join("zircon-project.toml")
}

#[test]
fn project_manifest_roundtrip_preserves_asset_manifest_path() {
    let root = unique_temp_project_root("manifest_asset_manifest");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let mut manifest = ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        3,
    );
    manifest.asset_manifest = Some("export/assets.json".to_string());
    manifest.save(paths.manifest_path()).unwrap();

    let loaded = ProjectManifest::load(paths.manifest_path()).unwrap();

    assert_eq!(loaded.asset_manifest.as_deref(), Some("export/assets.json"));
    assert_eq!(loaded, manifest);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manifest_roundtrip_preserves_plugins_and_export_profiles() {
    let root = unique_temp_project_root("manifest_plugins");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let mut manifest = ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        3,
    );
    manifest.plugins.set_enabled(
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
            .with_runtime_crate("zircon_plugin_sound_runtime"),
    );
    manifest.export_profiles.push(
        ExportProfile::new(
            "client",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Client3d,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate)
        .with_strategy(ExportPackagingStrategy::LibraryEmbed),
    );
    manifest.export_profiles.push(
        ExportProfile::new(
            "server",
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Headless,
            RuntimeProfileId::Server,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate),
    );

    manifest.save(paths.manifest_path()).unwrap();
    let loaded = ProjectManifest::load(paths.manifest_path()).unwrap();

    assert_eq!(loaded, manifest);

    let client = ExportBuildPlan::from_project_manifest(&loaded, "client").unwrap();
    assert_eq!(client.profile.name, "client");
    assert_eq!(
        client.profile.runtime_profile_id,
        Some(RuntimeProfileId::Client3d)
    );
    assert!(client
        .linked_runtime_crates
        .contains(&"zircon_plugin_sound_runtime".to_string()));
    assert!(client
        .generated_files
        .iter()
        .any(|file| file.path == "src/main.rs"));

    let server = ExportBuildPlan::from_project_manifest(&loaded, "server").unwrap();
    assert_eq!(server.profile.target_mode, RuntimeTargetMode::ServerRuntime);
    assert_eq!(
        server.profile.target_platform,
        ExportTargetPlatform::Headless
    );
    assert!(server
        .generated_files
        .iter()
        .any(|file| file.path == "Cargo.toml"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manifest_roundtrip_preserves_script_package_roots() {
    let root = unique_temp_project_root("manifest_scripts");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .unwrap();

    let mut manifest = ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        3,
    );
    manifest.scripts = ProjectScriptManifest {
        package_roots: vec!["scripts".to_string()],
        startup_packages: vec!["vampire_game".to_string()],
    };

    manifest.save(paths.manifest_path()).unwrap();
    let loaded = ProjectManifest::load(paths.manifest_path()).unwrap();

    assert_eq!(loaded.scripts.package_roots, ["scripts"]);
    assert_eq!(loaded.scripts.startup_packages, ["vampire_game"]);
    assert_eq!(loaded, manifest);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn export_profile_deserialization_preserves_missing_runtime_profile_id_for_validation() {
    let source = r#"
name = "Sandbox"
format_version = 3
project_guid = "0a347ab5-6dd3-4378-b1f0-00d9b3e48d96"
default_scene = "res://scenes/main.scene.toml"
library_version = 3

[[export_profiles]]
name = "client"
target_mode = "client_runtime"
target_platform = "windows"
strategies = ["source_template"]
output_name = "client"
"#;

    let manifest = ProjectManifest::from_toml_str(source).unwrap().value;

    assert_eq!(manifest.export_profiles.len(), 1);
    assert_eq!(manifest.export_profiles[0].runtime_profile_id, None);
    assert!(manifest.scripts.is_empty());

    let plan = ExportBuildPlan::from_project_manifest(&manifest, "client").unwrap();
    let expected = "export profile \"client\" must declare runtime_profile_id explicitly";

    assert!(plan.has_fatal_diagnostics());
    assert!(plan
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic == expected));
    assert!(plan
        .fatal_diagnostics
        .iter()
        .any(|diagnostic| diagnostic == expected));
    assert_eq!(plan.runtime_plugin_availability, Default::default());
}

#[test]
fn export_profile_map_table_parses_planned_profile_asset_fields() {
    let source = r#"
name = "Sandbox"
format_version = 3
project_guid = "7f1f12f3-d3c7-499d-8c8d-cb9809a3cdcf"
default_scene = "res://scenes/main.scene.toml"
library_version = 3

[export_profiles.windows-release]
runtime_profile_id = "client2d"
platform = "windows-x86_64"
path = "library_embed"
mode = "release"
plugins = ["sound", "net"]
features = { sound = ["timeline_animation_track"], net = ["http", "websocket"] }
asset_filter = "shipping"
"#;

    let manifest = ProjectManifest::from_toml_str(source).unwrap().value;

    assert_eq!(manifest.export_profiles.len(), 1);
    let profile = &manifest.export_profiles[0];
    assert_eq!(profile.name, "windows-release");
    assert_eq!(profile.target_mode, RuntimeTargetMode::ClientRuntime);
    assert_eq!(profile.runtime_profile_id, Some(RuntimeProfileId::Client2d));
    assert_eq!(profile.target_platform, ExportTargetPlatform::Windows);
    assert_eq!(
        profile.strategies,
        vec![ExportPackagingStrategy::LibraryEmbed]
    );
    assert_eq!(profile.build_mode, ExportBuildMode::Release);
    assert_eq!(profile.selected_plugins, ["sound", "net"]);
    assert_eq!(profile.asset_filter.as_deref(), Some("shipping"));
    assert_eq!(
        profile.features.get("sound"),
        Some(&vec!["timeline_animation_track".to_string()])
    );
    assert_eq!(
        profile.features.get("net"),
        Some(&vec!["http".to_string(), "websocket".to_string()])
    );
}
