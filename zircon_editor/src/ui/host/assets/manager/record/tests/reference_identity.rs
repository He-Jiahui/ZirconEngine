use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use zircon_runtime::asset::project::{AssetMetaDocument, PreviewState};
use zircon_runtime::asset::registry::AssetRegistryIndex;
use zircon_runtime::asset::{AssetId, AssetKind, AssetReference, AssetUri, AssetUuid};
use zircon_runtime_interface::ui::layout::{UiPoint, UiSize};

use super::super::details::build_details_generation;
use super::{record_to_view, reference_record};
use crate::ui::host::editor_asset_manager::{
    AssetCatalogRecord, EditorAssetCatalogGeneration, EditorAssetCatalogSnapshotRecord,
    EditorAssetFolderRecord,
};
use crate::ui::retained_host::asset_pointer::{
    asset_reference_viewport_y, AssetListPointerState, AssetPointerReferenceRoute,
    AssetReferenceListPointerBridge, AssetReferenceListPointerLayout,
};
use crate::ui::workbench::project::AssetWorkspaceState;
use crate::ui::workbench::snapshot::AssetSurfaceMode;

#[test]
fn editor57_reference_identity_rejects_missing_explicit_uuid_at_an_occupied_locator() {
    let occupied = record(uuid("occupant"), "res://occupied.png");
    let missing = uuid("missing");
    let reference = AssetReference::new(missing, occupied.locator.clone());
    let (catalog, locators) = indexes([occupied]);

    assert_ne!(
        missing,
        AssetReference::from_locator(reference.locator.clone()).uuid
    );
    assert!(reference_record(&reference, &catalog, &locators).is_none());
}

#[test]
fn editor57_reference_identity_allows_only_the_exact_locator_derived_uuid_to_fall_back() {
    let occupied = record(uuid("occupant"), "res://occupied.png");
    let reference = AssetReference::from_locator(occupied.locator.clone());
    let expected_uuid = occupied.asset_uuid;
    let (catalog, locators) = indexes([occupied]);

    assert_eq!(
        reference_record(&reference, &catalog, &locators).map(|record| record.asset_uuid),
        Some(expected_uuid)
    );
    let stale_derived = AssetReference::new(
        AssetReference::from_locator(uri("res://previous.png")).uuid,
        reference.locator.clone(),
    );
    assert!(reference_record(&stale_derived, &catalog, &locators).is_none());
    let absent = AssetReference::from_locator(uri("res://absent.png"));
    assert!(reference_record(&absent, &catalog, &locators).is_none());
}

#[test]
fn editor57_reference_identity_prefers_a_registered_uuid_over_an_occupied_hint() {
    let occupied = record(uuid("occupant"), "res://occupied.png");
    for target_uuid in [
        uuid("registered"),
        AssetReference::from_locator(occupied.locator.clone()).uuid,
    ] {
        let target = record(target_uuid, "res://moved.png");
        let reference = AssetReference::new(target_uuid, occupied.locator.clone());
        let (catalog, locators) = indexes([target, occupied.clone()]);

        let resolved = reference_record(&reference, &catalog, &locators).unwrap();
        assert_eq!(resolved.asset_uuid, target_uuid);
        assert_eq!(resolved.locator, uri("res://moved.png"));
    }
}

