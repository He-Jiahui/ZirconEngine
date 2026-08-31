use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadAxis;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GamepadAxisInput {
    pub gamepad: GamepadId,
    pub axis: GamepadAxis,
}

impl GamepadAxisInput {
    pub const fn new(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self { gamepad, axis }
    }
}
