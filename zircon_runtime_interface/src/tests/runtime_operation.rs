//! 固定操作请求/结果 JSON、poll V2 布局与 Runtime API V8 槽位顺序的契约测试。
//!
//! 这些测试构造接口值并检查编码/布局，不调度 Runtime 工作或证明 provider 的存活。
//! # 参考
//! [`ZrRuntimeOperationSubmitRequestV1`]、[`ZrRuntimeOperationStatusV2`]、[`ZrRuntimeApiV8`]。

use crate::{
    ZrRuntimeApiV8, ZrRuntimeOperationDetailKindV2, ZrRuntimeOperationHandle,
    ZrRuntimeOperationOutcomeV1, ZrRuntimeOperationPhase, ZrRuntimeOperationResultV1,
    ZrRuntimeOperationStatusV2, ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2,
};

/// 验证 V1 提交 JSON 往返保留处理器键和任意 payload，不依赖具体传输。
#[test]
fn runtime_operation_contracts_round_trip_without_transport_specific_types() {
    let request = ZrRuntimeOperationSubmitRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        "navigation.bake.scene",
        serde_json::json!({"force_full_rebuild": true}),
    );
    // 此处比较完整 DTO 与业务字段，避免只验证能解析却遗漏字段保值。
    let encoded = serde_json::to_vec(&request).unwrap();
    let decoded: ZrRuntimeOperationSubmitRequestV1 = serde_json::from_slice(&encoded).unwrap();

    assert_eq!(decoded, request);
    assert_eq!(decoded.operation_id, "navigation.bake.scene");
    assert_eq!(decoded.payload["force_full_rebuild"], true);
}

/// 验证状态固定布局、V2 头、保留字段、已知编码和终态分类，结果信封仍保留 V1 outcome。
///
/// 构造出的 2/5 进度只证明保值；测试不据此要求当前 Runtime producer 使用该进度口径。
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

    // 固定 ABI 状态与 JSON 结果的版本分别检查，防止把载体 V2 错当结果 JSON V2。
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
    // 枚举每个当前 detail 编码并核对反向映射，未知 11 必须保持不可识别。
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

/// 验证默认表无操作槽，并固定插件事件与 world sync 之间三个操作入口的 ABI 顺序。
///
/// offset 与当前 Option 函数指针大小共同计算，只检查本进程布局，不声称跨目标二进制兼容。
#[test]
fn runtime_operation_function_pointers_precede_world_sync_v6_tail() {
    let api = ZrRuntimeApiV8::empty();

    assert!(api.submit_operation.is_none());
    assert!(api.poll_operation.is_none());
    assert!(api.harvest_operation.is_none());
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV8, submit_operation),
        core::mem::offset_of!(ZrRuntimeApiV8, drain_plugin_events)
            + core::mem::size_of_val(&api.drain_plugin_events)
    );
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV8, poll_operation),
        core::mem::offset_of!(ZrRuntimeApiV8, submit_operation)
            + core::mem::size_of_val(&api.submit_operation)
    );
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV8, harvest_operation),
        core::mem::offset_of!(ZrRuntimeApiV8, poll_operation)
            + core::mem::size_of_val(&api.poll_operation)
    );
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV8, query_world),
        core::mem::offset_of!(ZrRuntimeApiV8, harvest_operation)
            + core::mem::size_of_val(&api.harvest_operation)
    );
}
