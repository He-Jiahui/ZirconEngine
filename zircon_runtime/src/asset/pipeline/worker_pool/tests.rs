use std::panic::{AssertUnwindSafe, catch_unwind};
#[cfg(windows)]
use std::process::Command;
use std::sync::{Arc, Barrier, mpsc};

use crate::asset::types::{CpuTexturePayload, TextureSource};
use crate::core::runtime::tasks::TaskPoolDescriptor;

use super::*;

#[derive(Clone)]
pub(super) struct AssetWorkerTestExecutionGate {
    worker_started: Arc<Barrier>,
    worker_release: Arc<Barrier>,
}

impl AssetWorkerTestExecutionGate {
    pub(super) fn new() -> Self {
        Self {
            worker_started: Arc::new(Barrier::new(2)),
            worker_release: Arc::new(Barrier::new(2)),
        }
    }

    pub(super) fn wait_for_worker_start(&self) {
        self.worker_started.wait();
    }

    pub(super) fn release_worker(&self) {
        self.worker_release.wait();
    }

    pub(super) fn wait_for_test_release(&self) {
        self.worker_started.wait();
        self.worker_release.wait();
    }
}

impl AssetWorkerPool {
    fn with_test_execution_gate(
        task_pool: TaskPool,
        options: AssetWorkerPoolOptions,
        test_execution_gate: AssetWorkerTestExecutionGate,
    ) -> Self {
        let mut pool = Self::new(task_pool, options);
        pool.test_execution_gate = Some(test_execution_gate);
        pool
    }
}

#[test]
fn asset_worker_pool_accessors_recover_poisoned_locks() {
    let pool = AssetWorkerPool::new(
        TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1)),
        AssetWorkerPoolOptions::new(),
    );
    let request = AssetRequest::Texture(TextureSource::Path(
        "missing-poison-recovery-texture.png".to_string(),
    ));

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = pool.completions.state.lock().unwrap();
        panic!("poison asset worker completion registry");
    }));
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _guard = pool.diagnostics.lock().unwrap();
        panic!("poison asset worker diagnostics lock");
    }));

    let ticket = pool
        .request(request)
        .expect("request should recover poisoned locks");
    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(2)),
        Ok(payload) if matches!(&*payload, CpuAssetPayload::Failure { .. })
    ));
    let diagnostics_deadline = Instant::now() + Duration::from_secs(2);
    while pool.diagnostics().in_flight != 0 {
        assert!(
            Instant::now() < diagnostics_deadline,
            "worker completion must settle its diagnostics after publishing the ticket"
        );
        std::thread::yield_now();
    }
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.in_flight, 0);
    assert_eq!(diagnostics.completed, 1);
    assert_eq!(diagnostics.failed, 1);
}

#[test]
fn expired_in_flight_entry_does_not_absorb_a_same_key_replacement() {
    let pool = AssetWorkerPool::new(
        TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1)),
        AssetWorkerPoolOptions::new(),
    );
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let expired = Arc::new(CompletionEntry::new(request.clone(), Instant::now()));
    expired.terminate(CompletionTerminal::Expired);
    lock_completion_registry(&pool.completions)
        .in_flight
        .insert(request.clone(), Arc::clone(&expired));

    let replacement = pool
        .request(request)
        .expect("an expired in-flight key must not absorb a replacement ticket");
    assert!(
        !Arc::ptr_eq(&replacement.entry, &expired),
        "replacement must own a fresh completion entry"
    );
    assert!(matches!(
        replacement.wait_timeout(Duration::from_secs(2)),
        Ok(payload) if matches!(&*payload, CpuAssetPayload::Texture(_))
    ));
}

#[test]
fn cancelling_a_running_entry_wakes_its_ticket_and_releases_admission() {
    let gate = AssetWorkerTestExecutionGate::new();
    let pool = AssetWorkerPool::with_test_execution_gate(
        TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1)),
        AssetWorkerPoolOptions::new(),
        gate.clone(),
    );
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let ticket = pool
        .request(request.clone())
        .expect("request should be admitted before its IO closure starts");

    gate.wait_for_worker_start();
    assert!(
        ticket.entry.is_running(),
        "the spawned IO closure must be in its running phase before cancellation"
    );
    assert!(pool.cancel(&request));
    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.in_flight, 0);
    assert_eq!(diagnostics.in_flight_waiters, 0);
    assert_eq!(diagnostics.cancelled, 1);

    gate.release_worker();
}

