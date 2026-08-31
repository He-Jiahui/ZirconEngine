use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::{AssetManager, AssetUri, ProjectAssetManager};
use crate::text::font::FontDatabase;

const TEXT_FONT_MANIFEST_WORK_DIRECTORY: &str = ".runtime_text_font_manifest_work";

#[test]
fn runtime_font_manifests_under_assets_stay_inside_runtime_assets_root() {
    let assets_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let fonts_root = assets_root.join("fonts");
    let entries = fs::read_dir(&fonts_root).expect("fonts assets directory should exist");

    for entry in entries.flatten() {
        let manifest_path = entry.path();
        if !manifest_path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with(".font.toml"))
        {
            continue;
        }

        let manifest = fs::read_to_string(&manifest_path).expect("font manifest should read");
        let manifest: toml::Value = toml::from_str(&manifest).expect("font manifest should parse");
        let source = manifest
            .get("source")
            .and_then(toml::Value::as_str)
            .expect("font manifest should declare a source");
        let source_path = PathBuf::from(source);
        assert!(
            !source_path.is_absolute(),
            "font manifest {:?} should not use an absolute source path",
            manifest_path
        );

        let resolved = fs::canonicalize(
            manifest_path
                .parent()
                .expect("font manifest should live in a folder")
                .join(&source_path),
        )
        .expect("font source should resolve on disk");
        let canonical_assets_root =
            fs::canonicalize(&assets_root).expect("assets root should resolve on disk");

        assert!(
            resolved.starts_with(&canonical_assets_root),
            "font manifest {:?} should keep its source inside runtime assets; got {:?}",
            manifest_path,
            resolved
        );
    }
}

#[test]
fn font_manifest_keeps_relative_source_paths_inside_allowed_root() {
    let temp = TempDirGuard::new("zircon-font-manifest-allow");
    let manifest_path = temp.path.join("allowed.font.toml");
    let local_font = temp.path.join("local.ttf");
    fs::copy(default_font_path(), &local_font).expect("font fixture should copy");
    write_manifest(&manifest_path, "local.ttf");

    let loaded = load_text_font_source(
        manifest_path
            .to_str()
            .expect("manifest path should convert to utf-8"),
        None,
    )
    .expect("relative in-scope source should load");

    assert_eq!(loaded.source_path, local_font);
    assert_eq!(loaded.family.as_deref(), Some("Test Family"));
}

#[test]
fn built_in_font_manifest_path_uses_the_selected_runtime_asset_root() {
    let product_asset_root = Path::new("E:/portable-product/assets");

    let resolved = resolve_font_asset_path_with("res://fonts/default.font.toml", |relative| {
        product_asset_root.join(relative)
    });

    assert_eq!(
        resolved,
        Some(product_asset_root.join("fonts/default.font.toml"))
    );
}

#[test]
fn production_res_font_manifest_path_uses_the_runtime_asset_resolver() {
    let asset_ref = "res://fonts/default.font.toml";

    assert_eq!(
        resolve_font_asset_path(asset_ref),
        Some(crate::asset::runtime_asset_path("fonts/default.font.toml"))
    );
}

#[test]
fn res_font_manifest_rejects_source_paths_that_escape_runtime_assets_root() {
    let temp = TempDirGuard::new("zircon-font-manifest-escape");
    let unique = format!(
        "codex-escape-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos()
    );
    let manifest_name = format!("{unique}.font.toml");
    let outside_name = format!("{unique}.ttf");
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fonts")
        .join(&manifest_name);
    let outside_path = temp.path.join(&outside_name);
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime manifest must have a workspace parent");
    let outside_from_assets = Path::new("..").join("..").join("..").join(
        outside_path
            .strip_prefix(workspace_root)
            .expect("workspace font sandbox should stay under the workspace"),
    );
    let _manifest_guard = TempFileGuard::new(manifest_path.clone());
    let _outside_guard = TempFileGuard::new(outside_path.clone());
    fs::copy(default_font_path(), &outside_path).expect("escape target font should exist");
    write_manifest(&manifest_path, &outside_from_assets.to_string_lossy());

    let loaded = load_text_font_source(&format!("res://fonts/{manifest_name}"), None);

    assert!(matches!(
        loaded,
        Err(FontLoadError::SourceOutsideAllowedRoot)
    ));
}

#[test]
fn font_manifest_rejects_absolute_source_paths() {
    let temp = TempDirGuard::new("zircon-font-manifest-absolute");
    let manifest_path = temp.path.join("absolute.font.toml");
    let absolute_font = default_font_path();
    write_manifest(&manifest_path, &absolute_font.to_string_lossy());

    let loaded = load_text_font_source(
        manifest_path
            .to_str()
            .expect("manifest path should convert to utf-8"),
        None,
    );

    assert!(matches!(loaded, Err(FontLoadError::AbsoluteManifestSource)));
}

