const DEFAULT_MAX_RETAINED_BYTES: usize = 1024 * 1024;

/// Combined admission limit for retained plugin deliveries and fault-receipt payloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorRuntimeEventConsumerRetentionBudget {
    max_retained_bytes: usize,
}

impl EditorRuntimeEventConsumerRetentionBudget {
    pub const fn new(max_retained_bytes: usize) -> Self {
        Self { max_retained_bytes }
    }

    pub const fn max_retained_bytes(self) -> usize {
        self.max_retained_bytes
    }
}

impl Default for EditorRuntimeEventConsumerRetentionBudget {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_RETAINED_BYTES)
    }
}
