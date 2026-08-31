use std::collections::HashMap;
use std::sync::Arc;

use zircon_runtime::asset::registry::AssetRegistryIndex;
use zircon_runtime::asset::{AssetReference, AssetUri, AssetUuid};

use super::super::preview_refresh::display_name_for_locator::display_name_for_locator;
use super::record::record_to_view;
use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, EditorAssetDetailsGeneration, EditorAssetReferenceRecord,
    EditorAssetSubassetRecord,
};

pub(super) fn build_details_generation(
    record: &AssetCatalogRecord,
    catalog_by_uuid: &HashMap<AssetUuid, AssetCatalogRecord>,
    uuid_by_locator: &HashMap<AssetUri, AssetUuid>,
    runtime_registry: &AssetRegistryIndex,
) -> Arc<EditorAssetDetailsGeneration> {
    let mut direct_references = record
        .direct_references
        .iter()
        .map(|reference| reference_to_view(reference, catalog_by_uuid, uuid_by_locator))
        .collect::<Vec<_>>();
    direct_references.sort_by(reference_order);

    let mut referenced_by = runtime_registry
        .get_referencers_by_uuid(record.asset_uuid)
        .into_iter()
        .filter_map(|source_uuid| catalog_by_uuid.get(&source_uuid))
        .map(|source| EditorAssetReferenceRecord {
            uuid: source.asset_uuid.to_string(),
            locator: source.locator.to_string(),
            display_name: source.display_name.clone(),
            kind: Some(source.kind),
            known_project_asset: true,
        })
        .collect::<Vec<_>>();
    referenced_by.sort_by(reference_order);

    Arc::new(EditorAssetDetailsGeneration {
        asset: Arc::new(record_to_view(record, catalog_by_uuid, uuid_by_locator)),
        direct_references: direct_references.into(),
        referenced_by: referenced_by.into(),
        package_id: record.locator.package_id().map(Arc::from),
        unit: record.meta.unit,
        included_files: record
            .meta
            .included_files
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .into(),
        subassets: record
            .meta
            .entries
            .iter()
            .filter(|entry| entry.url.label().is_some())
            .map(|entry| EditorAssetSubassetRecord {
                uuid: entry.uuid.to_string(),
                locator: entry.url.to_string(),
                kind: entry.asset_kind,
                artifact_locator: entry.artifact_locator.as_ref().map(ToString::to_string),
                dependency_locators: entry.dependencies.iter().map(ToString::to_string).collect(),
            })
            .collect::<Vec<_>>()
            .into(),
    })
}

fn reference_to_view(
    reference: &AssetReference,
    catalog_by_uuid: &HashMap<AssetUuid, AssetCatalogRecord>,
    uuid_by_locator: &HashMap<AssetUri, AssetUuid>,
) -> EditorAssetReferenceRecord {
    if let Some(record) = catalog_by_uuid.get(&reference.uuid).or_else(|| {
        uuid_by_locator
            .get(&reference.locator)
            .and_then(|uuid| catalog_by_uuid.get(uuid))
    }) {
        return EditorAssetReferenceRecord {
            uuid: record.asset_uuid.to_string(),
            locator: record.locator.to_string(),
            display_name: record.display_name.clone(),
            kind: Some(record.kind),
            known_project_asset: true,
        };
    }

    EditorAssetReferenceRecord {
        uuid: reference.uuid.to_string(),
        locator: reference.locator.to_string(),
        display_name: display_name_for_locator(&reference.locator),
        kind: None,
        known_project_asset: false,
    }
}

fn reference_order(
    left: &EditorAssetReferenceRecord,
    right: &EditorAssetReferenceRecord,
) -> std::cmp::Ordering {
    left.display_name
        .cmp(&right.display_name)
        .then(left.locator.cmp(&right.locator))
}
