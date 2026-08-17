use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde::Deserialize;
use zircon_runtime_interface::{ZrOwnedByteBuffer, ZrStatus};

use super::{
    ForeignOutputBudget, ForeignOutputKind, ForeignOutputState,
    FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH, HOST_REQUEST_OUTPUT_BUDGET,
};
use zircon_runtime_host::foreign_output::RuntimeForeignOutputErrorKind as RuntimeLibraryErrorKind;

static DEADLINE_RELEASED: AtomicBool = AtomicBool::new(false);
static DEADLINE_VALIDATOR_CALLED: AtomicBool = AtomicBool::new(false);
static NESTING_LIMIT_RELEASED: AtomicBool = AtomicBool::new(false);
static VALIDATION_TIME_RELEASED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Deserialize)]
struct TestPayload {
    values: Vec<u32>,
}

unsafe extern "C" fn release_deadline(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    DEADLINE_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_nesting_limited(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    NESTING_LIMIT_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_benchmark(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    ZrStatus::ok()
}

unsafe extern "C" fn release_validation_timed(output: ZrOwnedByteBuffer) -> ZrStatus {
    release_vec(output);
    VALIDATION_TIME_RELEASED.store(true, Ordering::Release);
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
