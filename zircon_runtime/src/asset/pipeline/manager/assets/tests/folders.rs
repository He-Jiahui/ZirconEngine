use std::fs;
use std::path::PathBuf;

use crate::asset::project::{AssetMetaDocument, ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    AssetManager, AssetUri, DataAsset, DataAssetFormat, FunctionAssetImporter, ImportedAsset,
};
use zircon_runtime_interface::project::RelPath;

use super::ProjectAssetManager;

struct Fixture {
    root: PathBuf,
    assets: PathBuf,
    manager: ProjectAssetManager,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "zircon_u09_{label}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
        ProjectManifest::new(
            "Folder mutation fixture",
            uri("res://scenes/default.scene.toml"),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();
        let assets = paths.asset_root(&RelPath::project_assets());
        fs::create_dir_all(assets.join("data/sub")).unwrap();
        fs::create_dir_all(assets.join("data/empty/nested")).unwrap();
        fs::write(assets.join("data/first.counted"), "first").unwrap();
        fs::write(assets.join("data/sub/second.counted"), "second").unwrap();
        let manager = ProjectAssetManager::default();
        manager.register_asset_importer(counted_importer()).unwrap();
        manager
            .open_prepared_project(ProjectManager::open(&root).unwrap())
            .unwrap();
        Self {
            root,
            assets,
            manager,
        }
    }

    fn folders(&self) -> Vec<AssetUri> {
        self.manager
            .current_project_manager()
            .unwrap()
            .project_folder_uris()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.manager.close_project();
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).unwrap()
}

#[test]
fn folder_create_rename_delete_preserves_empty_nested_directories_and_inventory() {
    let fixture = Fixture::new("empty_folder_ops");
    let generation = fixture
        .manager
        .current_project_generation_snapshot()
        .unwrap()
        .into_parts()
        .1;
    fixture
        .manager
        .create_project_folder(uri("res://new"))
        .unwrap();
    assert!(matches!(
        fixture.manager.check_project_generation(&generation),
        super::super::ProjectGenerationMatch::Superseded { .. }
    ));
    fixture
        .manager
        .create_project_folder(uri("res://new/nested"))
        .unwrap();
    assert!(fixture.folders().contains(&uri("res://new/nested")));
    let statuses = fixture
        .manager
        .relocate_project_folder(uri("res://new"), uri("res://renamed"))
        .expect("empty folder trees can be renamed");
    assert!(statuses.is_empty());
    assert!(!fixture.assets.join("new").exists());
    assert!(fixture.assets.join("renamed/nested").is_dir());
    assert!(fixture.folders().contains(&uri("res://renamed/nested")));
    fixture
        .manager
        .delete_project_folder(uri("res://renamed"))
        .expect_err("nonempty folder deletion never recurses");
    fixture
        .manager
        .delete_project_folder(uri("res://renamed/nested"))
        .unwrap();
    fixture
        .manager
        .delete_project_folder(uri("res://renamed"))
        .unwrap();
    assert!(!fixture.folders().contains(&uri("res://renamed")));
    assert!(
        AssetUri::parse("res://").is_err(),
        "root targets have no mutable AssetUri"
    );
}

#[test]
fn folder_relocation_commit_failure_rolls_back_all_authoring_files_and_registry() {
    use crate::core::resource::io::transaction::TransactionFault;
    let fixture = Fixture::new("folder_rename_rollback");
    let mut candidate = fixture.manager.current_project_manager().unwrap();
    fixture.manager.close_project().unwrap();
    let registry_path = candidate
        .paths()
        .registry_root()
        .join("asset-registry.json");
    let registry_before = fs::read(&registry_path).unwrap();
    let first_meta = fs::read(fixture.assets.join("data/first.counted.zmeta")).unwrap();
    let prepared = candidate
        .prepare_project_folder_relocation(uri("res://data"), uri("res://renamed"))
        .unwrap();
    assert!(prepared
        .commit_with_fault(TransactionFault::BeforeCommit(1))
        .is_err());
    assert_eq!(
        fs::read(fixture.assets.join("data/first.counted")).unwrap(),
        b"first"
    );
    assert_eq!(
        fs::read(fixture.assets.join("data/first.counted.zmeta")).unwrap(),
        first_meta
    );
    assert_eq!(
        fs::read(fixture.assets.join("data/sub/second.counted")).unwrap(),
        b"second"
    );
    assert_eq!(fs::read(&registry_path).unwrap(), registry_before);
    assert!(!fixture.assets.join("renamed").exists());
}

