use std::collections::{BTreeMap, BTreeSet};
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};

use crate::input::{
    GamepadAxisInput, InputActionMap, InputActionState, InputBinding, InputButton,
    InputFrameSnapshot,
};

mod consumed_input_index;
mod frame_axis_index;
mod generation;
mod workspace;

#[cfg(test)]
#[path = "action_evaluator/button_state_single_pass_tests.rs"]
mod button_state_single_pass_tests;

use consumed_input_index::ConsumedInputIndex;
use frame_axis_index::FrameAxisIndex;
use generation::ActionEvaluationGeneration;
use workspace::{ActionEvaluationWorkspace, EvaluatedAction};

#[derive(Debug)]
pub struct InputActionEvaluator {
    action_map: InputActionMap,
    generation: ActionEvaluationGeneration,
    // Direct evaluators synchronize this private scratch; the default manager reuses its outer lock.
    workspace: Mutex<ActionEvaluationWorkspace>,
    #[cfg(test)]
    metrics: EvaluationMetrics,
}

#[cfg(test)]
#[derive(Debug, Default)]
struct EvaluationMetrics {
    binding_visits: AtomicUsize,
    axis_source_visits: AtomicUsize,
    consumed_input_source_visits: AtomicUsize,
    generation_builds: AtomicUsize,
    output_actions: AtomicUsize,
}

#[cfg(test)]
impl EvaluationMetrics {
    fn reset_for_evaluation(&self) {
        self.binding_visits.store(0, Ordering::Relaxed);
        self.axis_source_visits.store(0, Ordering::Relaxed);
        self.consumed_input_source_visits
            .store(0, Ordering::Relaxed);
        self.output_actions.store(0, Ordering::Relaxed);
    }

    fn record_binding_visit(&self) {
        saturating_increment(&self.binding_visits);
    }

    fn record_generation_build(&self) {
        saturating_increment(&self.generation_builds);
    }

    fn record_axis_source_visits(&self, count: usize) {
        self.axis_source_visits.store(count, Ordering::Relaxed);
    }

    fn record_consumed_input_source_visits(&self, count: usize) {
        self.consumed_input_source_visits
            .store(count, Ordering::Relaxed);
    }

    fn record_output_actions(&self, count: usize) {
        self.output_actions.store(count, Ordering::Relaxed);
    }
}

#[cfg(test)]
fn saturating_increment(counter: &AtomicUsize) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        Some(value.saturating_add(1))
    });
}

impl Clone for InputActionEvaluator {
    fn clone(&self) -> Self {
        Self::new(self.action_map.clone())
    }
}

impl Default for InputActionEvaluator {
    fn default() -> Self {
        Self::new(InputActionMap::default())
    }
}

impl InputActionEvaluator {
    pub fn new(action_map: InputActionMap) -> Self {
        let generation = ActionEvaluationGeneration::from_action_map(&action_map);
        Self {
            action_map,
            generation,
            workspace: Mutex::default(),
            #[cfg(test)]
            metrics: EvaluationMetrics {
                generation_builds: AtomicUsize::new(1),
                ..EvaluationMetrics::default()
            },
        }
    }

    pub fn action_map(&self) -> &InputActionMap {
        &self.action_map
    }

    pub fn set_action_map(&mut self, action_map: InputActionMap) {
        let generation = ActionEvaluationGeneration::from_action_map(&action_map);
        self.action_map = action_map;
        self.generation = generation;
        self.workspace
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reset();
        #[cfg(test)]
        self.metrics.record_generation_build();
    }

    #[cfg(test)]
    pub(crate) fn indexed_binding_candidate_count(&self) -> usize {
        self.generation.candidate_count()
    }

