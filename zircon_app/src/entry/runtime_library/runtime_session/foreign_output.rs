use std::array;
use std::io::{self, BufReader, Read};
use std::slice;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use zircon_runtime_interface::{ZrOwnedByteBuffer, ZrStatus};

use super::{ensure_status, release_owned_buffer, validate_owned_buffer, RuntimeLibraryError};

const FOREIGN_OUTPUT_KIND_COUNT: usize = 5;
const DECODE_READER_CHUNK_BYTES: usize = 4 * 1024;
const FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH: usize = 128;

pub(super) const HOST_REQUEST_OUTPUT_BUDGET: ForeignOutputBudget =
    ForeignOutputBudget::new(256 * 1024, 256, Duration::from_millis(10)).allow_empty();
pub(super) const PROFILE_RESPONSE_OUTPUT_BUDGET: ForeignOutputBudget =
    ForeignOutputBudget::new(16 * 1024 * 1024, 65_536, Duration::from_millis(250)).allow_empty();
pub(super) const OPERATION_RESULT_OUTPUT_BUDGET: ForeignOutputBudget =
    ForeignOutputBudget::new(1024 * 1024, 16_384, Duration::from_millis(25));
pub(super) const PLUGIN_EVENT_OUTPUT_BUDGET: ForeignOutputBudget = ForeignOutputBudget::new(
    zircon_runtime_interface::ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
    zircon_runtime_interface::ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    Duration::from_millis(10),
)
.allow_empty();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ForeignOutputKind {
    SessionProtocol,
    HostRequests,
    ProfileResponse,
    OperationResult,
    PluginEvents,
}

impl ForeignOutputKind {
    const ALL: [Self; FOREIGN_OUTPUT_KIND_COUNT] = [
        Self::SessionProtocol,
        Self::HostRequests,
        Self::ProfileResponse,
        Self::OperationResult,
        Self::PluginEvents,
    ];

