use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadAxis;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadAxisTransition {
    pub gamepad: GamepadId,
    pub axis: GamepadAxis,
    pub previous_value: f32,
    pub value: f32,
}
