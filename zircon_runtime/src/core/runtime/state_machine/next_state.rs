use super::StateSpec;

/// Queued transition for a runtime-wide state machine.
/// 一个类型状态机的待提交请求；`Pending` 允许同值转移，
/// `PendingIfNeq` 在提交时按当前值抑制同值事件。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NextState<T: StateSpec> {
    Unchanged,
    Pending(T),
    PendingIfNeq(T),
}

impl<T: StateSpec> Default for NextState<T> {
    fn default() -> Self {
        Self::Unchanged
    }
}

impl<T: StateSpec> NextState<T> {
    pub fn set(&mut self, state: T) {
        *self = Self::Pending(state);
    }

    pub fn set_if_neq(&mut self, state: T) {
        // 既有显式 Pending 允许同值事件；重复设置同一值不能把这项请求降为 PendingIfNeq。
        if !matches!(self, Self::Pending(existing) if existing == &state) {
            *self = Self::PendingIfNeq(state);
        }
    }

    pub fn reset(&mut self) {
        *self = Self::Unchanged;
    }

    pub(crate) fn take_transition(&mut self) -> Option<(T, bool)> {
        match std::mem::take(self) {
            Self::Unchanged => None,
            Self::Pending(state) => Some((state, true)),
            Self::PendingIfNeq(state) => Some((state, false)),
        }
    }
}
