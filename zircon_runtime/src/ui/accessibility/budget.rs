use std::io::{self, Write};
use std::time::{Duration, Instant};

use serde::Serialize;
use zircon_runtime_interface::ZrRuntimePayloadLimitV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AccessibilitySnapshotBudgetError {
    EncodedBytes { observed: usize, limit: usize },
    Items { observed: usize, limit: usize },
    NestingDepth { observed: usize, limit: usize },
    ProcessingTime { limit_micros: u64 },
    Json(String),
}

pub(crate) struct AccessibilityBuildBudget {
    limit: Option<ZrRuntimePayloadLimitV1>,
    observed_items: usize,
    observed_encoded_bytes: usize,
    started: Instant,
}

impl AccessibilityBuildBudget {
    pub(crate) fn new(limit: ZrRuntimePayloadLimitV1) -> Self {
        Self {
            limit: Some(limit),
            observed_items: 0,
            observed_encoded_bytes: 0,
            started: Instant::now(),
        }
    }

    pub(super) fn unbounded() -> Self {
        Self {
            limit: None,
            observed_items: 0,
            observed_encoded_bytes: 0,
            started: Instant::now(),
        }
    }

    pub(super) fn observe_items(
        &mut self,
        count: usize,
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        let Some(limit) = self.limit else {
            return Ok(());
        };
        self.observed_items = self.observed_items.saturating_add(count);
        if self.observed_items > limit.max_items {
            return Err(AccessibilitySnapshotBudgetError::Items {
                observed: self.observed_items,
                limit: limit.max_items,
            });
        }
        self.check_deadline()
    }

    pub(crate) fn observe_value(
        &mut self,
        value: &(impl Serialize + ?Sized),
        nesting_offset: usize,
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        let encoded_bytes =
            self.measure_serialized(value, nesting_offset, self.observed_encoded_bytes)?;
        self.observed_encoded_bytes = self.observed_encoded_bytes.saturating_add(encoded_bytes);
        Ok(())
    }

    pub(crate) fn preflight_value(
        &self,
        value: &(impl Serialize + ?Sized),
        nesting_offset: usize,
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        self.measure_serialized(value, nesting_offset, self.observed_encoded_bytes)
            .map(|_| ())
    }

    pub(crate) fn observe_replacement<T: Serialize + ?Sized>(
        &mut self,
        previous: &T,
        replacement: &T,
        nesting_offset: usize,
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        let previous_bytes = self.measure_serialized(previous, nesting_offset, 0)?;
        let retained_without_previous = self.observed_encoded_bytes.saturating_sub(previous_bytes);
        let replacement_bytes =
            self.measure_serialized(replacement, nesting_offset, retained_without_previous)?;
        self.observed_encoded_bytes = retained_without_previous.saturating_add(replacement_bytes);
        Ok(())
    }

    pub(crate) fn validate_payload(
        &self,
        value: &(impl Serialize + ?Sized),
    ) -> Result<(), AccessibilitySnapshotBudgetError> {
        self.measure_serialized(value, 0, 0).map(|_| ())
    }

    pub(super) fn check_deadline(&self) -> Result<(), AccessibilitySnapshotBudgetError> {
        let Some(limit) = self.limit else {
            return Ok(());
        };
        if self.started.elapsed() > Duration::from_micros(limit.max_processing_time_micros) {
            return Err(AccessibilitySnapshotBudgetError::ProcessingTime {
                limit_micros: limit.max_processing_time_micros,
            });
        }
        Ok(())
    }

    fn measure_serialized(
        &self,
        value: &(impl Serialize + ?Sized),
        nesting_offset: usize,
        base_count: usize,
    ) -> Result<usize, AccessibilitySnapshotBudgetError> {
        let Some(limit) = self.limit else {
            return Ok(0);
        };
        self.check_deadline()?;
        if nesting_offset > limit.max_nesting_depth {
            return Err(AccessibilitySnapshotBudgetError::NestingDepth {
                observed: nesting_offset,
                limit: limit.max_nesting_depth,
            });
        }
        let mut writer =
            AccessibilityCountingWriter::new(limit, self.started, nesting_offset, base_count);
        let result = serde_json::to_writer(&mut writer, value);
        writer.finish(result)
    }
}

