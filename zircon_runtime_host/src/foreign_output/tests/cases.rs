use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc, Barrier, Mutex, OnceLock,
};
use std::time::{Duration, Instant};

use serde::de::IgnoredAny;
use serde::Deserialize;

use super::decode::decode_bounded_json;
use super::{
    profile_control_response_item_count, RuntimeForeignOutputBudget, RuntimeForeignOutputErrorKind,
    RuntimeForeignOutputKind, RuntimeForeignOutputState, RuntimeOwnedOutputReleaser,
    PROFILE_RESPONSE_OUTPUT_BUDGET,
};
use zircon_runtime_interface::{
    ProfileControlResponse, ZrByteSlice, ZrOwnedResultV2, ZrRuntimeAllocationId,
    ZrRuntimeSessionHandle, ZrStatus, ZrStatusCode,
};

const RELEASE_DIAGNOSTIC: &[u8] = b"test allocation is still in use";
const CALL_DIAGNOSTIC: &[u8] = b"test call failed";
// 由自定义 Deserialize 探针记录预检是否过早进入业务反序列化。
static BUSINESS_DESERIALIZE_CALLS: AtomicUsize = AtomicUsize::new(0);
// 由 ProfileControlResponse 包装探针记录类型化 DTO 是否在预检前已被构造。
static PROFILE_RESPONSE_DESERIALIZE_CALLS: AtomicUsize = AtomicUsize::new(0);

/// 记录业务反序列化是否被调用的测试 DTO。
#[derive(Debug)]
struct BusinessDeserializeProbe;

impl<'de> Deserialize<'de> for BusinessDeserializeProbe {
    /// 先计数，再跳过输入，以便测试断言预检失败时 DTO decoder 尚未运行。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BUSINESS_DESERIALIZE_CALLS.fetch_add(1, Ordering::SeqCst);
        IgnoredAny::deserialize(deserializer)?;
        Ok(Self)
    }
}

/// 以 Interface profile DTO 解码结果作为调用探针。
#[derive(Debug)]
struct ProfileResponseDeserializeProbe;

impl<'de> Deserialize<'de> for ProfileResponseDeserializeProbe {
    /// 计数后实际反序列化 ProfileControlResponse，保持探针输入契约真实。
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        PROFILE_RESPONSE_DESERIALIZE_CALLS.fetch_add(1, Ordering::SeqCst);
        ProfileControlResponse::deserialize(deserializer)?;
        Ok(Self)
    }
}

#[test]
/// 深层 JSON 图在通用预检阶段被拒绝，业务 Deserialize 探针仍为零。
fn bounded_decode_preflights_the_json_graph_before_business_deserialization() {
    BUSINESS_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let budget = RuntimeForeignOutputBudget::new(1024, 2, Duration::from_secs(1));
    let nested = format!("{}0{}", "[".repeat(160), "]".repeat(160));

    let (result, _) = decode_bounded_json::<BusinessDeserializeProbe, String>(
        nested.as_bytes(),
        budget,
        "JSON graph preflight probe",
        |_| Ok(1),
    );

    assert!(result.is_err());
    assert_eq!(BUSINESS_DESERIALIZE_CALLS.load(Ordering::SeqCst), 0);
}

/// 测试 provider registry 保存输出字节至 release callback 执行。
struct TestAllocation {
    /// 使 ABI 返回的 data 指针在 release 前保持有效。
    _bytes: Box<[u8]>,
    /// 可选地记录该测试 allocation 的释放次数。
    releases: Option<Arc<AtomicUsize>>,
    /// 是否模拟释放回调返回失败状态。
    reject_release: bool,
}

// 每项 fixture 分配的稳定 ID 用来验证 exactly-once release。
static NEXT_ALLOCATION_ID: AtomicU64 = AtomicU64::new(1);
// 模拟 provider 在调用者释放之前拥有结果缓冲区。
static TEST_ALLOCATIONS: OnceLock<Mutex<HashMap<u64, TestAllocation>>> = OnceLock::new();

/// 模拟 Runtime 按分配 ID 释放 Host 持有的测试输出。
unsafe extern "C" fn release_test_output(
    _session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    let allocation = TEST_ALLOCATIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&allocation.raw())
        .expect("test allocation must be released exactly once");
    if let Some(releases) = &allocation.releases {
        releases.fetch_add(1, Ordering::SeqCst);
    }
    if allocation.reject_release {
        ZrStatus::new(
            ZrStatusCode::Error,
            ZrByteSlice::from_static(RELEASE_DIAGNOSTIC),
        )
    } else {
        ZrStatus::ok()
    }
}

/// 将测试 session 与 fixture release callback 绑定。
fn test_releaser() -> RuntimeOwnedOutputReleaser {
    // The fixture registry and callback remain live for the complete test process.
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    unsafe { RuntimeOwnedOutputReleaser::new(ZrRuntimeSessionHandle::new(1), release_test_output) }
}

