use crate::{
    ZrRuntimeApiV6, ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle,
    ZrRuntimeOperationOutcomeV1, ZrRuntimeOperationPhase, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2,
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
fn runtime_operation_status_v2_is_fixed_layout_and_result_keeps_terminal_outcome() {
    let handle = ZrRuntimeOperationHandle::new(17);
    let status = ZrRuntimeOperationStatusV2::new(
        handle,
        ZrRuntimeOperationPhase::Queued,
        2,
        5,
        ZrRuntimeOperationDetailKindV2::QueueDepth,
        3,
    );
    let result = ZrRuntimeOperationResultV1::succeeded(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        handle,
        "navigation.bake.scene",
        serde_json::json!({"tiles": 5}),
    );

    assert_eq!(core::mem::size_of::<ZrRuntimeOperationStatusV2>(), 48);
    assert_eq!(status.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V2);
    assert_eq!(status.reserved, 0);
    assert_eq!(status.handle, handle);
    assert_eq!(status.phase(), Some(ZrRuntimeOperationPhase::Queued));
    assert_eq!(
        status.detail_kind(),
        Some(ZrRuntimeOperationDetailKindV2::QueueDepth)
    );
    assert_eq!(status.detail_value, 3);
    assert_eq!(result.handle, handle);
    assert!(matches!(
        result.outcome,
        ZrRuntimeOperationOutcomeV1::Succeeded { ref output }
            if output["tiles"] == 5
    ));
    assert_eq!(ZrRuntimeOperationPhase::Queued.raw(), 1);
    assert_eq!(ZrRuntimeOperationPhase::Preparing.raw(), 2);
    assert_eq!(ZrRuntimeOperationPhase::ReadyToApply.raw(), 3);
    assert_eq!(ZrRuntimeOperationPhase::Completed.raw(), 4);
    assert_eq!(ZrRuntimeOperationPhase::Failed.raw(), 5);
    assert_eq!(ZrRuntimeOperationPhase::Cancelled.raw(), 6);
    assert_eq!(ZrRuntimeOperationPhase::Expired.raw(), 7);
    assert_eq!(ZrRuntimeOperationPhase::Harvested.raw(), 8);
    assert!(ZrRuntimeOperationPhase::Completed.is_terminal());
    assert!(ZrRuntimeOperationPhase::Failed.is_terminal());
    assert!(ZrRuntimeOperationPhase::Cancelled.is_terminal());
    assert!(ZrRuntimeOperationPhase::Expired.is_terminal());
    assert!(ZrRuntimeOperationPhase::Harvested.is_terminal());
    assert!(!ZrRuntimeOperationPhase::Queued.is_terminal());
    assert!(!ZrRuntimeOperationPhase::Preparing.is_terminal());
    assert!(!ZrRuntimeOperationPhase::ReadyToApply.is_terminal());
    for (raw, kind) in [
        (0, ZrRuntimeOperationDetailKindV2::None),
        (1, ZrRuntimeOperationDetailKindV2::QueueDepth),
        (2, ZrRuntimeOperationDetailKindV2::AdmissionCountLimit),
        (3, ZrRuntimeOperationDetailKindV2::AdmissionByteLimit),
        (4, ZrRuntimeOperationDetailKindV2::DeadlineElapsed),
        (5, ZrRuntimeOperationDetailKindV2::Cancelled),
        (6, ZrRuntimeOperationDetailKindV2::WorkerPanic),
        (7, ZrRuntimeOperationDetailKindV2::OwnerApplyFailed),
        (8, ZrRuntimeOperationDetailKindV2::TerminalTtlElapsed),
        (9, ZrRuntimeOperationDetailKindV2::Harvested),
        (10, ZrRuntimeOperationDetailKindV2::WorkerChannelLost),
    ] {
        assert_eq!(kind.raw(), raw);
        assert_eq!(ZrRuntimeOperationDetailKindV2::from_raw(raw), Some(kind));
    }
    assert_eq!(ZrRuntimeOperationDetailKindV2::from_raw(11), None);
}

#[test]
fn runtime_operation_function_pointers_precede_world_sync_v6_tail() {
    let api = ZrRuntimeApiV6::empty();

    assert!(api.submit_operation.is_none());
    assert!(api.poll_operation.is_none());
    assert!(api.harvest_operation.is_none());
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV6, submit_operation),
        core::mem::offset_of!(ZrRuntimeApiV6, drain_plugin_events)
            + core::mem::size_of_val(&api.drain_plugin_events)
    );
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV6, poll_operation),
        core::mem::offset_of!(ZrRuntimeApiV6, submit_operation)
            + core::mem::size_of_val(&api.submit_operation)
    );
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV6, harvest_operation),
        core::mem::offset_of!(ZrRuntimeApiV6, poll_operation)
            + core::mem::size_of_val(&api.poll_operation)
    );
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV6, query_world),
        core::mem::offset_of!(ZrRuntimeApiV6, harvest_operation)
            + core::mem::size_of_val(&api.harvest_operation)
    );
}
