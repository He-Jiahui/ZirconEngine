use serde::{Deserialize, Serialize};

use super::{UiTextAlign, UiTextWrap};
use crate::ui::layout::UiFrame;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextLayout {
    pub text_align: UiTextAlign,
    pub wrap: UiTextWrap,
    pub font_size: f32,
    pub line_height: f32,
    pub lines: Vec<UiResolvedTextLine>,
    pub overflow_clipped: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedTextLine {
    pub text: String,
    pub frame: UiFrame,
}