/// 注册字节 owner 并构造指向它的 ABI 结果 DTO。
fn owned_output(
    bytes: impl Into<Vec<u8>>,
    releases: Arc<AtomicUsize>,
    reject_release: bool,
) -> ZrOwnedResultV2 {
    let bytes = bytes.into().into_boxed_slice();
    let data = bytes.as_ptr();
    let len = bytes.len() as u64;
    let allocation = ZrRuntimeAllocationId::new(NEXT_ALLOCATION_ID.fetch_add(1, Ordering::Relaxed));
    TEST_ALLOCATIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(
            allocation.raw(),
            TestAllocation {
                _bytes: bytes,
                releases: Some(releases),
                reject_release,
            },
        );
    ZrOwnedResultV2 {
        data,
        len,
        allocation,
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
/// 用于验证 JSON 解码路径的小型 typed payload。
struct TestPayload {
    values: Vec<u64>,
}

#[derive(Debug, Deserialize)]
/// 性能样本使用的 256 项 JSON payload。
struct BenchmarkPayload {
    values: Vec<u32>,
}

/// 通过真实测试释放实现记录独立性能 allocation 的释放。
unsafe extern "C" fn release_benchmark_output(
    session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    unsafe { release_test_output(session, allocation) }
}

/// 为整个性能样本共享有效的测试 provider 回调。
fn benchmark_releaser() -> RuntimeOwnedOutputReleaser {
    // The fixture registry and callback remain live for the complete benchmark.
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    unsafe {
        RuntimeOwnedOutputReleaser::new(ZrRuntimeSessionHandle::new(1), release_benchmark_output)
    }
}

/// 将性能样本字节保存在 registry 并返回其 ABI 描述。
fn benchmark_output(bytes: Vec<u8>) -> ZrOwnedResultV2 {
    let bytes = bytes.into_boxed_slice();
    let data = bytes.as_ptr();
    let len = bytes.len() as u64;
    let allocation = ZrRuntimeAllocationId::new(NEXT_ALLOCATION_ID.fetch_add(1, Ordering::Relaxed));
    TEST_ALLOCATIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(
            allocation.raw(),
            TestAllocation {
                _bytes: bytes,
                releases: None,
                reject_release: false,
            },
        );
    ZrOwnedResultV2 {
        data,
        len,
        allocation,
    }
}

/// 使用固定 25ms 时限建立边界测试预算。
fn test_budget(max_bytes: usize, max_items: usize) -> RuntimeForeignOutputBudget {
    RuntimeForeignOutputBudget::new(max_bytes, max_items, Duration::from_millis(25))
}

/// 构造含指定文件数并可追加未知字段的 profile JSON。
fn profile_files_payload(file_count: usize, extra_fields: &str) -> Vec<u8> {
    let mut files = String::with_capacity(file_count.saturating_mul(3));
    for index in 0..file_count {
        if index != 0 {
            files.push(',');
        }
        files.push_str("\"\"");
    }
    format!("{{\"status\":\"ok\",\"message\":\"complete\",\"files\":[{files}]{extra_fields}}}")
        .into_bytes()
}

/// 构造含指定 hotspot hints 数并可追加未知字段的 profile JSON。
fn profile_hotspot_hints_payload(hint_count: usize, extra_fields: &str) -> Vec<u8> {
    let mut hints = String::with_capacity(hint_count.saturating_mul(3));
    for index in 0..hint_count {
        if index != 0 {
            hints.push(',');
        }
        hints.push_str("\"\"");
    }
    format!(
        "{{\"status\":\"ok\",\"message\":\"complete\",\"hotspot_report\":{{\"session_id\":\"profile\",\"frame_budget_ms\":16.0,\"generated_from_span_count\":0,\"hotspots\":[],\"hints\":[{hints}]}}{extra_fields}}}"
    )
    .into_bytes()
}

#[test]
/// 确认 WorldQueryResult 每种 Interface 变体都映射为预期预算项数。
fn world_query_item_count_covers_every_result_variant() {
    use std::collections::BTreeMap;

    use zircon_runtime_interface::math::Transform;
    use zircon_runtime_interface::reflect::ReflectedValue;
    use zircon_runtime_interface::world_sync::{
        EntityRow, WorldHierarchyRow, WorldInspectionFieldRow, WorldQueryResult,
    };

    let mut components = BTreeMap::new();
    components.insert(
        "Transform".to_string(),
        serde_json::json!({ "translation": [0, 0, 0] }),
    );
    let results = [
        (
            WorldQueryResult::ComponentRows {
                generation: 11,
                rows: vec![EntityRow {
                    entity: 7,
                    components,
                }],
            },
            6,
        ),
        (
            WorldQueryResult::HierarchyRows {
                generation: 12,
                rows: vec![WorldHierarchyRow {
                    entity: 7,
                    parent: None,
                    depth: 0,
                    display_name: "Root".to_string(),
                    kind: "entity".to_string(),
                    subtree_hash: 41,
                    active_in_hierarchy: true,
                    has_children: false,
                }],
            },
            1,
        ),
        (
            WorldQueryResult::InspectionFields {
                generation: 13,
                entity: 7,
                fields: vec![WorldInspectionFieldRow {
                    component_type_path: "Transform".to_string(),
                    component_display_name: "Transform".to_string(),
                    field_name: "translation".to_string(),
                    field_display_name: "Translation".to_string(),
                    value_type_path: "Vec3".to_string(),
                    value: ReflectedValue::Null,
                    writable: true,
                    serializable: true,
                    plugin_owned: false,
                }],
            },
            1,
        ),
        (
            WorldQueryResult::TransformSnapshot {
                generation: 14,
                world_replacement_epoch: 3,
                entity: 7,
                transform: Transform::identity(),
            },
            1,
        ),
        (
            WorldQueryResult::EntityMissing {
                generation: 15,
                entity: 7,
            },
            1,
        ),
        (WorldQueryResult::NotModified { generation: 16 }, 1),
    ];

    for (result, expected) in results {
        assert_eq!(
            super::world_query_item_count(&result),
            expected,
            "unexpected structural item count for {result:?}"
        );
    }
}

#[test]
/// 成功路径解码后只释放一次、递增接受指标且不熔断 session。
fn bounded_json_acceptance_releases_once_and_records_metrics() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let payload = unsafe {
        state
            .decode_json(
                owned_output(br#"{"values":[1,2,3]}"#.to_vec(), releases.clone(), false),
                test_releaser(),
                RuntimeForeignOutputKind::WorldQuery,
                test_budget(1024, 4),
                "decode runtime world query",
                "free runtime world query",
                |payload: &TestPayload| Ok::<usize, &'static str>(1 + payload.values.len()),
            )
            .expect("bounded payload should decode")
            .expect("non-empty payload should remain present")
    };

    assert_eq!(
        payload,
        TestPayload {
            values: vec![1, 2, 3]
        }
    );
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    let metrics = state
        .metrics()
        .for_kind(RuntimeForeignOutputKind::WorldQuery);
    assert_eq!(metrics.accepted_payloads, 1);
    assert_eq!(metrics.accepted_bytes, 18);
    assert!(!state.is_protocol_failed());
}

#[test]
/// 字节超限先清理分配，再熔断并阻断其后的类别与 session 调用。
fn oversized_output_releases_then_fuses_every_session_call() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state
            .decode_json::<TestPayload, _>(
                owned_output(br#"{"values":[1]}"#.to_vec(), releases.clone(), false),
                test_releaser(),
                RuntimeForeignOutputKind::WorldQuery,
                test_budget(4, 8),
                "decode runtime world query",
                "free runtime world query",
                |_| Ok::<usize, &'static str>(2),
            )
            .expect_err("oversized foreign output must fail")
    };

    assert_eq!(
        error.kind(),
        RuntimeForeignOutputErrorKind::ProtocolViolation
    );
    assert!(error.to_string().contains("maximum is 4"));
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
    assert!(state
        .ensure_available(RuntimeForeignOutputKind::ProfileResponse)
        .unwrap_err()
        .to_string()
        .contains("prior foreign-output protocol violation"));
    assert!(state
        .ensure_session_available("tick runtime frame")
        .unwrap_err()
        .to_string()
        .contains("tick runtime frame"));
}

#[test]
/// 在预算明确允许时将规范空结果接受为 None 且不调用 release。
fn canonical_empty_pages_are_accepted_without_release() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let decoded = unsafe {
        state
            .decode_json::<Vec<u64>, _>(
                ZrOwnedResultV2::empty(),
                test_releaser(),
                RuntimeForeignOutputKind::WorldInvalidations,
                test_budget(1024, 8).allow_empty(),
                "drain runtime world invalidations",
                "free runtime world invalidations",
                |values| Ok::<usize, &'static str>(values.len()),
            )
            .expect("empty page is an allowed protocol outcome")
    };

    assert_eq!(decoded, None);
    assert_eq!(releases.load(Ordering::SeqCst), 0);
    assert!(!state.is_protocol_failed());
}

