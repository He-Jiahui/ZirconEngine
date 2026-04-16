use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InputButton {
    MouseLeft,
    MouseRight,
    MouseMiddle,
    Key(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    CursorMoved { x: f32, y: f32 },
    ButtonPressed(InputButton),
    ButtonReleased(InputButton),
    WheelScrolled { delta: f32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputEventRecord {
    pub sequence: u64,
    pub timestamp_millis: u64,
    pub event: InputEvent,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputSnapshot {
    pub cursor_position: [f32; 2],
    pub pressed_buttons: Vec<InputButton>,
    pub wheel_accumulator: f32,
}
