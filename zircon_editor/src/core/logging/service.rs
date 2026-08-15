use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use super::{
    EditorLogConfig, EditorLogDiagnostics, EditorLogError, EditorLogStore, LogEntry, LogFilter,
    LogRecord, RollingFileLogSink,
};

#[derive(Clone, Debug)]
pub struct LogWriteReport {
    record: LogRecord,
    persisted_to_disk: bool,
    persistence_error: Option<String>,
    event_delivery: LogEventDelivery,
}

impl LogWriteReport {
    fn new(
        record: LogRecord,
        persisted_to_disk: bool,
        persistence_error: Option<String>,
        event_delivery: LogEventDelivery,
    ) -> Self {
        Self {
            record,
            persisted_to_disk,
            persistence_error,
            event_delivery,
        }
    }

    pub fn record(&self) -> &LogRecord {
        &self.record
    }

    pub const fn persisted_to_disk(&self) -> bool {
        self.persisted_to_disk
    }

    pub fn persistence_error(&self) -> Option<&str> {
        self.persistence_error.as_deref()
    }

    pub const fn event_delivery(&self) -> LogEventDelivery {
        self.event_delivery
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogEventDelivery {
    NotConfigured,
    Queued,
    Delivered,
    Backpressured,
    Rejected,
}

pub trait EditorLogEventSink: Send + Sync {
    fn publish(&self, record: &LogRecord) -> LogEventDelivery;

    /// Delivery continuity may have broken at or before this sequence. The sink must rebuild
    /// its view from the service store instead of treating later events as complete.
    fn resync_required(&self, through_sequence: u64) -> LogEventDelivery;
}

#[derive(Default)]
struct LogEventDispatchState {
    pending: VecDeque<LogEventDispatch>,
    pending_bytes: usize,
    resync: Option<LogEventResync>,
    resync_required_records: u64,
    event_resyncs: u64,
    failed_event_resyncs: u64,
    dispatching: bool,
}

struct LogEventDispatch {
    record: LogRecord,
    sink: Arc<dyn EditorLogEventSink>,
    estimated_bytes: usize,
}

struct LogEventResync {
    sink: Arc<dyn EditorLogEventSink>,
    through_sequence: u64,
}

enum LogEventQueueOutcome {
    Enqueued { dispatch_now: bool },
    Backpressured { dispatch_now: bool },
}

enum PendingLogDispatch {
    Record(LogEventDispatch),
    Resync(LogEventResync),
}

pub struct EditorLogService {
    store: EditorLogStore,
    emission: Mutex<()>,
    event_sink: Mutex<Option<Arc<dyn EditorLogEventSink>>>,
    event_dispatch: Mutex<LogEventDispatchState>,
    event_queue_entry_capacity: usize,
    event_queue_retained_bytes: usize,
    rolling_file: Mutex<Option<RollingFileLogSink>>,
    rolling_file_configuration_error: Mutex<Option<Arc<str>>>,
    #[cfg(test)]
    before_emission_hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
    #[cfg(test)]
    after_store_hook: Mutex<Option<Arc<dyn Fn(&LogRecord) + Send + Sync>>>,
    #[cfg(test)]
    before_event_dispatch_hook: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,
}

impl Default for EditorLogService {
    fn default() -> Self {
        Self::new(EditorLogConfig::default())
    }
}

impl EditorLogService {
    pub fn new(config: EditorLogConfig) -> Self {
        Self {
            store: EditorLogStore::new(config),
            emission: Mutex::new(()),
            event_sink: Mutex::new(None),
            event_dispatch: Mutex::new(LogEventDispatchState::default()),
            event_queue_entry_capacity: config.event_queue_entry_capacity(),
            event_queue_retained_bytes: config.event_queue_retained_bytes(),
            rolling_file: Mutex::new(None),
            rolling_file_configuration_error: Mutex::new(None),
            #[cfg(test)]
            before_emission_hook: Mutex::new(None),
            #[cfg(test)]
            after_store_hook: Mutex::new(None),
            #[cfg(test)]
            before_event_dispatch_hook: Mutex::new(None),
        }
    }

    pub fn with_workspace_diagnostics(workspace_root: impl AsRef<Path>) -> Self {
        let service = Self::default();
        if let Err(error) = service.configure_workspace_diagnostics(workspace_root) {
            *service.lock_rolling_file_configuration_error() = Some(Arc::from(error.to_string()));
        }
        service
    }

    pub fn configure_workspace_diagnostics(
        &self,
        workspace_root: impl AsRef<Path>,
    ) -> Result<(), EditorLogError> {
        self.configure_rolling_file(
            workspace_root.as_ref().join(".zircon").join("logs"),
            32 * 1024 * 1024,
        )
    }

    pub fn configure_rolling_file(
        &self,
        root: impl Into<PathBuf>,
        max_file_bytes: u64,
    ) -> Result<(), EditorLogError> {
        let sink = RollingFileLogSink::new(root, max_file_bytes)?;
        let _emission = self.lock_emission();
        *self.lock_rolling_file() = Some(sink);
        *self.lock_rolling_file_configuration_error() = None;
        Ok(())
    }

    pub fn configure_event_sink(&self, sink: Arc<dyn EditorLogEventSink>) {
        let _emission = self.lock_emission();
        *self.lock_event_sink() = Some(sink);
    }

    pub fn disable_rolling_file(&self) {
        let _emission = self.lock_emission();
        *self.lock_rolling_file() = None;
        *self.lock_rolling_file_configuration_error() = None;
    }

    pub fn emit(&self, entry: LogEntry) -> Result<LogWriteReport, EditorLogError> {
        #[cfg(test)]
        self.run_before_emission_hook();
        let (record, persisted_to_disk, persistence_error, event_queue_outcome) = {
            let _emission = self.lock_emission();
            let record = self.store.push(entry)?;
            #[cfg(test)]
            self.run_after_store_hook(&record);
            let (persisted_to_disk, persistence_error) = match self.lock_rolling_file().as_ref() {
                Some(sink) => match sink.append(&record) {
                    Ok(_) => (true, None),
                    Err(error) => (false, Some(error.to_string())),
                },
                None => (false, None),
            };
            let event_queue_outcome = self
                .lock_event_sink()
                .clone()
                .map(|sink| self.enqueue_event(record.clone(), sink));
            (
                record,
                persisted_to_disk,
                persistence_error,
                event_queue_outcome,
            )
        };
        let event_delivery = match event_queue_outcome {
            Some(LogEventQueueOutcome::Enqueued { dispatch_now: true }) => {
                self.dispatch_pending_events()
            }
            Some(LogEventQueueOutcome::Enqueued {
                dispatch_now: false,
            }) => LogEventDelivery::Queued,
            Some(LogEventQueueOutcome::Backpressured { dispatch_now }) => {
                if dispatch_now {
                    let _ = self.dispatch_pending_events();
                }
                LogEventDelivery::Backpressured
            }
            None => LogEventDelivery::NotConfigured,
        };
        Ok(LogWriteReport::new(
            record,
            persisted_to_disk,
            persistence_error,
            event_delivery,
        ))
    }

    pub fn snapshot(&self, filter: &LogFilter) -> Vec<LogRecord> {
        self.store.snapshot(filter)
    }

    pub fn record(&self, sequence: u64) -> Option<LogRecord> {
        self.store.record(sequence)
    }

    pub fn clear(&self) -> usize {
        let (cleared, dispatch_now) = {
            let _emission = self.lock_emission();
            let (cleared, through_sequence) = self.store.clear();
            let dispatch_now = through_sequence
                .zip(self.lock_event_sink().clone())
                .is_some_and(|(through_sequence, sink)| {
                    self.enqueue_clear_resync(sink, through_sequence, cleared)
                });
            (cleared, dispatch_now)
        };
        if dispatch_now {
            let _ = self.dispatch_pending_events();
        }
        cleared
    }

    pub fn diagnostics(&self) -> EditorLogDiagnostics {
        let mut diagnostics = self.store.diagnostics();
        let state = self.lock_event_dispatch();
        diagnostics.queued_event_records = state.pending.len();
        diagnostics.queued_event_bytes = state.pending_bytes;
        diagnostics.resync_required_records = state.resync_required_records;
        diagnostics.event_resyncs = state.event_resyncs;
        diagnostics.failed_event_resyncs = state.failed_event_resyncs;
        diagnostics
    }

    pub fn rolling_file_configuration_error(&self) -> Option<Arc<str>> {
        self.lock_rolling_file_configuration_error().clone()
    }

    fn lock_rolling_file(&self) -> MutexGuard<'_, Option<RollingFileLogSink>> {
        self.rolling_file
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_event_sink(&self) -> MutexGuard<'_, Option<Arc<dyn EditorLogEventSink>>> {
        self.event_sink
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn enqueue_event(
        &self,
        record: LogRecord,
        sink: Arc<dyn EditorLogEventSink>,
    ) -> LogEventQueueOutcome {
        let estimated_bytes = record.entry().estimated_bytes();
        let mut state = self.lock_event_dispatch();
        if let Some(resync) = state.resync.as_mut() {
            resync.through_sequence = resync.through_sequence.max(record.sequence());
            state.resync_required_records = state.resync_required_records.saturating_add(1);
            let dispatch_now = !state.dispatching;
            if dispatch_now {
                state.dispatching = true;
            }
            return LogEventQueueOutcome::Backpressured { dispatch_now };
        }
        let queue_is_full = state.pending.len() >= self.event_queue_entry_capacity
            || estimated_bytes > self.event_queue_retained_bytes
            || state.pending_bytes > self.event_queue_retained_bytes - estimated_bytes;
        if queue_is_full {
            state.resync = Some(LogEventResync {
                sink,
                through_sequence: record.sequence(),
            });
            state.resync_required_records = state.resync_required_records.saturating_add(1);
            let dispatch_now = !state.dispatching;
            if dispatch_now {
                state.dispatching = true;
            }
            return LogEventQueueOutcome::Backpressured { dispatch_now };
        }
        state.pending.push_back(LogEventDispatch {
            record,
            sink,
            estimated_bytes,
        });
        state.pending_bytes += estimated_bytes;
        let dispatch_now = !state.dispatching;
        if dispatch_now {
            state.dispatching = true;
        }
        LogEventQueueOutcome::Enqueued { dispatch_now }
    }

    fn enqueue_clear_resync(
        &self,
        sink: Arc<dyn EditorLogEventSink>,
        through_sequence: u64,
        cleared: usize,
    ) -> bool {
        let mut state = self.lock_event_dispatch();
        state.pending.clear();
        state.pending_bytes = 0;
        let through_sequence = state.resync.take().map_or(through_sequence, |resync| {
            through_sequence.max(resync.through_sequence)
        });
        state.resync = Some(LogEventResync {
            sink,
            through_sequence,
        });
        state.resync_required_records =
            state.resync_required_records.saturating_add(cleared as u64);
        let dispatch_now = !state.dispatching;
        if dispatch_now {
            state.dispatching = true;
        }
        dispatch_now
    }

    fn dispatch_pending_events(&self) -> LogEventDelivery {
        #[cfg(test)]
        self.run_before_event_dispatch_hook();

        let mut first_delivery = None;
        loop {
            let dispatch = {
                let mut state = self.lock_event_dispatch();
                if let Some(dispatch) = state.pending.pop_front() {
                    state.pending_bytes -= dispatch.estimated_bytes;
                    Some(PendingLogDispatch::Record(dispatch))
                } else if let Some(resync) = state.resync.take() {
                    Some(PendingLogDispatch::Resync(resync))
                } else {
                    state.dispatching = false;
                    None
                }
            };
            let Some(dispatch) = dispatch else {
                return first_delivery.unwrap_or(LogEventDelivery::NotConfigured);
            };
            let delivery = match dispatch {
                PendingLogDispatch::Record(dispatch) => {
                    let delivery = dispatch.sink.publish(&dispatch.record);
                    if log_delivery_requires_resync(delivery) {
                        self.absorb_log_delivery_failure(dispatch.sink, dispatch.record.sequence());
                    }
                    delivery
                }
                PendingLogDispatch::Resync(resync) => {
                    let LogEventResync {
                        sink,
                        through_sequence,
                    } = resync;
                    let delivery = sink.resync_required(through_sequence);
                    if log_delivery_requires_resync(delivery) {
                        self.retry_log_resync(LogEventResync {
                            sink,
                            through_sequence,
                        });
                        return first_delivery.unwrap_or(delivery);
                    }
                    self.note_log_resync_delivered();
                    delivery
                }
            };
            if first_delivery.is_none() {
                first_delivery = Some(delivery);
            }
        }
    }

    fn absorb_log_delivery_failure(&self, sink: Arc<dyn EditorLogEventSink>, sequence: u64) {
        let mut state = self.lock_event_dispatch();
        let mut through_sequence = sequence;
        let mut newly_dropped_records = 1_u64;
        while let Some(dispatch) = state.pending.pop_front() {
            state.pending_bytes = state.pending_bytes.saturating_sub(dispatch.estimated_bytes);
            through_sequence = through_sequence.max(dispatch.record.sequence());
            newly_dropped_records = newly_dropped_records.saturating_add(1);
        }
        if let Some(resync) = state.resync.take() {
            through_sequence = through_sequence.max(resync.through_sequence);
        }
        state.resync = Some(LogEventResync {
            sink,
            through_sequence,
        });
        state.resync_required_records = state
            .resync_required_records
            .saturating_add(newly_dropped_records);
    }

    fn retry_log_resync(&self, resync: LogEventResync) {
        let mut state = self.lock_event_dispatch();
        let mut through_sequence = resync.through_sequence;
        let mut newly_dropped_records = 0_u64;
        while let Some(dispatch) = state.pending.pop_front() {
            state.pending_bytes = state.pending_bytes.saturating_sub(dispatch.estimated_bytes);
            through_sequence = through_sequence.max(dispatch.record.sequence());
            newly_dropped_records = newly_dropped_records.saturating_add(1);
        }
        if let Some(existing) = state.resync.take() {
            through_sequence = through_sequence.max(existing.through_sequence);
        }
        state.resync = Some(LogEventResync {
            sink: resync.sink,
            through_sequence,
        });
        state.resync_required_records = state
            .resync_required_records
            .saturating_add(newly_dropped_records);
        state.failed_event_resyncs = state.failed_event_resyncs.saturating_add(1);
        state.dispatching = false;
    }

    fn note_log_resync_delivered(&self) {
        let mut state = self.lock_event_dispatch();
        state.event_resyncs = state.event_resyncs.saturating_add(1);
    }

    fn lock_event_dispatch(&self) -> MutexGuard<'_, LogEventDispatchState> {
        self.event_dispatch
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_emission(&self) -> MutexGuard<'_, ()> {
        self.emission
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_rolling_file_configuration_error(&self) -> MutexGuard<'_, Option<Arc<str>>> {
        self.rolling_file_configuration_error
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[cfg(test)]
    pub(super) fn configure_emission_test_hooks(
        &self,
        before_emission: Arc<dyn Fn() + Send + Sync>,
        after_store: Arc<dyn Fn(&LogRecord) + Send + Sync>,
        before_event_dispatch: Arc<dyn Fn() + Send + Sync>,
    ) {
        *self
            .before_emission_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(before_emission);
        *self
            .after_store_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(after_store);
        *self
            .before_event_dispatch_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(before_event_dispatch);
    }

    #[cfg(test)]
    fn run_before_emission_hook(&self) {
        if let Some(hook) = self
            .before_emission_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            hook();
        }
    }

    #[cfg(test)]
    fn run_after_store_hook(&self, record: &LogRecord) {
        if let Some(hook) = self
            .after_store_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            hook(record);
        }
    }

    #[cfg(test)]
    fn run_before_event_dispatch_hook(&self) {
        if let Some(hook) = self
            .before_event_dispatch_hook
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
        {
            hook();
        }
    }
}

fn log_delivery_requires_resync(delivery: LogEventDelivery) -> bool {
    matches!(
        delivery,
        LogEventDelivery::Backpressured | LogEventDelivery::Rejected
    )
}