#[test]
/// 释放失败覆盖已解析值，保留 cleanup 诊断并熔断状态。
fn release_failure_preserves_cleanup_diagnostic_and_fuses_session() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state
            .decode_json(
                owned_output(br#"{"values":[1]}"#.to_vec(), releases.clone(), true),
                test_releaser(),
                RuntimeForeignOutputKind::OperationResult,
                test_budget(1024, 8),
                "harvest runtime operation",
                "free runtime operation output",
                |payload: &TestPayload| Ok::<usize, &'static str>(1 + payload.values.len()),
            )
            .expect_err("failed foreign release must reject the decoded value")
    };

    assert!(error.to_string().contains("cleanup failed"));
    assert!(error
        .to_string()
        .contains("test allocation is still in use"));
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
}

#[test]
/// 同时验证 Interface 最大嵌套深度与解码/业务校验的总时间预算。
fn nesting_and_total_decode_time_are_both_bounded() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let nested = format!("{}0{}", "[".repeat(129), "]".repeat(129));
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let depth_error = unsafe {
        state
            .decode_json::<serde_json::Value, _>(
                owned_output(nested.into_bytes(), releases.clone(), false),
                test_releaser(),
                RuntimeForeignOutputKind::ProfileResponse,
                test_budget(4096, 512),
                "decode runtime profile response",
                "free runtime profile response",
                |_| Ok::<usize, &'static str>(1),
            )
            .expect_err("payloads deeper than the shared limit must fail")
    };
    assert!(depth_error
        .to_string()
        .contains("maximum nesting depth 128"));
    assert_eq!(releases.load(Ordering::SeqCst), 1);

    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let time_error = unsafe {
        state
            .decode_json(
                owned_output(br#"{"values":[1]}"#.to_vec(), releases.clone(), false),
                test_releaser(),
                RuntimeForeignOutputKind::ProfileResponse,
                RuntimeForeignOutputBudget::new(1024, 8, Duration::from_millis(1)),
                "decode runtime profile response",
                "free runtime profile response",
                |payload: &TestPayload| {
                    std::thread::sleep(Duration::from_millis(3));
                    Ok::<usize, &'static str>(1 + payload.values.len())
                },
            )
            .expect_err("validation time belongs to the decode budget")
    };
    assert!(time_error.to_string().contains("decode time budget"));
    assert_eq!(releases.load(Ordering::SeqCst), 1);
}

