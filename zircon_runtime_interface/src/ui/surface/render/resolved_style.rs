use serde::{Deserialize, Serialize};

use crate::ui::style::{UiPainterFamily, UiPainterResolvedState};

use super::{UiTextAlign, UiTextDirection, UiTextOverflow, UiTextRenderMode, UiTextWrap};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
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
    pub text_direction: UiTextDirection,
    pub text_overflow: UiTextOverflow,
    pub rich_text: bool,
    pub text_render_mode: UiTextRenderMode,
    pub painter_family: UiPainterFamily,
    pub painter_state: UiPainterResolvedState,
}

impl UiResolvedStyle {
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;
    pub const DEFAULT_LINE_HEIGHT_SCALE: f32 = 1.2;

    pub fn default_line_height(font_size: f32) -> f32 {
        font_size * Self::DEFAULT_LINE_HEIGHT_SCALE
    }

    pub fn with_painter_state(
        mut self,
        family: UiPainterFamily,
        state: UiPainterResolvedState,
    ) -> Self {
        self.painter_family = family;
        self.painter_state = state;
        self
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
            text_direction: UiTextDirection::default(),
            text_overflow: UiTextOverflow::default(),
            rich_text: false,
            text_render_mode: UiTextRenderMode::default(),
            painter_family: UiPainterFamily::default(),
            painter_state: UiPainterResolvedState::default(),
        }
    }
}
