use crate::{
    ZrRuntimeApiV2, ZrRuntimeOperationHandle, ZrRuntimeOperationOutcomeV1, ZrRuntimeOperationPhase,
    ZrRuntimeOperationProgressV1, ZrRuntimeOperationResultV1, ZrRuntimeOperationSubmitRequestV1,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

#[test]
fn runtime_operation_contracts_round_trip_without_transport_specific_types() {
    let request = ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        "navigation.bake.scene",
        serde_json::json!({"force_full_rebuild": true}),
    );
    let encoded = serde_json::to_vec(&request).unwrap();
    let decoded: ZrRuntimeOperationSubmitRequestV1 = serde_json::from_slice(&encoded).unwrap();

    assert_eq!(decoded, request);
    assert_eq!(decoded.operation_id, "navigation.bake.scene");
    assert_eq!(decoded.payload["force_full_rebuild"], true);
}

#[test]
fn runtime_operation_progress_and_result_keep_handle_and_terminal_outcome() {
    let handle = ZrRuntimeOperationHandle::new(17);
    let progress = ZrRuntimeOperationProgressV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        handle,
        ZrRuntimeOperationPhase::Running,
        2,
        5,
        "building navigation tiles",
    );
    let result = ZrRuntimeOperationResultV1::succeeded(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        handle,
        "navigation.bake.scene",
        serde_json::json!({"tiles": 5}),
    );

    assert_eq!(progress.handle, handle);
    assert!(!progress.phase.is_terminal());
    assert_eq!(result.handle, handle);
    assert!(matches!(
        result.outcome,
        ZrRuntimeOperationOutcomeV1::Succeeded { ref output }
            if output["tiles"] == 5
    ));
    assert!(ZrRuntimeOperationPhase::Completed.is_terminal());
    assert!(ZrRuntimeOperationPhase::Failed.is_terminal());
    assert!(!ZrRuntimeOperationPhase::Queued.is_terminal());
    assert!(!ZrRuntimeOperationPhase::Running.is_terminal());
}

#[test]
fn runtime_operation_function_pointers_are_v2_api_table_tail() {
    let api = ZrRuntimeApiV2::empty();

    assert!(api.submit_operation.is_none());
    assert!(api.poll_operation.is_none());
    assert!(api.harvest_operation.is_none());
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV2, submit_operation),
        core::mem::offset_of!(ZrRuntimeApiV2, drain_plugin_events)
            + core::mem::size_of_val(&api.drain_plugin_events)
    );
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV2, poll_operation),
        core::mem::offset_of!(ZrRuntimeApiV2, submit_operation)
            + core::mem::size_of_val(&api.submit_operation)
    );
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV2, harvest_operation),
        core::mem::offset_of!(ZrRuntimeApiV2, poll_operation)
            + core::mem::size_of_val(&api.poll_operation)
    );
}