    const fn index(self) -> usize {
        match self {
            Self::SessionProtocol => 0,
            Self::HostRequests => 1,
            Self::ProfileResponse => 2,
            Self::OperationResult => 3,
            Self::PluginEvents => 4,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::SessionProtocol => "session_protocol",
            Self::HostRequests => "host_requests",
            Self::ProfileResponse => "profile_response",
            Self::OperationResult => "operation_result",
            Self::PluginEvents => "plugin_events",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ForeignOutputBudget {
    max_encoded_bytes: usize,
    max_items: usize,
    max_decode_time: Duration,
    allow_empty: bool,
}

impl ForeignOutputBudget {
    pub(super) const fn new(
        max_encoded_bytes: usize,
        max_items: usize,
        max_decode_time: Duration,
    ) -> Self {
        Self {
            max_encoded_bytes,
            max_items,
            max_decode_time,
            allow_empty: false,
        }
    }

    const fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    pub(super) fn validate_decode_duration(
        self,
        elapsed: Duration,
        operation: &'static str,
    ) -> Result<(), RuntimeLibraryError> {
        if elapsed <= self.max_decode_time {
            return Ok(());
        }
        Err(RuntimeLibraryError::protocol_violation(format!(
            "{operation} exceeded its decode time budget: observed {} microseconds; maximum is {} microseconds",
            elapsed.as_micros(),
            self.max_decode_time.as_micros()
        )))
    }

    fn validate_encoded_len(
        self,
        encoded_len: usize,
        operation: &'static str,
    ) -> Result<(), RuntimeLibraryError> {
        if encoded_len <= self.max_encoded_bytes {
            return Ok(());
        }
        Err(RuntimeLibraryError::protocol_violation(format!(
            "{operation} returned {encoded_len} encoded bytes; maximum is {}",
            self.max_encoded_bytes
        )))
    }

    fn validate_item_count(
        self,
        item_count: usize,
        operation: &'static str,
    ) -> Result<(), RuntimeLibraryError> {
        if item_count <= self.max_items {
            return Ok(());
        }
        Err(RuntimeLibraryError::protocol_violation(format!(
            "{operation} returned {item_count} items; maximum is {}",
            self.max_items
        )))
    }
}

#[derive(Default)]
struct ForeignOutputCounters {
    accepted_payloads: AtomicU64,
    accepted_bytes: AtomicU64,
    rejected_payloads: AtomicU64,
    rejected_bytes: AtomicU64,
    call_failures: AtomicU64,
    blocked_calls: AtomicU64,
    total_decode_nanoseconds: AtomicU64,
    max_decode_nanoseconds: AtomicU64,
}

impl ForeignOutputCounters {
    fn snapshot(&self) -> ForeignOutputMetrics {
        ForeignOutputMetrics {
            accepted_payloads: self.accepted_payloads.load(Ordering::Relaxed),
            accepted_bytes: self.accepted_bytes.load(Ordering::Relaxed),
            rejected_payloads: self.rejected_payloads.load(Ordering::Relaxed),
            rejected_bytes: self.rejected_bytes.load(Ordering::Relaxed),
            call_failures: self.call_failures.load(Ordering::Relaxed),
            blocked_calls: self.blocked_calls.load(Ordering::Relaxed),
            total_decode_nanoseconds: self.total_decode_nanoseconds.load(Ordering::Relaxed),
            max_decode_nanoseconds: self.max_decode_nanoseconds.load(Ordering::Relaxed),
        }
    }

    fn record_accepted(&self, encoded_len: usize, decode_time: Duration) {
        self.accepted_payloads.fetch_add(1, Ordering::Relaxed);
        self.accepted_bytes
            .fetch_add(usize_to_u64(encoded_len), Ordering::Relaxed);
        self.record_decode_time(decode_time);
    }

    fn record_rejected(&self, encoded_len: usize, decode_time: Duration) {
        self.rejected_payloads.fetch_add(1, Ordering::Relaxed);
        self.rejected_bytes
            .fetch_add(usize_to_u64(encoded_len), Ordering::Relaxed);
        self.record_decode_time(decode_time);
    }

    fn record_decode_time(&self, decode_time: Duration) {
        if decode_time.is_zero() {
            return;
        }
        let decode_nanoseconds = duration_to_u64_nanoseconds(decode_time).max(1);
        self.total_decode_nanoseconds
            .fetch_add(decode_nanoseconds, Ordering::Relaxed);
        self.max_decode_nanoseconds
            .fetch_max(decode_nanoseconds, Ordering::Relaxed);
    }
}

pub(super) struct ForeignOutputState {
    acceptance_gate: Mutex<()>,
    protocol_failed: AtomicBool,
    protocol_failures: AtomicU64,
    blocked_session_calls: AtomicU64,
    counters: [ForeignOutputCounters; FOREIGN_OUTPUT_KIND_COUNT],
}

impl Default for ForeignOutputState {
    fn default() -> Self {
        Self {
            acceptance_gate: Mutex::new(()),
            protocol_failed: AtomicBool::new(false),
            protocol_failures: AtomicU64::new(0),
            blocked_session_calls: AtomicU64::new(0),
            counters: array::from_fn(|_| ForeignOutputCounters::default()),
        }
    }
}

impl ForeignOutputState {
    pub(super) fn ensure_available(
        &self,
        kind: ForeignOutputKind,
    ) -> Result<(), RuntimeLibraryError> {
        if !self.is_protocol_failed() {
            return Ok(());
        }
        self.counters[kind.index()]
            .blocked_calls
            .fetch_add(1, Ordering::Relaxed);
        Err(fused_session_error(kind))
    }

    pub(super) fn is_protocol_failed(&self) -> bool {
        self.protocol_failed.load(Ordering::Acquire)
    }

    pub(super) fn ensure_session_available(
        &self,
        operation: &'static str,
    ) -> Result<(), RuntimeLibraryError> {
        if !self.is_protocol_failed() {
            return Ok(());
        }
        self.blocked_session_calls.fetch_add(1, Ordering::Relaxed);
        Err(fused_session_call_error(operation))
    }

    pub(super) fn reject_protocol<T>(
        &self,
        kind: ForeignOutputKind,
        error: RuntimeLibraryError,
    ) -> Result<T, RuntimeLibraryError> {
        let error = RuntimeLibraryError::protocol_violation(error.to_string());
        self.reject(kind, 0, Duration::ZERO, error)
    }

    pub(super) fn ensure_call_succeeded(
        &self,
        status: ZrStatus,
        output: ZrOwnedByteBuffer,
        kind: ForeignOutputKind,
        operation: &'static str,
        release_operation: &'static str,
    ) -> Result<(), RuntimeLibraryError> {
        let Err(call_error) = ensure_status(status, operation) else {
            return Ok(());
        };
        self.counters[kind.index()]
            .call_failures
            .fetch_add(1, Ordering::Relaxed);
        let encoded_len = output.len;
        let ownership_error = validate_owned_buffer(&output, release_operation).err();
        let release_error = release_owned_buffer(output, release_operation).err();
        match (ownership_error, release_error) {
            (None, None) => Err(call_error),
            (ownership_error, release_error) => {
                let mut error = RuntimeLibraryError::protocol_violation(match ownership_error {
                    Some(ownership_error) => format!(
                        "{call_error}; foreign call returned invalid output ownership: {ownership_error}"
                    ),
                    None => call_error.to_string(),
                });
                if let Some(release_error) = release_error {
                    error = error.with_cleanup_failure(&release_error);
                }
                self.reject(kind, encoded_len, Duration::ZERO, error)
            }
        }
    }

    pub(super) fn decode_json<T: DeserializeOwned>(
        &self,
        output: ZrOwnedByteBuffer,
        kind: ForeignOutputKind,
        budget: ForeignOutputBudget,
        operation: &'static str,
        release_operation: &'static str,
        validate: impl FnOnce(&T) -> Result<usize, RuntimeLibraryError>,
    ) -> Result<Option<T>, RuntimeLibraryError> {
        let encoded_len = output.len;
        if self.is_protocol_failed() {
            return self.reject_and_release(
                output,
                kind,
                encoded_len,
                Duration::ZERO,
                fused_session_error(kind),
                release_operation,
            );
        }
        if let Err(error) = validate_owned_buffer(&output, operation) {
            return self.reject_and_release(
                output,
                kind,
                encoded_len,
                Duration::ZERO,
                RuntimeLibraryError::protocol_violation(error.to_string()),
                release_operation,
            );
        }
        if let Err(error) = budget.validate_encoded_len(encoded_len, operation) {
            return self.reject_and_release(
                output,
                kind,
                encoded_len,
                Duration::ZERO,
                error,
                release_operation,
            );
        }
        if encoded_len == 0 {
            if !budget.allow_empty {
                let error = RuntimeLibraryError::protocol_violation(format!(
                    "{operation} returned an empty payload"
                ));
                return self.reject_and_release(
                    output,
                    kind,
                    0,
                    Duration::ZERO,
                    error,
                    release_operation,
                );
            }
            return self.finish_acceptance(
                output,
                kind,
                0,
                Duration::ZERO,
                operation,
                release_operation,
                None,
            );
        }

        let bytes = unsafe { slice::from_raw_parts(output.data.cast_const(), encoded_len) };
        let decode_started = Instant::now();
        let mut timed_out = false;
        let decoded = {
            let reader = DeadlineReader::new(
                bytes,
                decode_started + budget.max_decode_time,
                &mut timed_out,
            );
            serde_json::from_reader::<_, T>(BufReader::with_capacity(
                DECODE_READER_CHUNK_BYTES,
                reader,
            ))
        };
        let decoded = if timed_out {
            Err(RuntimeLibraryError::protocol_violation(format!(
                "{operation} exceeded its decode time budget while parsing: maximum is {} microseconds",
                budget.max_decode_time.as_micros()
            )))
        } else {
            decoded.map_err(|error| {
                RuntimeLibraryError::protocol_violation(format!(
                    "{operation} failed bounded JSON decode (maximum nesting depth {FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH}): {error}"
                ))
            })
        };
        let decoded = decoded.and_then(|decoded| {
            let item_count = validate(&decoded)
                .map_err(|error| RuntimeLibraryError::protocol_violation(error.to_string()))?;
            budget.validate_item_count(item_count, operation)?;
            Ok(decoded)
        });
        let decode_time = decode_started.elapsed();
        let decoded = decoded.and_then(|decoded| {
            budget.validate_decode_duration(decode_time, operation)?;
            Ok(decoded)
        });

        match decoded {
            Ok(decoded) => self.finish_acceptance(
                output,
                kind,
                encoded_len,
                decode_time,
                operation,
                release_operation,
                Some(decoded),
            ),
            Err(error) => self.reject_and_release(
                output,
                kind,
                encoded_len,
                decode_time,
                error,
                release_operation,
            ),
        }
    }

    pub(super) fn metrics(&self) -> ForeignOutputMetricsSnapshot {
        ForeignOutputMetricsSnapshot {
            protocol_failed: self.is_protocol_failed(),
            protocol_failures: self.protocol_failures.load(Ordering::Relaxed),
            blocked_session_calls: self.blocked_session_calls.load(Ordering::Relaxed),
            by_kind: array::from_fn(|index| self.counters[index].snapshot()),
        }
    }

    pub(super) fn diagnostic_line(&self) -> Option<String> {
        let metrics = self.metrics();
        if !metrics.has_activity() {
            return None;
        }
        let mut fields = vec![
            format!("protocol_failed={}", metrics.protocol_failed),
            format!("protocol_failures={}", metrics.protocol_failures),
            format!("blocked_session_calls={}", metrics.blocked_session_calls),
        ];
        for kind in ForeignOutputKind::ALL {
            let counters = metrics.for_kind(kind);
            fields.push(format!(
                "{}.accepted_payloads={} {}.accepted_bytes={} {}.rejected_payloads={} {}.rejected_bytes={} {}.call_failures={} {}.blocked_calls={} {}.total_decode_ns={} {}.max_decode_ns={}",
                kind.label(),
                counters.accepted_payloads,
                kind.label(),
                counters.accepted_bytes,
                kind.label(),
                counters.rejected_payloads,
                kind.label(),
                counters.rejected_bytes,
                kind.label(),
                counters.call_failures,
                kind.label(),
                counters.blocked_calls,
                kind.label(),
                counters.total_decode_nanoseconds,
                kind.label(),
                counters.max_decode_nanoseconds,
            ));
        }
        Some(fields.join(" "))
    }

    fn reject_and_release<T>(
        &self,
        output: ZrOwnedByteBuffer,
        kind: ForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        error: RuntimeLibraryError,
        release_operation: &'static str,
    ) -> Result<T, RuntimeLibraryError> {
        let error = match release_owned_buffer(output, release_operation) {
            Ok(()) => error,
            Err(release_error) => error.with_cleanup_failure(&release_error),
        };
        self.reject(kind, encoded_len, decode_time, error)
    }

    fn finish_acceptance<T>(
        &self,
        output: ZrOwnedByteBuffer,
        kind: ForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        operation: &'static str,
        release_operation: &'static str,
        value: Option<T>,
    ) -> Result<Option<T>, RuntimeLibraryError> {
        let release_error = release_owned_buffer(output, release_operation).err();
        let _acceptance = self
            .acceptance_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(release_error) = release_error {
            return self.reject_locked(
                kind,
                encoded_len,
                decode_time,
                RuntimeLibraryError::protocol_violation(format!(
                    "{operation} cleanup failed: {release_error}"
                )),
            );
        }
        if self.is_protocol_failed() {
            self.counters[kind.index()].record_rejected(encoded_len, decode_time);
            return Err(fused_session_error(kind));
        }
        self.counters[kind.index()].record_accepted(encoded_len, decode_time);
        Ok(value)
    }

    fn reject<T>(
        &self,
        kind: ForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        error: RuntimeLibraryError,
    ) -> Result<T, RuntimeLibraryError> {
        let _acceptance = self
            .acceptance_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.reject_locked(kind, encoded_len, decode_time, error)
    }

    fn reject_locked<T>(
        &self,
        kind: ForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        error: RuntimeLibraryError,
    ) -> Result<T, RuntimeLibraryError> {
        self.counters[kind.index()].record_rejected(encoded_len, decode_time);
        if !self.protocol_failed.swap(true, Ordering::AcqRel) {
            self.protocol_failures.fetch_add(1, Ordering::Relaxed);
        }
        Err(error)
    }
}

fn fused_session_error(kind: ForeignOutputKind) -> RuntimeLibraryError {
    RuntimeLibraryError::protocol_violation(format!(
        "runtime session rejected {} because a prior foreign-output protocol violation fused the session",
        kind.label()
    ))
}

fn fused_session_call_error(operation: &'static str) -> RuntimeLibraryError {
    RuntimeLibraryError::protocol_violation(format!(
        "runtime session rejected {operation} because a prior foreign-output protocol violation fused the session"
    ))
}

struct DeadlineReader<'a> {
    remaining: &'a [u8],
    deadline: Instant,
    timed_out: &'a mut bool,
}

impl<'a> DeadlineReader<'a> {
    fn new(remaining: &'a [u8], deadline: Instant, timed_out: &'a mut bool) -> Self {
        Self {
            remaining,
            deadline,
            timed_out,
        }
    }
}

impl Read for DeadlineReader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.remaining.is_empty() || output.is_empty() {
            return Ok(0);
        }
        if Instant::now() >= self.deadline {
            *self.timed_out = true;
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "foreign output decode deadline exceeded",
            ));
        }
        let count = output
            .len()
            .min(self.remaining.len())
            .min(DECODE_READER_CHUNK_BYTES);
        output[..count].copy_from_slice(&self.remaining[..count]);
        self.remaining = &self.remaining[count..];
        Ok(count)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ForeignOutputMetrics {
    pub(super) accepted_payloads: u64,
    pub(super) accepted_bytes: u64,
    pub(super) rejected_payloads: u64,
    pub(super) rejected_bytes: u64,
    pub(super) call_failures: u64,
    pub(super) blocked_calls: u64,
    pub(super) total_decode_nanoseconds: u64,
    pub(super) max_decode_nanoseconds: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ForeignOutputMetricsSnapshot {
    protocol_failed: bool,
    protocol_failures: u64,
    blocked_session_calls: u64,
    by_kind: [ForeignOutputMetrics; FOREIGN_OUTPUT_KIND_COUNT],
}

impl ForeignOutputMetricsSnapshot {
    pub(super) const fn for_kind(self, kind: ForeignOutputKind) -> ForeignOutputMetrics {
        self.by_kind[kind.index()]
    }

    fn has_activity(self) -> bool {
        self.protocol_failures > 0
            || self.blocked_session_calls > 0
            || self.by_kind.iter().any(|metrics| {
                metrics.accepted_payloads > 0
                    || metrics.rejected_payloads > 0
                    || metrics.call_failures > 0
                    || metrics.blocked_calls > 0
            })
    }
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn duration_to_u64_nanoseconds(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests;
