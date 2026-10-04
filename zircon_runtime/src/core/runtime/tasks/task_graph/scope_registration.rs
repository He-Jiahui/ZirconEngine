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

// 最后一个 registration 所有者释放时注销 graph 中的弱 scope 记录；graph 已销毁则无需重建其状态。
impl Drop for TaskGraphScopeRegistration {
    fn drop(&mut self) {
        if let Some(graph) = self.graph.upgrade() {
            graph.unregister_scope(self.scope_id);
        }
    }
}
