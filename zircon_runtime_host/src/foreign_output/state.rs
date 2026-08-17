//! Shared per-session protocol fuse and output accounting.

use std::slice;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use serde::de::DeserializeOwned;

use zircon_runtime_interface::{ZrOwnedByteBuffer, ZrStatus};

use super::decode::decode_bounded_json;
use super::kind::{RuntimeForeignOutputKind, RUNTIME_FOREIGN_OUTPUT_KIND_COUNT};
use super::metrics::{empty_counters, RuntimeForeignOutputCounters};
use super::{
    release_owned_buffer, validate_owned_buffer, RuntimeForeignOutputBudget,
    RuntimeForeignOutputError, RuntimeForeignOutputMetricsSnapshot,
};

pub struct RuntimeForeignOutputState {
    acceptance_gate: Mutex<()>,
    protocol_failed: AtomicBool,
    protocol_failures: AtomicU64,
    blocked_session_calls: AtomicU64,
    counters: [RuntimeForeignOutputCounters; RUNTIME_FOREIGN_OUTPUT_KIND_COUNT],
}

impl Default for RuntimeForeignOutputState {
    fn default() -> Self {
        Self {
            acceptance_gate: Mutex::new(()),
            protocol_failed: AtomicBool::new(false),
            protocol_failures: AtomicU64::new(0),
            blocked_session_calls: AtomicU64::new(0),
            counters: empty_counters(),
        }
    }
}

impl RuntimeForeignOutputState {
    pub fn ensure_available(
        &self,
        kind: RuntimeForeignOutputKind,
    ) -> Result<(), RuntimeForeignOutputError> {
        if !self.is_protocol_failed() {
            return Ok(());
        }
        self.counters[kind.index()].record_blocked_call();
        Err(fused_session_error(kind))
    }

    pub fn is_protocol_failed(&self) -> bool {
        self.protocol_failed.load(Ordering::Acquire)
    }

    pub fn ensure_session_available(
        &self,
        operation: &'static str,
    ) -> Result<(), RuntimeForeignOutputError> {
        if !self.is_protocol_failed() {
            return Ok(());
        }
        self.blocked_session_calls.fetch_add(1, Ordering::Relaxed);
        Err(fused_session_call_error(operation))
    }

    pub fn reject_protocol<T>(
        &self,
        kind: RuntimeForeignOutputKind,
        error: impl std::fmt::Display,
    ) -> Result<T, RuntimeForeignOutputError> {
        self.reject(
            kind,
            0,
            Duration::ZERO,
            RuntimeForeignOutputError::protocol_violation(error.to_string()),
        )
    }

    pub fn ensure_call_succeeded(
        &self,
        status: ZrStatus,
        output: ZrOwnedByteBuffer,
        kind: RuntimeForeignOutputKind,
        operation: &'static str,
        release_operation: &'static str,
    ) -> Result<(), RuntimeForeignOutputError> {
        let Some(call_error) = RuntimeForeignOutputError::from_status(status, operation) else {
            return Ok(());
        };
        self.counters[kind.index()].record_call_failure();
        let encoded_len = output.len;
        let ownership_error = validate_owned_buffer(&output, release_operation).err();
        let release_error = release_owned_buffer(output, release_operation).err();
        match (ownership_error, release_error) {
            (None, None) => Err(call_error),
            (ownership_error, release_error) => {
                let mut error = RuntimeForeignOutputError::protocol_violation(
                    match ownership_error {
                        Some(ownership_error) => format!(
                            "{call_error}; foreign call returned invalid output ownership: {ownership_error}"
                        ),
                        None => call_error.to_string(),
                    },
                );
                if let Some(release_error) = release_error {
                    error = error.with_cleanup_failure(&release_error);
                }
                self.reject(kind, encoded_len, Duration::ZERO, error)
            }
        }
    }

