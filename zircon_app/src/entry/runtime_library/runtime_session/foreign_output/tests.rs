use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier, Mutex, OnceLock};
use std::time::Duration;

use serde::Deserialize;
use zircon_runtime_interface::{ZrByteSlice, ZrOwnedByteBuffer, ZrStatus, ZrStatusCode};

use super::{ForeignOutputBudget, ForeignOutputKind, ForeignOutputState};
use zircon_runtime_host::foreign_output::RuntimeForeignOutputErrorKind as RuntimeLibraryErrorKind;

static ACCEPTED_RELEASED: AtomicBool = AtomicBool::new(false);
static CALL_FAILURE_RELEASED: AtomicBool = AtomicBool::new(false);
static EMPTY_RELEASED: AtomicBool = AtomicBool::new(false);
static FAILED_OWNERSHIP_RELEASED: AtomicBool = AtomicBool::new(false);
static FUSED_RETURN_RELEASED: AtomicBool = AtomicBool::new(false);
static OVERSIZED_RELEASED: AtomicBool = AtomicBool::new(false);
static ITEM_LIMIT_RELEASED: AtomicBool = AtomicBool::new(false);
static MALFORMED_RELEASED: AtomicBool = AtomicBool::new(false);
static RACE_OUTER_RELEASED: AtomicBool = AtomicBool::new(false);
static RACE_TRIGGER_RELEASED: AtomicBool = AtomicBool::new(false);
static RELEASE_RACE: OnceLock<Mutex<Option<Arc<ReleaseRace>>>> = OnceLock::new();

struct ReleaseRace {
    entered: Barrier,
    resume: Barrier,
}

impl ReleaseRace {
    fn new() -> Self {
        Self {
            entered: Barrier::new(2),
            resume: Barrier::new(2),
        }
    }
}

const CALL_FAILURE_DIAGNOSTIC: &[u8] = b"foreign call rejected";
const RELEASE_FAILURE_DIAGNOSTIC: &[u8] = b"foreign allocation remained owned";

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct TestPayload {
    values: Vec<u32>,
}

