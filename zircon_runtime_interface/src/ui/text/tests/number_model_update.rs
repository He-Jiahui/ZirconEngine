use super::*;

#[test]
fn request_validation_rejects_non_finite_values_and_debug_omits_value() {
    let key = UiNumberModelKey {
        model_id: UiNumberModelId::issue(),
        revision: UiNumberModelRevision::new(3),
    };
    let mut request = UiNumberModelUpdateRequest::new(
        UiTreeId::new("number.model.update"),
        UiNodeId::new(7),
        key,
        UiNumberModelUpdateOrigin::BoundRefresh,
        12345.5,
    );
    assert_eq!(request.validate(), Ok(()));
    assert!(!format!("{request:?}").contains("12345.5"));

    request.value = f64::NAN;
    assert_eq!(
        request.validate(),
        Err(UiNumberModelUpdateFailure::NonFiniteValue)
    );
}

#[test]
fn receipt_validation_requires_current_key_for_terminal_success() {
    let key = UiNumberModelKey {
        model_id: UiNumberModelId::issue(),
        revision: UiNumberModelRevision::new(3),
    };
    let mut receipt = UiNumberModelUpdateReceipt {
        schema_version: UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION,
        request_id: UiNumberModelUpdateId::issue(),
        tree_id: UiTreeId::new("number.model.update"),
        node_id: UiNodeId::new(7),
        origin: UiNumberModelUpdateOrigin::BoundRefresh,
        status: UiNumberModelUpdateStatus::Unchanged,
        expected_model: key,
        current_model: Some(key),
        failure: None,
    };
    assert_eq!(receipt.validate(), Ok(()));

    receipt.current_model = None;
    assert_eq!(
        receipt.validate(),
        Err(UiNumberModelUpdateFailure::PropertyRejected)
    );
}

#[test]
fn receipt_validation_rejects_inconsistent_failures_and_revision_relations() {
    let key = UiNumberModelKey {
        model_id: UiNumberModelId::issue(),
        revision: UiNumberModelRevision::new(3),
    };
    let mut receipt = UiNumberModelUpdateReceipt {
        schema_version: UI_NUMBER_MODEL_UPDATE_SCHEMA_VERSION,
        request_id: UiNumberModelUpdateId::issue(),
        tree_id: UiTreeId::new("number.model.update"),
        node_id: UiNodeId::new(7),
        origin: UiNumberModelUpdateOrigin::BoundRefresh,
        status: UiNumberModelUpdateStatus::Rejected,
        expected_model: key,
        current_model: Some(key),
        failure: Some(UiNumberModelUpdateFailure::InvalidRequestId),
    };
    assert_eq!(
        receipt.validate(),
        Err(UiNumberModelUpdateFailure::InvalidRequestId)
    );

    receipt.status = UiNumberModelUpdateStatus::Applied;
    receipt.failure = None;
    receipt.current_model = Some(UiNumberModelKey {
        revision: UiNumberModelRevision::new(5),
        ..key
    });
    assert_eq!(
        receipt.validate(),
        Err(UiNumberModelUpdateFailure::PropertyRejected)
    );

    receipt.status = UiNumberModelUpdateStatus::Conflict;
    receipt.failure = Some(UiNumberModelUpdateFailure::StaleModel);
    receipt.current_model = Some(key);
    assert_eq!(
        receipt.validate(),
        Err(UiNumberModelUpdateFailure::PropertyRejected)
    );
}