#[test]
/// 不可表示的 Instant 截止时刻必须释放输出、熔断并返回错误而不 panic。
fn unrepresentable_decode_deadline_releases_once_and_fuses_without_unwinding() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let attempt = std::panic::catch_unwind(|| unsafe {
        state.decode_json::<TestPayload, _>(
            owned_output(br#"{"values":[1]}"#.to_vec(), releases.clone(), false),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            RuntimeForeignOutputBudget::new(1024, 8, Duration::MAX),
            "decode runtime profile response",
            "free runtime profile response",
            |_| Ok::<usize, &'static str>(2),
        )
    });
    let error = attempt
        .expect("an unrepresentable decode deadline must not unwind")
        .expect_err("an unrepresentable decode deadline must be rejected");

    assert_eq!(
        error.kind(),
        RuntimeForeignOutputErrorKind::ProtocolViolation
    );
    assert!(error
        .to_string()
        .contains("deadline exceeds the host clock range"));
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
    let metrics = state
        .metrics()
        .for_kind(RuntimeForeignOutputKind::ProfileResponse);
    assert_eq!(metrics.accepted_payloads, 0);
    assert_eq!(metrics.rejected_payloads, 1);
}

#[test]
/// `files` 恰好达到专用类型化上限时接受 profile 响应。
fn profile_files_preflight_accepts_the_exact_typed_limit() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let file_limit = PROFILE_RESPONSE_OUTPUT_BUDGET.max_items.saturating_sub(1);
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let response = unsafe {
        state.decode_json::<ProfileControlResponse, &'static str>(
            owned_output(
                profile_files_payload(file_limit, ""),
                releases.clone(),
                false,
            ),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            PROFILE_RESPONSE_OUTPUT_BUDGET,
            "decode runtime profile response",
            "free runtime profile response",
            |response| Ok(profile_control_response_item_count(response)),
        )
    }
    .expect("the exact profile files limit must be accepted")
    .expect("the profile response is non-empty");

    assert_eq!(response.files.len(), file_limit);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(!state.is_protocol_failed());
}

#[test]
/// profile 文件数超过上限一项时，在构造业务 DTO 前拒绝并释放输出。
fn profile_files_preflight_rejects_limit_plus_one_before_typed_allocation() {
    BUSINESS_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let file_limit = PROFILE_RESPONSE_OUTPUT_BUDGET.max_items.saturating_sub(1);
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state.decode_json::<BusinessDeserializeProbe, &'static str>(
            owned_output(
                profile_files_payload(file_limit.saturating_add(1), ""),
                releases.clone(),
                false,
            ),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            PROFILE_RESPONSE_OUTPUT_BUDGET,
            "decode runtime profile response",
            "free runtime profile response",
            |_| Ok(1),
        )
    }
    .expect_err("profile files above the typed limit must be rejected");

    assert!(error
        .to_string()
        .contains("65536 profile files; maximum is 65535"));
    assert_eq!(BUSINESS_DESERIALIZE_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
}

