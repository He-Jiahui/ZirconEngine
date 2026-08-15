use crate::input::{GamepadAxisInput, InputButton, InputFrameSnapshot};

use super::consumed_input_index::ConsumedInputIndex;
use super::frame_axis_index::FrameAxisIndex;
use super::generation::ActionEvaluationGeneration;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct EvaluatedAction {
    pub(super) pressed: bool,
    pub(super) just_activated: bool,
    pub(super) just_deactivated: bool,
    pub(super) value: f32,
}

/// Reusable evaluator-local state. The default action manager serializes evaluator calls.
#[derive(Debug, Default)]
pub(super) struct ActionEvaluationWorkspace {
    frame_axes: FrameAxisIndex,
    consumed_inputs: ConsumedInputIndex,
    active_contexts: Vec<bool>,
    actions: Vec<EvaluatedAction>,
    #[cfg(test)]
    storage_growth_count: usize,
}

impl ActionEvaluationWorkspace {
    pub(super) fn prepare(
        &mut self,
        generation: &ActionEvaluationGeneration,
        frame: &InputFrameSnapshot,
        active_contexts: &[impl AsRef<str>],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) {
        self.prepare_actions(generation.actions().len());
        self.prepare_contexts(generation, active_contexts);
        self.prepare_frame_axes(generation, frame);
        self.prepare_consumed_inputs(consumed_buttons, consumed_axes);
    }

    pub(super) fn frame_axes(&self) -> &FrameAxisIndex {
        &self.frame_axes
    }

    pub(super) fn consumed_inputs(&self) -> &ConsumedInputIndex {
        &self.consumed_inputs
    }

    #[cfg(test)]
    pub(super) fn consumed_input_source_visit_count(&self) -> usize {
        self.consumed_inputs.source_visit_count()
    }

    pub(super) fn action(&self, slot: usize) -> EvaluatedAction {
        self.actions[slot]
    }

    pub(super) fn set_action(&mut self, slot: usize, action: EvaluatedAction) {
        self.actions[slot] = action;
    }

    pub(super) fn context_is_active(&self, slot: usize) -> bool {
        self.active_contexts.get(slot).copied().unwrap_or(false)
    }

    pub(super) fn reset(&mut self) {
        self.frame_axes.clear();
        self.consumed_inputs.clear();
        self.active_contexts.clear();
        self.actions.clear();
    }

    fn prepare_actions(&mut self, action_count: usize) {
        self.record_growth(
            self.actions.len() < action_count && self.actions.capacity() < action_count,
        );
        self.actions
            .resize(action_count, EvaluatedAction::default());
        self.actions.fill(EvaluatedAction::default());
    }

    fn prepare_contexts(
        &mut self,
        generation: &ActionEvaluationGeneration,
        active_contexts: &[impl AsRef<str>],
    ) {
        let context_count = generation.context_count();
        self.record_growth(
            self.active_contexts.len() < context_count
                && self.active_contexts.capacity() < context_count,
        );
        self.active_contexts.resize(context_count, false);
        self.active_contexts.fill(false);
        for context in active_contexts {
            if let Some(slot) = generation.context_slot(context.as_ref()) {
                self.active_contexts[slot] = true;
            }
        }
    }

    fn prepare_frame_axes(
        &mut self,
        generation: &ActionEvaluationGeneration,
        frame: &InputFrameSnapshot,
    ) {
        if !generation.has_axis_bindings() {
            self.frame_axes.clear();
            return;
        }
        let capacity_before = self.frame_axes.storage_capacity();
        self.frame_axes.load_frame(frame);
        self.record_growth(self.frame_axes.storage_capacity() > capacity_before);
    }

    fn prepare_consumed_inputs(
        &mut self,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) {
        let capacity_before = self.consumed_inputs.storage_capacity();
        self.consumed_inputs.load(consumed_buttons, consumed_axes);
        self.record_growth(self.consumed_inputs.storage_capacity() > capacity_before);
    }

    fn record_growth(&mut self, grew: bool) {
        #[cfg(test)]
        if grew {
            self.storage_growth_count = self.storage_growth_count.saturating_add(1);
        }
        #[cfg(not(test))]
        let _ = grew;
    }

    #[cfg(test)]
    pub(super) fn storage_growth_count(&self) -> usize {
        self.storage_growth_count
    }
}
