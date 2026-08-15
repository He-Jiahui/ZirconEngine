use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
use zircon_runtime::asset::{AssetUri, AssetUuid};

use super::details::build_details_generation;
use super::folders::build_folder_records;
use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, EditorAssetCatalogGeneration, ReferenceGraph,
};

pub(in crate::ui::host::editor_asset_manager::manager) fn build_catalog_generation(
    project: &ProjectManager,
    assets_root: &Path,
    catalog_revision: u64,
    publish_epoch: u64,
    catalog_by_uuid: &HashMap<AssetUuid, AssetCatalogRecord>,
    uuid_by_locator: &HashMap<AssetUri, AssetUuid>,
    reference_graph: &ReferenceGraph,
) -> Arc<EditorAssetCatalogGeneration> {
    let mut records = catalog_by_uuid.values().collect::<Vec<_>>();
    records.sort_by(|left, right| left.locator.cmp(&right.locator));
    let details = records
        .iter()
        .map(|record| {
            Some(build_details_generation(
                record,
                catalog_by_uuid,
                uuid_by_locator,
                reference_graph,
            ))
        })
        .collect::<Vec<_>>();
    let assets = details
        .iter()
        .map(|details| {
            Arc::clone(
                &details
                    .as_ref()
                    .expect("details are built for every asset")
                    .asset,
            )
        })
        .collect::<Vec<_>>();

    Arc::new(EditorAssetCatalogGeneration::from_parts(
        project.manifest().name.clone(),
        ProjectPaths::display_path(project.paths().root())
            .to_string_lossy()
            .into_owned(),
        ProjectPaths::display_path(assets_root)
            .to_string_lossy()
            .into_owned(),
        ProjectPaths::display_path(project.paths().cache_root())
            .to_string_lossy()
            .into_owned(),
        project.manifest().default_scene.to_string(),
        catalog_revision,
        publish_epoch,
        build_folder_records(catalog_by_uuid),
        assets,
        details,
    ))
}
