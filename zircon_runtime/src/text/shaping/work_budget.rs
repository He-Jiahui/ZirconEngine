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
mod tests {
    use super::{TextShapingWorkBudget, TextShapingWorkReport};

    #[test]
    fn shaping_work_budget_is_a_non_zero_execution_threshold() {
        assert_eq!(TextShapingWorkBudget::new(0), None);

        let budget = TextShapingWorkBudget::default();
        let boundary = budget.max_inline_input_bytes();

        assert!(!budget.exceeds_inline_threshold(boundary));
        assert!(budget.exceeds_inline_threshold(boundary + 1));
    }

    #[test]
    fn shaping_work_report_classifies_complete_synchronous_requests_without_slicing() {
        let budget = TextShapingWorkBudget::new(8).expect("non-zero budget");
        let mut report = TextShapingWorkReport::default();

        report.record_synchronous_request(budget, 5);
        report.record_synchronous_request(budget, 13);

        assert_eq!(report.inline_request_count, 1);
        assert_eq!(report.oversized_synchronous_request_count, 1);
        assert_eq!(report.synchronous_input_bytes, 18);
        assert_eq!(report.max_synchronous_input_bytes, 13);
    }

    #[test]
    fn shaping_work_report_merges_parallel_batch_receipts() {
        let budget = TextShapingWorkBudget::new(4).expect("non-zero budget");
        let mut first = TextShapingWorkReport::default();
        first.record_synchronous_request(budget, 3);
        let mut second = TextShapingWorkReport::default();
        second.record_synchronous_request(budget, 9);

        first.merge(second);

        assert_eq!(first.inline_request_count, 1);
        assert_eq!(first.oversized_synchronous_request_count, 1);
        assert_eq!(first.synchronous_input_bytes, 12);
        assert_eq!(first.max_synchronous_input_bytes, 9);
    }
}
