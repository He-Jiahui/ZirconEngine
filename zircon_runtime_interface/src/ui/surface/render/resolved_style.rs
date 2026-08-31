use serde::{Deserialize, Serialize};

use crate::ui::layout::UiPixelSnappingPolicy;
use crate::ui::style::{UiPainterFamily, UiPainterResolvedState};

use super::{
    UiRichTextFormat, UiTextAlign, UiTextDecorations, UiTextDirection, UiTextDistanceFieldEffects,
    UiTextOverflow, UiTextRenderMode, UiTextWrap, UiTextWritingMode,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiResolvedStyle {
    /// Paint-only device-pixel alignment. Text/layout cache keys intentionally
    /// derive only from their relevant fields and do not consume this policy.
    pub pixel_snapping: UiPixelSnappingPolicy,
    pub background_color: Option<String>,
    pub foreground_color: Option<String>,
    pub border_color: Option<String>,
    pub border_width: f32,
    pub corner_radius: f32,
    pub font: Option<String>,
    pub font_family: Option<String>,
    /// BCP 47 language tag used by shaping and locale-sensitive font fallback.
    pub language: Option<String>,
    pub font_weight: u16,
    pub font_size: f32,
    pub line_height: f32,
    pub tab_size: f32,
    pub text_align: UiTextAlign,
    pub wrap: UiTextWrap,
    pub text_direction: UiTextDirection,
    pub text_writing_mode: UiTextWritingMode,
    pub text_overflow: UiTextOverflow,
    pub rich_text_format: UiRichTextFormat,
    pub text_render_mode: UiTextRenderMode,
    pub text_effects: UiTextDistanceFieldEffects,
    pub text_decorations: UiTextDecorations,
    pub painter_family: UiPainterFamily,
    pub painter_state: UiPainterResolvedState,
}

impl UiResolvedStyle {
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;
    pub const DEFAULT_FONT_WEIGHT: u16 = 400;
    pub const DEFAULT_LINE_HEIGHT_SCALE: f32 = 1.2;
    pub const DEFAULT_TAB_SIZE: f32 = 4.0;
    pub const MIN_FONT_WEIGHT: u16 = 1;
    pub const MAX_FONT_WEIGHT: u16 = 1000;

    pub fn default_line_height(font_size: f32) -> f32 {
        font_size * Self::DEFAULT_LINE_HEIGHT_SCALE
    }

    pub const fn normalized_font_weight(font_weight: u16) -> u16 {
        if font_weight < Self::MIN_FONT_WEIGHT {
            Self::MIN_FONT_WEIGHT
        } else if font_weight > Self::MAX_FONT_WEIGHT {
            Self::MAX_FONT_WEIGHT
        } else {
            font_weight
        }
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
            pixel_snapping: UiPixelSnappingPolicy::Inherit,
            background_color: None,
            foreground_color: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
            font: None,
            font_family: None,
            language: None,
            font_weight: Self::DEFAULT_FONT_WEIGHT,
            font_size: Self::DEFAULT_FONT_SIZE,
            line_height: Self::default_line_height(Self::DEFAULT_FONT_SIZE),
            tab_size: Self::DEFAULT_TAB_SIZE,
            text_align: UiTextAlign::default(),
            wrap: UiTextWrap::default(),
            text_direction: UiTextDirection::default(),
            text_writing_mode: UiTextWritingMode::default(),
            text_overflow: UiTextOverflow::default(),
            rich_text_format: UiRichTextFormat::Plain,
            text_render_mode: UiTextRenderMode::default(),
            text_effects: UiTextDistanceFieldEffects::default(),
            text_decorations: UiTextDecorations::default(),
            painter_family: UiPainterFamily::default(),
            painter_state: UiPainterResolvedState::default(),
        }
    }
}
