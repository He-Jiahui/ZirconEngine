use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadAxis;

/// 帧快照中的当前轴样本；默认输入管理器在写入前完成死区映射和阈值过滤。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadAxisState {
    pub gamepad: GamepadId,
    pub axis: GamepadAxis,
    pub value: f32,
}
