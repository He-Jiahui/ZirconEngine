use std::collections::BTreeMap;

use crate::input::{GamepadAxisInput, GamepadAxisTransition, InputFrameSnapshot};

#[derive(Debug, Default)]
pub(super) struct FrameAxisIndex {
    values: BTreeMap<GamepadAxisInput, f32>,
    transitions: BTreeMap<GamepadAxisInput, GamepadAxisTransition>,
    #[cfg(test)]
    source_visits: usize,
}

impl FrameAxisIndex {
    pub(super) fn from_frame(frame: &InputFrameSnapshot) -> Self {
        let mut values = BTreeMap::new();
        for state in &frame.gamepad_axes {
            values.insert(
                GamepadAxisInput::new(state.gamepad, state.axis),
                state.value,
            );
        }

        let mut transitions = BTreeMap::new();
        for transition in &frame.gamepad_axis_transitions {
            transitions.insert(
                GamepadAxisInput::new(transition.gamepad, transition.axis),
                *transition,
            );
        }

        Self {
            values,
            transitions,
            #[cfg(test)]
            source_visits: frame
                .gamepad_axes
                .len()
                .saturating_add(frame.gamepad_axis_transitions.len()),
        }
    }

    pub(super) fn value(&self, axis: GamepadAxisInput) -> Option<f32> {
        self.values.get(&axis).copied()
    }

    pub(super) fn transition(&self, axis: GamepadAxisInput) -> Option<GamepadAxisTransition> {
        self.transitions.get(&axis).copied()
    }

    #[cfg(test)]
    pub(super) fn source_visit_count(&self) -> usize {
        self.source_visits
    }
}
