use std::fs::File;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, TryLockError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossbeam_channel::{
    bounded, Receiver, RecvTimeoutError, SendTimeoutError, Sender, TrySendError,
};

use super::super::settings::{DiagnosticLogSinkSettings, DEFAULT_DIAGNOSTIC_LOG_BATCH_BYTES};
use super::super::timestamp::{
    append_current_log_timestamp_prefix, write_current_log_timestamp_prefix,
};
use super::metrics::{DiagnosticLogSinkSnapshot, SinkMetrics};
use super::{append_diagnostic_log_line, write_diagnostic_log_line};
use crate::core::runtime::tasks::thread_is_join_ready;
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
    text: String,
    scope_len: usize,
    enqueued_at: Instant,
}

impl LogRecord {
    fn new(level: DiagnosticLogLevel, scope: &str, message: &str) -> Self {
        let mut text = String::with_capacity(scope.len().saturating_add(message.len()));
        text.push_str(scope);
        text.push_str(message);
        Self {
            level,
            text,
            scope_len: scope.len(),
            enqueued_at: Instant::now(),
        }
    }

    fn scope(&self) -> &str {
        &self.text[..self.scope_len]
    }

    fn message(&self) -> &str {
        &self.text[self.scope_len..]
    }
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

        if level < DiagnosticLogLevel::Warn && self.sender.is_full() {
            self.metrics.observe_queue_depth(self.queue_capacity);
            self.metrics.record_drop(level);
            return false;
        }

        let message = message();
        let command = SinkCommand::Record(LogRecord::new(level, scope, message.as_ref()));
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
            return self.wait_for_worker_close(deadline) && self.join_worker_until(deadline);
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
        self.join_worker_until(deadline)
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

    fn join_worker_until(&self, deadline: Instant) -> bool {
        loop {
            let mut slot = match self.worker.try_lock() {
                Ok(slot) => slot,
                Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
                Err(TryLockError::WouldBlock) => {
                    if Instant::now() >= deadline {
                        return false;
                    }
                    thread::yield_now();
                    continue;
                }
            };
            let Some(worker) = slot.as_ref() else {
                return self.metrics.is_closed();
            };
            // A shutdown acknowledgement precedes output destruction and thread exit.
            if thread_is_join_ready(worker) {
                return slot
                    .take()
                    .expect("finished worker retains join authority")
                    .join()
                    .is_ok();
            }
            drop(slot);
            if Instant::now() >= deadline {
                return false;
            }
            thread::yield_now();
        }
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
    command: SinkCommand,
    deadline: Instant,
) -> bool {
    sender.send_timeout(command, remaining(deadline)).is_ok()
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
    let mut encoding_buffer = Vec::new();
    let encoding_retention_limit = settings
        .max_batch_bytes
        .min(DEFAULT_DIAGNOSTIC_LOG_BATCH_BYTES);

    loop {
        if flush_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            flush_pending(
                &mut outputs,
                &mut pending,
                &metrics,
                &mut encoding_buffer,
                encoding_retention_limit,
            );
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
                    flush_pending(
                        &mut outputs,
                        &mut pending,
                        &metrics,
                        &mut encoding_buffer,
                        encoding_retention_limit,
                    );
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
                    flush_pending(
                        &mut outputs,
                        &mut pending,
                        &metrics,
                        &mut encoding_buffer,
                        encoding_retention_limit,
                    );
                    pending_bytes = 0;
                    flush_deadline = None;
                }
            }
            Ok(SinkCommand::Flush(acknowledged)) => {
                flush_pending(
                    &mut outputs,
                    &mut pending,
                    &metrics,
                    &mut encoding_buffer,
                    encoding_retention_limit,
                );
                let result = sync_outputs(&mut outputs, &metrics);
                pending_bytes = 0;
                flush_deadline = None;
                let _ = acknowledged.send(result);
            }
            Ok(SinkCommand::Shutdown(acknowledged)) => {
                flush_pending(
                    &mut outputs,
                    &mut pending,
                    &metrics,
                    &mut encoding_buffer,
                    encoding_retention_limit,
                );
                let result = sync_outputs(&mut outputs, &metrics);
                metrics.mark_closed();
                let _ = acknowledged.send(result);
                return;
            }
            Err(RecvTimeoutError::Timeout) => {
                flush_pending(
                    &mut outputs,
                    &mut pending,
                    &metrics,
                    &mut encoding_buffer,
                    encoding_retention_limit,
                );
                pending_bytes = 0;
                flush_deadline = None;
            }
            Err(RecvTimeoutError::Disconnected) => {
                flush_pending(
                    &mut outputs,
                    &mut pending,
                    &metrics,
                    &mut encoding_buffer,
                    encoding_retention_limit,
                );
                let _ = sync_outputs(&mut outputs, &metrics);
                metrics.mark_closed();
                return;
            }
        }
    }
}

