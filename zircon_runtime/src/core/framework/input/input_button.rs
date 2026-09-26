use serde::{Deserialize, Serialize};

use super::{GamepadButton, GamepadId};

/// 动作绑定使用的按钮身份；键盘物理键码与逻辑键名是不同项，同一宿主事件可同时更新两者。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InputButton {
    MouseLeft,
    MouseRight,
    MouseMiddle,
    MouseBack,
    MouseForward,
    MouseOther(u16),
    KeyCode(u32),
    Key(String),
    Gamepad {
        gamepad: GamepadId,
        button: GamepadButton,
    },
}