#[test]
fn completed_cache_ticket_drop_does_not_mutate_live_waiter_ledger() {
    let (io_pool, release) = occupied_io_pool();
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new());
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let owner = publish_test_payload(
        &pool,
        request.clone(),
        texture_payload(TextureSource::BuiltinChecker, 1, 1, 4),
    );
    let entry = Arc::clone(&owner.entry);

    let cached = pool
        .request(request)
        .expect("retained completion should admit a cache reader");
    drop(cached);

    assert_eq!(
        entry.waiter_count(),
        1,
        "completed-cache tickets are not live observers and cannot consume the owner ledger"
    );

    release
        .send(())
        .expect("occupied runtime IO worker should be released");
}

#[test]
fn cancelling_completed_unharvested_entry_releases_retention_budget() {
    let (io_pool, release) = occupied_io_pool();
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new());
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let ticket = publish_test_payload(
        &pool,
        request.clone(),
        texture_payload(TextureSource::BuiltinChecker, 1, 1, 4),
    );

    assert_eq!(pool.diagnostics().completion_entries, 1);
    assert!(pool.cancel(&request));
    assert!(matches!(
        ticket.try_result(),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.completion_entries, 0);
    assert_eq!(diagnostics.completion_bytes, 0);
    assert_eq!(diagnostics.cancelled, 1);

    release
        .send(())
        .expect("occupied runtime IO worker should be released");
}

#[test]
fn dropping_pool_cancels_completed_unharvested_entry_and_releases_retention_budget() {
    let (io_pool, release) = occupied_io_pool();
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new());
    let ticket = publish_test_payload(
        &pool,
        AssetRequest::Texture(TextureSource::BuiltinChecker),
        texture_payload(TextureSource::BuiltinChecker, 1, 1, 4),
    );

    drop(pool);

    assert!(matches!(
        ticket.try_result(),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
    let diagnostics = *lock_worker_diagnostics(&ticket.diagnostics);
    assert_eq!(diagnostics.completion_entries, 0);
    assert_eq!(diagnostics.completion_bytes, 0);
    assert_eq!(diagnostics.cancelled, 1);

    release
        .send(())
        .expect("occupied runtime IO worker should be released");
}

#[test]
fn payload_size_matrix_keeps_one_owner_and_rejects_oversize_retention() {
    const SMALL_PAYLOAD_BYTES: usize = 4 * 1024;
    const COMPLETION_BYTE_CAPACITY: usize = 16 * 1024;
    const OVERSIZE_PAYLOAD_BYTES: usize = 32 * 1024;

    let (io_pool, release) = occupied_io_pool();
    let timer = TaskTimer::new(2).expect("test timer should start");
    let options =
        AssetWorkerPoolOptions::new().with_completion_byte_capacity(COMPLETION_BYTE_CAPACITY);
    let pool = AssetWorkerPool::with_expiry_timer(io_pool, options, timer);
    let small_request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let first = publish_test_payload(
        &pool,
        small_request.clone(),
        texture_payload(TextureSource::BuiltinChecker, 32, 32, SMALL_PAYLOAD_BYTES),
    );
    let second = pool
        .request(small_request)
        .expect("a duplicate observer should join the retained shared payload");
    let first_payload = first
        .wait_timeout(Duration::from_secs(1))
        .expect("the 4 KiB payload should publish successfully");
    let second_payload = second
        .wait_timeout(Duration::from_secs(1))
        .expect("the duplicate ticket should observe the shared payload");
    assert!(
        Arc::ptr_eq(&first_payload, &second_payload),
        "duplicate observers must retain one immutable payload owner"
    );
    assert!(matches!(
        &*first_payload,
        CpuAssetPayload::Texture(texture) if texture.rgba.len() == SMALL_PAYLOAD_BYTES
    ));
    assert!(pool.diagnostics().completion_bytes >= SMALL_PAYLOAD_BYTES);

    let large = publish_test_payload(
        &pool,
        AssetRequest::Texture(TextureSource::BuiltinGrid),
        texture_payload(TextureSource::BuiltinGrid, 128, 64, OVERSIZE_PAYLOAD_BYTES),
    );
    assert!(matches!(
        large.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Rejected)
    ));
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.completion_rejected, 1);
    assert_eq!(diagnostics.payload_clone_bytes, 0);
    assert!(
        diagnostics.completion_bytes < OVERSIZE_PAYLOAD_BYTES,
        "an oversize payload must not become retained completion state"
    );

    release
        .send(())
        .expect("occupied runtime IO worker should be released");
}

