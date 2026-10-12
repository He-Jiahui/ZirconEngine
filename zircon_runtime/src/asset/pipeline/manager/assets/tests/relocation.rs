use std::fs;

use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{
    AssetImportContext, AssetImportError, AssetImportOutcome, AssetImporterDescriptor, AssetKind,
    AssetManager, AssetUri, DataAsset, DataAssetFormat, FunctionAssetImporter, ImportedAsset,
};
use zircon_runtime_interface::project::RelPath;

use super::ProjectAssetManager;

#[test]
fn pipeline_relocation_commits_live_resource_rename_and_change_event() {
    let root = unique_temp_project_root("pipeline_source_relocation");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout(&[RelPath::project_assets()]).unwrap();
    ProjectManifest::new(
        "PipelineSourceRelocation",
        AssetUri::parse("res://data/original.counted").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();
    let source_path = paths
        .asset_root(&RelPath::project_assets())
        .join("data/original.counted");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "pipeline-relocation-v1").unwrap();
    let source = AssetUri::parse("res://data/original.counted").unwrap();
    let target = AssetUri::parse("res://moved/renamed.counted").unwrap();

    let manager = ProjectAssetManager::default();
    manager
        .register_asset_importer(counted_data_importer())
        .unwrap();
    manager
        .open_prepared_project(ProjectManager::open(&root).unwrap())
        .unwrap();
    let source_uuid = manager
        .current_project_manager()
        .unwrap()
        .asset_registry()
        .entry_by_path(&source)
        .unwrap()
        .uuid();
    let source_id = manager.resolve_asset_id(&source).unwrap();
    let changes = AssetManager::subscribe_asset_changes(&manager);

    let statuses = manager
        .relocate_project_source(source_uuid, target.clone())
        .expect("pipeline relocation should commit the durable generation");

    assert!(statuses
        .iter()
        .any(|status| status.uri == target.to_string()));
    assert_eq!(manager.resolve_asset_id(&target), Some(source_id));
    assert_eq!(manager.resolve_asset_id(&source), None);
    let change = changes
        .try_recv()
        .expect("pipeline relocation publishes a renamed asset change");
    assert_eq!(change.kind, crate::asset::watch::AssetChangeKind::Renamed);
    assert_eq!(change.uri, target);
    assert_eq!(change.previous_uri, Some(source));

    let _ = fs::remove_dir_all(root);
}

fn counted_data_importer() -> FunctionAssetImporter {
    FunctionAssetImporter::new(
        AssetImporterDescriptor::new(
            "test.pipeline.counted.data",
            "test.pipeline.counted",
            AssetKind::Data,
            1,
        )
        .with_source_extensions(["counted"]),
        import_counted_data,
    )
}

fn import_counted_data(
    context: &AssetImportContext,
) -> Result<AssetImportOutcome, AssetImportError> {
    let text = context.source_text()?;
    Ok(AssetImportOutcome::new(
        context.uri.clone(),
        ImportedAsset::Data(DataAsset {
            uri: context.uri.clone(),
            format: DataAssetFormat::Json,
            text,
            canonical_json: serde_json::json!({ "pipeline": true }),
        }),
    ))
}

fn unique_temp_project_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "zircon_{label}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
