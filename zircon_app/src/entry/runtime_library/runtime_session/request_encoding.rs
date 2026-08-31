use std::io::{self, Write};
use std::time::Instant;

use zircon_runtime_interface::ZrRuntimePayloadLimitV1;

use super::RuntimeLibraryError;

const REQUEST_WRITER_INITIAL_CAPACITY_BYTES: usize = 4 * 1024;
const DEADLINE_CHECK_INTERVAL_BYTES: usize = 1024;

pub(super) fn encode_runtime_request<T: serde::Serialize + ?Sized>(
    value: &T,
    limit: ZrRuntimePayloadLimitV1,
    item_count: usize,
    operation: &'static str,
) -> Result<Vec<u8>, RuntimeLibraryError> {
    if item_count > limit.max_items {
        return Err(RuntimeLibraryError::new(format!(
            "{operation}: JSON item count {item_count} exceeds maximum {}",
            limit.max_items
        )));
    }
    let mut writer = RuntimeRequestWriter::new(limit);
    let result = serde_json::to_writer(&mut writer, value);
    writer
        .finish(result)
        .map_err(|error| RuntimeLibraryError::new(format!("{operation}: {error}")))
}

struct RuntimeRequestWriter {
    bytes: Vec<u8>,
    limit: ZrRuntimePayloadLimitV1,
    started: Instant,
    next_deadline_check_at: usize,
    depth: usize,
    in_string: bool,
    escaped: bool,
    failure: Option<String>,
    #[cfg(test)]
    deadline_checks: usize,
    #[cfg(test)]
    capacity_growths: usize,
}

impl RuntimeRequestWriter {
    fn new(limit: ZrRuntimePayloadLimitV1) -> Self {
        let initial_capacity = limit
            .max_encoded_bytes
            .min(REQUEST_WRITER_INITIAL_CAPACITY_BYTES);
        Self {
            bytes: Vec::with_capacity(initial_capacity),
            limit,
            started: Instant::now(),
            next_deadline_check_at: 0,
            depth: 0,
            in_string: false,
            escaped: false,
            failure: None,
            #[cfg(test)]
            deadline_checks: 0,
            #[cfg(test)]
            capacity_growths: 0,
        }
    }

    fn finish(mut self, result: Result<(), serde_json::Error>) -> Result<Vec<u8>, String> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| error.to_string())?;
        self.check_deadline()?;
        if self.bytes.is_empty() && !self.limit.allow_empty {
            return Err("empty JSON payload is not allowed".to_string());
        }
        Ok(self.bytes)
    }

    fn check_deadline(&mut self) -> Result<(), String> {
        #[cfg(test)]
        {
            self.deadline_checks = self.deadline_checks.saturating_add(1);
        }
        if self.started.elapsed().as_micros() > u128::from(self.limit.max_processing_time_micros) {
            return Err(format!(
                "JSON processing exceeded {} microseconds",
                self.limit.max_processing_time_micros
            ));
        }
        Ok(())
    }

    fn check_deadline_if_due(&mut self) -> Result<(), String> {
        if self.bytes.len() < self.next_deadline_check_at {
            return Ok(());
        }
        self.check_deadline()?;
        self.next_deadline_check_at = self
            .bytes
            .len()
            .saturating_add(DEADLINE_CHECK_INTERVAL_BYTES);
        Ok(())
    }

    fn inspect_nesting(&mut self, bytes: &[u8]) -> Result<(), String> {
        for byte in bytes {
            if self.in_string {
                if self.escaped {
                    self.escaped = false;
                } else if *byte == b'\\' {
                    self.escaped = true;
                } else if *byte == b'"' {
                    self.in_string = false;
                }
                continue;
            }
            match *byte {
                b'"' => self.in_string = true,
                b'{' | b'[' => {
                    self.depth = self.depth.saturating_add(1);
                    if self.depth > self.limit.max_nesting_depth {
                        return Err(format!(
                            "JSON nesting depth {} exceeds maximum {}",
                            self.depth, self.limit.max_nesting_depth
                        ));
                    }
                }
                b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                _ => {}
            }
        }
        Ok(())
    }
}

impl Write for RuntimeRequestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failure.is_some() {
            return Err(io::Error::other(
                "bounded runtime request writer already failed",
            ));
        }
        if let Err(error) = self.check_deadline_if_due() {
            self.failure = Some(error);
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "runtime request processing deadline exceeded",
            ));
        }
        let Some(encoded_len) = self.bytes.len().checked_add(bytes.len()) else {
            self.failure = Some("JSON encoded byte count overflowed usize".to_string());
            return Err(io::Error::other("runtime request byte count overflowed"));
        };
        if encoded_len > self.limit.max_encoded_bytes {
            self.failure = Some(format!(
                "JSON encoded length {encoded_len} exceeds maximum {}",
                self.limit.max_encoded_bytes
            ));
            return Err(io::Error::other("runtime request byte limit exceeded"));
        }
        if let Err(error) = self.inspect_nesting(bytes) {
            self.failure = Some(error);
            return Err(io::Error::other("runtime request nesting depth exceeded"));
        }
        #[cfg(test)]
        let prior_capacity = self.bytes.capacity();
        self.bytes.extend_from_slice(bytes);
        #[cfg(test)]
        if self.bytes.capacity() > prior_capacity {
            self.capacity_growths = self.capacity_growths.saturating_add(1);
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests;
