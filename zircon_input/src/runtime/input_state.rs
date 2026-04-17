use std::collections::BTreeSet;

use crate::{InputButton, InputEvent, InputEventRecord};

#[derive(Debug, Default)]
pub(crate) struct InputState {
    pub(crate) cursor_position: [f32; 2],
    pub(crate) pressed_buttons: BTreeSet<InputButton>,
    pub(crate) wheel_accumulator: f32,
    pub(crate) events: Vec<InputEvent>,
    pub(crate) records: Vec<InputEventRecord>,
    pub(crate) next_sequence: u64,
}
