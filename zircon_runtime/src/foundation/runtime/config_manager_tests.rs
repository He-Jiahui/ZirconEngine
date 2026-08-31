use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::core::framework::foundation::{ConfigManager, ConfigManagerError};
use crate::core::resource::io::{atomic_write_with_fault, stage_atomic_write, AtomicWriteFault};
use crate::core::CoreRuntime;

use super::config_manager::{
    ConfigCommitFenceForTest, ConfigFileWriterForTest, DefaultConfigManager,
};

const TEST_FLUSH_TIMEOUT: Duration = Duration::from_secs(2);
const TEST_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);
const LONG_TEST_DEBOUNCE: Duration = Duration::from_secs(60);
static NEXT_TEST_PATH_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
struct RecordingWriter {
    failures_remaining: AtomicUsize,
    writes: Mutex<Vec<RecordedWrite>>,
}

struct RecordedWrite {
    thread_id: ThreadId,
    bytes: Vec<u8>,
}

#[derive(Default)]
struct BlockingWriter {
    state: Mutex<BlockingWriterState>,
    changed: Condvar,
}

#[derive(Default)]
struct BlockingWriterState {
    entered: bool,
    released: bool,
}

impl BlockingWriter {
    fn wait_until_entered(&self) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _state = self
            .changed
            .wait_while(state, |state| !state.entered)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }

    fn release(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.released = true;
        self.changed.notify_all();
    }
}

impl ConfigFileWriterForTest for BlockingWriter {
    fn write(
        &self,
        _path: &Path,
        _bytes: &[u8],
        commit_fence: &ConfigCommitFenceForTest,
    ) -> io::Result<()> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.entered = true;
        self.changed.notify_all();
        let _state = self
            .changed
            .wait_while(state, |state| !state.released)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        commit_fence.commit(|| Ok(()))
    }
}

impl RecordingWriter {
    fn failing(failures: usize) -> Self {
        Self {
            failures_remaining: AtomicUsize::new(failures),
            writes: Mutex::new(Vec::new()),
        }
    }

    fn writes(&self) -> std::sync::MutexGuard<'_, Vec<RecordedWrite>> {
        self.writes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl ConfigFileWriterForTest for RecordingWriter {
    fn write(
        &self,
        _path: &Path,
        bytes: &[u8],
        commit_fence: &ConfigCommitFenceForTest,
    ) -> io::Result<()> {
        commit_fence.commit(|| {
            if self
                .failures_remaining
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |remaining| {
                    (remaining > 0).then_some(remaining - 1)
                })
                .is_ok()
            {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "injected config persistence failure",
                ));
            }
            self.writes().push(RecordedWrite {
                thread_id: std::thread::current().id(),
                bytes: bytes.to_vec(),
            });
            Ok(())
        })
    }
}

#[derive(Default)]
struct FirstReplaceFailsAtomicWriter {
    attempts: AtomicUsize,
}

impl ConfigFileWriterForTest for FirstReplaceFailsAtomicWriter {
    fn write(
        &self,
        path: &Path,
        bytes: &[u8],
        commit_fence: &ConfigCommitFenceForTest,
    ) -> io::Result<()> {
        let fault = if self.attempts.fetch_add(1, Ordering::AcqRel) == 0 {
            AtomicWriteFault::Replace
        } else {
            AtomicWriteFault::None
        };
        commit_fence.commit(|| atomic_write_with_fault(path, bytes, fault))
    }
}

#[derive(Default)]
struct BlockingAtomicWriter {
    state: Mutex<BlockingAtomicWriterState>,
    changed: Condvar,
}

#[derive(Default)]
struct BlockingAtomicWriterState {
    entered: bool,
    released: bool,
    completed: bool,
}

impl BlockingAtomicWriter {
    fn wait_until_entered(&self) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _state = self
            .changed
            .wait_while(state, |state| !state.entered)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }

    fn release(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.released = true;
        self.changed.notify_all();
    }

    fn wait_until_completed(&self) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _state = self
            .changed
            .wait_while(state, |state| !state.completed)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
}

