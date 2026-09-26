use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadAxis;

/// 记录一根设备轴的变化，供动作求值识别跨零边沿；常规同帧采样合并起点和最终值。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadAxisTransition {
    pub gamepad: GamepadId,
    pub axis: GamepadAxis,
    pub previous_value: f32,
    pub value: f32,
}
