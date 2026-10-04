use super::*;

#[test]
fn request_debug_omits_model_text() {
    let request = UiTextModelUpdateRequest::new(
        UiTreeId::new("text.model.update"),
        UiNodeId::new(7),
        UiTextDocumentKey {
            document_id: UiTextDocumentId::issue(),
            revision: UiTextDocumentRevision::new(3),
        },
        UiTextModelUpdateOrigin::BoundRefresh,
        "credential-like-model-value",
    );

    let debug = format!("{request:?}");
    assert!(!debug.contains("credential-like-model-value"));
    assert!(debug.contains("value_byte_len"));
}

#[test]
fn request_validation_rejects_unversioned_or_unqualified_identity() {
    let mut request = UiTextModelUpdateRequest::new(
        UiTreeId::new("text.model.update"),
        UiNodeId::new(7),
        UiTextDocumentKey {
            document_id: UiTextDocumentId::issue(),
            revision: UiTextDocumentRevision::new(0),
        },
        UiTextModelUpdateOrigin::BoundRefresh,
        "model",
    );
    assert_eq!(request.validate(), Ok(()));

    request.schema_version = 0;
    assert_eq!(
        request.validate(),
        Err(UiTextModelUpdateFailure::UnsupportedSchemaVersion)
    );
    request.schema_version = UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION;
    request.request_id = UiTextModelUpdateId::default();
    assert_eq!(
        request.validate(),
        Err(UiTextModelUpdateFailure::InvalidRequestId)
    );
    request.request_id = UiTextModelUpdateId::issue();
    request.expected_document.document_id = UiTextDocumentId::default();
    assert_eq!(
        request.validate(),
        Err(UiTextModelUpdateFailure::InvalidDocumentId)
    );
}

#[test]
fn receipt_validation_rejects_inconsistent_status_or_missing_current_key() {
    let key = UiTextDocumentKey {
        document_id: UiTextDocumentId::issue(),
        revision: UiTextDocumentRevision::new(2),
    };
    let mut receipt = UiTextModelUpdateReceipt {
        schema_version: UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION,
        request_id: UiTextModelUpdateId::issue(),
        tree_id: UiTreeId::new("text.model.update"),
        node_id: UiNodeId::new(7),
        origin: UiTextModelUpdateOrigin::BoundRefresh,
        status: UiTextModelUpdateStatus::Deferred,
        expected_document: key,
        current_document: Some(key),
        document_edit: None,
        failure: None,
    };
    assert_eq!(receipt.validate(), Ok(()));

    receipt.failure = Some(UiTextModelUpdateFailure::PendingQueueFull);
    assert_eq!(
        receipt.validate(),
        Err(UiTextModelUpdateFailure::DocumentRejected)
    );
    receipt.failure = None;
    receipt.current_document = None;
    assert_eq!(
        receipt.validate(),
        Err(UiTextModelUpdateFailure::DocumentRejected)
    );
}

#[test]
fn receipt_validation_accepts_malformed_identity_rejections_but_rejects_false_failures() {
    let valid_key = UiTextDocumentKey {
        document_id: UiTextDocumentId::issue(),
        revision: UiTextDocumentRevision::new(0),
    };
    let mut receipt = UiTextModelUpdateReceipt {
        schema_version: UI_TEXT_MODEL_UPDATE_SCHEMA_VERSION,
        request_id: UiTextModelUpdateId::default(),
        tree_id: UiTreeId::new("text.model.update"),
        node_id: UiNodeId::new(7),
        origin: UiTextModelUpdateOrigin::BoundRefresh,
        status: UiTextModelUpdateStatus::Rejected,
        expected_document: valid_key,
        current_document: None,
        document_edit: None,
        failure: Some(UiTextModelUpdateFailure::InvalidRequestId),
    };
    assert_eq!(receipt.validate(), Ok(()));

    receipt.request_id = UiTextModelUpdateId::issue();
    assert_eq!(
        receipt.validate(),
        Err(UiTextModelUpdateFailure::InvalidRequestId)
    );
    receipt.failure = Some(UiTextModelUpdateFailure::InvalidDocumentId);
    receipt.expected_document.document_id = UiTextDocumentId::default();
    assert_eq!(receipt.validate(), Ok(()));

    receipt.request_id = UiTextModelUpdateId::default();
    receipt.failure = Some(UiTextModelUpdateFailure::UnsupportedSchemaVersion);
    assert_eq!(receipt.validate(), Ok(()));
}