#[test]
fn editor57_reference_identity_details_and_catalog_drive_the_same_pointer_target() {
    let occupied = record(uuid("occupant"), "res://occupied.png");
    let registered = record(uuid("registered"), "res://moved.png");
    let missing = uuid("missing");
    let cases = [
        (
            AssetReference::new(missing, occupied.locator.clone()),
            missing,
            occupied.locator.clone(),
            false,
        ),
        (
            AssetReference::from_locator(occupied.locator.clone()),
            occupied.asset_uuid,
            occupied.locator.clone(),
            true,
        ),
        (
            AssetReference::new(registered.asset_uuid, occupied.locator.clone()),
            registered.asset_uuid,
            registered.locator.clone(),
            true,
        ),
    ];

    for (reference, expected_uuid, expected_locator, known) in cases {
        let mut source = record(uuid("source"), "res://source.png");
        source.direct_references.push(reference);
        let (catalog, locators) = indexes([source.clone(), occupied.clone(), registered.clone()]);
        let row = record_to_view(&source, &catalog, &locators);
        let details =
            build_details_generation(&source, &catalog, &locators, &AssetRegistryIndex::default());

        assert_eq!(row.direct_reference_uuids, vec![expected_uuid.to_string()]);
        assert_eq!(details.asset.as_ref(), &row);
        assert_eq!(details.direct_references.len(), 1);
        let projected = &details.direct_references[0];
        assert_eq!(projected.uuid, expected_uuid.to_string());
        assert_eq!(projected.locator, expected_locator.to_string());
        assert_eq!(projected.known_project_asset, known);
        assert_eq!(projected.kind, known.then_some(AssetKind::Texture));

        let mut workspace = workspace(&catalog, &locators);
        assert!(workspace.select_asset(Some(source.asset_uuid.to_string())));
        workspace.sync_selected_details(Some(Arc::clone(&details)));
        let snapshot = workspace.build_snapshot(AssetSurfaceMode::Explorer);
        assert_eq!(snapshot.selection.references.len(), 1);
        assert_eq!(snapshot.selection.references[0].uuid, projected.uuid);
        assert_eq!(snapshot.selection.references[0].known_project_asset, known);

        let mut pointer = AssetReferenceListPointerBridge::new();
        pointer.sync(
            AssetReferenceListPointerLayout::from_references(
                &snapshot.selection.references,
                UiSize::new(300.0, 140.0),
            ),
            AssetListPointerState::default(),
        );
        let dispatch = pointer
            .handle_click(UiPoint::new(8.0, asset_reference_viewport_y() + 8.0))
            .expect("reference row pointer dispatch");
        if known {
            assert_eq!(
                dispatch.route,
                Some(AssetPointerReferenceRoute::Item {
                    row_index: 0,
                    asset_uuid: expected_uuid.to_string(),
                })
            );
            if let Some(AssetPointerReferenceRoute::Item { asset_uuid, .. }) = dispatch.route {
                assert!(workspace.navigate_to_asset(&asset_uuid));
            }
            assert_eq!(
                workspace.selected_asset_uuid(),
                Some(expected_uuid.to_string().as_str())
            );
        } else {
            assert_eq!(
                dispatch.route,
                Some(AssetPointerReferenceRoute::ListSurface)
            );
            assert_eq!(
                workspace.selected_asset_uuid(),
                Some(source.asset_uuid.to_string().as_str())
            );
            assert_ne!(projected.uuid, occupied.asset_uuid.to_string());
        }
    }
}

fn workspace(
    catalog: &HashMap<AssetUuid, AssetCatalogRecord>,
    locators: &HashMap<AssetUri, AssetUuid>,
) -> AssetWorkspaceState {
    let assets = catalog
        .values()
        .map(|record| record_to_view(record, catalog, locators))
        .collect::<Vec<_>>();
    let mut workspace = AssetWorkspaceState::default();
    workspace.sync_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(
            EditorAssetCatalogSnapshotRecord {
                folders: vec![EditorAssetFolderRecord {
                    folder_id: "res://".to_string(),
                    parent_folder_id: None,
                    locator_prefix: "res://".to_string(),
                    display_name: "Assets".to_string(),
                    child_folder_ids: Vec::new(),
                    direct_asset_uuids: assets.iter().map(|asset| asset.uuid.clone()).collect(),
                    recursive_asset_count: assets.len(),
                }],
                assets,
                ..EditorAssetCatalogSnapshotRecord::default()
            },
            1,
        ),
    ));
    workspace
}

fn indexes(
    records: impl IntoIterator<Item = AssetCatalogRecord>,
) -> (
    HashMap<AssetUuid, AssetCatalogRecord>,
    HashMap<AssetUri, AssetUuid>,
) {
    let catalog = records
        .into_iter()
        .map(|record| (record.asset_uuid, record))
        .collect::<HashMap<_, _>>();
    let locators = catalog
        .values()
        .map(|record| (record.locator.clone(), record.asset_uuid))
        .collect();
    (catalog, locators)
}

fn record(asset_uuid: AssetUuid, locator: &str) -> AssetCatalogRecord {
    let locator = uri(locator);
    let mut meta = AssetMetaDocument::new(asset_uuid, locator.clone(), AssetKind::Texture);
    meta.preview_state = PreviewState::Ready;
    AssetCatalogRecord {
        asset_uuid,
        asset_id: AssetId::from_asset_uuid(asset_uuid),
        file_name: locator.path().to_string(),
        display_name: locator.path().to_string(),
        locator,
        kind: AssetKind::Texture,
        extension: "png".to_string(),
        meta_path: PathBuf::from("metadata/reference.zmeta"),
        meta,
        source_mtime_unix_ms: 0,
        source_hash: "reference-source".to_string(),
        preview_state: PreviewState::Ready,
        preview_artifact_path: PathBuf::new(),
        dirty: false,
        diagnostics: Vec::new(),
        direct_references: Vec::new(),
    }
}

fn uuid(label: &str) -> AssetUuid {
    AssetUuid::from_stable_label(&format!("editor57-reference-{label}"))
}

fn uri(locator: &str) -> AssetUri {
    AssetUri::parse(locator).expect("valid fixture locator")
}
