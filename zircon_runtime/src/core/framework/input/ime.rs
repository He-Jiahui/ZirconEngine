use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImeCursorRange {
    pub start: usize,
    pub end: usize,
}

impl ImeCursorRange {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImePreedit {
    pub value: String,
    pub cursor: Option<ImeCursorRange>,
}

impl ImePreedit {
    pub fn new(value: impl Into<String>, cursor: Option<ImeCursorRange>) -> Self {
        Self {
            value: value.into(),
            cursor,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImeDeleteSurrounding {
    pub before_bytes: usize,
    pub after_bytes: usize,
}

impl ImeDeleteSurrounding {
    pub const fn new(before_bytes: usize, after_bytes: usize) -> Self {
        Self {
            before_bytes,
            after_bytes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImeEvent {
    Enabled,
    Disabled,
    Preedit(ImePreedit),
    Commit(String),
    DeleteSurrounding(ImeDeleteSurrounding),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImeCursorArea {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ImeCursorArea {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImeSurroundingText {
    pub value: String,
    pub cursor: usize,
    pub anchor: usize,
}

impl ImeSurroundingText {
    pub fn new(value: impl Into<String>, cursor: usize, anchor: usize) -> Self {
        Self {
            value: value.into(),
            cursor,
            anchor,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImeHostRequest {
    Enable,
    Disable,
    SetCursorArea(ImeCursorArea),
    SetSurroundingText(ImeSurroundingText),
}
