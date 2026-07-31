use std::collections::BTreeMap;

use crate::input::InputActionMap;

#[derive(Clone, Debug, Default)]
pub(super) struct ActionBindingIndex {
    by_action: BTreeMap<String, Vec<usize>>,
    context_enabled: BTreeMap<String, bool>,
    has_axis_bindings: bool,
}

impl ActionBindingIndex {
    pub(super) fn from_action_map(action_map: &InputActionMap) -> Self {
        let mut by_action = BTreeMap::<String, Vec<usize>>::new();
        for (index, binding) in action_map.bindings.iter().enumerate() {
            by_action
                .entry(binding.action.clone())
                .or_default()
                .push(index);
        }
        let mut context_enabled = BTreeMap::new();
        for context in &action_map.contexts {
            context_enabled
                .entry(context.id.clone())
                .or_insert(context.enabled);
        }
        let has_axis_bindings = action_map
            .bindings
            .iter()
            .any(|binding| !binding.axes.is_empty());
        Self {
            by_action,
            context_enabled,
            has_axis_bindings,
        }
    }

    pub(super) fn indices_for_action(&self, action: &str) -> &[usize] {
        self.by_action.get(action).map(Vec::as_slice).unwrap_or(&[])
    }

    pub(super) fn has_axis_bindings(&self) -> bool {
        self.has_axis_bindings
    }

    pub(super) fn context_enabled(&self, context: &str) -> bool {
        self.context_enabled.get(context).copied().unwrap_or(true)
    }

    #[cfg(test)]
    pub(super) fn candidate_count(&self) -> usize {
        self.by_action.values().map(Vec::len).sum()
    }
}
