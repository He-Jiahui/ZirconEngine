use serde::{Deserialize, Serialize};

use super::{GamepadButton, GamepadId};

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
