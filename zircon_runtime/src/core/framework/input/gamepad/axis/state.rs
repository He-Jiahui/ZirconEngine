use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadAxis;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadAxisState {
    pub gamepad: GamepadId,
    pub axis: GamepadAxis,
    pub value: f32,
}
