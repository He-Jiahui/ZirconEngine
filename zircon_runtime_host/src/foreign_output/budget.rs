//! Per-output resource budgets enforced by the host.

use std::time::Duration;

use super::RuntimeForeignOutputError;

pub const RUNTIME_FOREIGN_OUTPUT_JSON_MAX_NESTING_DEPTH: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeForeignOutputBudget {
    pub(super) max_encoded_bytes: usize,
    pub(super) max_items: usize,
    pub(super) max_decode_time: Duration,
    pub(super) allow_empty: bool,
}

impl RuntimeForeignOutputBudget {
    pub const fn new(
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

    pub const fn allow_empty(mut self) -> Self {
        self.allow_empty = true;
        self
    }

    pub const fn max_decode_time(self) -> Duration {
        self.max_decode_time
    }

    pub fn validate_decode_duration(
        self,
        elapsed: Duration,
        operation: &'static str,
    ) -> Result<(), RuntimeForeignOutputError> {
        if elapsed <= self.max_decode_time {
            return Ok(());
        }
        Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} exceeded its decode time budget: observed {} microseconds; maximum is {} microseconds",
            elapsed.as_micros(),
            self.max_decode_time.as_micros()
        )))
    }

    pub(super) fn validate_encoded_len(
        self,
        encoded_len: usize,
        operation: &'static str,
    ) -> Result<(), RuntimeForeignOutputError> {
        if encoded_len <= self.max_encoded_bytes {
            return Ok(());
        }
        Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {encoded_len} encoded bytes; maximum is {}",
            self.max_encoded_bytes
        )))
    }

    pub(super) fn validate_item_count(
        self,
        item_count: usize,
        operation: &'static str,
    ) -> Result<(), RuntimeForeignOutputError> {
        if item_count <= self.max_items {
            return Ok(());
        }
        Err(RuntimeForeignOutputError::protocol_violation(format!(
            "{operation} returned {item_count} items; maximum is {}",
            self.max_items
        )))
    }
}