#[test]
/// 未知嵌套字段里的同名 files 与大标量不套用 profile files 集合计数。
fn profile_files_preflight_ignores_nested_names_and_large_scalar_fields() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let budget = RuntimeForeignOutputBudget::new(128 * 1024, 2, Duration::from_secs(1))
        .preflight_profile_files();
    let large_unknown = "x".repeat(64 * 1024);
    let extra = format!(",\"future\":{{\"files\":[0,1,2,3],\"large\":\"{large_unknown}\"}}");
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let response = unsafe {
        state.decode_json::<ProfileControlResponse, &'static str>(
            owned_output(profile_files_payload(0, &extra), releases.clone(), false),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            budget,
            "decode runtime profile response",
            "free runtime profile response",
            |response| Ok(profile_control_response_item_count(response)),
        )
    }
    .expect("nested names and legitimate large scalar fields must remain valid")
    .expect("the profile response is non-empty");

    assert!(response.files.is_empty());
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(!state.is_protocol_failed());
}

#[test]
/// hotspot hints 越过类型化项预算时在 DTO 反序列化前拒绝。
fn profile_hotspot_hints_preflight_rejects_before_typed_deserialization() {
    PROFILE_RESPONSE_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let budget =
        RuntimeForeignOutputBudget::new(2_048, 3, Duration::from_secs(1)).preflight_profile_files();
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state.decode_json::<ProfileResponseDeserializeProbe, &'static str>(
            owned_output(
                profile_hotspot_hints_payload(3, ""),
                releases.clone(),
                false,
            ),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            budget,
            "decode runtime profile response",
            "free runtime profile response",
            |_| Ok(1),
        )
    }
    .expect_err("profile hints above the typed item limit must be rejected in preflight");

    assert!(error.to_string().contains("4 profile items; maximum is 3"));
    assert_eq!(PROFILE_RESPONSE_DESERIALIZE_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
}

#[test]
/// hints 达到边界时接受，并忽略未知字段内的同名嵌套集合。
fn profile_hotspot_hints_preflight_accepts_typed_boundary_and_ignores_unknown_nesting() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let budget =
        RuntimeForeignOutputBudget::new(2_048, 3, Duration::from_secs(1)).preflight_profile_files();
    let extra = ",\"future\":{\"hotspot_report\":{\"hints\":[0,1,2,3]}}";
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let response = unsafe {
        state.decode_json::<ProfileControlResponse, &'static str>(
            owned_output(
                profile_hotspot_hints_payload(2, extra),
                releases.clone(),
                false,
            ),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            budget,
            "decode runtime profile response",
            "free runtime profile response",
            |response| Ok(profile_control_response_item_count(response)),
        )
    }
    .expect("the exact typed item limit and unknown nested fields must be accepted")
    .expect("the profile response is non-empty");

    assert_eq!(response.hotspot_report.unwrap().hints.len(), 2);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(!state.is_protocol_failed());
}

#[test]
/// profile 多类已知集合共享一个总预算而非分别获得独立上限。
fn profile_collections_share_one_preflight_item_budget() {
    use zircon_runtime_interface::HotspotReport;

    PROFILE_RESPONSE_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let mut response = ProfileControlResponse::ok("profile");
    response.files.push("report.json".to_owned());
    let mut report = HotspotReport::default();
    report.hints = vec!["first".to_owned(), "second".to_owned()];
    response.hotspot_report = Some(report);
    let encoded = serde_json::to_vec(&response).expect("encode a valid profile response");
    let budget =
        RuntimeForeignOutputBudget::new(2_048, 3, Duration::from_secs(1)).preflight_profile_files();

    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state.decode_json::<ProfileResponseDeserializeProbe, &'static str>(
            owned_output(encoded, releases.clone(), false),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            budget,
            "decode runtime profile response",
            "free runtime profile response",
            |_| Ok(1),
        )
    }
    .expect_err("combined profile collections must share the typed item limit");

    assert!(error.to_string().contains("4 profile items; maximum is 3"));
    assert_eq!(PROFILE_RESPONSE_DESERIALIZE_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
}

#[test]
/// 在 typed profile DTO 构造前限制快照中的嵌套集合。
fn profile_snapshot_collection_preflights_before_typed_deserialization() {
    use zircon_runtime_interface::ProfileSnapshot;

    PROFILE_RESPONSE_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let mut response = ProfileControlResponse::ok("profile");
    let mut snapshot = ProfileSnapshot::default();
    snapshot.recorder_retention.push(Default::default());
    response.snapshot = Some(snapshot);
    let encoded = serde_json::to_vec(&response).expect("encode a valid profile response");
    let budget =
        RuntimeForeignOutputBudget::new(2_048, 1, Duration::from_secs(1)).preflight_profile_files();

    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state.decode_json::<ProfileResponseDeserializeProbe, &'static str>(
            owned_output(encoded, releases.clone(), false),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            budget,
            "decode runtime profile response",
            "free runtime profile response",
            |_| Ok(1),
        )
    }
    .expect_err("nested profile snapshot collections must be bounded before typed decode");

    assert!(error.to_string().contains("2 profile items; maximum is 1"));
    assert_eq!(PROFILE_RESPONSE_DESERIALIZE_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
}

