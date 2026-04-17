use serde::{Deserialize, Serialize};

use crate::InputButton;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    CursorMoved { x: f32, y: f32 },
    ButtonPressed(InputButton),
    ButtonReleased(InputButton),
    WheelScrolled { delta: f32 },
}