impl ConfigFileWriterForTest for BlockingAtomicWriter {
    fn write(
        &self,
        path: &Path,
        bytes: &[u8],
        commit_fence: &ConfigCommitFenceForTest,
    ) -> io::Result<()> {
        let pending = stage_atomic_write(path, bytes)?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.entered = true;
        self.changed.notify_all();
        let _state = self
            .changed
            .wait_while(state, |state| !state.released)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        drop(_state);

        let result = commit_fence.commit(|| pending.commit());
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.completed = true;
        self.changed.notify_all();
        result
    }
}

#[derive(Default)]
struct FencedAtomicWriter;

impl ConfigFileWriterForTest for FencedAtomicWriter {
    fn write(
        &self,
        path: &Path,
        bytes: &[u8],
        commit_fence: &ConfigCommitFenceForTest,
    ) -> io::Result<()> {
        let pending = stage_atomic_write(path, bytes)?;
        commit_fence.commit(|| pending.commit())
    }
}

#[derive(Default)]
struct CommitAdmittedBlockingWriter {
    state: Mutex<BlockingAtomicWriterState>,
    changed: Condvar,
}

impl CommitAdmittedBlockingWriter {
    fn wait_until_entered(&self) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _state = self
            .changed
            .wait_while(state, |state| !state.entered)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }

    fn release(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.released = true;
        self.changed.notify_all();
    }

    fn wait_until_completed(&self) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _state = self
            .changed
            .wait_while(state, |state| !state.completed)
            .unwrap_or_else(|poisoned| poisoned.into_inner());
    }
}

impl ConfigFileWriterForTest for CommitAdmittedBlockingWriter {
    fn write(
        &self,
        path: &Path,
        bytes: &[u8],
        commit_fence: &ConfigCommitFenceForTest,
    ) -> io::Result<()> {
        let pending = stage_atomic_write(path, bytes)?;
        let result = commit_fence.commit(|| {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.entered = true;
            self.changed.notify_all();
            let _state = self
                .changed
                .wait_while(state, |state| !state.released)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            drop(_state);
            pending.commit()
        });
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.completed = true;
        self.changed.notify_all();
        result
    }
}

#[test]
fn identical_config_values_do_not_advance_the_dirty_generation() {
    let runtime = CoreRuntime::new();
    let writer = Arc::new(RecordingWriter::default());
    let manager = test_manager(&runtime, Arc::clone(&writer), LONG_TEST_DEBOUNCE);

    manager
        .set_value("editor.layout", json!({"dock": "main"}))
        .unwrap();
    manager.flush(TEST_FLUSH_TIMEOUT).unwrap();
    manager
        .set_value("editor.layout", json!({"dock": "main"}))
        .unwrap();

    let report = manager.persistence_report();
    assert_eq!(report.dirty_generation, 1);
    assert_eq!(report.persisted_generation, 1);
    assert_eq!(report.flush_attempts, 1);
    assert_eq!(writer.writes().len(), 1);
}

#[test]
fn config_burst_coalesces_and_never_writes_on_the_caller_thread() {
    let runtime = CoreRuntime::new();
    let writer = Arc::new(RecordingWriter::default());
    let manager = test_manager(&runtime, Arc::clone(&writer), LONG_TEST_DEBOUNCE);
    let caller_thread = std::thread::current().id();

    for value in 0..1_000 {
        manager.set_value("runtime.burst", json!(value)).unwrap();
    }
    assert_eq!(writer.writes().len(), 0);
    manager.flush(TEST_FLUSH_TIMEOUT).unwrap();

    let writes = writer.writes();
    assert_eq!(writes.len(), 1);
    assert_ne!(writes[0].thread_id, caller_thread);
    let document = serde_json::from_slice::<Value>(&writes[0].bytes).unwrap();
    assert_eq!(document["runtime.burst"], json!(999));
    drop(writes);

    let report = manager.persistence_report();
    assert_eq!(report.dirty_generation, 1_000);
    assert_eq!(report.persisted_generation, 1_000);
    assert_eq!(report.pending_flushes, 0);
    assert_eq!(report.peak_pending_flushes, 1);
    assert_eq!(report.flush_attempts, 1);
    assert_eq!(report.successful_writes, 1);
    assert_eq!(report.failed_writes, 0);
    assert!(report.serialized_bytes > 0);
}

