use std::sync::Arc;

use super::{
    resolve_ui_texture_candidate, resolve_ui_texture_id, ui_image_resource_id,
    UiTextureDependencyCache, UiTexturePrepareOutcome, UiTexturePrepareReceipt,
    UiTexturePrepareRow,
};
use crate::asset::ProjectAssetManager;
use crate::core::framework::render::UiRenderSubmission;
use crate::core::resource::{AssetUuid, ResourceId, ResourceKind, ResourceLocator, ResourceRecord};
use zircon_runtime_interface::ui::event_ui::{UiNodeId, UiTreeId};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiRenderCommandKind, UiRenderExtract, UiRenderList, UiResolvedStyle,
    UiVisualAssetRef,
};

#[test]
fn ui_image_resource_id_accepts_engine_assets_and_rejects_network_urls() {
    assert_eq!(
        ui_image_resource_id("res://ui/checker.png"),
        Some(ResourceId::from_stable_label("res://ui/checker.png"))
    );
    assert_eq!(
        ui_image_resource_id("https://example.com/checker.png"),
        None
    );
}

#[test]
fn ui_texture_dependencies_reuse_stable_submission_and_segment_products() {
    let first_id = ui_image_resource_id("res://ui/first.png").unwrap();
    let second_id = ui_image_resource_id("res://ui/second.png").unwrap();
    let submission = UiRenderSubmission::from_segments(vec![
        image_extract("first", &["res://ui/first.png"]),
        image_extract("second", &["res://ui/second.png", "res://ui/first.png"]),
    ]);
    let mut cache = UiTextureDependencyCache::default();

    let first = cache.prepare(&submission);
    let stable = cache.prepare(&submission);

    let mut expected = vec![first_id, second_id];
    expected.sort_unstable();
    assert_eq!(first.as_slice(), expected);
    assert!(first.change_journal().is_full_rebuild());
    assert!(Arc::ptr_eq(&first, &stable));
}

#[test]
fn ui_texture_dependencies_publish_exact_local_command_leaf_delta() {
    let mut sources = vec!["res://ui/shared.png"; 130];
    let flat = image_extract("local-delta", &sources);
    let first_frame =
        Arc::new(zircon_runtime_interface::ui::surface::UiRenderFrameExtract::from_extract(&flat));
    sources[65] = "res://ui/changed.png";
    let changed_flat = image_extract("local-delta", &sources);
    let (changed_frame, _) = first_frame
        .patch_ranges_from_extract(&changed_flat, &[65..66])
        .expect("fixed-cardinality image patch should retain untouched command leaves");
    let first_submission = UiRenderSubmission::single_frame(first_frame);
    let changed_submission = UiRenderSubmission::single_frame(Arc::new(changed_frame));
    let mut cache = UiTextureDependencyCache::default();

    let first = cache.prepare(&first_submission);
    let changed = cache.prepare(&changed_submission);
    let changed_id = ui_image_resource_id("res://ui/changed.png").unwrap();

    assert_eq!(
        changed.change_journal().base_generation(),
        Some(first.generation())
    );
    assert_eq!(changed.change_journal().added_ids(), &[changed_id]);
    assert!(changed.change_journal().removed_ids().is_empty());
    assert_eq!(changed.as_slice().len(), 2);
}

#[test]
fn ui_texture_resolution_maps_locator_identity_to_imported_asset_identity() {
    let manager = ProjectAssetManager::default();
    let locator = ResourceLocator::parse("res://ui/checker.png").unwrap();
    let requested = ResourceId::from_locator(&locator);
    let imported = ResourceId::from_asset_uuid(AssetUuid::from_stable_label("ui/checker"));
    manager
        .resource_manager()
        .register_record(ResourceRecord::new(
            imported,
            ResourceKind::Texture,
            locator,
        ))
        .unwrap();

    assert_ne!(requested, imported);
    assert_eq!(resolve_ui_texture_id(&manager, requested), imported);
}

