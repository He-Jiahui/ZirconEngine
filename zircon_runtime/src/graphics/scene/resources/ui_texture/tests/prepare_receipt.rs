use super::*;

#[test]
fn retained_receipt_preserves_equal_binding_product_and_indexes_frame_retry() {
    let manager = ProjectAssetManager::default();
    let projection = manager.resource_manager().projection_snapshot();
    let requested = ResourceId::from_stable_label("tests/ui-texture/retained-row");
    let ready = prepare_row(
        requested,
        Some(requested),
        UiTexturePrepareOutcome::Ready,
        Some(7),
    );
    let mut receipt = UiTexturePrepareReceipt::new(
        11,
        projection.management_identity(),
        projection.readiness_identity(),
        vec![ready],
    )
    .with_dependency_generation(1);

    receipt.begin_retained_frame(
        12,
        1,
        projection.management_identity(),
        projection.readiness_identity(),
    );
    assert!(!receipt.upsert_row(ready));
    assert_eq!(receipt.binding_product_generation(), 11);

    let failed = prepare_row(
        requested,
        Some(requested),
        UiTexturePrepareOutcome::UploadFailed,
        None,
    );
    assert!(receipt.upsert_row(failed));
    receipt.mark_binding_product_changed();
    assert_eq!(receipt.binding_product_generation(), 12);
    assert_eq!(
        receipt.frame_retry_ids.iter().copied().collect::<Vec<_>>(),
        vec![requested]
    );
    assert!(receipt.readiness_retry_ids.is_empty());
}

#[test]
fn retained_receipt_removes_dependency_from_ready_authority_and_summary() {
    let manager = ProjectAssetManager::default();
    let projection = manager.resource_manager().projection_snapshot();
    let requested = ResourceId::from_stable_label("tests/ui-texture/removed-row");
    let mut receipt = UiTexturePrepareReceipt::new(
        21,
        projection.management_identity(),
        projection.readiness_identity(),
        vec![prepare_row(
            requested,
            Some(requested),
            UiTexturePrepareOutcome::Ready,
            Some(3),
        )],
    );

    assert_eq!(receipt.ready_texture_id(requested), Some(requested));
    assert!(receipt.remove_requested(requested));
    assert_eq!(receipt.ready_texture_id(requested), None);
    assert_eq!(receipt.summary.requested_count, 0);
    assert_eq!(receipt.summary.ready_count, 0);
}

#[test]
fn retry_policy_separates_transient_frame_and_readiness_failures() {
    assert_eq!(
        retry_policy(UiTexturePrepareOutcome::UploadFailed),
        UiTextureRetryPolicy::EveryFrame
    );
    assert_eq!(
        retry_policy(UiTexturePrepareOutcome::GenerationChanged),
        UiTextureRetryPolicy::EveryFrame
    );
    assert_eq!(
        retry_policy(UiTexturePrepareOutcome::NotReady),
        UiTextureRetryPolicy::ReadinessGeneration
    );
    assert_eq!(
        retry_policy(UiTexturePrepareOutcome::Ready),
        UiTextureRetryPolicy::ManagementGeneration
    );
}
