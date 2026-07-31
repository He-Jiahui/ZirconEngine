use std::collections::HashMap;
use std::io::{self, Cursor, Read};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::core::framework::platform::{
    PreferenceDurabilityState, PreferenceKey, PreferenceMutationTerminal,
    PreferenceStorageBackendKind, PreferenceStorageError, PreferenceStorageErrorKind,
    PreferenceStorageOperation, PreferenceTicketWaitResult, PreferenceWorkDeadline,
};
use crate::platform::preferences::{PreferenceBackendWorkAuthority, PreferenceStorageBackend};

use super::adapter::{
    PreferencePersistenceAdapter, PreferencePersistenceLimits, MAX_PREFERENCE_FAILURE_DETAIL_BYTES,
};
use super::work::read_bounded;

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
fn platform_preference_storage_bounded_read_consumes_at_most_max_plus_one_bytes() {
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
    PreferencePersistenceAdapter::new(
        backend,
        PreferencePersistenceLimits {
            max_value_bytes,
            max_overlay_entries: 16,
            max_overlay_retained_bytes: 64 * 1024,
            max_lane_entries: 16,
            max_lane_retained_bytes: 64 * 1024,
        },
    )
}

#[derive(Default)]
struct MemoryBackend {
    values: Mutex<HashMap<PreferenceKey, Arc<[u8]>>>,
    operations: Mutex<Vec<String>>,
    fail_writes: AtomicUsize,
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
        if self.fail_writes.swap(0, Ordering::Relaxed) != 0 {
            return Err(PreferenceStorageError::new(
                PreferenceStorageErrorKind::TransientIo,
                PreferenceStorageOperation::Write,
                "memory",
                "é".repeat(MAX_PREFERENCE_FAILURE_DETAIL_BYTES + 8),
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
