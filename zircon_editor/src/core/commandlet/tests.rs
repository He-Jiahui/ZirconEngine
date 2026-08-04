use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use zircon_runtime::asset::project::AssetMetaDocument;
use zircon_runtime::asset::{AssetKind, AssetUri, AssetUuid};

use super::{
    CommandletExitCode, CommandletStatus, parse_commandlet_args, run_commandlet,
    run_commandlet_with_capabilities,
};
use crate::core::commands::{EditorCommandAction, EditorCommandRegistry};
use crate::core::plugin::EditorPluginManager;

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

#[test]
fn migrate_assets_is_registered_once_as_a_remote_callable_commandlet() {
    let registry = EditorCommandRegistry::default_workbench();
    let descriptor = registry
        .command("asset.migration.migrate_assets")
        .expect("migrate-assets commandlet should be registered in the editor command registry");

    assert!(descriptor.callable_from_remote());
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("editor.commandlet.migrate-assets")
    );
    assert_eq!(
        descriptor.required_capabilities(),
        &["asset.migration".to_owned()]
    );
    assert_eq!(
        descriptor
            .headless_commandlet_route()
            .map(crate::core::editor_operation::EditorOperationPath::as_str),
        Some("commandlet.route.migrate_assets")
    );
    assert_eq!(
        descriptor.headless_commandlet_name(),
        Some("migrate-assets")
    );
    assert_eq!(
        registry
            .command_for_headless_commandlet_name("migrate-assets")
            .map(crate::core::commands::EditorCommandDescriptor::id),
        Some(descriptor.id())
    );
    assert_eq!(
        descriptor.action(),
        &EditorCommandAction::HeadlessAssetMigration
    );
}

#[test]
fn plugin_list_is_registered_once_as_a_remote_callable_commandlet() {
    let registry = EditorCommandRegistry::default_workbench();
    let descriptor = registry
        .command("plugin.catalog.list")
        .expect("plugin-list commandlet should be registered in the editor command registry");

    assert!(descriptor.callable_from_remote());
    assert_eq!(
        descriptor.payload_schema_id(),
        Some("editor.commandlet.plugin-list")
    );
    assert_eq!(
        descriptor.required_capabilities(),
        &["plugin.catalog.read".to_owned()]
    );
    assert_eq!(
        descriptor
            .headless_commandlet_route()
            .map(crate::core::editor_operation::EditorOperationPath::as_str),
        Some("commandlet.route.plugin_list")
    );
    assert_eq!(descriptor.headless_commandlet_name(), Some("plugin-list"));
    assert_eq!(
        registry
            .command_for_headless_commandlet_name("plugin-list")
            .map(crate::core::commands::EditorCommandDescriptor::id),
        Some(descriptor.id())
    );
    assert_eq!(
        descriptor.action(),
        &EditorCommandAction::HeadlessPluginList
    );
}

#[test]
fn plugin_list_projects_the_existing_catalog_with_stable_json() {
    let first = run_commandlet(parse_request(["--run", "plugin-list"]));
    let second = run_commandlet(parse_request(["--run", "plugin-list"]));

    assert_eq!(first.exit_code(), CommandletExitCode::Success);
    assert_eq!(first.status(), CommandletStatus::Succeeded);
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap()
    );
    let plugins = first
        .plugins()
        .expect("plugin-list should return the canonical editor plugin catalog");
    assert!(!plugins.entries().is_empty());
    assert!(
        plugins
            .entries()
            .windows(2)
            .all(|pair| pair[0].package_id <= pair[1].package_id)
    );
}

#[test]
fn plugin_list_reuses_the_canonical_catalog_projection_without_rebuild() {
    let catalog = EditorPluginManager::builtin_shared()
        .expect("builtin editor plugin catalog should be admissible")
        .catalog_snapshot();
    let first = run_commandlet(parse_request(["--run", "plugin-list"]));
    let second = run_commandlet(parse_request(["--run", "plugin-list"]));

    assert!(Arc::ptr_eq(
        first
            .plugin_catalog_projection()
            .expect("first plugin-list report should retain the canonical projection"),
        second
            .plugin_catalog_projection()
            .expect("second plugin-list report should retain the canonical projection"),
    ));
    assert!(Arc::ptr_eq(
        first
            .plugin_catalog_projection()
            .expect("plugin-list should retain the shared catalog projection"),
        catalog.projection(),
    ));
}

#[test]
fn plugin_list_reports_missing_catalog_capability() {
    let report = run_commandlet_with_capabilities(
        parse_request(["--run", "plugin-list"]),
        std::iter::empty::<String>(),
    );

    assert_eq!(report.exit_code(), CommandletExitCode::MissingCapability);
    assert_eq!(report.status(), CommandletStatus::MissingCapabilities);
    assert!(
        report
            .error()
            .is_some_and(|error| error.contains("plugin.catalog.read"))
    );
}

