use std::sync::Arc;

use super::super::task_handle::TaskRecord;

/// A cooperative cancellation observation passed into runtime worker closures.
#[derive(Clone)]
pub struct TaskCancellationToken {
    pub(super) record: Arc<TaskRecord>,
}

impl TaskCancellationToken {
    pub fn is_cancellation_requested(&self) -> bool {
        self.record.lock_state().cancellation_requested
    }

    /// Confirms that running work observed a cancellation request and will
    /// return without continuing its user-visible operation.
    pub fn acknowledge_cancellation(&self) -> bool {
        let mut state = self.record.lock_state();
        if !state.cancellation_requested {
            return false;
        }
        state.cancellation_acknowledged = true;
        true
    }
}