#[test]
fn folder_mutations_refuse_ambiguous_source_directories_and_secondary_root_targets() {
    let fixture = Fixture::new("folder_multi_root");
    fixture.manager.close_project().unwrap();
    let mut manifest = ProjectManifest::load(fixture.root.join("zircon-project.toml")).unwrap();
    manifest
        .asset_roots
        .push(RelPath::parse("shared-assets").unwrap());
    manifest
        .save(fixture.root.join("zircon-project.toml"))
        .unwrap();
    fs::create_dir_all(fixture.root.join("shared-assets/data")).unwrap();
    fs::create_dir_all(fixture.root.join("shared-assets/occupied")).unwrap();
    fixture
        .manager
        .open_prepared_project(ProjectManager::open(&fixture.root).unwrap())
        .unwrap();
    fixture
        .manager
        .delete_project_folder(uri("res://data"))
        .expect_err("source directories with multiple physical roots are ambiguous");
    fixture
        .manager
        .relocate_project_folder(uri("res://data"), uri("res://renamed"))
        .expect_err("folder rename cannot choose an arbitrary source root");
    fixture
        .manager
        .create_project_folder(uri("res://occupied"))
        .expect_err("a secondary-root directory occupies the same logical target");
    assert!(!fixture.assets.join("occupied").exists());
    let source = uri("res://data/first.counted");
    let uuid = fixture
        .manager
        .current_project_manager()
        .unwrap()
        .asset_registry()
        .entry_by_path(&source)
        .unwrap()
        .uuid();
    // These files are written after import so neither target exists in the registry.
    fs::write(
        fixture.root.join("shared-assets/late.raw"),
        b"keep secondary",
    )
    .unwrap();
    fs::write(
        fixture.root.join("shared-assets/orphan.raw.zmeta"),
        b"keep orphan",
    )
    .unwrap();
    fixture
        .manager
        .relocate_project_source(uuid, uri("res://late.raw"))
        .expect_err("a secondary-root untracked target still occupies the logical URI");
    fixture
        .manager
        .relocate_project_source(uuid, uri("res://orphan.raw"))
        .expect_err("a secondary-root orphan sidecar also occupies the destination");
    assert!(!fixture.assets.join("late.raw").exists());
    assert!(!fixture.assets.join("orphan.raw").exists());
}

#[test]
fn asset_and_folder_rename_preserve_manifest_ui_root_locators() {
    let fixture = Fixture::new("manifest_ui_root_rename");
    fixture.manager.close_project().unwrap();
    let mut manifest = ProjectManifest::load(fixture.root.join("zircon-project.toml")).unwrap();
    let source = uri("res://data/first.counted");
    manifest.ui_roots.push(source.clone());
    manifest
        .save(fixture.root.join("zircon-project.toml"))
        .unwrap();
    fixture
        .manager
        .open_prepared_project(ProjectManager::open(&fixture.root).unwrap())
        .unwrap();
    let uuid = fixture
        .manager
        .current_project_manager()
        .unwrap()
        .asset_registry()
        .entry_by_path(&source)
        .unwrap()
        .uuid();
    fixture
        .manager
        .relocate_project_source(uuid, uri("res://renamed.counted"))
        .expect_err("manifest UI root locators cannot silently become stale");
    fixture
        .manager
        .relocate_project_folder(uri("res://data"), uri("res://renamed"))
        .expect_err("folder relocation also preserves manifest UI roots");
    assert!(fixture.assets.join("data/first.counted").is_file());
    assert!(!fixture.assets.join("renamed").exists());
}