#[test]
/// Interface DTO 的 positional 序列形式同样受 profile 集合预检约束。
fn positional_profile_collections_preflight_before_typed_deserialization() {
    PROFILE_RESPONSE_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let encoded =
        br#"["ok","complete",null,null,null,["profile",16.0,0,[],["","",""]],null,null,null,[]]"#;
    let decoded: ProfileControlResponse =
        serde_json::from_slice(encoded).expect("the positional fixture is accepted by typed serde");
    assert_eq!(decoded.hotspot_report.unwrap().hints.len(), 3);
    let budget =
        RuntimeForeignOutputBudget::new(1_024, 3, Duration::from_secs(1)).preflight_profile_files();

    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state.decode_json::<ProfileResponseDeserializeProbe, &'static str>(
            owned_output(encoded.to_vec(), releases.clone(), false),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            budget,
            "decode runtime profile response",
            "free runtime profile response",
            |_| Ok(1),
        )
    }
    .expect_err("positional profile collections must not bypass item preflight");

    assert!(error.to_string().contains("4 profile items; maximum is 3"));
    assert_eq!(PROFILE_RESPONSE_DESERIALIZE_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
}

#[test]
/// positional profile 数据在类型化预算边界上与 object 形式得到等价 DTO。
fn positional_profile_collections_accept_typed_boundary_with_object_parity() {
    use zircon_runtime_interface::HotspotReport;

    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let encoded =
        br#"["ok","complete",null,null,null,["profile",16.0,0,[],["",""]],null,null,null,[]]"#;
    let expected: ProfileControlResponse =
        serde_json::from_slice(encoded).expect("the positional fixture is accepted by typed serde");
    let mut object = ProfileControlResponse::ok("complete");
    let mut report = HotspotReport::default();
    report.session_id = "profile".to_owned();
    report.frame_budget_ms = 16.0;
    report.hints = vec![String::new(), String::new()];
    object.hotspot_report = Some(report);
    assert_eq!(expected, object);
    assert_eq!(profile_control_response_item_count(&expected), 3);
    let budget =
        RuntimeForeignOutputBudget::new(1_024, 3, Duration::from_secs(1)).preflight_profile_files();

    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let response = unsafe {
        state.decode_json::<ProfileControlResponse, &'static str>(
            owned_output(encoded.to_vec(), releases.clone(), false),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            budget,
            "decode runtime profile response",
            "free runtime profile response",
            |response| Ok(profile_control_response_item_count(response)),
        )
    }
    .expect("positional profile collections at the typed boundary must be accepted")
    .expect("the profile response is non-empty");

    assert_eq!(response, expected);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(!state.is_protocol_failed());
}

#[test]
/// positional 嵌套快照和诊断序列的子集合会计入同一 profile 项数。
fn positional_snapshot_and_diagnostic_series_preflight_nested_items() {
    use zircon_runtime_interface::{
        ProfileRecorderRetentionSnapshot, ProfileSnapshot, RuntimeInputDiagnosticsSnapshot,
    };

    PROFILE_RESPONSE_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let retention = serde_json::to_value(ProfileRecorderRetentionSnapshot::default())
        .expect("encode profile retention");
    let input = serde_json::to_value(RuntimeInputDiagnosticsSnapshot::default())
        .expect("encode runtime input diagnostics");
    let profile =
        serde_json::to_value(ProfileSnapshot::default()).expect("encode nested profile snapshot");
    let snapshot = serde_json::json!([
        "profile",
        "root",
        false,
        false,
        16.0,
        [],
        [],
        [],
        [retention]
    ]);
    let series = serde_json::json!([
        "series",
        null,
        ["runtime"],
        null,
        null,
        null,
        null,
        [{ "frame_index": 0, "value": 0.0 }]
    ]);
    let diagnostics = serde_json::json!([
        0,
        null,
        null,
        null,
        null,
        null,
        null,
        input,
        [series],
        null,
        profile
    ]);
    let encoded = serde_json::to_vec(&serde_json::json!([
        "ok",
        "complete",
        snapshot,
        diagnostics,
        null,
        null,
        null,
        null,
        null,
        []
    ]))
    .expect("encode positional profile response");
    let decoded: ProfileControlResponse =
        serde_json::from_slice(&encoded).expect("the positional response is valid for typed serde");
    assert_eq!(profile_control_response_item_count(&decoded), 5);
    let budget =
        RuntimeForeignOutputBudget::new(4_096, 4, Duration::from_secs(1)).preflight_profile_files();

    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state.decode_json::<ProfileResponseDeserializeProbe, &'static str>(
            owned_output(encoded, releases.clone(), false),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            budget,
            "decode runtime profile response",
            "free runtime profile response",
            |_| Ok(1),
        )
    }
    .expect_err("nested positional collections must be bounded before typed decode");

    assert!(error.to_string().contains("5 profile items; maximum is 4"));
    assert_eq!(PROFILE_RESPONSE_DESERIALIZE_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
}

