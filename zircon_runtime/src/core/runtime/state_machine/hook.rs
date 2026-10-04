use std::sync::Arc;

use super::{StateSpec, StateTransitionEvent};

pub(crate) type StateHook<T> = Arc<dyn Fn(&StateTransitionEvent<T>) + Send + Sync + 'static>;

/// 状态写入时复制有序钩子快照；调用方释放 StateRegistry 锁后再逐个执行。
/// Deferred hook invocation bundle for one state transition.
pub(crate) struct StateTransitionDispatch<T: StateSpec> {
    event: StateTransitionEvent<T>,
    ordered_hooks: Vec<StateHook<T>>,
}

impl<T: StateSpec> StateTransitionDispatch<T> {
    pub(crate) fn new(event: StateTransitionEvent<T>, ordered_hooks: Vec<StateHook<T>>) -> Self {
        Self {
            event,
            ordered_hooks,
        }
    }

    pub(crate) fn event(&self) -> &StateTransitionEvent<T> {
        &self.event
    }

    pub(crate) fn run(self) {
        for hook in self.ordered_hooks {
            hook(&self.event);
        }
    }
}
