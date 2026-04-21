use serde::{Deserialize, Serialize};

use super::{UiTextAlign, UiTextRenderMode, UiTextWrap};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiResolvedStyle {
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub font: Option<String>,
    pub font_family: Option<String>,
    pub font_size: f32,
    pub line_height: f32,
    pub text_align: UiTextAlign,
    pub wrap: UiTextWrap,
    pub text_render_mode: UiTextRenderMode,
}

impl UiResolvedStyle {
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;
    pub const DEFAULT_LINE_HEIGHT_SCALE: f32 = 1.2;

    pub fn default_line_height(font_size: f32) -> f32 {
        font_size * Self::DEFAULT_LINE_HEIGHT_SCALE
    }
}

impl Default for UiResolvedStyle {
    fn default() -> Self {
        Self {
            background_color: None,
            foreground_color: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            font: None,
            font_family: None,
            font_size: Self::DEFAULT_FONT_SIZE,
            line_height: Self::default_line_height(Self::DEFAULT_FONT_SIZE),
            text_align: UiTextAlign::default(),
            wrap: UiTextWrap::default(),
            text_render_mode: UiTextRenderMode::default(),
        }
    }
}
