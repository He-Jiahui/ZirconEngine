use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Deserialize;
use zircon_runtime_host::foreign_output::RuntimeOwnedOutputReleaser;
use zircon_runtime_interface::{
    ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeReleaseAllocationFnV2, ZrRuntimeSessionHandle,
    ZrStatus,
};

use super::{
    ForeignOutputBudget, ForeignOutputKind, ForeignOutputState,
    FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH, HOST_REQUEST_OUTPUT_BUDGET,
};
use zircon_runtime_host::foreign_output::RuntimeForeignOutputErrorKind as RuntimeLibraryErrorKind;

static DEADLINE_RELEASED: AtomicBool = AtomicBool::new(false);
static DEADLINE_VALIDATOR_CALLED: AtomicBool = AtomicBool::new(false);
static NESTING_LIMIT_RELEASED: AtomicBool = AtomicBool::new(false);
static VALIDATION_TIME_RELEASED: AtomicBool = AtomicBool::new(false);
static NEXT_ALLOCATION_ID: AtomicU64 = AtomicU64::new(1);
static TEST_ALLOCATIONS: OnceLock<Mutex<HashMap<u64, Box<[u8]>>>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct TestPayload {
    values: Vec<u32>,
}

unsafe extern "C" fn release_deadline(
    _session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    release_vec(allocation);
    DEADLINE_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_nesting_limited(
    _session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    release_vec(allocation);
    NESTING_LIMIT_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_benchmark(
    _session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    release_vec(allocation);
    ZrStatus::ok()
}

unsafe extern "C" fn release_validation_timed(
    _session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    release_vec(allocation);
    VALIDATION_TIME_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

fn release_vec(allocation: ZrRuntimeAllocationId) {
    TEST_ALLOCATIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&allocation.raw())
        .expect("benchmark allocation must be released exactly once");
}

fn owned_json(
    value: &impl serde::Serialize,
    release: ZrRuntimeReleaseAllocationFnV2,
) -> ZrOwnedResultV2 {
    owned_bytes(
        serde_json::to_vec(value).expect("serialize test payload"),
        release,
    )
}

fn owned_bytes(bytes: Vec<u8>, _release: ZrRuntimeReleaseAllocationFnV2) -> ZrOwnedResultV2 {
    let bytes = bytes.into_boxed_slice();
    let data = bytes.as_ptr();
    let len = bytes.len() as u64;
    let allocation = ZrRuntimeAllocationId::new(NEXT_ALLOCATION_ID.fetch_add(1, Ordering::Relaxed));
    TEST_ALLOCATIONS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(allocation.raw(), bytes);
    ZrOwnedResultV2 {
        data,
        len,
        allocation,
    }
}

fn releaser(release: ZrRuntimeReleaseAllocationFnV2) -> RuntimeOwnedOutputReleaser {
    RuntimeOwnedOutputReleaser::new(ZrRuntimeSessionHandle::new(1), release)
}

#[test]
fn decode_deadline_interrupts_parsing_before_schema_validation() {
    DEADLINE_RELEASED.store(false, Ordering::Release);
    DEADLINE_VALIDATOR_CALLED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let output = owned_json(
        &serde_json::json!({ "values": [1, 2, 3] }),
        release_deadline,
    );
    let budget = ForeignOutputBudget::new(1_024, 4, Duration::ZERO);

    let error = state
        .decode_json::<TestPayload, &'static str>(
            output,
            releaser(release_deadline),
            ForeignOutputKind::HostRequests,
            budget,
            "decode deadline test host requests",
            "free deadline test host requests",
            |_| {
                DEADLINE_VALIDATOR_CALLED.store(true, Ordering::Release);
                Ok(3)
            },
        )
        .expect_err("an expired deadline must interrupt parsing");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error.to_string().contains("while parsing"));
    assert!(DEADLINE_RELEASED.load(Ordering::Acquire));
    assert!(!DEADLINE_VALIDATOR_CALLED.load(Ordering::Acquire));
    assert!(state.is_protocol_failed());
}

#[test]
fn decode_time_budget_includes_schema_validation_and_item_counting() {
    VALIDATION_TIME_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let output = owned_json(
        &serde_json::json!({ "values": [1, 2, 3] }),
        release_validation_timed,
    );
    let budget = ForeignOutputBudget::new(1_024, 4, Duration::from_millis(1));

    let error = state
        .decode_json::<TestPayload, &'static str>(
            output,
            releaser(release_validation_timed),
            ForeignOutputKind::HostRequests,
            budget,
            "decode validation-time test host requests",
            "free validation-time test host requests",
            |payload| {
                std::thread::sleep(Duration::from_millis(10));
                Ok(payload.values.len())
            },
        )
        .expect_err("schema validation beyond the decode budget must fail");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error.to_string().contains("decode time budget"));
    assert!(VALIDATION_TIME_RELEASED.load(Ordering::Acquire));
    assert!(state.is_protocol_failed());
}