#[test]
fn concurrent_config_updates_are_persisted_without_loss() {
    const THREAD_COUNT: usize = 4;
    const KEYS_PER_THREAD: usize = 50;

    let runtime = CoreRuntime::new();
    let writer = Arc::new(RecordingWriter::default());
    let manager = test_manager(&runtime, Arc::clone(&writer), LONG_TEST_DEBOUNCE);
    let threads = (0..THREAD_COUNT)
        .map(|thread_index| {
            let manager = manager.clone();
            std::thread::spawn(move || {
                for key_index in 0..KEYS_PER_THREAD {
                    manager
                        .set_value(
                            &format!("runtime.concurrent.{thread_index}.{key_index}"),
                            json!(key_index),
                        )
                        .unwrap();
                }
            })
        })
        .collect::<Vec<_>>();
    for thread in threads {
        thread.join().unwrap();
    }
    manager.flush(TEST_FLUSH_TIMEOUT).unwrap();

    let writes = writer.writes();
    assert_eq!(writes.len(), 1);
    let document =
        serde_json::from_slice::<serde_json::Map<String, Value>>(&writes[0].bytes).unwrap();
    assert_eq!(document.len(), THREAD_COUNT * KEYS_PER_THREAD);
    for thread_index in 0..THREAD_COUNT {
        for key_index in 0..KEYS_PER_THREAD {
            assert_eq!(
                document.get(&format!("runtime.concurrent.{thread_index}.{key_index}")),
                Some(&json!(key_index))
            );
        }
    }
}

#[test]
fn identical_value_retries_a_failed_dirty_generation() {
    let runtime = CoreRuntime::new();
    let writer = Arc::new(RecordingWriter::failing(1));
    let manager = test_manager(&runtime, Arc::clone(&writer), LONG_TEST_DEBOUNCE);

    manager.set_value("runtime.retry", json!(7)).unwrap();
    assert!(matches!(
        manager.flush(TEST_FLUSH_TIMEOUT),
        Err(ConfigManagerError::Persistence { .. })
    ));
    let failed = manager.persistence_report();
    assert_eq!(failed.dirty_generation, 1);
    assert_eq!(failed.persisted_generation, 0);
    assert_eq!(failed.failed_writes, 1);
    assert!(failed.last_error.is_some());

    manager.set_value("runtime.retry", json!(7)).unwrap();
    manager.flush(TEST_FLUSH_TIMEOUT).unwrap();
    let recovered = manager.persistence_report();
    assert_eq!(recovered.dirty_generation, 1);
    assert_eq!(recovered.persisted_generation, 1);
    assert_eq!(recovered.flush_attempts, 2);
    assert_eq!(recovered.successful_writes, 1);
    assert_eq!(recovered.failed_writes, 1);
    assert!(recovered.last_error.is_none());
}

