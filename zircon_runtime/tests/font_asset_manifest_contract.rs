#[path = "../src/graphics/scene/scene_renderer/ui/font_asset.rs"]
mod font_asset;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use font_asset::load_ui_font_manifest;
use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
use zircon_runtime::asset::{AssetManager, AssetUri, ProjectAssetManager};
use zircon_runtime_interface::ui::surface::UiTextRenderMode;

mod asset {
    pub use zircon_runtime::asset::*;
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
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("temp dir should be created");
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

        let resolved = std::fs::canonicalize(
            manifest_path
                .parent()
                .expect("font manifest should live in a folder")
                .join(&source_path),
        )
        .expect("font source should resolve on disk");
        let canonical_assets_root =
            std::fs::canonicalize(&assets_root).expect("assets root should resolve on disk");

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

    let loaded = load_ui_font_manifest(
        manifest_path
            .to_str()
            .expect("manifest path should convert to utf-8"),
    )
    .expect("relative in-scope source should load");

    assert_eq!(loaded.source_path, local_font);
    assert_eq!(loaded.family.as_deref(), Some("Test Family"));
    assert_eq!(loaded.render_mode, Some(UiTextRenderMode::Sdf));
}

#[test]
fn res_font_manifest_rejects_source_paths_that_escape_runtime_assets_root() {
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
    let outside_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(&outside_name);
    let _manifest_guard = TempFileGuard::new(manifest_path.clone());
    let _outside_guard = TempFileGuard::new(outside_path.clone());
    fs::copy(default_font_path(), &outside_path).expect("escape target font should exist");
    write_manifest(&manifest_path, &format!("../../{outside_name}"));

    let loaded = load_ui_font_manifest(&format!("res://fonts/{manifest_name}"));

    assert!(
        loaded.is_none(),
        "res font manifests should reject source paths that escape the runtime assets root"
    );
}

#[test]
fn font_manifest_rejects_absolute_source_paths() {
    let temp = TempDirGuard::new("zircon-font-manifest-absolute");
    let manifest_path = temp.path.join("absolute.font.toml");
    let absolute_font = default_font_path();
    write_manifest(&manifest_path, &absolute_font.to_string_lossy());

    let loaded = load_ui_font_manifest(
        manifest_path
            .to_str()
            .expect("manifest path should convert to utf-8"),
    );

    assert!(
        loaded.is_none(),
        "font manifests should reject absolute source paths and keep source resolution scoped"
    );
}

#[test]
fn project_font_manifest_resolves_through_project_asset_manager() {
    let temp = TempDirGuard::new("zircon-project-font-manifest");
    let paths = ProjectPaths::from_root(&temp.path).expect("project paths should build");
    paths.ensure_layout().expect("project layout should exist");
    ProjectManifest::new(
        "FontSandbox",
        AssetUri::parse("res://fonts/project.font.toml").expect("startup uri should parse"),
        1,
    )
    .save(paths.manifest_path())
    .expect("project manifest should save");

    let font_dir = paths.assets_root().join("fonts");
    fs::create_dir_all(&font_dir).expect("font dir should exist");
    let project_font = font_dir.join("project.ttf");
    fs::copy(default_font_path(), &project_font).expect("font fixture should copy");
    let project_manifest = font_dir.join("project.font.toml");
    write_manifest(&project_manifest, "project.ttf");

    let manager = ProjectAssetManager::default();
    manager
        .open_project(
            temp.path
                .to_str()
                .expect("project root should convert to utf-8"),
        )
        .expect("project should open");

    let loaded = font_asset::load_ui_font_manifest_with_asset_manager(
        "res://fonts/project.font.toml",
        Some(&manager),
    )
    .expect("project font manifest should resolve through the active project asset manager");

    assert_eq!(loaded.source_path, project_font);
    assert_eq!(loaded.family.as_deref(), Some("Test Family"));
    assert_eq!(loaded.render_mode, Some(UiTextRenderMode::Sdf));
}
