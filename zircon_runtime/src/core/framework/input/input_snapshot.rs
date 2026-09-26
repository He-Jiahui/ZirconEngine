use serde::{Deserialize, Serialize};

use super::InputButton;

/// 兼容只需要光标、按住按钮和纵向滚动的简化视图；它不携带帧边沿及扩展设备事件。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputSnapshot {
    pub cursor_position: [f32; 2],
    pub pressed_buttons: Vec<InputButton>,
    pub wheel_accumulator: f32,
}
