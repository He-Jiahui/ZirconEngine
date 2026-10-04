use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::migration::{
    migrate_project_assets, AssetMigrationIssueKind, AssetMigrationMode, AssetMigrationOptions,
    AssetMigrationReport,
};
use crate::asset::project::{AssetMetaDocument, ProjectManifest, ProjectPaths};
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{AssetKind, AssetUri, AssetUuid};

#[test]
fn unreadable_utf8_document_is_not_counted_as_a_parse() {
    let root = fixture("migration-metrics-utf8");
    let source = root.join("assets/broken.zmaterial");
    let bytes = [0xff, 0xfe, 0xff];
    fs::write(&source, bytes).unwrap();

    let report = dry_run(&root);
    assert_eq!(report.scanned_files(), 1);
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::InvalidDocument
    );
    assert_eq!(report.metrics().document_reads(), 1);
    assert_eq!(report.metrics().document_parses(), 0);
    assert_eq!(report.metrics().reference_visits(), 0);
    assert_eq!(fs::read(source).unwrap(), bytes);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_toml_document_records_the_attempted_parse() {
    let root = fixture("migration-metrics-toml");
    fs::write(root.join("assets/broken.zmaterial"), "[unfinished\n").unwrap();

    let report = dry_run(&root);
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::InvalidDocument
    );
    assert_eq!(report.metrics().document_reads(), 1);
    assert_eq!(report.metrics().document_parses(), 1);
    assert_eq!(report.metrics().reference_visits(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejected_retired_reference_retains_its_resolver_visit() {
    let root = fixture("migration-metrics-retired-reference");
    let missing = AssetUuid::new();
    fs::write(
        root.join("assets/broken.zmaterial"),
        format!("version = 2\n\n[shader]\nuuid = \"{missing}\"\nurl = \"res://missing.zshader\"\n"),
    )
    .unwrap();

    let report = dry_run(&root);
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::DanglingReference
    );
    assert_eq!(report.metrics().reference_visits(), 1);
    assert_eq!(report.metrics().resolver_index_lookups(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn later_reference_failure_preserves_all_preceding_document_visits() {
    let root = fixture("migration-metrics-current-reference");
    let shader = root.join("assets/registered.zshader");
    fs::write(&shader, "fixture shader").unwrap();
    let shader_guid = AssetUuid::new();
    AssetMetaDocument::new(
        shader_guid,
        AssetUri::parse("res://registered.zshader").unwrap(),
        AssetKind::Shader,
    )
    .save(shader.with_extension("zshader.zmeta"))
    .unwrap();
    let missing = AssetUuid::new();
    let material = root.join("assets/broken.zmaterial");
    let bytes = format!(
        "version = 2\n\n[shader]\nkind = \"project\"\nguid = \"{shader_guid}\"\npath_hint = \"assets/registered.zshader\"\n\n[textures.albedo]\nkind = \"project\"\nguid = \"{missing}\"\npath_hint = \"assets/missing.ztexture\"\n"
    );
    fs::write(&material, &bytes).unwrap();

    let report = dry_run(&root);
    assert_eq!(
        report.issues()[0].kind(),
        AssetMigrationIssueKind::DanglingReference
    );
    assert_eq!(report.metrics().document_reads(), 1);
    assert_eq!(report.metrics().document_parses(), 1);
    assert_eq!(report.metrics().reference_visits(), 2);
    assert_eq!(fs::read_to_string(material).unwrap(), bytes);
    fs::remove_dir_all(root).unwrap();
}

fn fixture(label: &str) -> PathBuf {
    let root = unique_temp_project_root(label);
    fs::create_dir_all(root.join("assets")).unwrap();
    let paths = ProjectPaths::from_root(&root).unwrap();
    ProjectManifest::new(
        "MigrationMetricErrors",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    root
}

fn dry_run(root: &Path) -> AssetMigrationReport {
    migrate_project_assets(AssetMigrationOptions::new(root, AssetMigrationMode::DryRun)).unwrap()
}