#[test]
fn folder_relocation_preserves_all_asset_identities_sidecars_and_source_path_index() {
    let fixture = Fixture::new("populated_folder_rename");
    let source_a = uri("res://data/first.counted");
    let source_b = uri("res://data/sub/second.counted");
    let old = fixture.manager.current_project_manager().unwrap();
    let uuid_a = old
        .asset_registry()
        .entry_by_path(&source_a)
        .unwrap()
        .uuid();
    let uuid_b = old
        .asset_registry()
        .entry_by_path(&source_b)
        .unwrap()
        .uuid();
    let target_a = uri("res://renamed/first.counted");
    let target_b = uri("res://renamed/sub/second.counted");
    let statuses = fixture
        .manager
        .relocate_project_folder(uri("res://data"), uri("res://renamed"))
        .expect("all registered descendants commit together");
    assert!(statuses
        .iter()
        .any(|status| status.uri == target_a.to_string()));
    assert!(statuses
        .iter()
        .any(|status| status.uri == target_b.to_string()));
    let project = fixture.manager.current_project_manager().unwrap();
    assert!(project.asset_registry().entry_by_path(&source_a).is_none());
    assert!(project.asset_registry().entry_by_path(&source_b).is_none());
    assert_eq!(
        project
            .asset_registry()
            .entry_by_path(&target_a)
            .unwrap()
            .uuid(),
        uuid_a
    );
    assert_eq!(
        project
            .asset_registry()
            .entry_by_path(&target_b)
            .unwrap()
            .uuid(),
        uuid_b
    );
    assert_eq!(
        fixture
            .manager
            .current_project_source_path(&target_a)
            .unwrap(),
        Some(fixture.assets.join("renamed/first.counted"))
    );
    assert!(matches!(
        fixture.manager.current_project_source_path(&source_a),
        Err(AssetImportError::MissingProjectAssetUri { uri }) if uri == source_a
    ));
    assert_eq!(
        AssetMetaDocument::load(fixture.assets.join("renamed/sub/second.counted.zmeta"))
            .unwrap()
            .url,
        target_b
    );
    assert_eq!(
        fs::read(fixture.assets.join("renamed/first.counted")).unwrap(),
        b"first"
    );
    assert!(fixture.assets.join("renamed/empty/nested").is_dir());
    assert!(!fixture.assets.join("data").exists());
}

#[test]
fn folder_delete_and_rename_refuse_untracked_files_and_occupied_or_overlapping_targets() {
    let fixture = Fixture::new("folder_refusals");
    fs::write(fixture.assets.join("data/unknown.raw"), "keep").unwrap();
    fixture
        .manager
        .delete_project_folder(uri("res://data"))
        .expect_err("deletion refuses both imported and unknown contents");
    let error = fixture
        .manager
        .relocate_project_folder(uri("res://data"), uri("res://renamed"))
        .unwrap_err();
    assert!(error.to_string().contains("untracked or unsupported file"));
    assert_eq!(
        fs::read(fixture.assets.join("data/unknown.raw")).unwrap(),
        b"keep"
    );
    assert!(!fixture.assets.join("renamed").exists());
    fixture
        .manager
        .relocate_project_folder(uri("res://data"), uri("res://data/sub/moved"))
        .expect_err("a folder cannot move inside itself");
    fixture
        .manager
        .create_project_folder(uri("res://occupied"))
        .unwrap();
    fixture
        .manager
        .relocate_project_folder(uri("res://data"), uri("res://occupied"))
        .expect_err("an existing empty target is still occupied");
    fixture
        .manager
        .create_project_folder(uri("package://readonly/new"))
        .expect_err("folder mutations are project-only");
}

#[test]
fn folder_mutations_reject_a_retired_project_activation() {
    let fixture = Fixture::new("folder_activation");
    let activation = fixture
        .manager
        .current_project_manager()
        .unwrap()
        .activation_identity()
        .clone();
    fixture.manager.close_project().unwrap();
    fixture
        .manager
        .open_prepared_project(ProjectManager::open(&fixture.root).unwrap())
        .unwrap();
    fixture
        .manager
        .create_project_folder_for_project(&activation, uri("res://stale"))
        .expect_err("old queued creates cannot enter the new activation");
    fixture
        .manager
        .delete_project_folder_for_project(&activation, uri("res://data/empty/nested"))
        .expect_err("old queued deletes cannot enter the new activation");
    fixture
        .manager
        .relocate_project_folder_for_project(&activation, uri("res://data"), uri("res://stale"))
        .expect_err("old queued renames cannot enter the new activation");
    assert!(!fixture.assets.join("stale").exists());
    assert!(fixture.assets.join("data/empty/nested").is_dir());
}

