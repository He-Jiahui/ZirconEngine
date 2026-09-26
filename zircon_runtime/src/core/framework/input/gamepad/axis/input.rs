use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadAxis;

/// 用设备与轴的组合标识一次可消费的模拟输入，避免上层消费一根轴时屏蔽其他设备的同名轴。
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
