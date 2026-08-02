use std::collections::HashMap;
use std::io::{self, Cursor, Read};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

use crate::core::framework::platform::{
    PreferenceDurabilityState, PreferenceKey, PreferenceMutationTerminal,
    PreferenceStorageBackendKind, PreferenceStorageError, PreferenceStorageErrorKind,
    PreferenceStorageOperation, PreferenceTicketWaitResult, PreferenceWorkDeadline,
};
use crate::core::runtime::{JobHandle, JobScheduler, TaskPool, TaskPoolDescriptor};
use crate::platform::preferences::{PreferenceBackendWorkAuthority, PreferenceStorageBackend};

use super::adapter::{
    PreferencePersistenceAdapter, PreferencePersistenceLimits, MAX_PREFERENCE_FAILURE_DETAIL_BYTES,
    MAX_PREFERENCE_VALUE_BYTES,
};
use super::work::read_bounded;

mod quota_and_failure;

struct CountingRead {
    remaining: usize,
    consumed: Arc<AtomicUsize>,
}

impl Read for CountingRead {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = buffer.len().min(self.remaining);
        buffer[..count].fill(b'x');
        self.remaining -= count;
        self.consumed.fetch_add(count, Ordering::Relaxed);
        Ok(count)
    }
}

#[test]
fn platform_preference_storage_bounded_backend_read_consumes_at_most_max_plus_one_bytes() {
    let consumed = Arc::new(AtomicUsize::new(0));
    let reader = CountingRead {
        remaining: 1024,
        consumed: Arc::clone(&consumed),
    };

    let failure = read_bounded(Box::new(reader), 8, "counting")
        .expect_err("oversized stream must be rejected");

    assert_eq!(failure.kind(), PreferenceStorageErrorKind::CapacityExceeded);
    assert_eq!(consumed.load(Ordering::Relaxed), 9);
}

