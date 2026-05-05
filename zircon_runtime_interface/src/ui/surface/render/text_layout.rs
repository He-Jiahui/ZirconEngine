use serde::{Deserialize, Serialize};

use super::{
    UiEditableTextState, UiTextAlign, UiTextDirection, UiTextOverflow, UiTextRunKind, UiTextWrap,
};
use crate::ui::layout::UiFrame;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextLayout {
    pub text_align: UiTextAlign,
    pub wrap: UiTextWrap,
    pub direction: UiTextDirection,
    pub overflow: UiTextOverflow,
    pub font_size: f32,
    pub line_height: f32,
    pub measured_width: f32,
    pub measured_height: f32,
    pub source_range: UiTextRange,
    pub lines: Vec<UiResolvedTextLine>,
    pub overflow_clipped: bool,
    pub editable: Option<UiEditableTextState>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextLine {
    pub text: String,
    pub frame: UiFrame,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub measured_width: f32,
    pub baseline: f32,
    pub direction: UiTextDirection,
    pub runs: Vec<UiResolvedTextRun>,
    pub ellipsized: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextRun {
    pub kind: UiTextRunKind,
    pub text: String,
    pub source_range: UiTextRange,
    pub visual_range: UiTextRange,
    pub direction: UiTextDirection,
}