struct AccessibilityCountingWriter {
    limit: ZrRuntimePayloadLimitV1,
    started: Instant,
    base_count: usize,
    count: usize,
    depth: usize,
    in_string: bool,
    escaped: bool,
    failure: Option<AccessibilitySnapshotBudgetError>,
}

impl AccessibilityCountingWriter {
    fn new(
        limit: ZrRuntimePayloadLimitV1,
        started: Instant,
        nesting_offset: usize,
        base_count: usize,
    ) -> Self {
        Self {
            limit,
            started,
            base_count,
            count: 0,
            depth: nesting_offset,
            in_string: false,
            escaped: false,
            failure: None,
        }
    }

    fn finish(
        mut self,
        result: Result<(), serde_json::Error>,
    ) -> Result<usize, AccessibilitySnapshotBudgetError> {
        if let Some(error) = self.failure.take() {
            return Err(error);
        }
        result.map_err(|error| AccessibilitySnapshotBudgetError::Json(error.to_string()))?;
        if self.started.elapsed() > Duration::from_micros(self.limit.max_processing_time_micros) {
            return Err(AccessibilitySnapshotBudgetError::ProcessingTime {
                limit_micros: self.limit.max_processing_time_micros,
            });
        }
        Ok(self.count)
    }
}

impl Write for AccessibilityCountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.started.elapsed() > Duration::from_micros(self.limit.max_processing_time_micros) {
            self.failure = Some(AccessibilitySnapshotBudgetError::ProcessingTime {
                limit_micros: self.limit.max_processing_time_micros,
            });
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "accessibility snapshot build deadline exceeded",
            ));
        }
        self.count = self.count.saturating_add(bytes.len());
        let observed = self.base_count.saturating_add(self.count);
        if observed > self.limit.max_encoded_bytes {
            self.failure = Some(AccessibilitySnapshotBudgetError::EncodedBytes {
                observed,
                limit: self.limit.max_encoded_bytes,
            });
            return Err(io::Error::other(
                "accessibility snapshot byte budget exceeded",
            ));
        }
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
                        self.failure = Some(AccessibilitySnapshotBudgetError::NestingDepth {
                            observed: self.depth,
                            limit: self.limit.max_nesting_depth,
                        });
                        return Err(io::Error::other(
                            "accessibility snapshot nesting budget exceeded",
                        ));
                    }
                }
                b'}' | b']' => self.depth = self.depth.saturating_sub(1),
                _ => {}
            }
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limit(max_encoded_bytes: usize, max_nesting_depth: usize) -> ZrRuntimePayloadLimitV1 {
        ZrRuntimePayloadLimitV1 {
            max_encoded_bytes,
            max_items: 16,
            max_nesting_depth,
            max_processing_time_micros: 100_000,
            allow_empty: false,
        }
    }

    #[test]
    fn build_budget_accumulates_serialized_bytes_before_retention() {
        let mut budget = AccessibilityBuildBudget::new(limit(8, 8));
        budget.observe_value("1234", 2).unwrap();

        let error = budget
            .observe_value("a", 2)
            .expect_err("the second value must exceed the cumulative byte budget");

        assert_eq!(
            error,
            AccessibilitySnapshotBudgetError::EncodedBytes {
                observed: 9,
                limit: 8
            }
        );
    }

    #[test]
    fn build_budget_accounts_for_the_snapshot_nesting_offset() {
        let mut budget = AccessibilityBuildBudget::new(limit(1024, 2));

        let error = budget
            .observe_value(&serde_json::json!({"value": 1}), 2)
            .expect_err("the value object must exceed the remaining nesting budget");

        assert_eq!(
            error,
            AccessibilitySnapshotBudgetError::NestingDepth {
                observed: 3,
                limit: 2
            }
        );
    }
}
