#![cfg(feature = "dynamic-api")]

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Instant;

use zircon_runtime::dynamic_api::zircon_runtime_get_api_v7;
use zircon_runtime_interface::{
    ProfileControlCommand, ProfileControlRequest, ProfileControlResponse, ZrByteSlice,
    ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeApiV7, ZrRuntimeSessionConfigV3,
    ZrRuntimeSessionHandle, ZrRuntimeWakeSinkV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V3, ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

const WARMUP_ITERATIONS: usize = 128;
const MEASURED_ITERATIONS: usize = 2_000;
const MAX_P99_RELEASE_NS: u128 = 1_000_000;
const MIN_RELEASES_PER_SECOND: f64 = 10_000.0;
static RUNTIME_TEST_LOCK: Mutex<()> = Mutex::new(());

fn runtime_test_guard() -> MutexGuard<'static, ()> {
    RUNTIME_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn runtime_api() -> &'static ZrRuntimeApiV7 {
    let api = unsafe { zircon_runtime_get_api_v7(core::ptr::null()) };
    assert!(!api.is_null(), "runtime rejected the default host API");
    unsafe { &*api }
}

fn status_diagnostics(status: ZrStatus) -> String {
    let bytes = unsafe {
        status
            .diagnostics
            .checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
    }
    .expect("runtime status diagnostics must use a valid bounded byte slice");
    String::from_utf8_lossy(bytes).into_owned()
}

fn create_session(api: &ZrRuntimeApiV7) -> ZrRuntimeSessionHandle {
    let create = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let status = unsafe {
        create(
            ZrRuntimeSessionConfigV3 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
                profile: ZrByteSlice::from_static(b"headless"),
                project_root: ZrByteSlice::empty(),
                play_scene: ZrByteSlice::empty(),
                play_report_pipe: ZrByteSlice::empty(),
                wake_sink: ZrRuntimeWakeSinkV1::disabled(),
            },
            &mut session,
        )
    };
    assert_eq!(
        status.status_code(),
        ZrStatusCode::Ok,
        "{status:?}: {}",
        status_diagnostics(status)
    );
    assert!(session.is_valid());
    session
}

fn destroy_session(api: &ZrRuntimeApiV7, session: ZrRuntimeSessionHandle) -> ZrStatusCode {
    let destroy = api.destroy_session.expect("destroy_session");
    unsafe { destroy(session) }.status_code()
}

