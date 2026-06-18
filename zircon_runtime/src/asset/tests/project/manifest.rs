use std::fs;

use crate::asset::project::{ProjectManifest, ProjectPaths, ProjectScriptManifest};
use crate::asset::AssetUri;
use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::{
    plugin::ExportBuildMode, plugin::ExportBuildPlan, plugin::ExportPackagingStrategy,
    plugin::ExportProfile, plugin::ExportTargetPlatform, plugin::ProjectPluginSelection,
    plugin::RuntimeProfileId,
};

use super::unique_temp_project_root;

#[test]
fn project_manifest_roundtrip_preserves_default_scene_and_paths() {
    let root = unique_temp_project_root("manifest");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

    let manifest = ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        3,
    );
    manifest.save(paths.manifest_path()).unwrap();

    let loaded = ProjectManifest::load(paths.manifest_path()).unwrap();

    assert_eq!(loaded, manifest);
    assert!(paths.assets_root().is_dir());
    assert!(paths.library_root().is_dir());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn project_manifest_roundtrip_preserves_plugins_and_export_profiles() {
    let root = unique_temp_project_root("manifest_plugins");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();

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
        )
        .with_runtime_profile_id(RuntimeProfileId::Client3d)
        .with_strategy(ExportPackagingStrategy::SourceTemplate)
        .with_strategy(ExportPackagingStrategy::LibraryEmbed),
    );
    manifest.export_profiles.push(
        ExportProfile::new(
            "server",
            RuntimeTargetMode::ServerRuntime,
            ExportTargetPlatform::Headless,
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
    paths.ensure_layout().unwrap();

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
fn export_profile_runtime_profile_id_is_backward_compatible() {
    let source = r#"
name = "Sandbox"
default_scene = "res://scenes/main.scene.toml"
schema_version = 3

[[export_profiles]]
name = "client"
target_mode = "client_runtime"
target_platform = "windows"
strategies = ["source_template"]
output_name = "client"
"#;

    let manifest: ProjectManifest = toml::from_str(source).unwrap();

    assert_eq!(manifest.export_profiles.len(), 1);
    assert_eq!(manifest.export_profiles[0].runtime_profile_id, None);
    assert!(manifest.scripts.is_empty());
}

#[test]
fn export_profile_map_table_parses_planned_profile_asset_fields() {
    let source = r#"
name = "Sandbox"
default_scene = "res://scenes/main.scene.toml"
schema_version = 3

[export_profiles.windows-release]
platform = "windows-x86_64"
path = "library_embed"
mode = "release"
plugins = ["sound", "net"]
features = { sound = ["timeline_animation_track"], net = ["http", "websocket"] }
asset_filter = "shipping"
"#;

    let manifest: ProjectManifest = toml::from_str(source).unwrap();

    assert_eq!(manifest.export_profiles.len(), 1);
    let profile = &manifest.export_profiles[0];
    assert_eq!(profile.name, "windows-release");
    assert_eq!(profile.target_mode, RuntimeTargetMode::ClientRuntime);
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
