use std::time::Duration;

const DEFAULT_MAX_TASKS: usize = 1_024;
const DEFAULT_MAX_IN_FLIGHT_PREPARES: usize = 32;
const DEFAULT_MAX_RETAINED_BYTES: usize = 4 * 1024 * 1024;
const DEFAULT_MAX_OWNER_APPLIES_PER_TICK: usize = 8;
const DEFAULT_TERMINAL_RESULT_TTL: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug)]
pub(in crate::operation) struct RuntimeOperationLimits {
    pub(in crate::operation) max_tasks: usize,
    pub(in crate::operation) max_in_flight_prepares: usize,
    pub(in crate::operation) max_retained_bytes: usize,
    pub(in crate::operation) max_owner_applies_per_tick: usize,
    pub(in crate::operation) terminal_result_ttl: Duration,
}

impl Default for RuntimeOperationLimits {
    fn default() -> Self {
        Self {
            max_tasks: DEFAULT_MAX_TASKS,
            max_in_flight_prepares: DEFAULT_MAX_IN_FLIGHT_PREPARES,
            max_retained_bytes: DEFAULT_MAX_RETAINED_BYTES,
            max_owner_applies_per_tick: DEFAULT_MAX_OWNER_APPLIES_PER_TICK,
            terminal_result_ttl: DEFAULT_TERMINAL_RESULT_TTL,
        }
    }
}