fn snapshot_output(api: &ZrRuntimeApiV7, session: ZrRuntimeSessionHandle) -> ZrOwnedResultV2 {
    let request = serde_json::to_vec(&ProfileControlRequest {
        command: ProfileControlCommand::Snapshot,
        config: None,
    })
    .expect("serialize profile snapshot request");
    let profile_control = api.profile_control.expect("profile_control");
    let mut output = ZrOwnedResultV2::empty();
    let status = unsafe {
        profile_control(
            session,
            ZrByteSlice {
                data: request.as_ptr(),
                len: request.len(),
            },
            &mut output,
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert!(!output.data.is_null());
    assert!(output.len > 0);
    assert!(output.allocation.is_valid());

    let len = usize::try_from(output.len).expect("runtime output length fits host address space");
    let bytes = unsafe { core::slice::from_raw_parts(output.data, len) };
    let response: ProfileControlResponse =
        serde_json::from_slice(bytes).expect("decode profile snapshot response");
    assert_eq!(response.status, "ok");
    output
}

fn release(
    api: &ZrRuntimeApiV7,
    session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatusCode {
    let release = api.release_allocation.expect("release_allocation");
    unsafe { release(session, allocation) }.status_code()
}

fn percentile(sorted: &[u128], numerator: usize, denominator: usize) -> u128 {
    let rank = (sorted.len() * numerator)
        .div_ceil(denominator)
        .saturating_sub(1);
    sorted[rank.min(sorted.len() - 1)]
}

fn record_performance_metric(metric: &str) {
    println!("{metric}");
    let Some(path) = std::env::var_os("ZIRCON_RUNTIME_V7_PERF_OUTPUT") else {
        return;
    };
    let mut output = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open V7 performance output");
    writeln!(output, "{metric}").expect("write V7 performance output");
}

#[test]
fn runtime_v7_owned_results_require_opaque_exactly_once_release() {
    let _guard = runtime_test_guard();
    let api = runtime_api();
    let session = create_session(api);
    let output = snapshot_output(api, session);
    let allocation = output.allocation;

    assert_eq!(release(api, session, allocation), ZrStatusCode::Ok);
    assert_eq!(release(api, session, allocation), ZrStatusCode::NotFound);
    assert_eq!(
        release(api, session, ZrRuntimeAllocationId::new(u64::MAX)),
        ZrStatusCode::NotFound
    );
    assert_eq!(destroy_session(api, session), ZrStatusCode::Ok);
}

#[test]
fn runtime_v7_release_is_concurrent_and_exactly_once() {
    let _guard = runtime_test_guard();
    let api = runtime_api();
    let session = create_session(api);
    let output = snapshot_output(api, session);
    let allocation = output.allocation;
    let release_fn = api.release_allocation.expect("release_allocation");

    let first = thread::spawn(move || unsafe { release_fn(session, allocation) }.status_code());
    let second = thread::spawn(move || unsafe { release_fn(session, allocation) }.status_code());
    let statuses = [first.join().unwrap(), second.join().unwrap()];

    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == ZrStatusCode::Ok)
            .count(),
        1
    );
    assert_eq!(
        statuses
            .iter()
            .filter(|status| **status == ZrStatusCode::NotFound)
            .count(),
        1
    );
    assert_eq!(destroy_session(api, session), ZrStatusCode::Ok);
}

#[test]
fn runtime_v7_destroy_is_retryable_after_outstanding_result_release() {
    let _guard = runtime_test_guard();
    let api = runtime_api();
    let session = create_session(api);
    let output = snapshot_output(api, session);

    assert_eq!(destroy_session(api, session), ZrStatusCode::Error);
    assert_eq!(release(api, session, output.allocation), ZrStatusCode::Ok);
    assert_eq!(destroy_session(api, session), ZrStatusCode::Ok);
}

#[test]
fn runtime_v7_release_rejects_a_different_session_without_changing_owner_census() {
    let _guard = runtime_test_guard();
    let api = runtime_api();
    let owner = create_session(api);
    let foreign = create_session(api);
    let output = snapshot_output(api, owner);

    assert_eq!(
        release(api, foreign, output.allocation),
        ZrStatusCode::NotFound
    );
    assert_eq!(destroy_session(api, foreign), ZrStatusCode::Ok);
    assert_eq!(destroy_session(api, owner), ZrStatusCode::Error);
    assert_eq!(release(api, owner, output.allocation), ZrStatusCode::Ok);
    assert_eq!(destroy_session(api, owner), ZrStatusCode::Ok);
}

#[test]
fn runtime_v7_release_performance_acceptance() {
    let _guard = runtime_test_guard();
    let api = runtime_api();
    let session = create_session(api);

    for _ in 0..WARMUP_ITERATIONS {
        let output = snapshot_output(api, session);
        assert_eq!(release(api, session, output.allocation), ZrStatusCode::Ok);
    }

    let outputs: Vec<_> = (0..MEASURED_ITERATIONS)
        .map(|_| snapshot_output(api, session))
        .collect();
    let total_start = Instant::now();
    let mut release_ns = Vec::with_capacity(MEASURED_ITERATIONS);
    for output in outputs {
        let release_start = Instant::now();
        assert_eq!(release(api, session, output.allocation), ZrStatusCode::Ok);
        release_ns.push(release_start.elapsed().as_nanos());
    }
    let elapsed = total_start.elapsed();
    release_ns.sort_unstable();

    let p50_ns = percentile(&release_ns, 50, 100);
    let p95_ns = percentile(&release_ns, 95, 100);
    let p99_ns = percentile(&release_ns, 99, 100);
    let throughput = MEASURED_ITERATIONS as f64 / elapsed.as_secs_f64();
    record_performance_metric(&format!(
        "RUNTIME_INTERFACE01_V7_RELEASE_PERF iterations={MEASURED_ITERATIONS} p50_ns={p50_ns} p95_ns={p95_ns} p99_ns={p99_ns} throughput_releases_per_second={throughput:.2}"
    ));

    assert!(
        p99_ns <= MAX_P99_RELEASE_NS,
        "release p99 {p99_ns} ns exceeds {MAX_P99_RELEASE_NS} ns"
    );
    assert!(
        throughput >= MIN_RELEASES_PER_SECOND,
        "release throughput {throughput:.2}/s is below {MIN_RELEASES_PER_SECOND:.2}/s"
    );
    assert_eq!(destroy_session(api, session), ZrStatusCode::Ok);
}