    #[cfg(test)]
    pub(crate) fn evaluation_binding_visit_count(&self) -> usize {
        self.metrics.binding_visits.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn evaluation_axis_source_visit_count(&self) -> usize {
        self.metrics.axis_source_visits.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn evaluation_consumed_input_source_visit_count(&self) -> usize {
        self.metrics
            .consumed_input_source_visits
            .load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn evaluation_generation_build_count(&self) -> usize {
        self.metrics.generation_builds.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn evaluation_output_action_count(&self) -> usize {
        self.metrics.output_actions.load(Ordering::Relaxed)
    }

    #[cfg(test)]
    pub(crate) fn workspace_storage_growth_count(&self) -> usize {
        self.lock_workspace().storage_growth_count()
    }

    pub fn evaluate(&self, frame: &InputFrameSnapshot) -> InputActionState {
        self.evaluate_with_consumed_buttons(frame, &[])
    }

    pub fn evaluate_with_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.evaluate_with_consumed_input(frame, consumed_buttons, &[])
    }

    pub fn evaluate_with_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        self.evaluate_with_active_contexts_and_consumed_input(
            frame,
            &[] as &[&str],
            consumed_buttons,
            consumed_axes,
        )
    }

    pub fn evaluate_with_active_contexts(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[impl AsRef<str>],
    ) -> InputActionState {
        self.evaluate_with_active_contexts_and_consumed_buttons(frame, active_contexts, &[])
    }

    pub fn evaluate_with_active_contexts_and_consumed_buttons(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[impl AsRef<str>],
        consumed_buttons: &[InputButton],
    ) -> InputActionState {
        self.evaluate_with_active_contexts_and_consumed_input(
            frame,
            active_contexts,
            consumed_buttons,
            &[],
        )
    }

    pub fn evaluate_with_active_contexts_and_consumed_input(
        &self,
        frame: &InputFrameSnapshot,
        active_contexts: &[impl AsRef<str>],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        let mut workspace = self.lock_workspace();
        evaluate_with_workspace(
            &self.action_map,
            &self.generation,
            frame,
            active_contexts,
            consumed_buttons,
            consumed_axes,
            #[cfg(test)]
            &self.metrics,
            &mut workspace,
        )
    }

    pub(super) fn evaluate_while_manager_locked(
        &mut self,
        frame: &InputFrameSnapshot,
        active_contexts: &[&str],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) -> InputActionState {
        let action_map = &self.action_map;
        let generation = &self.generation;
        #[cfg(test)]
        let metrics = &self.metrics;
        let workspace = self
            .workspace
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        evaluate_with_workspace(
            action_map,
            generation,
            frame,
            active_contexts,
            consumed_buttons,
            consumed_axes,
            #[cfg(test)]
            metrics,
            workspace,
        )
    }

    fn lock_workspace(&self) -> MutexGuard<'_, ActionEvaluationWorkspace> {
        self.workspace
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn evaluate_with_workspace(
    action_map: &InputActionMap,
    generation: &ActionEvaluationGeneration,
    frame: &InputFrameSnapshot,
    active_contexts: &[impl AsRef<str>],
    consumed_buttons: &[InputButton],
    consumed_axes: &[GamepadAxisInput],
    #[cfg(test)] metrics: &EvaluationMetrics,
    workspace: &mut ActionEvaluationWorkspace,
) -> InputActionState {
    #[cfg(test)]
    metrics.reset_for_evaluation();

    let all_contexts_active = active_contexts.is_empty();
    workspace.prepare(
        generation,
        frame,
        active_contexts,
        consumed_buttons,
        consumed_axes,
    );
    #[cfg(test)]
    metrics.record_axis_source_visits(workspace.frame_axes().source_visit_count());
    #[cfg(test)]
    metrics.record_consumed_input_source_visits(workspace.consumed_input_source_visit_count());

    for (slot, compiled) in generation.actions().iter().enumerate() {
        if !action_context_is_active(
            generation,
            compiled.context_slot,
            all_contexts_active,
            workspace,
        ) {
            continue;
        }

        let mut action_pressed = false;
        let mut action_just_activated = false;
        let mut action_just_deactivated = false;
        let mut action_value = 0.0;

        for &binding_index in compiled.binding_indices(generation) {
            #[cfg(test)]
            metrics.record_binding_visit();
            let binding = &action_map.bindings[binding_index];
            if binding.buttons.iter().any(|button| {
                workspace
                    .consumed_inputs()
                    .button_is_consumed(consumed_buttons, button)
            }) {
                continue;
            }

            let has_buttons = !binding.buttons.is_empty();
            let has_axes = !binding.axes.is_empty();
            let mut all_pressed = true;
            let mut any_just_pressed = false;
            let mut any_just_released = false;
            for button in &binding.buttons {
                if all_pressed {
                    all_pressed = frame.buttons.pressed(button);
                }
                if !any_just_pressed {
                    any_just_pressed = frame.buttons.just_pressed(button);
                }
                if !any_just_released {
                    any_just_released = frame.buttons.just_released(button);
                }
                if !all_pressed && any_just_pressed && any_just_released {
                    break;
                }
            }

            if has_axes {
                if !all_pressed {
                    action_just_deactivated |= has_buttons && any_just_released;
                    continue;
                }

                let axis = evaluate_binding_axes(
                    workspace.frame_axes(),
                    workspace.consumed_inputs(),
                    binding,
                    consumed_axes,
                );
                if axis.value != 0.0 {
                    action_pressed = true;
                    action_value = dominant_action_value(action_value, axis.value);
                    action_just_activated |= (has_buttons && any_just_pressed) || axis.activated;
                } else {
                    action_just_deactivated |= axis.deactivated;
                }
                continue;
            }

            if all_pressed {
                action_pressed = true;
                action_value = dominant_action_value(action_value, 1.0);
                action_just_activated |= any_just_pressed;
            } else {
                action_just_deactivated |= any_just_released;
            }
        }

        workspace.set_action(
            slot,
            EvaluatedAction {
                pressed: action_pressed,
                just_activated: action_just_activated,
                just_deactivated: action_just_deactivated,
                value: action_value,
            },
        );
    }

    project_action_state(
        action_map,
        generation,
        #[cfg(test)]
        metrics,
        workspace,
    )
}

fn action_context_is_active(
    generation: &ActionEvaluationGeneration,
    context_slot: Option<usize>,
    all_contexts_active: bool,
    workspace: &ActionEvaluationWorkspace,
) -> bool {
    let Some(context_slot) = context_slot else {
        return true;
    };
    generation.context_enabled(context_slot)
        && (all_contexts_active || workspace.context_is_active(context_slot))
}

fn project_action_state(
    action_map: &InputActionMap,
    generation: &ActionEvaluationGeneration,
    #[cfg(test)] metrics: &EvaluationMetrics,
    workspace: &ActionEvaluationWorkspace,
) -> InputActionState {
    let mut pressed = BTreeSet::new();
    let mut just_activated = BTreeSet::new();
    let mut just_deactivated = BTreeSet::new();
    let mut values = BTreeMap::new();

    for (slot, compiled) in generation.actions().iter().enumerate() {
        let action = &action_map.actions[compiled.action_index];
        let evaluated = workspace.action(slot);
        if evaluated.pressed {
            pressed.insert(action.id.clone());
            values.insert(action.id.clone(), evaluated.value);
            if evaluated.just_activated {
                just_activated.insert(action.id.clone());
            }
        } else if evaluated.just_deactivated {
            just_deactivated.insert(action.id.clone());
        }
    }

    #[cfg(test)]
    metrics.record_output_actions(pressed.len().saturating_add(just_deactivated.len()));
    InputActionState::from_sets_and_values(pressed, just_activated, just_deactivated, values)
}

fn evaluate_binding_axes(
    frame_axes: &FrameAxisIndex,
    consumed_inputs: &ConsumedInputIndex,
    binding: &InputBinding,
    consumed_axes: &[GamepadAxisInput],
) -> BindingAxisEvaluation {
    let mut any_transition = false;
    let mut previous_value = 0.0;
    let mut transition_value = 0.0;
    let mut action_value = 0.0;

    for binding_axis in &binding.axes {
        let axis_input = GamepadAxisInput::new(binding_axis.gamepad, binding_axis.axis);
        if consumed_inputs.axis_is_consumed(consumed_axes, axis_input) {
            continue;
        }

        let current_state = frame_axes.value(axis_input);
        if let Some(current_state) = current_state {
            action_value = dominant_action_value(action_value, binding_axis.value(current_state));
        }
        let axis_transition = frame_axes.transition(axis_input);
        let current_source = axis_transition
            .map(|axis| axis.value)
            .or(current_state)
            .unwrap_or(0.0);
        let previous_source = axis_transition
            .map(|axis| axis.previous_value)
            .unwrap_or(current_source);

        any_transition |= axis_transition.is_some();
        previous_value = dominant_action_value(previous_value, binding_axis.value(previous_source));
        transition_value =
            dominant_action_value(transition_value, binding_axis.value(current_source));
    }

    BindingAxisEvaluation {
        value: action_value,
        activated: any_transition && previous_value == 0.0 && transition_value != 0.0,
        deactivated: any_transition && previous_value != 0.0 && transition_value == 0.0,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BindingAxisEvaluation {
    value: f32,
    activated: bool,
    deactivated: bool,
}

fn dominant_action_value(current: f32, candidate: f32) -> f32 {
    if candidate.abs() > current.abs() {
        candidate
    } else {
        current
    }
}