    pub fn decode_json<T, E>(
        &self,
        output: ZrOwnedByteBuffer,
        kind: RuntimeForeignOutputKind,
        budget: RuntimeForeignOutputBudget,
        operation: &'static str,
        release_operation: &'static str,
        validate: impl FnOnce(&T) -> Result<usize, E>,
    ) -> Result<Option<T>, RuntimeForeignOutputError>
    where
        T: DeserializeOwned,
        E: std::fmt::Display,
    {
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
                error,
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
                return self.reject_and_release(
                    output,
                    kind,
                    0,
                    Duration::ZERO,
                    RuntimeForeignOutputError::protocol_violation(format!(
                        "{operation} returned an empty payload"
                    )),
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
        let (decoded, decode_time) = decode_bounded_json(bytes, budget, operation, validate);
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

    pub fn metrics(&self) -> RuntimeForeignOutputMetricsSnapshot {
        RuntimeForeignOutputMetricsSnapshot {
            protocol_failed: self.is_protocol_failed(),
            protocol_failures: self.protocol_failures.load(Ordering::Relaxed),
            blocked_session_calls: self.blocked_session_calls.load(Ordering::Relaxed),
            by_kind: std::array::from_fn(|index| self.counters[index].snapshot()),
        }
    }

    pub fn diagnostic_line(&self) -> Option<String> {
        let metrics = self.metrics();
        if !metrics.has_activity() {
            return None;
        }
        let mut fields = vec![
            format!("protocol_failed={}", metrics.protocol_failed),
            format!("protocol_failures={}", metrics.protocol_failures),
            format!("blocked_session_calls={}", metrics.blocked_session_calls),
        ];
        for kind in RuntimeForeignOutputKind::ALL {
            let counters = metrics.for_kind(kind);
            fields.push(format!(
                "{}.accepted_payloads={} {}.accepted_bytes={} {}.rejected_payloads={} {}.rejected_bytes={} {}.call_failures={} {}.blocked_calls={} {}.total_decode_ns={} {}.max_decode_ns={}",
                kind.label(), counters.accepted_payloads,
                kind.label(), counters.accepted_bytes,
                kind.label(), counters.rejected_payloads,
                kind.label(), counters.rejected_bytes,
                kind.label(), counters.call_failures,
                kind.label(), counters.blocked_calls,
                kind.label(), counters.total_decode_nanoseconds,
                kind.label(), counters.max_decode_nanoseconds,
            ));
        }
        Some(fields.join(" "))
    }

    fn reject_and_release<T>(
        &self,
        output: ZrOwnedByteBuffer,
        kind: RuntimeForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        error: RuntimeForeignOutputError,
        release_operation: &'static str,
    ) -> Result<T, RuntimeForeignOutputError> {
        let error = match release_owned_buffer(output, release_operation) {
            Ok(()) => error,
            Err(release_error) => error.with_cleanup_failure(&release_error),
        };
        self.reject(kind, encoded_len, decode_time, error)
    }

    fn finish_acceptance<T>(
        &self,
        output: ZrOwnedByteBuffer,
        kind: RuntimeForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        operation: &'static str,
        release_operation: &'static str,
        value: Option<T>,
    ) -> Result<Option<T>, RuntimeForeignOutputError> {
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
                RuntimeForeignOutputError::protocol_violation(format!(
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
        kind: RuntimeForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        error: RuntimeForeignOutputError,
    ) -> Result<T, RuntimeForeignOutputError> {
        let _acceptance = self
            .acceptance_gate
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.reject_locked(kind, encoded_len, decode_time, error)
    }

    fn reject_locked<T>(
        &self,
        kind: RuntimeForeignOutputKind,
        encoded_len: usize,
        decode_time: Duration,
        error: RuntimeForeignOutputError,
    ) -> Result<T, RuntimeForeignOutputError> {
        self.counters[kind.index()].record_rejected(encoded_len, decode_time);
        if !self.protocol_failed.swap(true, Ordering::AcqRel) {
            self.protocol_failures.fetch_add(1, Ordering::Relaxed);
        }
        Err(error)
    }
}

fn fused_session_error(kind: RuntimeForeignOutputKind) -> RuntimeForeignOutputError {
    RuntimeForeignOutputError::protocol_violation(format!(
        "runtime session rejected {} because a prior foreign-output protocol violation fused the session",
        kind.label()
    ))
}

fn fused_session_call_error(operation: &'static str) -> RuntimeForeignOutputError {
    RuntimeForeignOutputError::protocol_violation(format!(
        "runtime session rejected {operation} because a prior foreign-output protocol violation fused the session"
    ))
}