#[test]
/// 零项预算连 profile 响应外壳本身也会拒绝，并且尚未调用 DTO decoder。
fn zero_profile_item_budget_rejects_the_envelope_before_typed_deserialization() {
    PROFILE_RESPONSE_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let budget =
        RuntimeForeignOutputBudget::new(1_024, 0, Duration::from_secs(1)).preflight_profile_files();

    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state.decode_json::<ProfileResponseDeserializeProbe, &'static str>(
            owned_output(profile_files_payload(0, ""), releases.clone(), false),
            test_releaser(),
            RuntimeForeignOutputKind::ProfileResponse,
            budget,
            "decode runtime profile response",
            "free runtime profile response",
            |_| Ok(1),
        )
    }
    .expect_err("the response envelope consumes one profile item");

    assert!(error.to_string().contains("1 profile items; maximum is 0"));
    assert_eq!(PROFILE_RESPONSE_DESERIALIZE_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(state.is_protocol_failed());
}

#[test]
/// 普通 Runtime 调用失败会释放返回输出，但不会仅因调用失败触发协议 fuse。
fn ordinary_call_failure_releases_output_without_fusing_protocol() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let error = unsafe {
        state
            .ensure_call_succeeded(
                ZrStatus::new(
                    ZrStatusCode::Error,
                    ZrByteSlice::from_static(CALL_DIAGNOSTIC),
                ),
                owned_output(vec![0_u8], releases.clone(), false),
                test_releaser(),
                RuntimeForeignOutputKind::HostRequests,
                "drain runtime host requests",
                "free runtime host requests",
            )
            .expect_err("runtime call failure must propagate")
    };

    assert_eq!(error.kind(), RuntimeForeignOutputErrorKind::RuntimeCall);
    assert!(error.to_string().contains("test call failed"));
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(!state.is_protocol_failed());
}

#[test]
/// 在解码校验进行时提交并发协议违规，会阻止在途结果随后被接受。
fn concurrent_protocol_rejection_prevents_inflight_acceptance() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = Arc::new(RuntimeForeignOutputState::default());
    let validation_entered = Arc::new(Barrier::new(2));
    let validation_may_finish = Arc::new(Barrier::new(2));

    let decode_state = state.clone();
    let decode_releases = releases.clone();
    let decode_entered = validation_entered.clone();
    let decode_may_finish = validation_may_finish.clone();
    // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
    let decode = std::thread::spawn(move || unsafe {
        decode_state.decode_json(
            owned_output(br#"{"values":[1,2,3]}"#.to_vec(), decode_releases, false),
            test_releaser(),
            RuntimeForeignOutputKind::WorldQuery,
            RuntimeForeignOutputBudget::new(1024, 8, Duration::from_secs(5)),
            "decode concurrent runtime world query",
            "free concurrent runtime world query",
            |payload: &TestPayload| {
                decode_entered.wait();
                decode_may_finish.wait();
                Ok::<usize, &'static str>(payload.values.len())
            },
        )
    });

    validation_entered.wait();
    state
        .reject_protocol::<()>(
            RuntimeForeignOutputKind::WorldInvalidations,
            "concurrent invalidation violated the protocol",
        )
        .expect_err("the competing protocol violation must fuse the session");
    validation_may_finish.wait();

    let error = decode
        .join()
        .expect("the inflight decoder thread must complete")
        .expect_err("an inflight result must not be accepted after the session fuses");
    assert!(error
        .to_string()
        .contains("prior foreign-output protocol violation"));
    assert_eq!(releases.load(Ordering::SeqCst), 1);

    let metrics = state.metrics();
    assert_eq!(metrics.protocol_failures, 1);
    let query = metrics.for_kind(RuntimeForeignOutputKind::WorldQuery);
    assert_eq!(query.accepted_payloads, 0);
    assert_eq!(query.rejected_payloads, 1);
}

