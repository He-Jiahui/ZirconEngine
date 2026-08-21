use std::fs::File;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{
    Receiver, RecvTimeoutError, SendTimeoutError, Sender, TrySendError, bounded,
};

use super::super::settings::DiagnosticLogSinkSettings;
use super::super::timestamp::current_log_timestamp;
use super::diagnostic_log_line;
use super::metrics::{DiagnosticLogSinkSnapshot, SinkMetrics};
use crate::diagnostic_log::DiagnosticLogLevel;

pub(super) const SINK_THREAD_NAME: &str = "zircon-diagnostic-log";
const MAX_INITIAL_BATCH_CAPACITY: usize = 1_024;

pub(super) struct SinkRuntime {
    sender: Sender<SinkCommand>,
    metrics: Arc<SinkMetrics>,
    closed: AtomicBool,
    active_senders: AtomicUsize,
    queue_capacity: usize,
    critical_enqueue_timeout: Duration,
    worker: Mutex<Option<JoinHandle<()>>>,
}

struct ActiveSenderGuard<'a> {
    active_senders: &'a AtomicUsize,
}

impl<'a> ActiveSenderGuard<'a> {
    fn enter(active_senders: &'a AtomicUsize) -> Self {
        active_senders.fetch_add(1, Ordering::AcqRel);
        Self { active_senders }
    }
}

impl Drop for ActiveSenderGuard<'_> {
    fn drop(&mut self) {
        self.active_senders.fetch_sub(1, Ordering::AcqRel);
    }
}

struct LogRecord {
    level: DiagnosticLogLevel,
    scope: String,
    message: String,
    enqueued_at: Instant,
}

enum SinkCommand {
    Record(LogRecord),
    Flush(Sender<io::Result<()>>),
    Shutdown(Sender<io::Result<()>>),
}

struct SinkOutputs {
    file: Option<Box<dyn DurableOutput>>,
    console_enabled: bool,
    failed: bool,
}

pub(super) trait DurableOutput: Write + Send {
    fn sync_data(&mut self) -> io::Result<()>;
}

impl DurableOutput for File {
    fn sync_data(&mut self) -> io::Result<()> {
        File::sync_data(self)
    }
}

impl SinkRuntime {
    pub(super) fn start(
        file: Option<Box<dyn DurableOutput>>,
        console_enabled: bool,
        settings: DiagnosticLogSinkSettings,
    ) -> io::Result<Self> {
        let settings = settings.normalized();
        let queue_capacity = settings.queue_capacity;
        let critical_enqueue_timeout = settings.critical_enqueue_timeout;
        let (sender, receiver) = bounded(queue_capacity);
        let metrics = Arc::new(SinkMetrics::new());
        let worker_metrics = Arc::clone(&metrics);
        let worker = thread::Builder::new()
            .name(SINK_THREAD_NAME.to_string())
            .spawn(move || {
                run_sink_worker(
                    receiver,
                    SinkOutputs {
                        file,
                        console_enabled,
                        failed: false,
                    },
                    settings,
                    worker_metrics,
                );
            })?;
        Ok(Self {
            sender,
            metrics,
            closed: AtomicBool::new(false),
            active_senders: AtomicUsize::new(0),
            queue_capacity,
            critical_enqueue_timeout,
            worker: Mutex::new(Some(worker)),
        })
    }

    pub(super) fn enqueue(&self, level: DiagnosticLogLevel, scope: &str, message: &str) -> bool {
        self.enqueue_lazy(level, scope, || message)
    }

    pub(super) fn enqueue_lazy<F, M>(
        &self,
        level: DiagnosticLogLevel,
        scope: &str,
        message: F,
    ) -> bool
    where
        F: FnOnce() -> M,
        M: AsRef<str>,
    {
        if self.closed.load(Ordering::Acquire) {
            self.metrics.record_drop(level);
            return false;
        }

        let _active_sender = ActiveSenderGuard::enter(&self.active_senders);
        if self.closed.load(Ordering::Acquire) {
            self.metrics.record_drop(level);
            return false;
        }

        let message = message();
        let command = SinkCommand::Record(LogRecord {
            level,
            scope: scope.to_owned(),
            message: message.as_ref().to_owned(),
            enqueued_at: Instant::now(),
        });
        let accepted = match self.sender.try_send(command) {
            Ok(()) => {
                self.metrics.observe_queue_depth(self.sender.len());
                true
            }
            Err(TrySendError::Full(command)) if level >= DiagnosticLogLevel::Warn => {
                self.metrics.observe_queue_depth(self.queue_capacity);
                self.metrics.record_critical_backpressure();
                let accepted = match self
                    .sender
                    .send_timeout(command, self.critical_enqueue_timeout)
                {
                    Ok(()) => true,
                    Err(SendTimeoutError::Timeout(_) | SendTimeoutError::Disconnected(_)) => false,
                };
                if accepted {
                    self.metrics.observe_queue_depth(self.sender.len());
                }
                accepted
            }
            Err(TrySendError::Full(_)) => {
                self.metrics.observe_queue_depth(self.queue_capacity);
                false
            }
            Err(TrySendError::Disconnected(_)) => false,
        };

        if !accepted {
            self.metrics.record_drop(level);
        }
        accepted
    }