#[test]
fn json_nesting_limit_releases_and_fuses_before_schema_validation() {
    NESTING_LIMIT_RELEASED.store(false, Ordering::Release);
    let state = ForeignOutputState::default();
    let depth = FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH + 1;
    let mut bytes = vec![b'['; depth];
    bytes.push(b'0');
    bytes.extend(std::iter::repeat_n(b']', depth));

    let error = state
        .decode_json::<serde_json::Value, &'static str>(
            owned_bytes(bytes, release_nesting_limited),
            releaser(release_nesting_limited),
            ForeignOutputKind::OperationResult,
            ForeignOutputBudget::new(1_024, 1_024, Duration::from_millis(25)),
            "decode nested test operation",
            "free nested test operation",
            |_| Ok(1),
        )
        .expect_err("JSON beyond the bounded recursion depth must fail");

    assert_eq!(error.kind(), RuntimeLibraryErrorKind::ProtocolViolation);
    assert!(error.to_string().contains("maximum nesting depth 128"));
    assert!(NESTING_LIMIT_RELEASED.load(Ordering::Acquire));
    assert!(state.is_protocol_failed());
}

#[test]
fn foreign_output_decode_performance_acceptance() {
    const WARMUP_ITERATIONS: usize = 64;
    const MEASURED_ITERATIONS: usize = 2_000;

    let payload = serde_json::json!({
        "values": (0_u32..256).collect::<Vec<_>>()
    });
    let encoded_bytes = serde_json::to_vec(&payload)
        .expect("serialize benchmark payload")
        .len();
    let warmup_state = ForeignOutputState::default();
    for _ in 0..WARMUP_ITERATIONS {
        let decoded = warmup_state
            .decode_json::<TestPayload, &'static str>(
                owned_json(&payload, release_benchmark),
                releaser(release_benchmark),
                ForeignOutputKind::HostRequests,
                HOST_REQUEST_OUTPUT_BUDGET,
                "decode benchmark host requests",
                "free benchmark host requests",
                |payload| Ok(payload.values.len()),
            )
            .expect("warmup payload must remain within the host-request budget");
        std::hint::black_box(decoded);
    }

    let state = ForeignOutputState::default();
    let mut samples = Vec::with_capacity(MEASURED_ITERATIONS);
    for _ in 0..MEASURED_ITERATIONS {
        let output = owned_json(&payload, release_benchmark);
        let started = Instant::now();
        let decoded = state
            .decode_json::<TestPayload, &'static str>(
                output,
                releaser(release_benchmark),
                ForeignOutputKind::HostRequests,
                HOST_REQUEST_OUTPUT_BUDGET,
                "decode benchmark host requests",
                "free benchmark host requests",
                |payload| Ok(payload.values.len()),
            )
            .expect("benchmark payload must remain within the host-request budget");
        samples.push(started.elapsed().as_nanos());
        std::hint::black_box(decoded);
    }
    samples.sort_unstable();

    let p50_ns = percentile_nanoseconds(&samples, 50);
    let p95_ns = percentile_nanoseconds(&samples, 95);
    let p99_ns = percentile_nanoseconds(&samples, 99);
    let total_ns = samples.iter().copied().sum::<u128>();
    let throughput = (MEASURED_ITERATIONS as f64) * 1_000_000_000.0 / (total_ns as f64);
    println!(
        "APP01_FOREIGN_OUTPUT_PERF iterations={MEASURED_ITERATIONS} encoded_bytes={encoded_bytes} items=256 p50_ns={p50_ns} p95_ns={p95_ns} p99_ns={p99_ns} throughput_payloads_per_second={throughput:.0}"
    );

    let metrics = state.metrics().for_kind(ForeignOutputKind::HostRequests);
    assert_eq!(metrics.accepted_payloads, MEASURED_ITERATIONS as u64);
    assert_eq!(
        metrics.accepted_bytes,
        (encoded_bytes * MEASURED_ITERATIONS) as u64
    );
    assert_eq!(metrics.rejected_payloads, 0);
    assert!(
        p99_ns <= HOST_REQUEST_OUTPUT_BUDGET.max_decode_time().as_nanos(),
        "p99 boundary latency {p99_ns}ns exceeded the 10ms host-request budget"
    );
}

fn percentile_nanoseconds(samples: &[u128], percentile: usize) -> u128 {
    let rank = samples
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    samples[rank]
}