unsafe extern "C" fn release_accepted(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    ACCEPTED_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_oversized(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    OVERSIZED_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_call_failure(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    CALL_FAILURE_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn reject_release(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    ZrStatus::new(
        ZrStatusCode::Error,
        ZrByteSlice::from_static(RELEASE_FAILURE_DIAGNOSTIC),
    )
}

unsafe extern "C" fn release_empty(output: ZrOwnedByteBuffer) -> ZrStatus {
    if !output.data.is_null() {
        unsafe {
            drop(Box::from_raw(output.data));
        }
    }
    EMPTY_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_fused_return(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    FUSED_RETURN_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_malformed(output: ZrOwnedByteBuffer) -> ZrStatus {
    if !output.data.is_null() {
        unsafe {
            drop(Box::from_raw(output.data));
        }
    }
    MALFORMED_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_failed_ownership(output: ZrOwnedByteBuffer) -> ZrStatus {
    if !output.data.is_null() {
        unsafe {
            drop(Box::from_raw(output.data));
        }
    }
    FAILED_OWNERSHIP_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_item_limited(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    ITEM_LIMIT_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_race_outer(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    RACE_OUTER_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_race_trigger(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    RACE_TRIGGER_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_while_another_call_fuses(output: ZrOwnedByteBuffer) -> ZrStatus {
    let synchronization = RELEASE_RACE
        .get()
        .expect("release-race synchronization should be installed")
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .expect("release-race synchronization should remain active")
        .clone();
    synchronization.entered.wait();
    synchronization.resume.wait();
    release_vec(output);
    ZrStatus::ok()
}

fn release_vec(output: ZrOwnedByteBuffer) {
    if !output.data.is_null() {
        unsafe {
            drop(Vec::from_raw_parts(
                output.data,
                output.len,
                output.capacity,
            ));
        }
    }
}

fn owned_json(
    value: &impl serde::Serialize,
    release: unsafe extern "C" fn(ZrOwnedByteBuffer) -> ZrStatus,
) -> ZrOwnedByteBuffer {
    owned_bytes(
        serde_json::to_vec(value).expect("serialize test payload"),
        release,
    )
}

fn owned_bytes(
    mut bytes: Vec<u8>,
    release: unsafe extern "C" fn(ZrOwnedByteBuffer) -> ZrStatus,
) -> ZrOwnedByteBuffer {
    let output = ZrOwnedByteBuffer {
        data: bytes.as_mut_ptr(),
        len: bytes.len(),
        capacity: bytes.capacity(),
        owner_token: 1,
        free: Some(release),
    };
    std::mem::forget(bytes);
    output
}

fn budget(max_bytes: usize, max_items: usize) -> ForeignOutputBudget {
    ForeignOutputBudget::new(max_bytes, max_items, Duration::from_millis(25))
}

#[test]
fn accepted_payload_is_released_and_records_per_kind_metrics() {
    ACCEPTED_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let output = owned_json(&serde_json::json!({ "values": [1, 2] }), release_accepted);
    let encoded_len = output.len as u64;

    let decoded = state
        .decode_json::<TestPayload, &'static str>(
            output,
            ForeignOutputKind::HostRequests,
            budget(1_024, 4),
            "decode test host requests",
            "free test host requests",
            |payload| Ok(payload.values.len()),
        )
        .expect("bounded payload should decode");

    assert_eq!(decoded, Some(TestPayload { values: vec![1, 2] }));
    assert!(ACCEPTED_RELEASED.load(Ordering::Acquire));
    assert!(!state.is_protocol_failed());
    let metrics = state.metrics().for_kind(ForeignOutputKind::HostRequests);
    assert_eq!(metrics.accepted_payloads, 1);
    assert_eq!(metrics.accepted_bytes, encoded_len);
    assert_eq!(metrics.rejected_payloads, 0);
    assert!(metrics.max_decode_nanoseconds > 0);
}

#[test]
fn failed_foreign_call_releases_output_without_fusing_when_cleanup_succeeds() {
    CALL_FAILURE_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let output = owned_json(&serde_json::json!({ "values": [1] }), release_call_failure);

    let error = state
        .ensure_call_succeeded(
            ZrStatus::new(
                ZrStatusCode::Error,
                ZrByteSlice::from_static(CALL_FAILURE_DIAGNOSTIC),
            ),
            output,
            ForeignOutputKind::ProfileResponse,
            "control test profiling",
            "free test profile response",
        )
        .expect_err("runtime call failure should propagate");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::RuntimeCall);
    assert!(CALL_FAILURE_RELEASED.load(Ordering::Acquire));
    assert!(!state.is_protocol_failed());
    let metrics = state.metrics().for_kind(ForeignOutputKind::ProfileResponse);
    assert_eq!(metrics.call_failures, 1);
    assert_eq!(metrics.rejected_payloads, 0);
}

#[test]
fn failed_call_cleanup_failure_fuses_with_both_diagnostics() {
    let state = ForeignOutputState::default();
    let output = owned_json(&serde_json::json!({ "values": [1] }), reject_release);

    let error = state
        .ensure_call_succeeded(
            ZrStatus::new(
                ZrStatusCode::Error,
                ZrByteSlice::from_static(CALL_FAILURE_DIAGNOSTIC),
            ),
            output,
            ForeignOutputKind::OperationResult,
            "harvest test operation",
            "free test operation output",
        )
        .expect_err("cleanup failure must turn a call failure into a protocol fault");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error.to_string().contains("foreign call rejected"));
    assert!(error
        .to_string()
        .contains("foreign allocation remained owned"));
    assert!(state.is_protocol_failed());
    let metrics = state.metrics().for_kind(ForeignOutputKind::OperationResult);
    assert_eq!(metrics.call_failures, 1);
    assert_eq!(metrics.rejected_payloads, 1);
}

#[test]
fn failed_call_with_invalid_output_ownership_releases_and_fuses() {
    FAILED_OWNERSHIP_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let output = ZrOwnedByteBuffer {
        data: Box::into_raw(Box::new(0_u8)),
        len: 2,
        capacity: 1,
        owner_token: 4,
        free: Some(release_failed_ownership),
    };

    let error = state
        .ensure_call_succeeded(
            ZrStatus::new(
                ZrStatusCode::Error,
                ZrByteSlice::from_static(CALL_FAILURE_DIAGNOSTIC),
            ),
            output,
            ForeignOutputKind::ProfileResponse,
            "control test profiling",
            "free failed test profile response",
        )
        .expect_err("invalid failure output ownership must fuse the session");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error.to_string().contains("invalid output ownership"));
    assert!(error.to_string().contains("len 2 exceeds capacity 1"));
    assert!(FAILED_OWNERSHIP_RELEASED.load(Ordering::Acquire));
    assert!(state.is_protocol_failed());
}

#[test]
fn empty_allowed_payload_is_released_without_decode() {
    EMPTY_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let output = ZrOwnedByteBuffer {
        data: Box::into_raw(Box::new(0_u8)),
        len: 0,
        capacity: 1,
        owner_token: 2,
        free: Some(release_empty),
    };

    let decoded = state
        .decode_json::<TestPayload, &'static str>(
            output,
            ForeignOutputKind::HostRequests,
            budget(1_024, 4).allow_empty(),
            "decode empty test host requests",
            "free empty test host requests",
            |_| Ok(0),
        )
        .expect("an empty page is a valid no-work result");

    assert_eq!(decoded, None);
    assert!(EMPTY_RELEASED.load(Ordering::Acquire));
    assert!(!state.is_protocol_failed());
}

#[test]
fn malformed_storage_is_released_before_protocol_failure() {
    MALFORMED_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let output = ZrOwnedByteBuffer {
        data: Box::into_raw(Box::new(0_u8)),
        len: 2,
        capacity: 1,
        owner_token: 3,
        free: Some(release_malformed),
    };

    let error = state
        .decode_json::<TestPayload, &'static str>(
            output,
            ForeignOutputKind::ProfileResponse,
            budget(1_024, 4),
            "decode malformed test profile response",
            "free malformed test profile response",
            |_| Ok(1),
        )
        .expect_err("malformed ownership metadata must fail before slicing");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error.to_string().contains("len 2 exceeds capacity 1"));
    assert!(MALFORMED_RELEASED.load(Ordering::Acquire));
    assert!(state.is_protocol_failed());
}

#[test]
fn oversized_payload_is_released_before_the_session_fuses() {
    OVERSIZED_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let output = owned_json(
        &serde_json::json!({ "values": [1, 2, 3] }),
        release_oversized,
    );
    let encoded_len = output.len;

    let error = state
        .decode_json::<TestPayload, &'static str>(
            output,
            ForeignOutputKind::OperationResult,
            budget(encoded_len - 1, 4),
            "decode test operation result",
            "free test operation result",
            |payload| Ok(payload.values.len()),
        )
        .expect_err("oversized foreign output must fail before decode");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error.to_string().contains("encoded bytes"));
    assert!(OVERSIZED_RELEASED.load(Ordering::Acquire));
    assert!(state.is_protocol_failed());
    assert!(state
        .ensure_available(ForeignOutputKind::ProfileResponse)
        .expect_err("a protocol failure must fuse later foreign calls")
        .to_string()
        .contains("prior foreign-output protocol violation"));
    let metrics = state.metrics().for_kind(ForeignOutputKind::OperationResult);
    assert_eq!(metrics.accepted_payloads, 0);
    assert_eq!(metrics.rejected_payloads, 1);
    let diagnostic = state
        .diagnostic_line()
        .expect("foreign output activity must be observable at teardown");
    assert!(diagnostic.contains("protocol_failed=true"));
    assert!(diagnostic.contains("operation_result.rejected_payloads=1"));
}

#[test]
fn buffer_returned_after_another_thread_fuses_is_still_released() {
    FUSED_RETURN_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let first = owned_json(&serde_json::json!({ "values": [1, 2] }), release_oversized);
    let first_len = first.len;
    let _ = state.decode_json::<TestPayload, &'static str>(
        first,
        ForeignOutputKind::OperationResult,
        budget(first_len - 1, 4),
        "decode first test operation",
        "free first test operation",
        |payload| Ok(payload.values.len()),
    );
    let returned_after_fuse =
        owned_json(&serde_json::json!({ "values": [3] }), release_fused_return);

    let error = state
        .decode_json::<TestPayload, &'static str>(
            returned_after_fuse,
            ForeignOutputKind::HostRequests,
            budget(1_024, 4),
            "decode raced test host requests",
            "free raced test host requests",
            |payload| Ok(payload.values.len()),
        )
        .expect_err("a raced return after the fuse must be discarded");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(FUSED_RETURN_RELEASED.load(Ordering::Acquire));
}

