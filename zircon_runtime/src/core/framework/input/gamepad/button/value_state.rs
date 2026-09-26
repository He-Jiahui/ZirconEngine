use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadButton;

/// 保留模拟按钮的当前幅值，与布尔按住状态并列供帧快照观察；动作触发依据滞回后的按钮边沿。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadButtonValueState {
    pub gamepad: GamepadId,
    pub button: GamepadButton,
    pub value: f32,
}
