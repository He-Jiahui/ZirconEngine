use std::sync::Weak;

use super::engine_task_graph::EngineTaskGraphInner;

/// Retires a scope's weak graph entry when its final live owner is released.
pub(super) struct TaskGraphScopeRegistration {
    graph: Weak<EngineTaskGraphInner>,
    scope_id: u64,
}

impl TaskGraphScopeRegistration {
    pub(super) const fn new(graph: Weak<EngineTaskGraphInner>, scope_id: u64) -> Self {
        Self { graph, scope_id }
    }
}

impl Drop for TaskGraphScopeRegistration {
    fn drop(&mut self) {
        if let Some(graph) = self.graph.upgrade() {
            graph.unregister_scope(self.scope_id);
        }
    }
}