    pub(super) fn flush(&self, timeout: Duration) -> bool {
        if self.closed.load(Ordering::Acquire) {
            return false;
        }
        let deadline = deadline_after(timeout);
        let (acknowledged, receiver) = bounded(1);
        if !send_control_until(&self.sender, SinkCommand::Flush(acknowledged), deadline) {
            return false;
        }
        receiver
            .recv_timeout(remaining(deadline))
            .is_ok_and(|result| result.is_ok())
    }

    pub(super) fn shutdown(&self, timeout: Duration) -> bool {
        self.shutdown_for_library_unload(timeout) && self.outputs_succeeded()
    }

    /// Stops and joins the worker without conflating an output error with worker liveness.
    ///
    /// Dynamic-library owners must not unload while the worker can still execute library code.
    pub(super) fn shutdown_for_library_unload(&self, timeout: Duration) -> bool {
        let deadline = deadline_after(timeout);
        if self.closed.swap(true, Ordering::AcqRel) {
            return self.wait_for_worker_close(deadline) && self.join_worker();
        }

        while self.active_senders.load(Ordering::Acquire) != 0 {
            if Instant::now() >= deadline {
                self.closed.store(false, Ordering::Release);
                return false;
            }
            thread::yield_now();
        }

        let (acknowledged, receiver) = bounded(1);
        if !send_control_until(&self.sender, SinkCommand::Shutdown(acknowledged), deadline) {
            self.closed.store(false, Ordering::Release);
            return false;
        }
        let Ok(_output_result) = receiver.recv_timeout(remaining(deadline)) else {
            return false;
        };
        self.join_worker()
    }

    pub(super) fn outputs_succeeded(&self) -> bool {
        self.metrics.outputs_succeeded()
    }

    fn wait_for_worker_close(&self, deadline: Instant) -> bool {
        while !self.metrics.is_closed() {
            if Instant::now() >= deadline {
                return false;
            }
            thread::yield_now();
        }
        true
    }

    fn join_worker(&self) -> bool {
        let Some(worker) = self.worker.lock().ok().and_then(|mut worker| worker.take()) else {
            return self.metrics.is_closed();
        };
        worker.join().is_ok()
    }

    pub(super) fn snapshot(&self) -> DiagnosticLogSinkSnapshot {
        self.metrics.snapshot(self.sender.len())
    }

    pub(super) fn is_open(&self) -> bool {
        !self.closed.load(Ordering::Acquire)
    }
}

