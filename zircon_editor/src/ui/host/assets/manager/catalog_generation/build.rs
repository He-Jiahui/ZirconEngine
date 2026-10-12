use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
use zircon_runtime::asset::registry::AssetRegistryIndex;
use zircon_runtime::asset::{AssetUri, AssetUuid};

use super::details::build_details_generation;
use super::folders::build_folder_records;
use crate::ui::host::editor_asset_manager::{AssetCatalogRecord, EditorAssetCatalogGeneration};

pub(in crate::ui::host::editor_asset_manager::manager) fn build_catalog_generation(
    project: &ProjectManager,
    runtime_registry: &AssetRegistryIndex,
    assets_root: &Path,
    catalog_revision: u64,
    publish_epoch: u64,
    catalog_by_uuid: &HashMap<AssetUuid, AssetCatalogRecord>,
    uuid_by_locator: &HashMap<AssetUri, AssetUuid>,
) -> Arc<EditorAssetCatalogGeneration> {
    let mut records = Vec::with_capacity(catalog_by_uuid.len());
    records.extend(catalog_by_uuid.values());
    records.sort_by(|left, right| left.locator.cmp(&right.locator));
    let record_count = records.len();
    let mut details = Vec::with_capacity(record_count);
    for record in &records {
        details.push(Some(build_details_generation(
            record,
            catalog_by_uuid,
            uuid_by_locator,
            runtime_registry,
        )));
    }

    let mut assets = Vec::with_capacity(record_count);
    for details in &details {
        assets.push(Arc::clone(
            &details
                .as_ref()
                .expect("details are built for every asset")
                .asset,
        ));
    }

    let mut catalog_records = Vec::with_capacity(record_count);
    for record in &records {
        catalog_records.push(Some(Arc::new((*record).clone())));
    }

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
        catalog_records,
    ))
}