#[test]
fn decode_rechecks_the_session_fuse_after_schema_validation() {
    RACE_OUTER_RELEASED.store(false, Ordering::Release);
    RACE_TRIGGER_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let outer = owned_json(&serde_json::json!({ "values": [1, 2] }), release_race_outer);

    let error = state
        .decode_json::<TestPayload, &'static str>(
            outer,
            ForeignOutputKind::HostRequests,
            budget(1_024, 4),
            "decode raced outer host requests",
            "free raced outer host requests",
            |payload| {
                let trigger = owned_json(
                    &serde_json::json!({ "values": [3, 4] }),
                    release_race_trigger,
                );
                let trigger_len = trigger.len;
                let _ = state.decode_json::<TestPayload, &'static str>(
                    trigger,
                    ForeignOutputKind::OperationResult,
                    budget(trigger_len - 1, 4),
                    "decode raced fuse trigger",
                    "free raced fuse trigger",
                    |payload| Ok(payload.values.len()),
                );
                Ok(payload.values.len())
            },
        )
        .expect_err("a payload validated after the session fuses must be discarded");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error
        .to_string()
        .contains("prior foreign-output protocol violation"));
    assert!(RACE_TRIGGER_RELEASED.load(Ordering::Acquire));
    assert!(RACE_OUTER_RELEASED.load(Ordering::Acquire));
    let metrics = state.metrics().for_kind(ForeignOutputKind::HostRequests);
    assert_eq!(metrics.accepted_payloads, 0);
    assert_eq!(metrics.rejected_payloads, 1);
}