#[test]
fn migrate_assets_dry_run_reports_success_without_writing() {
    let root = fixture_root("dry-run");
    let material = write_migratable_fixture(&root);
    let before = fs::read(&material).unwrap();
    let request = parse_request([
        "--run",
        "migrate-assets",
        "--project",
        root.to_str().unwrap(),
        "--dry-run",
    ]);

    let report = run_commandlet(request);

    assert_eq!(report.exit_code(), CommandletExitCode::Success);
    assert_eq!(report.status(), CommandletStatus::Succeeded);
    assert!(report.migration().is_some());
    assert_eq!(fs::read(&material).unwrap(), before);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn migrate_assets_apply_writes_the_runtime_migration_result() {
    let root = fixture_root("apply");
    let material = write_migratable_fixture(&root);
    let request = parse_request([
        "--run",
        "migrate-assets",
        "--project",
        root.to_str().unwrap(),
        "--apply",
    ]);

    let report = run_commandlet(request);

    assert_eq!(report.exit_code(), CommandletExitCode::Success);
    assert!(
        report
            .migration()
            .is_some_and(|migration| migration.applied)
    );
    let migrated = fs::read_to_string(&material).unwrap();
    assert!(migrated.contains("guid"));
    assert!(!migrated.contains("uuid"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn commandlet_parser_reports_unknown_commands_and_mutually_exclusive_modes() {
    for args in [
        vec!["--run", "unknown", "--project", "fixture", "--dry-run"],
        vec![
            "--run",
            "migrate-assets",
            "--project",
            "fixture",
            "--dry-run",
            "--apply",
        ],
    ] {
        let report = parse_commandlet_args(args).unwrap_err();

        assert_eq!(report.exit_code(), CommandletExitCode::InvalidArguments);
        assert_eq!(report.status(), CommandletStatus::InvalidArguments);
        assert!(report.error().is_some());
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["exit_code"], 2);
        assert_eq!(json["status"], "invalid_arguments");
    }
}

#[test]
fn commandlet_reports_missing_capability_without_starting_a_ui_host() {
    let root = fixture_root("missing-capability");
    write_migratable_fixture(&root);
    let request = parse_request([
        "--run",
        "migrate-assets",
        "--project",
        root.to_str().unwrap(),
        "--dry-run",
    ]);

    let report = run_commandlet_with_capabilities(request, std::iter::empty::<String>());

    assert_eq!(report.exit_code(), CommandletExitCode::MissingCapability);
    assert_eq!(report.status(), CommandletStatus::MissingCapabilities);
    assert!(
        report
            .error()
            .is_some_and(|error| error.contains("asset.migration"))
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn commandlet_maps_runtime_errors_to_failure_json() {
    let root = fixture_root("runtime-error");
    let request = parse_request([
        "--run",
        "migrate-assets",
        "--project",
        root.to_str().unwrap(),
        "--dry-run",
    ]);

    let report = run_commandlet(request);

    assert_eq!(report.exit_code(), CommandletExitCode::Failed);
    assert_eq!(report.status(), CommandletStatus::Failed);
    assert!(
        report
            .error()
            .is_some_and(|error| error.contains("zircon-project.toml"))
    );
    fs::remove_dir_all(root).unwrap();
}

fn parse_request<const N: usize>(args: [&str; N]) -> super::CommandletRequest {
    parse_commandlet_args(args)
        .expect("commandlet arguments should parse")
        .expect("--run should produce a commandlet request")
}

fn fixture_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "zircon_editor_commandlet_{label}_{}_{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn write_migratable_fixture(root: &Path) -> PathBuf {
    fs::write(
        root.join("zircon-project.toml"),
        "name = \"Commandlet Fixture\"\nformat_version = 2\ndefault_scene = \"res://scenes/main.scene.toml\"\nasset_roots = [\"assets\"]\nlibrary_version = 1\n",
    )
    .unwrap();
    let shader = root.join("assets/shaders/pbr.zshader");
    fs::create_dir_all(shader.parent().unwrap()).unwrap();
    fs::write(&shader, b"fixture shader").unwrap();
    let guid: AssetUuid = "11111111-2222-4333-8444-555555555555".parse().unwrap();
    AssetMetaDocument::new(
        guid,
        AssetUri::parse("res://shaders/pbr.zshader").unwrap(),
        AssetKind::Shader,
    )
    .save(shader.with_file_name("pbr.zshader.zmeta"))
    .unwrap();

    let material = root.join("assets/materials/hero.zmaterial");
    fs::create_dir_all(material.parent().unwrap()).unwrap();
    fs::write(
        &material,
        format!(
            "version = 2\nname = \"Hero\"\n\n[shader]\nuuid = \"{guid}\"\nurl = \"res://shaders/pbr.zshader\"\n"
        ),
    )
    .unwrap();
    material
}
