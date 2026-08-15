use std::collections::BTreeMap;

use crate::input::InputActionMap;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CompiledAction {
    pub(super) action_index: usize,
    pub(super) context_slot: Option<usize>,
    binding_start: usize,
    binding_end: usize,
}

impl CompiledAction {
    pub(super) fn binding_indices<'a>(
        &self,
        generation: &'a ActionEvaluationGeneration,
    ) -> &'a [usize] {
        &generation.binding_indices[self.binding_start..self.binding_end]
    }
}

/// Immutable map-change-time lookup data for one persisted action map.
#[derive(Clone, Debug, Default)]
pub(super) struct ActionEvaluationGeneration {
    actions: Vec<CompiledAction>,
    binding_indices: Vec<usize>,
    context_slots: BTreeMap<String, usize>,
    context_enabled: Vec<bool>,
    has_axis_bindings: bool,
    #[cfg(test)]
    source_binding_count: usize,
}

impl ActionEvaluationGeneration {
    pub(super) fn from_action_map(action_map: &InputActionMap) -> Self {
        let mut context_slots = BTreeMap::new();
        let mut context_enabled = Vec::new();
        for context in &action_map.contexts {
            insert_context_slot(
                &mut context_slots,
                &mut context_enabled,
                &context.id,
                context.enabled,
            );
        }
        for action in &action_map.actions {
            if let Some(context) = action.context.as_deref() {
                insert_context_slot(&mut context_slots, &mut context_enabled, context, true);
            }
        }

        let mut bindings_by_action = BTreeMap::<String, Vec<usize>>::new();
        for (index, binding) in action_map.bindings.iter().enumerate() {
            bindings_by_action
                .entry(binding.action.clone())
                .or_default()
                .push(index);
        }

        let mut actions = Vec::with_capacity(action_map.actions.len());
        let mut binding_indices = Vec::with_capacity(action_map.bindings.len());
        for (action_index, action) in action_map.actions.iter().enumerate() {
            let binding_start = binding_indices.len();
            if let Some(indices) = bindings_by_action.get(&action.id) {
                binding_indices.extend(indices.iter().copied());
            }
            let binding_end = binding_indices.len();
            actions.push(CompiledAction {
                action_index,
                context_slot: action
                    .context
                    .as_deref()
                    .and_then(|context| context_slots.get(context).copied()),
                binding_start,
                binding_end,
            });
        }

        Self {
            actions,
            binding_indices,
            context_slots,
            context_enabled,
            has_axis_bindings: action_map
                .bindings
                .iter()
                .any(|binding| !binding.axes.is_empty()),
            #[cfg(test)]
            source_binding_count: action_map.bindings.len(),
        }
    }

    pub(super) fn actions(&self) -> &[CompiledAction] {
        &self.actions
    }

    pub(super) fn has_axis_bindings(&self) -> bool {
        self.has_axis_bindings
    }

    pub(super) fn context_count(&self) -> usize {
        self.context_enabled.len()
    }

    pub(super) fn context_slot(&self, context: &str) -> Option<usize> {
        self.context_slots.get(context).copied()
    }

    pub(super) fn context_enabled(&self, slot: usize) -> bool {
        self.context_enabled.get(slot).copied().unwrap_or(true)
    }

    #[cfg(test)]
    pub(super) fn candidate_count(&self) -> usize {
        self.source_binding_count
    }
}

fn insert_context_slot(
    context_slots: &mut BTreeMap<String, usize>,
    context_enabled: &mut Vec<bool>,
    context: &str,
    enabled: bool,
) {
    if context_slots.contains_key(context) {
        return;
    }
    let slot = context_enabled.len();
    context_slots.insert(context.to_owned(), slot);
    context_enabled.push(enabled);
}
