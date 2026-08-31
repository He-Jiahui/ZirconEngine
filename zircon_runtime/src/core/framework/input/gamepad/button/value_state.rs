use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadButton;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadButtonValueState {
    pub gamepad: GamepadId,
    pub button: GamepadButton,
    pub value: f32,
}