fn flush_pending(
    outputs: &mut SinkOutputs,
    pending: &mut Vec<LogRecord>,
    metrics: &SinkMetrics,
    buffer: &mut Vec<u8>,
    retained_byte_limit: usize,
) {
    flush_pending_with_timestamp(
        outputs,
        pending,
        metrics,
        buffer,
        retained_byte_limit,
        || {
            let mut prefix = TimestampPrefix::default();
            write_current_log_timestamp_prefix(&mut prefix)
                .expect("writing a diagnostic log timestamp to fixed storage");
            prefix
        },
    );
}

fn flush_pending_with_timestamp<F>(
    outputs: &mut SinkOutputs,
    pending: &mut Vec<LogRecord>,
    metrics: &SinkMetrics,
    buffer: &mut Vec<u8>,
    retained_byte_limit: usize,
    prepare_timestamp: F,
) where
    F: FnOnce() -> TimestampPrefix,
{
    if pending.is_empty() {
        return;
    }

    let record_count = pending.len();
    buffer.clear();
    let estimated_bytes = pending.iter().fold(0usize, |total, record| {
        total.saturating_add(estimated_record_bytes(record))
    });
    let prepared_timestamp = if estimated_bytes > retained_byte_limit {
        Some(prepare_timestamp())
    } else {
        None
    };
    if let Some(prefix) = prepared_timestamp.as_ref() {
        let encoded_bytes = encoded_pending_bytes(pending, prefix.as_bytes()).unwrap_or(usize::MAX);
        if encoded_bytes > retained_byte_limit || encoded_bytes == usize::MAX {
            let failed = if outputs.console_enabled {
                let stderr = io::stderr().lock();
                flush_oversized_pending(
                    Some(stderr),
                    outputs.file.as_deref_mut(),
                    pending,
                    (prefix.as_bytes(), encoded_bytes),
                    retained_byte_limit,
                    metrics,
                    buffer,
                )
            } else {
                flush_oversized_pending(
                    None::<io::Sink>,
                    outputs.file.as_deref_mut(),
                    pending,
                    (prefix.as_bytes(), encoded_bytes),
                    retained_byte_limit,
                    metrics,
                    buffer,
                )
            };
            outputs.failed |= failed;
            return;
        }
    }

    buffer.reserve_exact(estimated_bytes.min(retained_byte_limit));
    let timestamp_prefix_start = buffer.len();
    if let Some(prefix) = prepared_timestamp.as_ref() {
        buffer.extend_from_slice(prefix.as_bytes());
    } else {
        append_current_log_timestamp_prefix(buffer);
    }
    let timestamp_prefix_len = buffer.len() - timestamp_prefix_start;
    for (index, record) in pending.drain(..).enumerate() {
        if index != 0 {
            buffer.extend_from_within(
                timestamp_prefix_start..timestamp_prefix_start + timestamp_prefix_len,
            );
        }
        append_diagnostic_log_line(buffer, record.level, record.scope(), record.message());
    }

    let mut configured_outputs = 0usize;
    let mut successful_outputs = 0usize;
    if outputs.console_enabled {
        configured_outputs += 1;
        let mut stderr = io::stderr().lock();
        if stderr.write_all(buffer.as_slice()).is_err() || stderr.flush().is_err() {
            metrics.record_output_error();
            outputs.failed = true;
        } else {
            successful_outputs += 1;
        }
    }
    if let Some(file) = outputs.file.as_mut() {
        configured_outputs += 1;
        if file.write_all(buffer.as_slice()).is_err() || file.flush().is_err() {
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
    buffer.clear();
    if buffer.capacity() > retained_byte_limit {
        // An oversized accepted record must not pin its temporary encoding allocation.
        *buffer = Vec::new();
    }
}

// The signed i32 year takes at most 11 bytes; the other prefix fields take 18.
#[derive(Default)]
struct TimestampPrefix {
    bytes: [u8; 32],
    len: usize,
}

impl TimestampPrefix {
    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl std::fmt::Write for TimestampPrefix {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        let end = self
            .len
            .checked_add(value.len())
            .filter(|&end| end <= self.bytes.len())
            .ok_or(std::fmt::Error)?;
        self.bytes[self.len..end].copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[derive(Default)]
struct EncodedByteCounter(usize);

impl Write for EncodedByteCounter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .ok_or_else(|| io::Error::other("diagnostic log encoded size overflow"))?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn encoded_pending_bytes(pending: &[LogRecord], prefix: &[u8]) -> io::Result<usize> {
    let mut counter = EncodedByteCounter::default();
    for record in pending {
        counter.write_all(prefix)?;
        write_diagnostic_log_line(&mut counter, record.level, record.scope(), record.message())?;
    }
    Ok(counter.0)
}

#[derive(Clone, Copy)]
enum OutputChunkMode {
    Utf8,
    RawBytes,
}

struct BoundedOutput<'a, W: Write + ?Sized> {
    output: &'a mut W,
    byte_limit: usize,
    mode: OutputChunkMode,
    written_bytes: usize,
}

impl<W: Write + ?Sized> Write for BoundedOutput<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut end = bytes.len().min(self.byte_limit);
        if matches!(self.mode, OutputChunkMode::Utf8) {
            // Formatter pieces are valid UTF-8. Windows console writes must keep scalars whole.
            while end != 0 && end < bytes.len() && bytes[end] & 0xc0 == 0x80 {
                end -= 1;
            }
            if end == 0 && !bytes.is_empty() {
                // A scalar wider than the unchanged limit is borrowed whole, at most four bytes.
                end = 1;
                while end < bytes.len() && bytes[end] & 0xc0 == 0x80 {
                    end += 1;
                }
            }
        }
        let written = self.output.write(&bytes[..end])?;
        self.written_bytes = self
            .written_bytes
            .checked_add(written)
            .ok_or_else(|| io::Error::other("diagnostic log encoded size overflow"))?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

fn write_pending_bounded<W: Write + ?Sized>(
    output: &mut W,
    pending: &[LogRecord],
    prefix: &[u8],
    byte_limit: usize,
    mode: OutputChunkMode,
) -> io::Result<usize> {
    // Runtime settings normalize the existing byte limit to at least one.
    debug_assert!(byte_limit != 0);
    let mut output = BoundedOutput {
        output,
        byte_limit,
        mode,
        written_bytes: 0,
    };
    for record in pending {
        output.write_all(prefix)?;
        write_diagnostic_log_line(&mut output, record.level, record.scope(), record.message())?;
    }
    output.flush()?;
    Ok(output.written_bytes)
}

fn flush_oversized_pending<C: Write, F: Write + ?Sized>(
    console: Option<C>,
    file: Option<&mut F>,
    pending: &mut Vec<LogRecord>,
    encoded: (&[u8], usize),
    byte_limit: usize,
    metrics: &SinkMetrics,
    buffer: &mut Vec<u8>,
) -> bool {
    let (prefix, encoded_bytes) = encoded;
    let mut configured_outputs = 0usize;
    let mut successful_outputs = 0usize;
    if let Some(mut console) = console {
        configured_outputs += 1;
        if write_pending_bounded(
            &mut console,
            pending,
            prefix,
            byte_limit,
            OutputChunkMode::Utf8,
        )
        .is_err()
        {
            metrics.record_output_error();
        } else {
            successful_outputs += 1;
        }
    }
    if let Some(file) = file {
        configured_outputs += 1;
        if write_pending_bounded(file, pending, prefix, byte_limit, OutputChunkMode::RawBytes)
            .is_err()
        {
            metrics.record_output_error();
        } else {
            successful_outputs += 1;
        }
    }
    metrics.record_batch(
        pending.len(),
        encoded_bytes,
        configured_outputs != 0 && configured_outputs == successful_outputs,
    );
    pending.clear();
    *buffer = Vec::new();
    configured_outputs != successful_outputs
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
        .scope_len
        .saturating_add(
            record
                .text
                .len()
                .saturating_sub(record.scope_len)
                .saturating_mul(2),
        )
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

#[cfg(test)]
#[path = "worker/batch_buffer_tests.rs"]
mod batch_buffer_tests;

#[cfg(test)]
#[path = "worker/oversized_encoding_tests.rs"]
mod oversized_encoding_tests;
