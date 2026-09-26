use serde::{Deserialize, Serialize};

/// 预编辑或周围文本中的 UTF-8 字节范围；转给 UI 或宿主前应相对相应文本校验边界。
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

/// 输入法请求删除光标两侧的字节数，消费方须按文本边界约束实际删除范围。
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

/// 宿主送入运行时的输入法状态与编辑事件；预编辑、提交和删除需按到达顺序处理。
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

/// 运行时回传给宿主输入法的编辑上下文；光标、锚点和组合范围均以本文本的 UTF-8 字节计。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImeSurroundingText {
    pub value: String,
    pub cursor: usize,
    pub anchor: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_range: Option<ImeCursorRange>,
}

impl ImeSurroundingText {
    pub fn new(value: impl Into<String>, cursor: usize, anchor: usize) -> Self {
        Self {
            value: value.into(),
            cursor,
            anchor,
            composition_range: None,
        }
    }

    pub fn with_composition_range(mut self, composition_range: Option<ImeCursorRange>) -> Self {
        self.composition_range = composition_range;
        self
    }
}

/// 编辑控件或失焦处理发往宿主输入法的反向请求，须通过宿主请求排空路径执行。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImeHostRequest {
    Enable,
    Disable,
    SetCursorArea(ImeCursorArea),
    SetSurroundingText(ImeSurroundingText),
}
