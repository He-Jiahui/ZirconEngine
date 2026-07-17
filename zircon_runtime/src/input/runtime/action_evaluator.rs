#[cfg(test)]
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use crate::input::{
    GamepadAxisInput, InputActionMap, InputActionState, InputBinding, InputButton,
    InputFrameSnapshot,
};

mod binding_index;
mod frame_axis_index;

use binding_index::ActionBindingIndex;
use frame_axis_index::FrameAxisIndex;

#[derive(Clone, Debug, Default)]
pub struct InputActionEvaluator {
    action_map: InputActionMap,
    binding_index: ActionBindingIndex,
    #[cfg(test)]
    evaluation_binding_visits: Cell<usize>,
    #[cfg(test)]
    evaluation_axis_source_visits: Cell<usize>,
}

impl InputActionEvaluator {
    pub fn new(action_map: InputActionMap) -> Self {
        let binding_index = ActionBindingIndex::from_action_map(&action_map);
        Self {
            action_map,
            binding_index,
            #[cfg(test)]
            evaluation_binding_visits: Cell::new(0),
            #[cfg(test)]
            evaluation_axis_source_visits: Cell::new(0),
        }
    }

    pub fn action_map(&self) -> &InputActionMap {
        &self.action_map
    }

    pub fn set_action_map(&mut self, action_map: InputActionMap) {
        self.binding_index = ActionBindingIndex::from_action_map(&action_map);
        self.action_map = action_map;
    }

    #[cfg(test)]
    pub(crate) fn indexed_binding_candidate_count(&self) -> usize {
        self.binding_index.candidate_count()
    }

    #[cfg(test)]
    pub(crate) fn evaluation_binding_visit_count(&self) -> usize {
        self.evaluation_binding_visits.get()
    }

    #[cfg(test)]
    pub(crate) fn evaluation_axis_source_visit_count(&self) -> usize {
        self.evaluation_axis_source_visits.get()
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
        #[cfg(test)]
        self.evaluation_binding_visits.set(0);
        #[cfg(test)]
        self.evaluation_axis_source_visits.set(0);
        let frame_axes = if self.binding_index.has_axis_bindings() {
            FrameAxisIndex::from_frame(frame)
        } else {
            FrameAxisIndex::default()
        };
        #[cfg(test)]
        self.evaluation_axis_source_visits
            .set(frame_axes.source_visit_count());
        let consumed_buttons = consumed_buttons.iter().cloned().collect::<BTreeSet<_>>();
        let consumed_axes = consumed_axes.iter().cloned().collect::<BTreeSet<_>>();
        let active_contexts = active_contexts
            .iter()
            .map(|context| context.as_ref())
            .collect::<BTreeSet<_>>();
        let mut pressed = BTreeSet::new();
        let mut just_activated = BTreeSet::new();
        let mut just_deactivated = BTreeSet::new();
        let mut values = BTreeMap::new();

        for action in &self.action_map.actions {
            if !self.action_context_is_active(action.context.as_deref(), &active_contexts) {
                continue;
            }

            let mut action_pressed = false;
            let mut action_just_activated = false;
            let mut action_just_deactivated = false;
            let mut action_value = 0.0;

            for &binding_index in self.binding_index.indices_for_action(&action.id) {
                #[cfg(test)]
                self.evaluation_binding_visits
                    .set(self.evaluation_binding_visits.get().saturating_add(1));
                let binding = &self.action_map.bindings[binding_index];
                if binding
                    .buttons
                    .iter()
                    .any(|button| consumed_buttons.contains(button))
                {
                    continue;
                }

                let has_buttons = !binding.buttons.is_empty();
                let has_axes = !binding.axes.is_empty();
                let all_pressed = binding
                    .buttons
                    .iter()
                    .all(|button| frame.buttons.pressed(button));
                let any_just_pressed = binding
                    .buttons
                    .iter()
                    .any(|button| frame.buttons.just_pressed(button));
                let any_just_released = binding
                    .buttons
                    .iter()
                    .any(|button| frame.buttons.just_released(button));

                if has_axes {
                    if !all_pressed {
                        action_just_deactivated |= has_buttons && any_just_released;
                        continue;
                    }

                    let axis_transition =
                        binding_axis_transition(&frame_axes, binding, &consumed_axes);
                    let axis_value = binding_axis_value(&frame_axes, binding, &consumed_axes);
                    if axis_value != 0.0 {
                        action_pressed = true;
                        action_value = dominant_action_value(action_value, axis_value);
                        action_just_activated |=
                            (has_buttons && any_just_pressed) || axis_transition.activated;
                    } else {
                        action_just_deactivated |= axis_transition.deactivated;
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

            if action_pressed {
                pressed.insert(action.id.clone());
                values.insert(action.id.clone(), action_value);
                if action_just_activated {
                    just_activated.insert(action.id.clone());
                }
            } else if action_just_deactivated {
                just_deactivated.insert(action.id.clone());
            }
        }

        InputActionState::from_sets_and_values(pressed, just_activated, just_deactivated, values)
    }

    fn action_context_is_active(
        &self,
        action_context: Option<&str>,
        active_contexts: &BTreeSet<&str>,
    ) -> bool {
        let Some(context) = action_context else {
            return true;
        };

        if !self.action_map.context_enabled(context) {
            return false;
        }

        active_contexts.is_empty() || active_contexts.contains(context)
    }
}

fn binding_axis_consumed(
    gamepad_axis: GamepadAxisInput,
    consumed_axes: &BTreeSet<GamepadAxisInput>,
) -> bool {
    consumed_axes.contains(&gamepad_axis)
}

fn binding_axis_value(
    frame_axes: &FrameAxisIndex,
    binding: &InputBinding,
    consumed_axes: &BTreeSet<GamepadAxisInput>,
) -> f32 {
    binding
        .axes
        .iter()
        .filter(|binding_axis| {
            !binding_axis_consumed(
                GamepadAxisInput::new(binding_axis.gamepad, binding_axis.axis),
                consumed_axes,
            )
        })
        .filter_map(|binding_axis| {
            frame_axes
                .value(GamepadAxisInput::new(
                    binding_axis.gamepad,
                    binding_axis.axis,
                ))
                .map(|value| binding_axis.value(value))
        })
        .fold(0.0, dominant_action_value)
}

fn binding_axis_transition(
    frame_axes: &FrameAxisIndex,
    binding: &InputBinding,
    consumed_axes: &BTreeSet<GamepadAxisInput>,
) -> BindingAxisTransition {
    let mut any_transition = false;
    let mut previous_value = 0.0;
    let mut value = 0.0;

    for binding_axis in &binding.axes {
        if binding_axis_consumed(
            GamepadAxisInput::new(binding_axis.gamepad, binding_axis.axis),
            consumed_axes,
        ) {
            continue;
        }

        let axis_input = GamepadAxisInput::new(binding_axis.gamepad, binding_axis.axis);
        let axis_transition = frame_axes.transition(axis_input);
        let current_source = axis_transition
            .map(|axis| axis.value)
            .or_else(|| frame_axes.value(axis_input))
            .unwrap_or(0.0);
        let previous_source = axis_transition
            .map(|axis| axis.previous_value)
            .unwrap_or(current_source);

        any_transition |= axis_transition.is_some();
        previous_value = dominant_action_value(previous_value, binding_axis.value(previous_source));
        value = dominant_action_value(value, binding_axis.value(current_source));
    }

    BindingAxisTransition {
        activated: any_transition && previous_value == 0.0 && value != 0.0,
        deactivated: any_transition && previous_value != 0.0 && value == 0.0,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BindingAxisTransition {
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