#[test]
fn atomic_replace_failure_keeps_old_json_and_same_value_retry_commits_new_json() {
    let root = unique_temp_root("atomic_retry");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("config.json");
    std::fs::write(&path, br#"{"version":1}"#).unwrap();
    let runtime = CoreRuntime::new();
    let manager = DefaultConfigManager::new_with_options(
        &runtime.handle(),
        path.clone(),
        Arc::new(FirstReplaceFailsAtomicWriter::default()),
        LONG_TEST_DEBOUNCE,
        TEST_SHUTDOWN_TIMEOUT,
    )
    .unwrap();

    manager.set_value("version", json!(2)).unwrap();
    assert!(manager.flush(TEST_FLUSH_TIMEOUT).is_err());
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&path).unwrap()).unwrap(),
        json!({"version": 1})
    );

    manager.set_value("version", json!(2)).unwrap();
    manager.flush(TEST_FLUSH_TIMEOUT).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&path).unwrap()).unwrap(),
        json!({"version": 2})
    );
    drop(manager);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn last_config_manager_owner_performs_a_bounded_shutdown_flush() {
    let runtime = CoreRuntime::new();
    let writer = Arc::new(RecordingWriter::default());
    let manager = test_manager(&runtime, Arc::clone(&writer), LONG_TEST_DEBOUNCE);

    manager.set_value("runtime.shutdown", json!(true)).unwrap();
    drop(manager);

    let writes = writer.writes();
    assert_eq!(writes.len(), 1);
    let document = serde_json::from_slice::<Value>(&writes[0].bytes).unwrap();
    assert_eq!(document["runtime.shutdown"], json!(true));
}

#[test]
fn explicit_flush_returns_when_its_timeout_expires() {
    let runtime = CoreRuntime::new();
    let writer = Arc::new(BlockingWriter::default());
    let manager = DefaultConfigManager::new_with_options(
        &runtime.handle(),
        unique_temp_root("flush_timeout").join("config.json"),
        writer.clone(),
        Duration::ZERO,
        TEST_SHUTDOWN_TIMEOUT,
    )
    .unwrap();

    manager.set_value("runtime.blocked", json!(true)).unwrap();
    writer.wait_until_entered();
    let timeout = Duration::from_millis(25);
    let error = manager.flush(timeout).unwrap_err();
    assert!(matches!(
        error,
        ConfigManagerError::FlushTimedOut {
            timeout: actual_timeout,
            ..
        } if actual_timeout == timeout
    ));

    writer.release();
    manager.flush(TEST_FLUSH_TIMEOUT).unwrap();
}

