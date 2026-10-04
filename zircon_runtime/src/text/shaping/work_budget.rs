const DEFAULT_MAX_INLINE_INPUT_BYTES: usize = 64 * 1024;

/// Scheduling threshold for one synchronous shaping request.
///
/// This budget classifies work; it is never a source-line, script-run, or cluster boundary.
/// Classification never authorizes slicing a request. Until a typed deferred outcome exists,
/// the synchronous fallback must retain the complete semantic context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextShapingWorkBudget {
    max_inline_input_bytes: usize,
}

/// Aggregate receipt for complete requests that actually reached a shaping backend.
///
/// Oversized requests remain synchronous until a typed deferred work-unit owner exists. The
/// counters therefore describe observed work without authorizing a semantic split or defer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextShapingWorkReport {
    pub(crate) inline_request_count: usize,
    pub(crate) oversized_synchronous_request_count: usize,
    pub(crate) synchronous_input_bytes: usize,
    pub(crate) max_synchronous_input_bytes: usize,
}

impl TextShapingWorkBudget {
    pub(crate) const fn new(max_inline_input_bytes: usize) -> Option<Self> {
        if max_inline_input_bytes == 0 {
            return None;
        }
        Some(Self {
            max_inline_input_bytes,
        })
    }

    pub(crate) const fn max_inline_input_bytes(self) -> usize {
        self.max_inline_input_bytes
    }

    pub(crate) const fn exceeds_inline_threshold(self, input_bytes: usize) -> bool {
        input_bytes > self.max_inline_input_bytes
    }
}

impl Default for TextShapingWorkBudget {
    fn default() -> Self {
        Self {
            max_inline_input_bytes: DEFAULT_MAX_INLINE_INPUT_BYTES,
        }
    }
}

impl TextShapingWorkReport {
    pub(crate) fn record_synchronous_request(
        &mut self,
        budget: TextShapingWorkBudget,
        input_bytes: usize,
    ) {
        if budget.exceeds_inline_threshold(input_bytes) {
            self.oversized_synchronous_request_count =
                self.oversized_synchronous_request_count.saturating_add(1);
        } else {
            self.inline_request_count = self.inline_request_count.saturating_add(1);
        }
        self.synchronous_input_bytes = self.synchronous_input_bytes.saturating_add(input_bytes);
        self.max_synchronous_input_bytes = self.max_synchronous_input_bytes.max(input_bytes);
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.inline_request_count = self
            .inline_request_count
            .saturating_add(other.inline_request_count);
        self.oversized_synchronous_request_count = self
            .oversized_synchronous_request_count
            .saturating_add(other.oversized_synchronous_request_count);
        self.synchronous_input_bytes = self
            .synchronous_input_bytes
            .saturating_add(other.synchronous_input_bytes);
        self.max_synchronous_input_bytes = self
            .max_synchronous_input_bytes
            .max(other.max_synchronous_input_bytes);
    }
}

#[cfg(test)]
#[path = "tests/work_budget.rs"]
mod tests;