#[test]
fn ui_texture_candidate_rejects_unresolved_and_wrong_kind_resources() {
    let manager = ProjectAssetManager::default();
    let missing = ResourceId::from_stable_label("missing-ui-texture");
    let projection = manager.resource_manager().projection_snapshot();
    assert_eq!(
        resolve_ui_texture_candidate(projection.management(), missing),
        Err(UiTexturePrepareOutcome::UnresolvedIdentity)
    );

    let locator = ResourceLocator::parse("res://ui/not-a-texture.asset").unwrap();
    let wrong_kind = ResourceId::from_locator(&locator);
    manager
        .resource_manager()
        .register_record(ResourceRecord::new(
            wrong_kind,
            ResourceKind::Material,
            locator,
        ))
        .unwrap();
    let projection = manager.resource_manager().projection_snapshot();
    assert_eq!(
        resolve_ui_texture_candidate(projection.management(), wrong_kind),
        Err(UiTexturePrepareOutcome::InvalidResourceKind)
    );
}

#[test]
fn ui_texture_receipt_only_exposes_exact_ready_rows_to_binding() {
    let manager = ProjectAssetManager::default();
    let projection = manager.resource_manager().projection_snapshot();
    let ready = ResourceId::from_stable_label("ready-ui-texture");
    let failed = ResourceId::from_stable_label("failed-ui-texture");
    let unqualified = ResourceId::from_stable_label("unqualified-ui-texture");
    let receipt = UiTexturePrepareReceipt::new(
        1,
        projection.management_identity(),
        projection.readiness_identity(),
        vec![
            UiTexturePrepareRow {
                requested: ready,
                resolved: Some(ready),
                outcome: UiTexturePrepareOutcome::Ready,
                prepared_revision: Some(7),
            },
            UiTexturePrepareRow {
                requested: failed,
                resolved: Some(failed),
                outcome: UiTexturePrepareOutcome::UploadFailed,
                prepared_revision: None,
            },
            UiTexturePrepareRow {
                requested: unqualified,
                resolved: Some(unqualified),
                outcome: UiTexturePrepareOutcome::Ready,
                prepared_revision: None,
            },
        ],
    );

    assert_eq!(receipt.ready_texture_id(ready), Some(ready));
    assert_eq!(receipt.ready_texture_id(failed), None);
    assert_eq!(receipt.ready_texture_id(unqualified), None);
}

#[test]
fn ui_texture_receipt_reuses_binding_product_generation_only_for_equal_rows() {
    let manager = ProjectAssetManager::default();
    let projection = manager.resource_manager().projection_snapshot();
    let requested = ResourceId::from_stable_label("stable-ui-texture");
    let row = UiTexturePrepareRow {
        requested,
        resolved: Some(requested),
        outcome: UiTexturePrepareOutcome::Ready,
        prepared_revision: Some(7),
    };
    let first = UiTexturePrepareReceipt::new(
        11,
        projection.management_identity(),
        projection.readiness_identity(),
        vec![row],
    );
    let mut stable = UiTexturePrepareReceipt::new(
        12,
        projection.management_identity(),
        projection.readiness_identity(),
        vec![row],
    );
    stable.reuse_equivalent_binding_product_generation(&first);
    assert_eq!(stable.binding_product_generation(), 11);

    let mut changed = UiTexturePrepareReceipt::new(
        13,
        projection.management_identity(),
        projection.readiness_identity(),
        vec![UiTexturePrepareRow {
            outcome: UiTexturePrepareOutcome::UploadFailed,
            prepared_revision: None,
            ..row
        }],
    );
    changed.reuse_equivalent_binding_product_generation(&stable);
    assert_eq!(changed.binding_product_generation(), 13);
}

fn image_extract(tree_id: &str, sources: &[&str]) -> Arc<UiRenderExtract> {
    Arc::new(UiRenderExtract {
        tree_id: UiTreeId::new(tree_id),
        list: UiRenderList {
            commands: sources
                .iter()
                .enumerate()
                .map(|(index, source)| UiRenderCommand {
                    node_id: UiNodeId::new(index as u64 + 1),
                    kind: UiRenderCommandKind::Image,
                    frame: UiFrame::new(0.0, 0.0, 1.0, 1.0),
                    clip_frame: None,
                    z_index: index as i32,
                    style: UiResolvedStyle::default(),
                    text_layout: None,
                    text: None,
                    image: Some(UiVisualAssetRef::Image((*source).to_string())),
                    opacity: 1.0,
                })
                .collect(),
        },
        raster_scale: 1.0,
    })
}
