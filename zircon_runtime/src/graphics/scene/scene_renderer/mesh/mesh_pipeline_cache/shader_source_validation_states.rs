use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ShaderSourceValidationStatus<V, E> {
    Missing,
    Pending,
    Ready(V),
    Failed(E),
}

enum ShaderSourceValidationState<V, E> {
    Pending,
    Ready(V),
    Failed(E),
}

pub(super) struct ShaderSourceValidationStates<K, V, E> {
    states: HashMap<K, ShaderSourceValidationState<V, E>>,
    pending_count: usize,
    ready_count: usize,
    failed_count: usize,
}

impl<K, V, E> Default for ShaderSourceValidationStates<K, V, E> {
    fn default() -> Self {
        Self {
            states: HashMap::new(),
            pending_count: 0,
            ready_count: 0,
            failed_count: 0,
        }
    }
}

impl<K, V, E> ShaderSourceValidationStates<K, V, E>
where
    K: Eq + Hash,
    V: Clone,
    E: Clone,
{
    pub(super) fn status(&self, key: &K) -> ShaderSourceValidationStatus<V, E> {
        match self.states.get(key) {
            None => ShaderSourceValidationStatus::Missing,
            Some(ShaderSourceValidationState::Pending) => ShaderSourceValidationStatus::Pending,
            Some(ShaderSourceValidationState::Ready(value)) => {
                ShaderSourceValidationStatus::Ready(value.clone())
            }
            Some(ShaderSourceValidationState::Failed(error)) => {
                ShaderSourceValidationStatus::Failed(error.clone())
            }
        }
    }

    pub(super) fn mark_pending(&mut self, key: K) -> bool {
        if self.states.contains_key(&key) {
            return false;
        }
        self.states
            .insert(key, ShaderSourceValidationState::Pending);
        self.pending_count += 1;
        true
    }

    pub(super) fn publish_ready(&mut self, key: &K, value: V) -> bool {
        let Some(state) = self.states.get_mut(key) else {
            return false;
        };
        if !matches!(state, ShaderSourceValidationState::Pending) {
            return false;
        }
        *state = ShaderSourceValidationState::Ready(value);
        self.pending_count -= 1;
        self.ready_count += 1;
        true
    }

    pub(super) fn publish_failed(&mut self, key: &K, error: E) -> bool {
        let Some(state) = self.states.get_mut(key) else {
            return false;
        };
        if !matches!(state, ShaderSourceValidationState::Pending) {
            return false;
        }
        *state = ShaderSourceValidationState::Failed(error);
        self.pending_count -= 1;
        self.failed_count += 1;
        true
    }

    pub(super) fn take_ready(&mut self, key: &K) -> Option<V> {
        if !matches!(
            self.states.get(key),
            Some(ShaderSourceValidationState::Ready(_))
        ) {
            return None;
        }
        let Some(ShaderSourceValidationState::Ready(value)) = self.states.remove(key) else {
            unreachable!("ready validation state was checked before removal")
        };
        self.ready_count -= 1;
        Some(value)
    }

    pub(super) fn pending_count(&self) -> usize {
        self.pending_count
    }

    pub(super) fn ready_count(&self) -> usize {
        self.ready_count
    }

    pub(super) fn failed_count(&self) -> usize {
        self.failed_count
    }

    pub(super) fn len(&self) -> usize {
        self.states.len()
    }
}