#[test]
/// 对共享 Host 请求预算执行 warmup/样本循环并核对计数和 P99 时限边界。
fn foreign_output_decode_performance_acceptance() {
    const WARMUP_ITERATIONS: usize = 64;
    const MEASURED_ITERATIONS: usize = 2_000;

    let payload = serde_json::to_vec(&serde_json::json!({
        "values": (0_u32..256).collect::<Vec<_>>()
    }))
    .expect("serialize benchmark payload");
    let warmup = RuntimeForeignOutputState::default();
    for _ in 0..WARMUP_ITERATIONS {
        // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
        let decoded = unsafe {
            warmup
                .decode_json(
                    benchmark_output(payload.clone()),
                    benchmark_releaser(),
                    RuntimeForeignOutputKind::HostRequests,
                    super::HOST_REQUEST_OUTPUT_BUDGET,
                    "decode benchmark host requests",
                    "free benchmark host requests",
                    |decoded: &BenchmarkPayload| Ok::<usize, &'static str>(decoded.values.len()),
                )
                .expect("warmup payload must remain within the shared budget")
        };
        std::hint::black_box(decoded);
    }

    let state = RuntimeForeignOutputState::default();
    let mut samples = Vec::with_capacity(MEASURED_ITERATIONS);
    for _ in 0..MEASURED_ITERATIONS {
        let output = benchmark_output(payload.clone());
        let started = Instant::now();
        // SAFETY: 测试夹具绑定仍存活的 registry callback/session；非空输出字节由 registry 持有至释放，规范空输出不读取指针且不调用释放回调。
        let decoded = unsafe {
            state
                .decode_json(
                    output,
                    benchmark_releaser(),
                    RuntimeForeignOutputKind::HostRequests,
                    super::HOST_REQUEST_OUTPUT_BUDGET,
                    "decode benchmark host requests",
                    "free benchmark host requests",
                    |decoded: &BenchmarkPayload| Ok::<usize, &'static str>(decoded.values.len()),
                )
                .expect("benchmark payload must remain within the shared budget")
        };
        samples.push(started.elapsed().as_nanos());
        std::hint::black_box(decoded);
    }
    samples.sort_unstable();

    let p50_ns = percentile_nanoseconds(&samples, 50);
    let p95_ns = percentile_nanoseconds(&samples, 95);
    let p99_ns = percentile_nanoseconds(&samples, 99);
    let total_ns = samples.iter().copied().sum::<u128>();
    let throughput = (MEASURED_ITERATIONS as f64) * 1_000_000_000.0 / total_ns as f64;
    println!(
        "RUNTIME_HOST_FOREIGN_OUTPUT_PERF iterations={MEASURED_ITERATIONS} encoded_bytes={} items=256 p50_ns={p50_ns} p95_ns={p95_ns} p99_ns={p99_ns} throughput_payloads_per_second={throughput:.0}",
        payload.len()
    );

    let metrics = state
        .metrics()
        .for_kind(RuntimeForeignOutputKind::HostRequests);
    assert_eq!(metrics.accepted_payloads, MEASURED_ITERATIONS as u64);
    assert_eq!(
        metrics.accepted_bytes,
        (payload.len() * MEASURED_ITERATIONS) as u64
    );
    assert_eq!(metrics.rejected_payloads, 0);
    assert!(
        p99_ns
            <= super::HOST_REQUEST_OUTPUT_BUDGET
                .max_decode_time()
                .as_nanos(),
        "p99 boundary latency {p99_ns}ns exceeded the shared 10ms host-request budget"
    );
}

#[test]
/// 每类共享 Host 预算的字节、条目、耗时、空页和嵌套值来自 Interface 常量。
fn foreign_output_policies_derive_from_interface_limits() {
    let policies = [
        (
            super::HOST_REQUEST_OUTPUT_BUDGET,
            zircon_runtime_interface::ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1,
        ),
        (
            super::PROFILE_RESPONSE_OUTPUT_BUDGET,
            zircon_runtime_interface::ZR_RUNTIME_PROFILE_RESPONSE_OUTPUT_LIMIT_V1,
        ),
        (
            super::OPERATION_RESULT_OUTPUT_BUDGET,
            zircon_runtime_interface::ZR_RUNTIME_OPERATION_RESULT_OUTPUT_LIMIT_V1,
        ),
        (
            super::PLUGIN_EVENT_OUTPUT_BUDGET,
            zircon_runtime_interface::ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1,
        ),
        (
            super::WORLD_QUERY_OUTPUT_BUDGET,
            zircon_runtime_interface::ZR_RUNTIME_WORLD_QUERY_OUTPUT_LIMIT_V1,
        ),
        (
            super::WORLD_INVALIDATION_OUTPUT_BUDGET,
            zircon_runtime_interface::ZR_RUNTIME_WORLD_INVALIDATION_OUTPUT_LIMIT_V1,
        ),
    ];

    for (policy, interface) in policies {
        assert_eq!(policy.max_encoded_bytes, interface.max_encoded_bytes);
        assert_eq!(policy.max_items, interface.max_items);
        assert_eq!(
            policy.max_decode_time.as_micros(),
            u128::from(interface.max_processing_time_micros)
        );
        assert_eq!(policy.allow_empty, interface.allow_empty);
        assert_eq!(
            super::RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH,
            interface.max_nesting_depth
        );
    }
}

/// 按样本数和请求百分位数选择排序后的纳秒值。
fn percentile_nanoseconds(samples: &[u128], percentile: usize) -> u128 {
    let rank = samples
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    samples[rank]
}