#[test]
fn asset_rename_rejects_the_active_default_scene() {
    let fixture = Fixture::new("default_scene_asset_rename");
    fixture.manager.close_project().unwrap();
    let mut manifest = ProjectManifest::load(fixture.root.join("zircon-project.toml")).unwrap();
    manifest.default_scene = uri("res://data/first.counted");
    manifest
        .save(fixture.root.join("zircon-project.toml"))
        .unwrap();
    fixture
        .manager
        .open_prepared_project(ProjectManager::open(&fixture.root).unwrap())
        .unwrap();
    let uuid = fixture
        .manager
        .current_project_manager()
        .unwrap()
        .asset_registry()
        .entry_by_path(&manifest.default_scene)
        .unwrap()
        .uuid();
    fixture
        .manager
        .relocate_project_source(uuid, uri("res://renamed.counted"))
        .expect_err("the manifest default scene cannot silently become a missing locator");
    fixture
        .manager
        .relocate_project_folder(uri("res://data"), uri("res://renamed"))
        .expect_err("folder relocation preserves the active default scene too");
    assert!(fixture.assets.join("data/first.counted").is_file());
}

#[cfg(target_os = "linux")]
#[test]
fn folder_and_asset_rename_refuse_symlink_sources_sidecars_and_target_parents() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new("folder_asset_symlinks");
    let outside = fixture.root.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("keep.raw"), "external").unwrap();
    symlink(&outside, fixture.assets.join("linked")).unwrap();
    fixture
        .manager
        .create_project_folder(uri("res://linked/new"))
        .expect_err("linked target parents are never followed");
    fixture
        .manager
        .delete_project_folder(uri("res://linked"))
        .expect_err("a linked source is never removed as a real folder");
    fixture
        .manager
        .relocate_project_folder(uri("res://linked"), uri("res://renamed"))
        .expect_err("a linked source is never followed");
    symlink(
        outside.join("keep.raw"),
        fixture.assets.join("data/linked.raw"),
    )
    .unwrap();
    fixture
        .manager
        .relocate_project_folder(uri("res://data"), uri("res://renamed"))
        .expect_err("linked descendants block the entire folder rename");
    fs::remove_file(fixture.assets.join("data/linked.raw")).unwrap();
    let source = uri("res://data/first.counted");
    let uuid = fixture
        .manager
        .current_project_manager()
        .unwrap()
        .asset_registry()
        .entry_by_path(&source)
        .unwrap()
        .uuid();
    let sidecar = fixture.assets.join("data/first.counted.zmeta");
    let external_meta = outside.join("first.counted.zmeta");
    fs::copy(&sidecar, &external_meta).unwrap();
    fs::remove_file(&sidecar).unwrap();
    symlink(&external_meta, &sidecar).unwrap();
    fixture
        .manager
        .relocate_project_source(uuid, uri("res://data/renamed.counted"))
        .expect_err("asset rename refuses a linked sidecar before parsing or publication");
    assert_eq!(fs::read(outside.join("keep.raw")).unwrap(), b"external");
    assert!(fixture.assets.join("data/first.counted").is_file());
    assert!(!fixture.assets.join("data/renamed.counted").exists());
    assert!(!outside.join("new").exists());
}

fn counted_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new(
            "test.u09.counted.data",
            "test.u09.counted",
            AssetKind::Data,
            1,
        )
        .with_source_extensions(["counted"]),
        |context: &AssetImportContext| -> Result<AssetImportOutcome, AssetImportError> {
            Ok(AssetImportOutcome::new(
                context.uri.clone(),
                ImportedAsset::Data(DataAsset {
                    uri: context.uri.clone(),
                    format: DataAssetFormat::Json,
                    text: context.source_text()?,
                    canonical_json: serde_json::json!({ "u09": true }),
                }),
            ))
        },
    )
}