#[test]
fn shutdown_timeout_fences_a_late_writer_from_overwriting_a_new_manager() {
    let root = unique_temp_root("shutdown_fence");
    let path = root.join("config.json");

    let old_runtime = CoreRuntime::new();
    let old_writer = Arc::new(BlockingAtomicWriter::default());
    let old_manager = DefaultConfigManager::new_with_options(
        &old_runtime.handle(),
        path.clone(),
        old_writer.clone(),
        Duration::ZERO,
        Duration::from_millis(25),
    )
    .unwrap();
    old_manager.set_value("owner", json!("stale")).unwrap();
    old_writer.wait_until_entered();

    let shutdown_started = Instant::now();
    drop(old_manager);
    assert!(shutdown_started.elapsed() < Duration::from_millis(500));

    let new_runtime = CoreRuntime::new();
    let new_manager = DefaultConfigManager::new_with_options(
        &new_runtime.handle(),
        path.clone(),
        Arc::new(FencedAtomicWriter),
        Duration::ZERO,
        TEST_SHUTDOWN_TIMEOUT,
    )
    .unwrap();
    new_manager.set_value("owner", json!("fresh")).unwrap();
    new_manager.flush(TEST_FLUSH_TIMEOUT).unwrap();

    old_writer.release();
    old_writer.wait_until_completed();
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&path).unwrap()).unwrap(),
        json!({"owner": "fresh"})
    );
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);

    drop(new_manager);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn replacement_activation_fails_fast_while_an_admitted_commit_is_still_running() {
    let root = unique_temp_root("commit_in_progress");
    let path = root.join("config.json");
    let old_runtime = CoreRuntime::new();
    let old_writer = Arc::new(CommitAdmittedBlockingWriter::default());
    let old_manager = DefaultConfigManager::new_with_options(
        &old_runtime.handle(),
        path.clone(),
        old_writer.clone(),
        Duration::ZERO,
        Duration::from_millis(25),
    )
    .unwrap();
    old_manager.set_value("owner", json!("stale")).unwrap();
    old_writer.wait_until_entered();
    drop(old_manager);

    let new_runtime = CoreRuntime::new();
    let activation_started = Instant::now();
    let error = DefaultConfigManager::new_with_options(
        &new_runtime.handle(),
        path.clone(),
        Arc::new(FencedAtomicWriter),
        Duration::ZERO,
        TEST_SHUTDOWN_TIMEOUT,
    )
    .unwrap_err();
    assert!(activation_started.elapsed() < Duration::from_millis(500));
    assert!(error.to_string().contains("still in progress"));

    old_writer.release();
    old_writer.wait_until_completed();
    let new_manager = DefaultConfigManager::new_with_options(
        &new_runtime.handle(),
        path.clone(),
        Arc::new(FencedAtomicWriter),
        Duration::ZERO,
        TEST_SHUTDOWN_TIMEOUT,
    )
    .unwrap();
    new_manager.set_value("owner", json!("fresh")).unwrap();
    new_manager.flush(TEST_FLUSH_TIMEOUT).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&path).unwrap()).unwrap(),
        json!({"owner": "fresh"})
    );

    drop(new_manager);
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn startup_recovers_a_missing_config_from_its_single_atomic_backup() {
    let root = unique_temp_root("startup_backup_recovery");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("config.json");
    let backup = root.join(format!(".config.json.zr-backup-{}-1", std::process::id()));
    std::fs::write(&backup, br#"{"owner":"recovered"}"#).unwrap();
    let runtime = CoreRuntime::new();

    let manager = DefaultConfigManager::new_with_options(
        &runtime.handle(),
        path.clone(),
        Arc::new(RecordingWriter::default()),
        LONG_TEST_DEBOUNCE,
        TEST_SHUTDOWN_TIMEOUT,
    )
    .unwrap();

    assert_eq!(manager.get_value("owner"), Some(json!("recovered")));
    assert!(path.is_file());
    assert!(!backup.exists());
    drop(manager);
    std::fs::remove_dir_all(root).unwrap();
}

#[cfg(windows)]
#[test]
fn startup_rejects_multiple_atomic_backups_for_a_missing_config() {
    let root = unique_temp_root("startup_backup_ambiguous");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("config.json");
    for id in 1..=2 {
        std::fs::write(
            root.join(format!(
                ".config.json.zr-backup-{}-{id}",
                std::process::id()
            )),
            br#"{"owner":"ambiguous"}"#,
        )
        .unwrap();
    }
    let runtime = CoreRuntime::new();

    let error = DefaultConfigManager::new_with_options(
        &runtime.handle(),
        path.clone(),
        Arc::new(RecordingWriter::default()),
        LONG_TEST_DEBOUNCE,
        TEST_SHUTDOWN_TIMEOUT,
    )
    .unwrap_err();

    assert!(error.to_string().contains("2 backup candidates"));
    assert!(!path.exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_startup_config_returns_an_actionable_error() {
    let root = unique_temp_root("malformed");
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("config.json");
    std::fs::write(&path, b"{not-json").unwrap();
    let runtime = CoreRuntime::new();

    let error = DefaultConfigManager::new_with_options(
        &runtime.handle(),
        path.clone(),
        Arc::new(RecordingWriter::default()),
        LONG_TEST_DEBOUNCE,
        TEST_SHUTDOWN_TIMEOUT,
    )
    .unwrap_err();

    assert!(error.to_string().contains(path.to_string_lossy().as_ref()));
    std::fs::remove_dir_all(root).unwrap();
}

fn test_manager(
    runtime: &CoreRuntime,
    writer: Arc<RecordingWriter>,
    debounce: Duration,
) -> DefaultConfigManager {
    DefaultConfigManager::new_with_options(
        &runtime.handle(),
        unique_temp_root("memory").join("config.json"),
        writer,
        debounce,
        TEST_SHUTDOWN_TIMEOUT,
    )
    .unwrap()
}

fn unique_temp_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon_config_manager_{name}_{}_{}",
        std::process::id(),
        NEXT_TEST_PATH_ID.fetch_add(1, Ordering::Relaxed)
    ))
}
