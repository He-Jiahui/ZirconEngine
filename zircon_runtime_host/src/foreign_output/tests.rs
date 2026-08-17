use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Barrier,
};
use std::time::{Duration, Instant};

use serde::Deserialize;

use super::{
    RuntimeForeignOutputBudget, RuntimeForeignOutputErrorKind, RuntimeForeignOutputKind,
    RuntimeForeignOutputState,
};
use zircon_runtime_interface::{ZrByteSlice, ZrOwnedByteBuffer, ZrStatus, ZrStatusCode};

const RELEASE_DIAGNOSTIC: &[u8] = b"test allocation is still in use";
const CALL_DIAGNOSTIC: &[u8] = b"test call failed";

struct TestAllocation {
    bytes: Vec<u8>,
    releases: Arc<AtomicUsize>,
    reject_release: bool,
}

unsafe extern "C" fn release_test_output(output: ZrOwnedByteBuffer) -> ZrStatus {
    let allocation = unsafe { Box::from_raw(output.owner_token as usize as *mut TestAllocation) };
    allocation.releases.fetch_add(1, Ordering::SeqCst);
    if allocation.reject_release {
        ZrStatus::new(
            ZrStatusCode::Error,
            ZrByteSlice::from_static(RELEASE_DIAGNOSTIC),
        )
    } else {
        ZrStatus::ok()
    }
}

