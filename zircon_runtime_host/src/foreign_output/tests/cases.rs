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
static BUSINESS_DESERIALIZE_CALLS: AtomicUsize = AtomicUsize::new(0);
static PROFILE_RESPONSE_DESERIALIZE_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
struct BusinessDeserializeProbe;

impl<'de> Deserialize<'de> for BusinessDeserializeProbe {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BUSINESS_DESERIALIZE_CALLS.fetch_add(1, Ordering::SeqCst);
        IgnoredAny::deserialize(deserializer)?;
        Ok(Self)
    }
}

#[derive(Debug)]
struct ProfileResponseDeserializeProbe;

impl<'de> Deserialize<'de> for ProfileResponseDeserializeProbe {
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

struct TestAllocation {
    _bytes: Box<[u8]>,
    releases: Option<Arc<AtomicUsize>>,
    reject_release: bool,
}

static NEXT_ALLOCATION_ID: AtomicU64 = AtomicU64::new(1);
static TEST_ALLOCATIONS: OnceLock<Mutex<HashMap<u64, TestAllocation>>> = OnceLock::new();

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

fn test_releaser() -> RuntimeOwnedOutputReleaser {
    // The fixture registry and callback remain live for the complete test process.
    unsafe { RuntimeOwnedOutputReleaser::new(ZrRuntimeSessionHandle::new(1), release_test_output) }
}

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
struct TestPayload {
    values: Vec<u64>,
}

#[derive(Debug, Deserialize)]
struct BenchmarkPayload {
    values: Vec<u32>,
}

unsafe extern "C" fn release_benchmark_output(
    session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    unsafe { release_test_output(session, allocation) }
}

fn benchmark_releaser() -> RuntimeOwnedOutputReleaser {
    // The fixture registry and callback remain live for the complete benchmark.
    unsafe {
        RuntimeOwnedOutputReleaser::new(ZrRuntimeSessionHandle::new(1), release_benchmark_output)
    }
}

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

fn test_budget(max_bytes: usize, max_items: usize) -> RuntimeForeignOutputBudget {
    RuntimeForeignOutputBudget::new(max_bytes, max_items, Duration::from_millis(25))
}

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
fn bounded_json_acceptance_releases_once_and_records_metrics() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
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
fn oversized_output_releases_then_fuses_every_session_call() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
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
fn canonical_empty_pages_are_accepted_without_release() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
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
fn release_failure_preserves_cleanup_diagnostic_and_fuses_session() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
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
fn nesting_and_total_decode_time_are_both_bounded() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let nested = format!("{}0{}", "[".repeat(129), "]".repeat(129));
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
fn unrepresentable_decode_deadline_releases_once_and_fuses_without_unwinding() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
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
fn profile_files_preflight_accepts_the_exact_typed_limit() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let file_limit = PROFILE_RESPONSE_OUTPUT_BUDGET.max_items.saturating_sub(1);
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
fn profile_files_preflight_rejects_limit_plus_one_before_typed_allocation() {
    BUSINESS_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let file_limit = PROFILE_RESPONSE_OUTPUT_BUDGET.max_items.saturating_sub(1);
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
fn profile_files_preflight_ignores_nested_names_and_large_scalar_fields() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let budget = RuntimeForeignOutputBudget::new(128 * 1024, 2, Duration::from_secs(1))
        .preflight_profile_files();
    let large_unknown = "x".repeat(64 * 1024);
    let extra = format!(",\"future\":{{\"files\":[0,1,2,3],\"large\":\"{large_unknown}\"}}");
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
fn profile_hotspot_hints_preflight_rejects_before_typed_deserialization() {
    PROFILE_RESPONSE_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let budget =
        RuntimeForeignOutputBudget::new(2_048, 3, Duration::from_secs(1)).preflight_profile_files();
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
fn profile_hotspot_hints_preflight_accepts_typed_boundary_and_ignores_unknown_nesting() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let budget =
        RuntimeForeignOutputBudget::new(2_048, 3, Duration::from_secs(1)).preflight_profile_files();
    let extra = ",\"future\":{\"hotspot_report\":{\"hints\":[0,1,2,3]}}";
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
fn zero_profile_item_budget_rejects_the_envelope_before_typed_deserialization() {
    PROFILE_RESPONSE_DESERIALIZE_CALLS.store(0, Ordering::SeqCst);
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let budget =
        RuntimeForeignOutputBudget::new(1_024, 0, Duration::from_secs(1)).preflight_profile_files();

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
fn ordinary_call_failure_releases_output_without_fusing_protocol() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
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
fn concurrent_protocol_rejection_prevents_inflight_acceptance() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = Arc::new(RuntimeForeignOutputState::default());
    let validation_entered = Arc::new(Barrier::new(2));
    let validation_may_finish = Arc::new(Barrier::new(2));

    let decode_state = state.clone();
    let decode_releases = releases.clone();
    let decode_entered = validation_entered.clone();
    let decode_may_finish = validation_may_finish.clone();
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
fn foreign_output_decode_performance_acceptance() {
    const WARMUP_ITERATIONS: usize = 64;
    const MEASURED_ITERATIONS: usize = 2_000;

    let payload = serde_json::to_vec(&serde_json::json!({
        "values": (0_u32..256).collect::<Vec<_>>()
    }))
    .expect("serialize benchmark payload");
    let warmup = RuntimeForeignOutputState::default();
    for _ in 0..WARMUP_ITERATIONS {
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

fn percentile_nanoseconds(samples: &[u128], percentile: usize) -> u128 {
    let rank = samples
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    samples[rank]
}