#[test]
#[ignore = "the Runtime11 256 MiB RSS matrix is an explicit pressure validation"]
fn payload_256_mib_matrix_rejects_oversize_retention() {
    const LARGE_PAYLOAD_BYTES: usize = 256 * 1024 * 1024;

    let rss_before = current_process_rss_bytes();
    let (io_pool, release) = occupied_io_pool();
    let timer = TaskTimer::new(2).expect("pressure-matrix timer should start");
    let options = AssetWorkerPoolOptions::new().with_completion_byte_capacity(64 * 1024 * 1024);
    let pool = AssetWorkerPool::with_expiry_timer(io_pool, options, timer);
    let ticket = publish_test_payload(
        &pool,
        AssetRequest::Texture(TextureSource::BuiltinGrid),
        texture_payload(
            TextureSource::BuiltinGrid,
            8_192,
            8_192,
            LARGE_PAYLOAD_BYTES,
        ),
    );

    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Rejected)
    ));
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.completion_rejected, 1);
    assert_eq!(diagnostics.payload_clone_bytes, 0);
    let rss_after = current_process_rss_bytes();
    println!(
        "RUNTIME11_ASSET_WORKER_MATRIX payload_bytes={LARGE_PAYLOAD_BYTES} waiters=1 workers=1 stall_seconds=0 rss_before={} rss_after={} queue_age_total_ms={:.3} queue_age_max_ms={:.3} payload_clone_bytes={} cancel_wall_total_ms={:.3} drop_wall_total_ms={:.3}",
        rss_value(rss_before),
        rss_value(rss_after),
        diagnostics.queue_age_total.as_secs_f64() * 1_000.0,
        diagnostics.queue_age_max.as_secs_f64() * 1_000.0,
        diagnostics.payload_clone_bytes,
        diagnostics.cancel_wall_total.as_secs_f64() * 1_000.0,
        diagnostics.drop_wall_total.as_secs_f64() * 1_000.0,
    );
    release
        .send(())
        .expect("occupied runtime IO worker should be released");
}

#[test]
fn dropping_pool_records_a_nonblocking_drop_wall_measurement() {
    let (io_pool, release) = occupied_io_pool();
    let pool = AssetWorkerPool::new(io_pool, AssetWorkerPoolOptions::new());
    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .expect("queued request should be admitted before the pool is dropped");

    drop(pool);

    assert!(matches!(
        ticket.wait_timeout(Duration::from_secs(1)),
        Err(AssetWorkerCompletionError::Cancelled)
    ));
    let diagnostics = *lock_worker_diagnostics(&ticket.diagnostics);
    assert_eq!(diagnostics.drop_wall_samples, 1);
    assert!(diagnostics.drop_wall_total > Duration::ZERO);
    release
        .send(())
        .expect("occupied runtime IO worker should be released");
}

#[test]
#[ignore = "the Runtime11 1/1k/100k waiter, 1/8/64 worker, and 0/1/60 second stall matrix is explicit pressure validation"]
fn runtime11_pressure_matrix_records_shared_completion_backpressure() {
    let rss_before = current_process_rss_bytes();
    for worker_count in [1, 8, 64] {
        for waiter_capacity in [1, 1_000, 100_000] {
            let diagnostics = run_waiter_pressure_sample(worker_count, waiter_capacity);
            println!(
                "RUNTIME11_ASSET_WORKER_MATRIX payload_bytes=4096 workers={worker_count} waiters={waiter_capacity} stall_seconds=0 queue_age_total_ms={:.3} queue_age_max_ms={:.3} queue_age_samples={} payload_clone_bytes={}",
                diagnostics.queue_age_total.as_secs_f64() * 1_000.0,
                diagnostics.queue_age_max.as_secs_f64() * 1_000.0,
                diagnostics.queue_age_samples,
                diagnostics.payload_clone_bytes,
            );
        }
    }

    for stall_seconds in [0, 1, 60] {
        let diagnostics = run_stalled_worker_pressure_sample(stall_seconds);
        println!(
            "RUNTIME11_ASSET_WORKER_MATRIX payload_bytes=4096 workers=1 waiters=1 stall_seconds={stall_seconds} queue_age_total_ms={:.3} queue_age_max_ms={:.3} queue_age_samples={} payload_clone_bytes={}",
            diagnostics.queue_age_total.as_secs_f64() * 1_000.0,
            diagnostics.queue_age_max.as_secs_f64() * 1_000.0,
            diagnostics.queue_age_samples,
            diagnostics.payload_clone_bytes,
        );
    }

    println!(
        "RUNTIME11_ASSET_WORKER_MATRIX rss_before={} rss_after={}",
        rss_value(rss_before),
        rss_value(current_process_rss_bytes()),
    );
}

