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

#[cfg(test)]
mod tests {
    use super::TextShapingWorkBudget;

    #[test]
    fn shaping_work_budget_is_a_non_zero_execution_threshold() {
        assert_eq!(TextShapingWorkBudget::new(0), None);

        let budget = TextShapingWorkBudget::default();
        let boundary = budget.max_inline_input_bytes();

        assert!(!budget.exceeds_inline_threshold(boundary));
        assert!(budget.exceeds_inline_threshold(boundary + 1));
    }
}