fn send_control_until(
    sender: &Sender<SinkCommand>,
    mut command: SinkCommand,
    deadline: Instant,
) -> bool {
    loop {
        match sender.try_send(command) {
            Ok(()) => return true,
            Err(TrySendError::Full(returned)) => command = returned,
            Err(TrySendError::Disconnected(_)) => return false,
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::yield_now();
    }
}

fn run_sink_worker(
    receiver: Receiver<SinkCommand>,
    mut outputs: SinkOutputs,
    settings: DiagnosticLogSinkSettings,
    metrics: Arc<SinkMetrics>,
) {
    let mut pending = Vec::with_capacity(
        settings
            .max_batch_records
            .min(settings.queue_capacity)
            .min(MAX_INITIAL_BATCH_CAPACITY),
    );
    let mut pending_bytes = 0usize;
    let mut flush_deadline = None;

    loop {
        if flush_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            flush_pending(&mut outputs, &mut pending, &metrics);
            pending_bytes = 0;
            flush_deadline = None;
            continue;
        }
        let received = match flush_deadline {
            Some(deadline) => receiver.recv_timeout(remaining(deadline)),
            None => receiver.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match received {
            Ok(SinkCommand::Record(record)) => {
                metrics.observe_queue_depth(
                    receiver
                        .len()
                        .saturating_add(1)
                        .min(settings.queue_capacity),
                );
                metrics.record_dequeued(record.enqueued_at);
                let record_bytes = estimated_record_bytes(&record);
                if !pending.is_empty()
                    && pending_bytes.saturating_add(record_bytes) > settings.max_batch_bytes
                {
                    flush_pending(&mut outputs, &mut pending, &metrics);
                    pending_bytes = 0;
                    flush_deadline = None;
                }
                if pending.is_empty() {
                    flush_deadline = Some(deadline_after(settings.flush_interval));
                }
                pending_bytes = pending_bytes.saturating_add(record_bytes);
                pending.push(record);
                if pending.len() >= settings.max_batch_records
                    || pending_bytes >= settings.max_batch_bytes
                    || settings.flush_interval.is_zero()
                    || flush_deadline.is_some_and(|deadline| Instant::now() >= deadline)
                {
                    flush_pending(&mut outputs, &mut pending, &metrics);
                    pending_bytes = 0;
                    flush_deadline = None;
                }
            }
            Ok(SinkCommand::Flush(acknowledged)) => {
                flush_pending(&mut outputs, &mut pending, &metrics);
                let result = sync_outputs(&mut outputs, &metrics);
                pending_bytes = 0;
                flush_deadline = None;
                let _ = acknowledged.send(result);
            }
            Ok(SinkCommand::Shutdown(acknowledged)) => {
                flush_pending(&mut outputs, &mut pending, &metrics);
                let result = sync_outputs(&mut outputs, &metrics);
                metrics.mark_closed();
                let _ = acknowledged.send(result);
                return;
            }
            Err(RecvTimeoutError::Timeout) => {
                flush_pending(&mut outputs, &mut pending, &metrics);
                pending_bytes = 0;
                flush_deadline = None;
            }
            Err(RecvTimeoutError::Disconnected) => {
                flush_pending(&mut outputs, &mut pending, &metrics);
                let _ = sync_outputs(&mut outputs, &metrics);
                metrics.mark_closed();
                return;
            }
        }
    }
}

fn flush_pending(outputs: &mut SinkOutputs, pending: &mut Vec<LogRecord>, metrics: &SinkMetrics) {
    if pending.is_empty() {
        return;
    }

    let record_count = pending.len();
    let mut buffer = Vec::with_capacity(pending.iter().map(estimated_record_bytes).sum::<usize>());
    for record in pending.drain(..) {
        let line = diagnostic_log_line(
            &current_log_timestamp(),
            record.level,
            &record.scope,
            &record.message,
        );
        buffer.extend_from_slice(line.as_bytes());
    }

    let mut configured_outputs = 0usize;
    let mut successful_outputs = 0usize;
    if outputs.console_enabled {
        configured_outputs += 1;
        let mut stderr = io::stderr().lock();
        if stderr.write_all(&buffer).is_err() || stderr.flush().is_err() {
            metrics.record_output_error();
            outputs.failed = true;
        } else {
            successful_outputs += 1;
        }
    }
    if let Some(file) = outputs.file.as_mut() {
        configured_outputs += 1;
        if file.write_all(&buffer).is_err() || file.flush().is_err() {
            metrics.record_output_error();
            outputs.failed = true;
        } else {
            successful_outputs += 1;
        }
    }
    metrics.record_batch(
        record_count,
        buffer.len(),
        configured_outputs != 0 && configured_outputs == successful_outputs,
    );
}

fn sync_outputs(outputs: &mut SinkOutputs, metrics: &SinkMetrics) -> io::Result<()> {
    let mut sync_failed = outputs.failed;
    if outputs.console_enabled && io::stderr().flush().is_err() {
        metrics.record_output_error();
        sync_failed = true;
    }
    if let Some(file) = outputs.file.as_mut() {
        if file.flush().is_err() || file.sync_data().is_err() {
            metrics.record_output_error();
            sync_failed = true;
        }
    }
    if outputs.file.is_none() && !outputs.console_enabled {
        sync_failed = true;
    }
    outputs.failed = sync_failed;
    if sync_failed {
        Err(io::Error::other("diagnostic log output was not durable"))
    } else {
        Ok(())
    }
}

fn estimated_record_bytes(record: &LogRecord) -> usize {
    record
        .scope
        .len()
        .saturating_add(record.message.len().saturating_mul(2))
        .saturating_add(64)
}

fn deadline_after(duration: Duration) -> Instant {
    Instant::now()
        .checked_add(duration)
        .unwrap_or_else(Instant::now)
}

fn remaining(deadline: Instant) -> Duration {
    deadline.saturating_duration_since(Instant::now())
}
