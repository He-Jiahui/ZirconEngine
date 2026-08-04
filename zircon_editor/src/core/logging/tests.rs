use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Barrier, Condvar, Mutex};
use std::thread;

use super::{
    EditorLogConfig, EditorLogError, EditorLogEventSink, EditorLogService, EditorLogStore,
    LogChannel, LogEntry, LogEventDelivery, LogFilter, LogJump, LogJumpTarget, LogRecord,
    LogSeverity, LogSource, RollingFileLogSink,
};

static NEXT_TEMP_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct ControlledEmissionState {
    before_count: usize,
    first_stored: bool,
    release_first: bool,
}

struct ControlledEmission {
    state: Mutex<ControlledEmissionState>,
    changed: Condvar,
}

impl ControlledEmission {
    fn new() -> Self {
        Self {
            state: Mutex::new(ControlledEmissionState::default()),
            changed: Condvar::new(),
        }
    }

    fn before_emission(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.before_count += 1;
        self.changed.notify_all();
    }

    fn after_store(&self, record: &LogRecord) {
        if record.sequence() != 1 {
            return;
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.first_stored = true;
        self.changed.notify_all();
        while !state.release_first {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn wait_until(&self, predicate: impl Fn(&ControlledEmissionState) -> bool) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !predicate(&state) {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn release_first(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.release_first = true;
        self.changed.notify_all();
    }
}

struct ReentrantEventSink {
    service: std::sync::Weak<EditorLogService>,
    reentering: AtomicBool,
}

struct SaturatingEventSink {
    gate: Arc<Barrier>,
    deliveries: Mutex<Vec<String>>,
}

struct RetryingResyncEventSink {
    resync_attempts: AtomicU64,
}

#[derive(Default)]
struct OrderedEventSink {
    sequences: Mutex<Vec<u64>>,
}

impl EditorLogEventSink for OrderedEventSink {
    fn publish(&self, record: &LogRecord) -> LogEventDelivery {
        self.sequences
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(record.sequence());
        LogEventDelivery::Delivered
    }

    fn resync_required(&self, _through_sequence: u64) -> LogEventDelivery {
        LogEventDelivery::Delivered
    }
}

impl EditorLogEventSink for ReentrantEventSink {
    fn publish(&self, _record: &LogRecord) -> LogEventDelivery {
        if self.reentering.swap(true, Ordering::SeqCst) {
            return LogEventDelivery::Delivered;
        }
        let delivery = match self.service.upgrade() {
            Some(service) => match service.emit(entry(
                "event sink reentry",
                LogSeverity::Warning,
                LogSource::editor(),
            )) {
                Ok(_) => LogEventDelivery::Delivered,
                Err(_) => LogEventDelivery::Rejected,
            },
            None => LogEventDelivery::Rejected,
        };
        self.reentering.store(false, Ordering::SeqCst);
        delivery
    }

    fn resync_required(&self, _through_sequence: u64) -> LogEventDelivery {
        LogEventDelivery::Delivered
    }
}

impl EditorLogEventSink for SaturatingEventSink {
    fn publish(&self, record: &LogRecord) -> LogEventDelivery {
        self.deliveries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(format!("record:{}", record.sequence()));
        if record.sequence() == 1 {
            self.gate.wait();
            self.gate.wait();
        }
        LogEventDelivery::Delivered
    }

    fn resync_required(&self, through_sequence: u64) -> LogEventDelivery {
        self.deliveries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(format!("resync:{through_sequence}"));
        LogEventDelivery::Delivered
    }
}

impl EditorLogEventSink for RetryingResyncEventSink {
    fn publish(&self, _record: &LogRecord) -> LogEventDelivery {
        LogEventDelivery::Rejected
    }

    fn resync_required(&self, _through_sequence: u64) -> LogEventDelivery {
        if self.resync_attempts.fetch_add(1, Ordering::SeqCst) == 0 {
            LogEventDelivery::Rejected
        } else {
            LogEventDelivery::Delivered
        }
    }
}

fn entry(message: &str, severity: LogSeverity, source: LogSource) -> LogEntry {
    LogEntry::new(source, severity, message, 42, None).unwrap()
}

#[test]
fn byte_bounded_store_evicts_oldest_records_before_accepting_new_entries() {
    let first = entry("first", LogSeverity::Info, LogSource::editor());
    let second = entry("second", LogSeverity::Warning, LogSource::runtime());
    let max_bytes = second.estimated_bytes();
    let store = EditorLogStore::new(EditorLogConfig::new(2, max_bytes).unwrap());

    store.push(first).unwrap();
    store.push(second.clone()).unwrap();
    store
        .push(entry("third", LogSeverity::Error, LogSource::import()))
        .unwrap();

    let records = store.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].entry().message(), "third");
    assert_eq!(store.diagnostics().dropped_records, 2);
    assert!(store.diagnostics().retained_bytes <= max_bytes);
}

#[test]
fn event_dispatch_limits_cannot_exceed_the_authoritative_store_budget() {
    let config = EditorLogConfig::new(2, 128).unwrap();

    assert!(matches!(
        config.with_event_queue_limits(3, 128),
        Err(EditorLogError::EventQueueEntryCapacityExceedsStore {
            maximum: 2,
            actual: 3,
        })
    ));
    assert!(matches!(
        config.with_event_queue_limits(2, 129),
        Err(EditorLogError::EventQueueByteCapacityExceedsStore {
            maximum: 128,
            actual: 129,
        })
    ));
}

#[test]
fn filters_channel_and_minimum_severity_without_reordering_records() {
    let store = EditorLogStore::new(EditorLogConfig::new(8, 4096).unwrap());
    store
        .push(entry("editor-info", LogSeverity::Info, LogSource::editor()))
        .unwrap();
    store
        .push(entry(
            "runtime-warning",
            LogSeverity::Warning,
            LogSource::runtime(),
        ))
        .unwrap();
    store
        .push(entry(
            "runtime-error",
            LogSeverity::Error,
            LogSource::runtime(),
        ))
        .unwrap();

    let filter = LogFilter::new(BTreeSet::from([LogChannel::Runtime]), LogSeverity::Warning);
    let records = store.snapshot(&filter);

    assert_eq!(
        records
            .iter()
            .map(|record| record.entry().message())
            .collect::<Vec<_>>(),
        vec!["runtime-warning", "runtime-error"]
    );
}

#[test]
fn entry_preserves_typed_jump_target() {
    let jump = LogJump::script_location("res://scripts/player.zr", 12, 4).unwrap();
    let entry = LogEntry::new(
        LogSource::script_build(),
        LogSeverity::Error,
        "script type mismatch",
        9,
        Some(jump.clone()),
    )
    .unwrap();

    assert_eq!(entry.jump(), Some(&jump));
    assert!(matches!(
        jump.target(),
        LogJumpTarget::ScriptLocation { path, line: 12, column: 4 }
            if path.as_ref() == "res://scripts/player.zr"
    ));
}

#[test]
fn typed_sources_and_jump_targets_reject_blank_external_identifiers() {
    assert!(matches!(
        LogSource::plugin("  "),
        Err(EditorLogError::EmptyPluginSource)
    ));
    assert!(matches!(
        LogJump::asset("\t"),
        Err(EditorLogError::EmptyJumpTarget)
    ));
}

#[test]
fn rolling_sink_starts_a_new_segment_when_daily_file_is_full() {
    let directory = std::env::temp_dir().join(format!(
        "zircon_editor_logging_test_{}_{}",
        std::process::id(),
        NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed),
    ));
    let sink = RollingFileLogSink::new(&directory, 1).unwrap();
    let record = EditorLogStore::new(EditorLogConfig::new(1, 4096).unwrap())
        .push(entry("roll", LogSeverity::Info, LogSource::editor()))
        .unwrap();

    let first = sink.append_for_day(&record, 20_000).unwrap();
    drop(sink);
    let reloaded_sink = RollingFileLogSink::new(&directory, 1).unwrap();
    let second = reloaded_sink.append_for_day(&record, 20_000).unwrap();
    drop(reloaded_sink);
    let second_reload = RollingFileLogSink::new(&directory, 1).unwrap();
    let third = second_reload.append_for_day(&record, 20_000).unwrap();
    let next_day = second_reload.append_for_day(&record, 20_001).unwrap();

    assert_ne!(first, second);
    assert_ne!(second, third);
    assert!(
        first
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("-0.log")
    );
    assert!(
        second
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("-1.log")
    );
    assert!(
        third
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("-2.log")
    );
    assert!(
        next_day
            .file_name()
            .unwrap()
            .to_string_lossy()
            .contains("editor-20001-0.log")
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn rolling_file_preserves_one_record_per_line_for_multiline_messages() {
    let directory = std::env::temp_dir().join(format!(
        "zircon_editor_logging_test_{}_{}",
        std::process::id(),
        NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed),
    ));
    let sink = RollingFileLogSink::new(&directory, 4096).unwrap();
    let record = EditorLogStore::new(EditorLogConfig::new(1, 4096).unwrap())
        .push(entry(
            "first\nsecond",
            LogSeverity::Info,
            LogSource::editor(),
        ))
        .unwrap();

    let path = sink.append_for_day(&record, 20_000).unwrap();
    let file = std::fs::read_to_string(path).unwrap();

    assert_eq!(file.lines().count(), 1);
    assert!(file.contains("message=first\\nsecond"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn service_serializes_sequence_assignment_and_file_append_across_threads() {
    let directory = std::env::temp_dir().join(format!(
        "zircon_editor_logging_test_{}_{}",
        std::process::id(),
        NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed),
    ));
    let service = Arc::new(EditorLogService::new(
        EditorLogConfig::new(16, 4096).unwrap(),
    ));
    service.configure_rolling_file(&directory, 4096).unwrap();
    let control = Arc::new(ControlledEmission::new());
    let before_control = Arc::clone(&control);
    let after_control = Arc::clone(&control);
    service.configure_emission_test_hooks(
        Arc::new(move || before_control.before_emission()),
        Arc::new(move |record| after_control.after_store(record)),
        Arc::new(|| {}),
    );
    let first_service = Arc::clone(&service);
    let first = thread::spawn(move || {
        first_service
            .emit(entry("first", LogSeverity::Info, LogSource::editor()))
            .unwrap()
    });
    control.wait_until(|state| state.first_stored);
    let second_service = Arc::clone(&service);
    let second = thread::spawn(move || {
        second_service
            .emit(entry("second", LogSeverity::Info, LogSource::editor()))
            .unwrap()
    });
    control.wait_until(|state| state.before_count == 2);
    assert_eq!(service.snapshot(&LogFilter::default()).len(), 1);
    control.release_first();
    let mut reports = vec![first.join().unwrap(), second.join().unwrap()];
    reports.sort_by_key(|report| report.record().sequence());
    let expected = reports
        .iter()
        .map(|report| format!("sequence={}", report.record().sequence()))
        .collect::<Vec<_>>();
    let file = std::fs::read_dir(&directory)
        .unwrap()
        .find_map(|entry| entry.ok().map(|entry| entry.path()))
        .unwrap();
    let actual = std::fs::read_to_string(file)
        .unwrap()
        .lines()
        .map(|line| line.split_whitespace().next().unwrap().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(actual, expected);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn event_sink_can_reenter_logging_without_deadlocking_the_emission_lock() {
    let service = Arc::new(EditorLogService::new(
        EditorLogConfig::new(8, 4096).unwrap(),
    ));
    service.configure_event_sink(Arc::new(ReentrantEventSink {
        service: Arc::downgrade(&service),
        reentering: AtomicBool::new(false),
    }));

    let report = service
        .emit(entry("outer", LogSeverity::Info, LogSource::editor()))
        .unwrap();

    assert_eq!(report.event_delivery(), LogEventDelivery::Delivered);
    assert_eq!(service.snapshot(&LogFilter::default()).len(), 2);
}

#[test]
fn event_dispatch_preserves_sequence_order_when_the_first_publish_is_paused() {
    let service = Arc::new(EditorLogService::new(
        EditorLogConfig::new(8, 4096).unwrap(),
    ));
    let sink = Arc::new(OrderedEventSink::default());
    let event_sink: Arc<dyn EditorLogEventSink> = Arc::clone(&sink);
    service.configure_event_sink(event_sink);
    let gate = Arc::new(Barrier::new(2));
    let gate_hook = Arc::clone(&gate);
    service.configure_emission_test_hooks(
        Arc::new(|| {}),
        Arc::new(|_| {}),
        Arc::new(move || {
            gate_hook.wait();
            gate_hook.wait();
        }),
    );
    let first_service = Arc::clone(&service);
    let first = thread::spawn(move || {
        first_service
            .emit(entry("first", LogSeverity::Info, LogSource::editor()))
            .unwrap()
    });
    gate.wait();

    let second = service
        .emit(entry("second", LogSeverity::Info, LogSource::editor()))
        .unwrap();
    assert_eq!(second.event_delivery(), LogEventDelivery::Queued);
    gate.wait();
    assert_eq!(
        first.join().unwrap().event_delivery(),
        LogEventDelivery::Delivered
    );
    assert_eq!(
        sink.sequences
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_slice(),
        [1, 2]
    );
}

#[test]
fn slow_event_sink_bounds_queued_records_and_requires_a_sequence_resync() {
    let service = Arc::new(EditorLogService::new(
        EditorLogConfig::new(8, 4096)
            .unwrap()
            .with_event_queue_limits(1, 4096)
            .unwrap(),
    ));
    let gate = Arc::new(Barrier::new(2));
    let sink = Arc::new(SaturatingEventSink {
        gate: Arc::clone(&gate),
        deliveries: Mutex::new(Vec::new()),
    });
    let event_sink: Arc<dyn EditorLogEventSink> = Arc::clone(&sink);
    service.configure_event_sink(event_sink);

    let first_service = Arc::clone(&service);
    let first = thread::spawn(move || {
        first_service
            .emit(entry("first", LogSeverity::Info, LogSource::editor()))
            .unwrap()
    });
    gate.wait();

    let second = service
        .emit(entry("second", LogSeverity::Info, LogSource::editor()))
        .unwrap();
    let third = service
        .emit(entry("third", LogSeverity::Info, LogSource::editor()))
        .unwrap();
    assert_eq!(second.event_delivery(), LogEventDelivery::Queued);
    assert_eq!(third.event_delivery(), LogEventDelivery::Backpressured);
    assert_eq!(service.diagnostics().queued_event_records, 1);
    assert_eq!(service.diagnostics().resync_required_records, 1);

    gate.wait();
    assert_eq!(
        first.join().unwrap().event_delivery(),
        LogEventDelivery::Delivered
    );
    assert_eq!(
        sink.deliveries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_slice(),
        ["record:1", "record:2", "resync:3"]
    );
    let diagnostics = service.diagnostics();
    assert_eq!(diagnostics.queued_event_records, 0);
    assert_eq!(diagnostics.resync_required_records, 1);
    assert_eq!(diagnostics.event_resyncs, 1);
}

#[test]
fn slow_event_sink_uses_the_byte_budget_before_its_entry_budget() {
    let event = entry("repeat", LogSeverity::Info, LogSource::editor());
    let event_bytes = event.estimated_bytes();
    let service = Arc::new(EditorLogService::new(
        EditorLogConfig::new(3, event_bytes.saturating_mul(3))
            .unwrap()
            .with_event_queue_limits(2, event_bytes)
            .unwrap(),
    ));
    let gate = Arc::new(Barrier::new(2));
    let sink = Arc::new(SaturatingEventSink {
        gate: Arc::clone(&gate),
        deliveries: Mutex::new(Vec::new()),
    });
    let event_sink: Arc<dyn EditorLogEventSink> = Arc::clone(&sink);
    service.configure_event_sink(event_sink);

    let first_service = Arc::clone(&service);
    let first = thread::spawn(move || first_service.emit(event).unwrap());
    gate.wait();

    let second = service
        .emit(entry("repeat", LogSeverity::Info, LogSource::editor()))
        .unwrap();
    let third = service
        .emit(entry("repeat", LogSeverity::Info, LogSource::editor()))
        .unwrap();
    assert_eq!(second.event_delivery(), LogEventDelivery::Queued);
    assert_eq!(third.event_delivery(), LogEventDelivery::Backpressured);
    assert_eq!(service.diagnostics().queued_event_records, 1);
    assert!(service.diagnostics().queued_event_records < 2);
    assert_eq!(service.diagnostics().queued_event_bytes, event_bytes);

    gate.wait();
    assert_eq!(
        first.join().unwrap().event_delivery(),
        LogEventDelivery::Delivered
    );
    assert_eq!(service.diagnostics().event_resyncs, 1);
}

#[test]
fn rejected_resync_is_retained_until_a_later_emit_can_deliver_it() {
    let service = EditorLogService::new(EditorLogConfig::new(4, 4096).unwrap());
    service.configure_event_sink(Arc::new(RetryingResyncEventSink {
        resync_attempts: AtomicU64::new(0),
    }));

    let first = service
        .emit(entry("first", LogSeverity::Info, LogSource::editor()))
        .unwrap();
    assert_eq!(first.event_delivery(), LogEventDelivery::Rejected);
    assert_eq!(service.diagnostics().event_resyncs, 0);
    assert_eq!(service.diagnostics().failed_event_resyncs, 1);

    let second = service
        .emit(entry("second", LogSeverity::Info, LogSource::editor()))
        .unwrap();
    assert_eq!(second.event_delivery(), LogEventDelivery::Backpressured);
    assert_eq!(service.diagnostics().event_resyncs, 1);
    assert_eq!(service.diagnostics().failed_event_resyncs, 1);
}

#[test]
fn rolling_file_io_failure_keeps_the_authoritative_memory_record() {
    let root_file = std::env::temp_dir().join(format!(
        "zircon_editor_logging_file_{}_{}",
        std::process::id(),
        NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed),
    ));
    std::fs::write(&root_file, "not a directory").unwrap();
    let service = EditorLogService::new(EditorLogConfig::new(2, 4096).unwrap());
    service.configure_rolling_file(&root_file, 4096).unwrap();

    let report = service
        .emit(entry("retained", LogSeverity::Error, LogSource::runtime()))
        .unwrap();

    assert!(!report.persisted_to_disk());
    assert!(report.persistence_error().is_some());
    assert_eq!(
        service
            .record(report.record().sequence())
            .unwrap()
            .entry()
            .message(),
        "retained"
    );
    std::fs::remove_file(root_file).unwrap();
}

#[test]
fn workspace_default_configures_the_project_diagnostics_directory() {
    let workspace = std::env::temp_dir().join(format!(
        "zircon_editor_workspace_{}_{}",
        std::process::id(),
        NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed),
    ));
    let service = EditorLogService::with_workspace_diagnostics(&workspace);

    let report = service
        .emit(entry("persisted", LogSeverity::Info, LogSource::editor()))
        .unwrap();

    assert!(service.rolling_file_configuration_error().is_none());
    assert!(report.persisted_to_disk());
    assert!(workspace.join(".zircon").join("logs").is_dir());
    service.disable_rolling_file();
    let after_close = service
        .emit(entry("memory only", LogSeverity::Info, LogSource::editor()))
        .unwrap();
    assert!(!after_close.persisted_to_disk());
    std::fs::remove_dir_all(workspace).unwrap();
}
