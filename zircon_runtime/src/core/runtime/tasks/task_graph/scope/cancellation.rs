use std::sync::Arc;

use crate::core::runtime::tasks::TaskNode;

/// A cooperative cancellation observation passed into runtime worker closures.
#[derive(Clone)]
pub struct TaskCancellationToken {
    pub(in crate::core::runtime::tasks::task_graph) node: Arc<TaskNode>,
}

impl TaskCancellationToken {
    pub fn is_cancellation_requested(&self) -> bool {
        self.node.lock_inner().cancellation_requested
    }

    /// Confirms that running work observed a cancellation request and will
    /// return without continuing its user-visible operation.
    pub fn acknowledge_cancellation(&self) -> bool {
        let mut state = self.node.lock_inner();
        if !state.cancellation_requested {
            return false;
        }
        state.cancellation_acknowledged = true;
        true
    }

    #[cfg(test)]
    pub(crate) fn task_node_identity(&self) -> usize {
        TaskNode::identity(&self.node)
    }
}