fn occupied_io_pool() -> (TaskPool, mpsc::SyncSender<()>) {
    let io_pool = TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(1));
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let (release_tx, release_rx) = mpsc::sync_channel(1);
    io_pool.spawn(move || {
        let _ = started_tx.send(());
        let _ = release_rx.recv();
    });
    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("test should occupy the only runtime IO worker");
    (io_pool, release_tx)
}

fn publish_test_payload(
    pool: &AssetWorkerPool,
    request: AssetRequest,
    payload: CpuAssetPayload,
) -> AssetWorkerCompletionTicket {
    let ticket = pool
        .request(request)
        .expect("test request should register before its IO task can start");
    let entry = Arc::clone(&ticket.entry);
    publish_completion(
        &pool.expiry_timer,
        &pool.completions,
        &pool.diagnostics,
        &pool.options,
        entry,
        Some(payload),
    );
    ticket
}

fn texture_payload(
    source: TextureSource,
    width: u32,
    height: u32,
    bytes: usize,
) -> CpuAssetPayload {
    CpuAssetPayload::Texture(CpuTexturePayload {
        source,
        width,
        height,
        rgba: vec![0; bytes],
    })
}

fn run_waiter_pressure_sample(
    worker_count: usize,
    waiter_capacity: usize,
) -> AssetWorkerPoolDiagnostics {
    let pool = AssetWorkerPool::new(
        TaskPool::new(TaskPoolDescriptor::io().with_worker_threads(worker_count)),
        AssetWorkerPoolOptions::new().with_waiter_capacity(waiter_capacity),
    );
    let request = AssetRequest::Texture(TextureSource::BuiltinChecker);
    let first = pool
        .request(request.clone())
        .expect("the first pressure-matrix request should be admitted");
    let mut duplicates = Vec::with_capacity(waiter_capacity.saturating_sub(1));
    for _ in 1..waiter_capacity {
        duplicates.push(
            pool.request(request.clone())
                .expect("a pressure-matrix duplicate must join the one shared completion owner"),
        );
    }

    let first_payload = first
        .wait_timeout(Duration::from_secs(2))
        .expect("the pressure-matrix owner should complete");
    for duplicate in duplicates {
        let payload = duplicate
            .wait_timeout(Duration::from_secs(2))
            .expect("each pressure-matrix ticket should observe the shared completion");
        assert!(Arc::ptr_eq(&first_payload, &payload));
    }
    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.payload_clone_bytes, 0);
    assert_eq!(diagnostics.queue_age_samples, 1);
    diagnostics
}

fn run_stalled_worker_pressure_sample(stall_seconds: u64) -> AssetWorkerPoolDiagnostics {
    let (io_pool, release) = occupied_io_pool();
    let pool = AssetWorkerPool::new(
        io_pool,
        AssetWorkerPoolOptions::new()
            .with_queue_depth(0)
            .with_request_max_age(Duration::from_secs(75)),
    );
    let ticket = pool
        .request(AssetRequest::Texture(TextureSource::BuiltinChecker))
        .expect("the stalled pressure-matrix request should be admitted");
    std::thread::sleep(Duration::from_secs(stall_seconds));
    release
        .send(())
        .expect("the stalled runtime IO worker should be released");
    ticket
        .wait_timeout(Duration::from_secs(2))
        .expect("the stalled pressure-matrix request should complete after release");

    let diagnostics = pool.diagnostics();
    assert_eq!(diagnostics.payload_clone_bytes, 0);
    assert_eq!(diagnostics.queue_age_samples, 1);
    assert!(diagnostics.queue_age_total >= Duration::from_secs(stall_seconds));
    diagnostics
}

#[cfg(windows)]
fn current_process_rss_bytes() -> Option<u64> {
    let command = format!("(Get-Process -Id {}).WorkingSet64", std::process::id());
    let output = Command::new("powershell.exe")
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"])
        .arg(command)
        .output()
        .ok()?;
    output.status.success().then_some(())?;
    String::from_utf8(output.stdout).ok()?.trim().parse().ok()
}

#[cfg(not(windows))]
fn current_process_rss_bytes() -> Option<u64> {
    None
}

fn rss_value(value: Option<u64>) -> String {
    value.map_or_else(|| "unavailable".to_string(), |value| value.to_string())
}
