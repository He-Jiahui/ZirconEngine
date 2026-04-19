use serde::{Deserialize, Serialize};

use super::InputButton;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputSnapshot {
    pub cursor_position: [f32; 2],
    pub pressed_buttons: Vec<InputButton>,
    pub wheel_accumulator: f32,
}