#[test]
fn payload_released_while_another_call_fuses_is_not_accepted() {
    let synchronization = Arc::new(ReleaseRace::new());
    *RELEASE_RACE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(synchronization.clone());
    let state = Arc::new(ForeignOutputState::default());

    std::thread::scope(|scope| {
        let state_for_decode = state.clone();
        let decoding = scope.spawn(move || {
            let output = owned_json(
                &serde_json::json!({ "values": [1, 2] }),
                release_while_another_call_fuses,
            );
            state_for_decode.decode_json::<TestPayload, &'static str>(
                output,
                ForeignOutputKind::HostRequests,
                budget(1_024, 4),
                "decode release-raced host requests",
                "free release-raced host requests",
                |payload| Ok(payload.values.len()),
            )
        });

        synchronization.entered.wait();
        let trigger = owned_json(
            &serde_json::json!({ "values": [3, 4] }),
            release_race_trigger,
        );
        let trigger_len = trigger.len;
        state
            .decode_json::<TestPayload, &'static str>(
                trigger,
                ForeignOutputKind::OperationResult,
                budget(trigger_len - 1, 4),
                "decode concurrent fuse trigger",
                "free concurrent fuse trigger",
                |payload| Ok(payload.values.len()),
            )
            .expect_err("the concurrent oversized payload must fuse the session");
        synchronization.resume.wait();

        let error = decoding
            .join()
            .expect("release-race decode thread should not panic")
            .expect_err("payload acceptance must recheck the serialized fuse state");
        assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
        assert!(error
            .to_string()
            .contains("prior foreign-output protocol violation"));
    });

    *RELEASE_RACE
        .get()
        .expect("release-race synchronization should exist")
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    let metrics = state.metrics().for_kind(ForeignOutputKind::HostRequests);
    assert_eq!(metrics.accepted_payloads, 0);
    assert_eq!(metrics.rejected_payloads, 1);
}

#[test]
fn item_budget_violation_releases_and_fuses_after_schema_decode() {
    ITEM_LIMIT_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let output = owned_json(
        &serde_json::json!({ "values": [1, 2, 3] }),
        release_item_limited,
    );

    let error = state
        .decode_json::<TestPayload, &'static str>(
            output,
            ForeignOutputKind::HostRequests,
            budget(1_024, 2),
            "decode test host requests",
            "free test host requests",
            |payload| Ok(payload.values.len()),
        )
        .expect_err("decoded item counts above the contract must fail");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error.to_string().contains("3 items; maximum is 2"));
    assert!(ITEM_LIMIT_RELEASED.load(Ordering::Acquire));
    assert!(state.is_protocol_failed());
}

#[test]
fn decode_time_budget_rejects_elapsed_work() {
    let budget = ForeignOutputBudget::new(1_024, 4, Duration::from_millis(5));

    let error = budget
        .validate_decode_duration(Duration::from_millis(6), "decode test output")
        .expect_err("decode work above the deadline must fail");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error.to_string().contains("decode time"));
}

#[test]
fn protocol_rejection_blocks_every_session_call() {
    let state = ForeignOutputState::default();
    let rejection: Result<(), _> = state.reject_protocol(
        ForeignOutputKind::OperationResult,
        crate::entry::runtime_library::RuntimeLibraryError::protocol_violation(
            "runtime returned invalid operation metadata",
        ),
    );

    assert!(rejection.is_err());
    let blocked = state
        .ensure_session_available("tick runtime frame")
        .expect_err("the entire session must remain fused");
    assert_eq!(blocked.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(blocked.to_string().contains("tick runtime frame"));
    assert_eq!(state.metrics().blocked_session_calls, 1);
}
