use crate::input::{GamepadAxisInput, GamepadAxisTransition, InputFrameSnapshot};

#[derive(Debug, Default)]
pub(super) struct FrameAxisIndex {
    values: Vec<AxisValue>,
    transitions: Vec<AxisTransition>,
    #[cfg(test)]
    source_visits: usize,
}

#[derive(Clone, Copy, Debug)]
struct AxisValue {
    input: GamepadAxisInput,
    source_index: usize,
    value: f32,
}

#[derive(Clone, Copy, Debug)]
struct AxisTransition {
    input: GamepadAxisInput,
    source_index: usize,
    transition: GamepadAxisTransition,
}

impl FrameAxisIndex {
    pub(super) fn load_frame(&mut self, frame: &InputFrameSnapshot) {
        self.values.clear();
        self.values.extend(
            frame
                .gamepad_axes
                .iter()
                .enumerate()
                .map(|(source_index, state)| AxisValue {
                    input: GamepadAxisInput::new(state.gamepad, state.axis),
                    source_index,
                    value: state.value,
                }),
        );
        self.values
            .sort_unstable_by_key(|value| (value.input, value.source_index));
        retain_latest_value(&mut self.values);

        self.transitions.clear();
        self.transitions
            .extend(frame.gamepad_axis_transitions.iter().enumerate().map(
                |(source_index, transition)| AxisTransition {
                    input: GamepadAxisInput::new(transition.gamepad, transition.axis),
                    source_index,
                    transition: *transition,
                },
            ));
        self.transitions
            .sort_unstable_by_key(|transition| (transition.input, transition.source_index));
        retain_latest_transition(&mut self.transitions);

        #[cfg(test)]
        {
            self.source_visits = frame
                .gamepad_axes
                .len()
                .saturating_add(frame.gamepad_axis_transitions.len());
        }
    }

    pub(super) fn value(&self, axis: GamepadAxisInput) -> Option<f32> {
        self.values
            .binary_search_by_key(&axis, |value| value.input)
            .ok()
            .map(|index| self.values[index].value)
    }

    pub(super) fn transition(&self, axis: GamepadAxisInput) -> Option<GamepadAxisTransition> {
        self.transitions
            .binary_search_by_key(&axis, |transition| transition.input)
            .ok()
            .map(|index| self.transitions[index].transition)
    }

    pub(super) fn clear(&mut self) {
        self.values.clear();
        self.transitions.clear();
        #[cfg(test)]
        {
            self.source_visits = 0;
        }
    }

    pub(super) fn storage_capacity(&self) -> usize {
        self.values
            .capacity()
            .saturating_add(self.transitions.capacity())
    }

    #[cfg(test)]
    pub(super) fn source_visit_count(&self) -> usize {
        self.source_visits
    }
}

fn retain_latest_value(values: &mut Vec<AxisValue>) {
    let mut retained = 0;
    for index in 0..values.len() {
        if retained > 0 && values[retained - 1].input == values[index].input {
            values[retained - 1] = values[index];
        } else {
            values.swap(retained, index);
            retained += 1;
        }
    }
    values.truncate(retained);
}

fn retain_latest_transition(transitions: &mut Vec<AxisTransition>) {
    let mut retained = 0;
    for index in 0..transitions.len() {
        if retained > 0 && transitions[retained - 1].input == transitions[index].input {
            transitions[retained - 1] = transitions[index];
        } else {
            transitions.swap(retained, index);
            retained += 1;
        }
    }
    transitions.truncate(retained);
}