#[test]
fn platform_preference_storage_flush_before_later_different_key_preserves_read_your_write() {
    let backend = Arc::new(MemoryBackend::default());
    let adapter = adapter(backend.clone(), 64);
    let key = PreferenceKey::new("woc.input", "bindings").unwrap();
    let later_key = PreferenceKey::new("woc.window", "placement").unwrap();

    let first = adapter
        .submit_write(
            key.clone(),
            Arc::from(&b"first"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    let visible = adapter.snapshot(&key).unwrap();
    assert_eq!(visible.value(), Some(&b"first"[..]));
    assert!(matches!(
        visible.durability(),
        PreferenceDurabilityState::Pending | PreferenceDurabilityState::Durable
    ));

    let fence = adapter.flush_fence(PreferenceWorkDeadline::none()).unwrap();
    let second = adapter
        .submit_write(
            later_key.clone(),
            Arc::from(&b"second"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();

    assert!(matches!(
        first
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    assert!(matches!(
        fence.wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    assert!(matches!(
        second
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    assert_eq!(adapter.snapshot(&key).unwrap().value(), Some(&b"first"[..]));
    assert_eq!(
        adapter.snapshot(&later_key).unwrap().value(),
        Some(&b"second"[..])
    );

    let operations = backend.operations.lock().unwrap().clone();
    let flush = operations.iter().position(|item| item == "flush").unwrap();
    let second_write = operations
        .iter()
        .rposition(|item| item == "write:woc.window/placement")
        .unwrap();
    assert!(flush < second_write, "post-fence work crossed global flush");
}

#[test]
fn platform_preference_storage_projects_oversized_read_and_error_detail() {
    let backend = Arc::new(MemoryBackend::default());
    let oversized_key = PreferenceKey::new("woc.input", "oversized").unwrap();
    backend
        .values
        .lock()
        .unwrap()
        .insert(oversized_key.clone(), Arc::from(&b"12345"[..]));
    let adapter = adapter(backend.clone(), 4);

    let pending = adapter.snapshot(&oversized_key).unwrap();
    assert_eq!(pending.durability(), PreferenceDurabilityState::Pending);
    let terminal = wait_snapshot_terminal(&adapter, &oversized_key);
    let PreferenceMutationTerminal::Failed(read_failure) = terminal else {
        panic!("oversized read should fail with a projected terminal");
    };
    assert_eq!(
        read_failure.kind(),
        PreferenceStorageErrorKind::CapacityExceeded
    );

    backend.fail_writes.store(1, Ordering::Relaxed);
    let failure_key = PreferenceKey::new("woc.input", "failure").unwrap();
    let submission = adapter
        .submit_write(
            failure_key,
            Arc::from(&b"ok"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    let PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(failure)) =
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2))
    else {
        panic!("backend error should become projected failure");
    };
    assert!(failure.detail().len() <= MAX_PREFERENCE_FAILURE_DETAIL_BYTES);
    assert!(failure.detail().is_char_boundary(failure.detail().len()));
}

#[test]
fn platform_preference_storage_rejects_value_limit_above_hard_maximum() {
    let error = PreferencePersistenceAdapter::new(
        Arc::new(MemoryBackend::default()),
        PreferencePersistenceLimits {
            max_value_bytes: MAX_PREFERENCE_VALUE_BYTES + 1,
            ..PreferencePersistenceLimits::default()
        },
    )
    .unwrap_err();

    assert_eq!(
        error.configured_max_value_bytes(),
        MAX_PREFERENCE_VALUE_BYTES + 1
    );
    assert_eq!(error.hard_max_value_bytes(), MAX_PREFERENCE_VALUE_BYTES);
}

#[test]
fn platform_preference_storage_failed_generation_blocks_every_fence_until_retry_is_durable() {
    let backend = Arc::new(MemoryBackend::default());
    backend.fail_writes.store(1, Ordering::Relaxed);
    let adapter = adapter_with_limits(backend, 64, 1);
    let key = PreferenceKey::new("woc.input", "retry").unwrap();
    let failed = adapter
        .submit_write(
            key.clone(),
            Arc::from(&b"failed"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    assert!(matches!(
        failed
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(_))
    ));

    for _ in 0..2 {
        let fence = adapter.flush_fence(PreferenceWorkDeadline::none()).unwrap();
        assert!(matches!(
            fence.wait_until(Instant::now() + Duration::from_secs(2)),
            PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(_))
        ));
    }

    let retry = adapter
        .submit_write(
            key,
            Arc::from(&b"durable"[..]),
            PreferenceWorkDeadline::none(),
        )
        .expect("same-key recovery must not consume a second overlay entry");
    assert!(matches!(
        retry
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    let fence = adapter.flush_fence(PreferenceWorkDeadline::none()).unwrap();
    assert!(matches!(
        fence.wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
}

#[test]
fn platform_preference_storage_inflight_failure_is_replaced_by_captured_durable_successor() {
    let backend = Arc::new(MemoryBackend::default());
    backend.fail_writes.store(1, Ordering::Relaxed);
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    *backend.write_started.lock().unwrap() = Some(started_tx);
    *backend.write_release.lock().unwrap() = Some(release_rx);
    let adapter = adapter(backend, 64);
    let key = PreferenceKey::new("woc.input", "inflight-retry").unwrap();
    let failed = adapter
        .submit_write(
            key.clone(),
            Arc::from(&b"failed"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    started_rx.recv().unwrap();
    let retry = adapter
        .submit_write(
            key,
            Arc::from(&b"durable"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    let fence = adapter.flush_fence(PreferenceWorkDeadline::none()).unwrap();

    release_tx.send(()).unwrap();
    assert!(matches!(
        failed
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(_))
    ));
    assert!(matches!(
        retry
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    assert!(matches!(
        fence.wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
}

#[test]
fn platform_preference_storage_explicit_eviction_releases_overlay_entry_capacity() {
    let backend = Arc::new(MemoryBackend::default());
    backend.fail_writes.store(1, Ordering::Relaxed);
    let adapter = adapter_with_limits(backend, 64, 1);
    let first_key = PreferenceKey::new("woc.input", "first").unwrap();
    let second_key = PreferenceKey::new("woc.input", "second").unwrap();
    let first = adapter
        .submit_write(
            first_key.clone(),
            Arc::from(&b"first"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    assert!(matches!(
        first
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(_))
    ));
    assert!(adapter
        .submit_write(
            second_key.clone(),
            Arc::from(&b"second"[..]),
            PreferenceWorkDeadline::none(),
        )
        .is_err());

    let eviction = adapter
        .evict(&first_key)
        .expect("terminal non-durable generation is explicitly evictable");
    assert_eq!(
        eviction.durability(),
        PreferenceDurabilityState::VisibleNotDurable
    );
    assert!(adapter
        .submit_write(
            second_key,
            Arc::from(&b"second"[..]),
            PreferenceWorkDeadline::none(),
        )
        .is_ok());
}

#[test]
fn platform_preference_storage_rejects_pending_and_durable_eviction() {
    let backend = Arc::new(MemoryBackend::default());
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    *backend.write_started.lock().unwrap() = Some(started_tx);
    *backend.write_release.lock().unwrap() = Some(release_rx);
    let adapter = adapter(backend, 64);
    let key = PreferenceKey::new("woc.input", "eviction-terminal-only").unwrap();
    let submission = adapter
        .submit_write(
            key.clone(),
            Arc::from(&b"durable"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    started_rx.recv().unwrap();

    assert_eq!(adapter.evict(&key), None);
    assert_eq!(submission.ticket().terminal(), None);

    release_tx.send(()).unwrap();
    assert!(matches!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    assert_eq!(adapter.evict(&key), None);
    assert_eq!(
        adapter.snapshot(&key).unwrap().value(),
        Some(&b"durable"[..])
    );
}

#[test]
fn platform_preference_storage_default_limits_allow_maximum_value_failure_retry() {
    let backend = Arc::new(MemoryBackend::default());
    backend.fail_writes.store(1, Ordering::Relaxed);
    let adapter =
        PreferencePersistenceAdapter::new(backend, PreferencePersistenceLimits::default()).unwrap();
    let key = PreferenceKey::new("woc.input", "maximum-value-retry").unwrap();
    let value = Arc::<[u8]>::from(vec![b'x'; MAX_PREFERENCE_VALUE_BYTES]);
    let failed = adapter
        .submit_write(
            key.clone(),
            Arc::clone(&value),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    assert!(matches!(
        failed
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(_))
    ));

    let retry = adapter
        .submit_write(key.clone(), value, PreferenceWorkDeadline::none())
        .expect("same-key maximum value retry must replace the failed overlay quote");
    assert!(matches!(
        retry
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(5)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    assert_eq!(
        adapter.snapshot(&key).unwrap().value().map(<[u8]>::len),
        Some(MAX_PREFERENCE_VALUE_BYTES)
    );
}

#[test]
fn platform_preference_storage_one_second_backend_stall_remains_off_caller_filesystem_wall() {
    let backend = Arc::new(MemoryBackend::default());
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    *backend.write_started.lock().unwrap() = Some(started_tx);
    *backend.write_release.lock().unwrap() = Some(release_rx);
    let adapter = adapter(backend, 64);
    let key = PreferenceKey::new("woc.input", "one-second-stall").unwrap();
    let submission = adapter
        .submit_write(
            key.clone(),
            Arc::from(&b"visible"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    started_rx.recv().unwrap();
    std::thread::sleep(Duration::from_millis(1_000));

    assert_eq!(submission.ticket().terminal(), None);
    assert_eq!(
        adapter.snapshot(&key).unwrap().durability(),
        PreferenceDurabilityState::Pending
    );
    assert_eq!(adapter.diagnostics().caller_filesystem_wall, Duration::ZERO);

    release_tx.send(()).unwrap();
    assert!(matches!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
}

#[test]
fn platform_preference_storage_prestart_deadline_projects_without_ticket_polling() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let adapter = PreferencePersistenceAdapter::with_scheduler(
        Arc::new(MemoryBackend::default()),
        test_limits(64, 4),
        scheduler,
    )
    .unwrap();
    let key = PreferenceKey::new("woc.input", "deadline").unwrap();
    adapter
        .submit_write(
            key.clone(),
            Arc::from(&b"visible"[..]),
            PreferenceWorkDeadline::at(Instant::now() + Duration::from_millis(10)),
        )
        .unwrap();

    let terminal = wait_snapshot_terminal(&adapter, &key);
    assert_eq!(terminal, PreferenceMutationTerminal::DeadlineBeforeStart);
    assert_eq!(
        adapter.snapshot(&key).unwrap().durability(),
        PreferenceDurabilityState::VisibleNotDurable
    );
    release_tx.send(()).unwrap();
    blocker.wait();
}

#[test]
fn platform_preference_storage_stale_initial_read_cannot_replace_newer_visible_generation() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let backend = Arc::new(MemoryBackend::default());
    let key = PreferenceKey::new("woc.input", "stale-read").unwrap();
    backend
        .values
        .lock()
        .unwrap()
        .insert(key.clone(), Arc::from(&b"persisted-old"[..]));
    let adapter = PreferencePersistenceAdapter::with_scheduler(
        backend.clone(),
        test_limits(64, 4),
        scheduler,
    )
    .unwrap();

    assert_eq!(
        adapter.snapshot(&key).unwrap().durability(),
        PreferenceDurabilityState::Pending
    );
    let latest = adapter
        .submit_write(
            key.clone(),
            Arc::from(&b"visible-new"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    assert_eq!(
        adapter.snapshot(&key).unwrap().value(),
        Some(&b"visible-new"[..])
    );

    release_tx.send(()).unwrap();
    assert!(matches!(
        latest
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
    blocker.wait();
    assert_eq!(
        adapter.snapshot(&key).unwrap().value(),
        Some(&b"visible-new"[..])
    );
    assert_eq!(
        *backend.operations.lock().unwrap(),
        vec!["write:woc.input/stale-read".to_owned()]
    );
}

#[test]
fn platform_preference_storage_deadline_during_inflight_does_not_override_backend_terminal() {
    let backend = Arc::new(MemoryBackend::default());
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    *backend.write_started.lock().unwrap() = Some(started_tx);
    *backend.write_release.lock().unwrap() = Some(release_rx);
    let adapter = adapter(backend, 64);
    let key = PreferenceKey::new("woc.input", "inflight-deadline").unwrap();
    let submission = adapter
        .submit_write(
            key.clone(),
            Arc::from(&b"durable"[..]),
            PreferenceWorkDeadline::at(Instant::now() + Duration::from_millis(10)),
        )
        .unwrap();
    started_rx.recv().unwrap();
    std::thread::sleep(Duration::from_millis(20));

    assert_eq!(submission.ticket().terminal(), None);
    assert_eq!(
        adapter.snapshot(&key).unwrap().durability(),
        PreferenceDurabilityState::Pending
    );
    release_tx.send(()).unwrap();
    assert!(matches!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Durable)
    ));
}

#[test]
fn platform_preference_storage_backend_panic_projects_failure_and_releases_lane() {
    let backend = Arc::new(MemoryBackend::default());
    backend.panic_writes.store(1, Ordering::Relaxed);
    let adapter = adapter(backend, 64);
    let key = PreferenceKey::new("woc.input", "panic").unwrap();
    let submission = adapter
        .submit_write(
            key.clone(),
            Arc::from(&b"visible"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();

    assert!(matches!(
        submission
            .ticket()
            .wait_until(Instant::now() + Duration::from_secs(2)),
        PreferenceTicketWaitResult::Terminal(PreferenceMutationTerminal::Failed(_))
    ));
    assert_eq!(
        adapter.snapshot(&key).unwrap().durability(),
        PreferenceDurabilityState::VisibleNotDurable
    );
    assert_eq!(adapter.diagnostics().lane.queue_entries, 0);
}

#[test]
fn platform_preference_storage_diagnostics_keep_caller_filesystem_wall_zero() {
    let adapter = adapter(Arc::new(MemoryBackend::default()), 64);
    let submission = adapter
        .submit_write(
            PreferenceKey::new("woc.input", "diagnostics").unwrap(),
            Arc::from(&b"value"[..]),
            PreferenceWorkDeadline::none(),
        )
        .unwrap();
    let _ = submission
        .ticket()
        .wait_until(Instant::now() + Duration::from_secs(2));

    let diagnostics = adapter.diagnostics();
    assert_eq!(diagnostics.caller_filesystem_wall, Duration::ZERO);
    assert!(diagnostics.backend_wall > Duration::ZERO);
    assert_eq!(diagnostics.overlay.durable, 1);
}

fn wait_snapshot_terminal(
    adapter: &PreferencePersistenceAdapter,
    key: &PreferenceKey,
) -> PreferenceMutationTerminal {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let snapshot = adapter.snapshot(key).unwrap();
        if let Some(terminal) = snapshot.last_terminal() {
            return terminal.clone();
        }
        assert!(
            Instant::now() < deadline,
            "snapshot did not reach terminal state"
        );
        std::thread::yield_now();
    }
}

fn adapter(backend: Arc<MemoryBackend>, max_value_bytes: usize) -> PreferencePersistenceAdapter {
    adapter_with_limits(backend, max_value_bytes, 16)
}

fn adapter_with_limits(
    backend: Arc<MemoryBackend>,
    max_value_bytes: usize,
    max_overlay_entries: usize,
) -> PreferencePersistenceAdapter {
    PreferencePersistenceAdapter::new(backend, test_limits(max_value_bytes, max_overlay_entries))
        .unwrap()
}

fn test_limits(max_value_bytes: usize, max_overlay_entries: usize) -> PreferencePersistenceLimits {
    PreferencePersistenceLimits {
        max_value_bytes,
        max_overlay_entries,
        max_overlay_retained_bytes: 64 * 1024,
        max_lane_entries: 16,
        max_lane_retained_bytes: 64 * 1024,
    }
}

fn blocked_scheduler() -> (JobScheduler, mpsc::SyncSender<()>, JobHandle) {
    let scheduler = JobScheduler::from_pool(TaskPool::new(
        TaskPoolDescriptor::io().with_worker_threads(1),
    ));
    let (started_tx, started_rx) = mpsc::sync_channel(0);
    let (release_tx, release_rx) = mpsc::sync_channel(0);
    let blocker = scheduler.schedule(move || {
        started_tx.send(()).unwrap();
        release_rx.recv().unwrap();
    });
    started_rx.recv().unwrap();
    (scheduler, release_tx, blocker)
}

#[derive(Default)]
struct MemoryBackend {
    values: Mutex<HashMap<PreferenceKey, Arc<[u8]>>>,
    operations: Mutex<Vec<String>>,
    fail_writes: AtomicUsize,
    write_failure_kind: Mutex<Option<PreferenceStorageErrorKind>>,
    panic_writes: AtomicUsize,
    write_started: Mutex<Option<mpsc::SyncSender<()>>>,
    write_release: Mutex<Option<mpsc::Receiver<()>>>,
}

impl PreferenceStorageBackend for MemoryBackend {
    fn backend_kind(&self) -> PreferenceStorageBackendKind {
        PreferenceStorageBackendKind::HostProvided
    }

    fn open_read(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<Option<Box<dyn Read + Send>>, PreferenceStorageError> {
        self.operations
            .lock()
            .unwrap()
            .push(format!("read:{}/{}", key.namespace(), key.key()));
        Ok(self
            .values
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .map(|value| Box::new(Cursor::new(value)) as Box<dyn Read + Send>))
    }

    fn write(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
        value: &[u8],
    ) -> Result<(), PreferenceStorageError> {
        if let Some(started) = self.write_started.lock().unwrap().take() {
            started.send(()).unwrap();
        }
        if let Some(release) = self.write_release.lock().unwrap().take() {
            release.recv().unwrap();
        }
        if self.panic_writes.swap(0, Ordering::Relaxed) != 0 {
            panic!("injected preference backend panic");
        }
        if self.fail_writes.swap(0, Ordering::Relaxed) != 0 {
            return Err(PreferenceStorageError::new(
                PreferenceStorageErrorKind::TransientIo,
                PreferenceStorageOperation::Write,
                "memory",
                "é".repeat(MAX_PREFERENCE_FAILURE_DETAIL_BYTES + 8),
            ));
        }
        if let Some(kind) = self.write_failure_kind.lock().unwrap().take() {
            return Err(PreferenceStorageError::new(
                kind,
                PreferenceStorageOperation::Write,
                "memory",
                "injected typed backend failure",
            ));
        }
        self.operations
            .lock()
            .unwrap()
            .push(format!("write:{}/{}", key.namespace(), key.key()));
        self.values
            .lock()
            .unwrap()
            .insert(key.clone(), Arc::from(value));
        Ok(())
    }

    fn remove(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<(), PreferenceStorageError> {
        self.operations
            .lock()
            .unwrap()
            .push(format!("remove:{}/{}", key.namespace(), key.key()));
        self.values.lock().unwrap().remove(key);
        Ok(())
    }

    fn flush(
        &self,
        _authority: &PreferenceBackendWorkAuthority,
    ) -> Result<(), PreferenceStorageError> {
        self.operations.lock().unwrap().push("flush".to_owned());
        Ok(())
    }
}