#[test]
fn font_manifest_reports_parse_failure_instead_of_missing_source() {
    let temp = TempDirGuard::new("zircon-font-manifest-parse");
    let manifest_path = temp.path.join("malformed.font.toml");
    fs::write(&manifest_path, "source = [").expect("malformed manifest should write");

    let loaded = load_text_font_source(
        manifest_path
            .to_str()
            .expect("manifest path should convert to utf-8"),
        None,
    );

    assert!(matches!(loaded, Err(FontLoadError::ManifestParseFailed)));
}

#[test]
fn font_manifest_reports_missing_source_with_a_stable_io_cause() {
    let temp = TempDirGuard::new("zircon-font-manifest-missing-source");
    let manifest_path = temp.path.join("missing-source.font.toml");
    write_manifest(&manifest_path, "missing.ttf");

    let loaded = load_text_font_source(
        manifest_path
            .to_str()
            .expect("manifest path should convert to utf-8"),
        None,
    );

    assert!(matches!(
        loaded,
        Err(FontLoadError::ManifestSourceUnavailable(
            FontLoadIoFailure::NotFound
        ))
    ));
}

#[test]
fn project_font_manifest_resolves_through_project_asset_manager() {
    let temp = TempDirGuard::new("zircon-project-font-manifest");
    let paths = ProjectPaths::from_root(&temp.path).expect("project paths should build");
    paths
        .ensure_layout(&[zircon_runtime_interface::project::RelPath::project_assets()])
        .expect("project layout should exist");
    ProjectManifest::new(
        "FontSandbox",
        AssetUri::parse("res://fonts/project.font.toml").expect("startup uri should parse"),
        1,
    )
    .save(paths.manifest_path())
    .expect("project manifest should save");

    let font_dir = paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("fonts");
    fs::create_dir_all(&font_dir).expect("font dir should exist");
    let project_font = font_dir.join("project.ttf");
    fs::copy(default_font_path(), &project_font).expect("font fixture should copy");
    let project_manifest = font_dir.join("project.font.toml");
    write_manifest(&project_manifest, "project.ttf");

    let expected_cooked_bytes = fs::read(&project_font).expect("project font bytes should exist");
    let expected_asset_uuid = {
        let manager = ProjectAssetManager::default();
        manager
            .open_project(
                temp.path
                    .to_str()
                    .expect("project root should convert to utf-8"),
            )
            .expect("project should import the source font into its artifact cache");
        manager
            .current_project_manager()
            .and_then(|project| {
                project
                    .asset_registry()
                    .entry_by_path(&AssetUri::parse("res://fonts/project.font.toml").unwrap())
                    .map(|entry| entry.uuid())
            })
            .expect("opened project font should retain a registry UUID")
    };
    fs::remove_file(&project_font).expect("source font should be removable after import");

    let restarted = ProjectAssetManager::default();
    restarted
        .open_project(
            temp.path
                .to_str()
                .expect("project root should convert to utf-8"),
        )
        .expect("project should restore the font manifest from its persisted artifact cache");
    let loaded = load_text_font_source("res://fonts/project.font.toml", Some(&restarted))
        .expect("project font manifest should resolve through the restarted asset manager");

    assert_eq!(loaded.family.as_deref(), Some("Test Family"));
    assert_eq!(loaded.asset_uuid, Some(expected_asset_uuid));
    let cooked_blob = loaded
        .cooked_blob
        .expect("project font resolution must use its cooked payload after source removal");
    assert_eq!(cooked_blob.bytes(), expected_cooked_bytes);
    assert!(cooked_blob.has_valid_content_hash());

    let asset = loaded
        .asset
        .as_ref()
        .expect("project font resolution should retain the deserialized font asset metadata");
    let owner = "res://fonts/project.font.toml";
    let mut font_database = FontDatabase::default();
    let registration = font_database
        .replace_font_asset_blob(owner, asset, &loaded.source_path, &cooked_blob)
        .expect("the restarted runtime should register its deserialized cooked font blob");
    assert!(
        !registration.faces.is_empty(),
        "a cooked font blob should produce at least one renderable face"
    );
    assert!(
        font_database.font_asset_primary_face(owner).is_some(),
        "the restarted runtime should resolve the registered cooked font face"
    );
}

fn write_manifest(path: &Path, source: &str) {
    fs::write(
        path,
        format!("source = {source:?}\nfamily = \"Test Family\"\nrender_mode = \"sdf\"\n"),
    )
    .expect("manifest should be written");
}

fn default_font_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fonts")
        .join("FiraMono-subset.ttf")
}

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        );
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("zircon_runtime manifest must have a workspace parent")
            .join("docs")
            .join("tests")
            .join("runtime")
            .join("text")
            .join(TEXT_FONT_MANIFEST_WORK_DIRECTORY);
        fs::create_dir_all(&root).expect("workspace font manifest directory should exist");
        let path = root.join(unique);
        fs::create_dir_all(&path).expect("workspace font manifest sandbox should be created");
        Self { path }
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct TempFileGuard {
    path: PathBuf,
}

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