fn owned_output(
    bytes: impl Into<Vec<u8>>,
    releases: Arc<AtomicUsize>,
    reject_release: bool,
) -> ZrOwnedByteBuffer {
    let allocation = Box::new(TestAllocation {
        bytes: bytes.into(),
        releases,
        reject_release,
    });
    let data = allocation.bytes.as_ptr().cast_mut();
    let len = allocation.bytes.len();
    let capacity = allocation.bytes.capacity();
    ZrOwnedByteBuffer {
        data,
        len,
        capacity,
        owner_token: Box::into_raw(allocation) as usize as u64,
        free: Some(release_test_output),
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

unsafe extern "C" fn release_benchmark_output(output: ZrOwnedByteBuffer) -> ZrStatus {
    if !output.data.is_null() {
        unsafe {
            drop(Vec::from_raw_parts(
                output.data,
                output.len,
                output.capacity,
            ));
        }
    }
    ZrStatus::ok()
}

fn benchmark_output(mut bytes: Vec<u8>) -> ZrOwnedByteBuffer {
    let output = ZrOwnedByteBuffer {
        data: bytes.as_mut_ptr(),
        len: bytes.len(),
        capacity: bytes.capacity(),
        owner_token: 1,
        free: Some(release_benchmark_output),
    };
    std::mem::forget(bytes);
    output
}

fn test_budget(max_bytes: usize, max_items: usize) -> RuntimeForeignOutputBudget {
    RuntimeForeignOutputBudget::new(max_bytes, max_items, Duration::from_millis(25))
}

#[test]
fn bounded_json_acceptance_releases_once_and_records_metrics() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let payload = state
        .decode_json(
            owned_output(br#"{"values":[1,2,3]}"#.to_vec(), releases.clone(), false),
            RuntimeForeignOutputKind::WorldQuery,
            test_budget(1024, 4),
            "decode runtime world query",
            "free runtime world query",
            |payload: &TestPayload| Ok::<usize, &'static str>(1 + payload.values.len()),
        )
        .expect("bounded payload should decode")
        .expect("non-empty payload should remain present");

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
    let error = state
        .decode_json::<TestPayload, _>(
            owned_output(br#"{"values":[1]}"#.to_vec(), releases.clone(), false),
            RuntimeForeignOutputKind::WorldQuery,
            test_budget(4, 8),
            "decode runtime world query",
            "free runtime world query",
            |_| Ok::<usize, &'static str>(2),
        )
        .expect_err("oversized foreign output must fail");

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
fn explicitly_empty_pages_are_accepted_and_released() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let decoded = state
        .decode_json::<Vec<u64>, _>(
            owned_output(Vec::new(), releases.clone(), false),
            RuntimeForeignOutputKind::WorldInvalidations,
            test_budget(1024, 8).allow_empty(),
            "drain runtime world invalidations",
            "free runtime world invalidations",
            |values| Ok::<usize, &'static str>(values.len()),
        )
        .expect("empty page is an allowed protocol outcome");

    assert_eq!(decoded, None);
    assert_eq!(releases.load(Ordering::SeqCst), 1);
    assert!(!state.is_protocol_failed());
}

#[test]
fn release_failure_preserves_cleanup_diagnostic_and_fuses_session() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let error = state
        .decode_json(
            owned_output(br#"{"values":[1]}"#.to_vec(), releases.clone(), true),
            RuntimeForeignOutputKind::OperationResult,
            test_budget(1024, 8),
            "harvest runtime operation",
            "free runtime operation output",
            |payload: &TestPayload| Ok::<usize, &'static str>(1 + payload.values.len()),
        )
        .expect_err("failed foreign release must reject the decoded value");

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
    let depth_error = state
        .decode_json::<serde_json::Value, _>(
            owned_output(nested.into_bytes(), releases.clone(), false),
            RuntimeForeignOutputKind::ProfileResponse,
            test_budget(4096, 512),
            "decode runtime profile response",
            "free runtime profile response",
            |_| Ok::<usize, &'static str>(1),
        )
        .expect_err("payloads deeper than the shared limit must fail");
    assert!(depth_error
        .to_string()
        .contains("maximum nesting depth 128"));
    assert_eq!(releases.load(Ordering::SeqCst), 1);

    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let time_error = state
        .decode_json(
            owned_output(br#"{"values":[1]}"#.to_vec(), releases.clone(), false),
            RuntimeForeignOutputKind::ProfileResponse,
            RuntimeForeignOutputBudget::new(1024, 8, Duration::from_millis(1)),
            "decode runtime profile response",
            "free runtime profile response",
            |payload: &TestPayload| {
                std::thread::sleep(Duration::from_millis(3));
                Ok::<usize, &'static str>(1 + payload.values.len())
            },
        )
        .expect_err("validation time belongs to the decode budget");
    assert!(time_error.to_string().contains("decode time budget"));
    assert_eq!(releases.load(Ordering::SeqCst), 1);
}

#[test]
fn ordinary_call_failure_releases_output_without_fusing_protocol() {
    let releases = Arc::new(AtomicUsize::new(0));
    let state = RuntimeForeignOutputState::default();
    let error = state
        .ensure_call_succeeded(
            ZrStatus::new(
                ZrStatusCode::Error,
                ZrByteSlice::from_static(CALL_DIAGNOSTIC),
            ),
            owned_output(Vec::new(), releases.clone(), false),
            RuntimeForeignOutputKind::HostRequests,
            "drain runtime host requests",
            "free runtime host requests",
        )
        .expect_err("runtime call failure must propagate");

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
    let decode = std::thread::spawn(move || {
        decode_state.decode_json(
            owned_output(br#"{"values":[1,2,3]}"#.to_vec(), decode_releases, false),
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
        let decoded = warmup
            .decode_json(
                benchmark_output(payload.clone()),
                RuntimeForeignOutputKind::HostRequests,
                super::HOST_REQUEST_OUTPUT_BUDGET,
                "decode benchmark host requests",
                "free benchmark host requests",
                |decoded: &BenchmarkPayload| Ok::<usize, &'static str>(decoded.values.len()),
            )
            .expect("warmup payload must remain within the shared budget");
        std::hint::black_box(decoded);
    }

    let state = RuntimeForeignOutputState::default();
    let mut samples = Vec::with_capacity(MEASURED_ITERATIONS);
    for _ in 0..MEASURED_ITERATIONS {
        let output = benchmark_output(payload.clone());
        let started = Instant::now();
        let decoded = state
            .decode_json(
                output,
                RuntimeForeignOutputKind::HostRequests,
                super::HOST_REQUEST_OUTPUT_BUDGET,
                "decode benchmark host requests",
                "free benchmark host requests",
                |decoded: &BenchmarkPayload| Ok::<usize, &'static str>(decoded.values.len()),
            )
            .expect("benchmark payload must remain within the shared budget");
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

fn percentile_nanoseconds(samples: &[u128], percentile: usize) -> u128 {
    let rank = samples
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    samples[rank]
}
