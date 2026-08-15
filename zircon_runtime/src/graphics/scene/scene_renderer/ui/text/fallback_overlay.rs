use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextDirection, UiTextWrap};

use super::super::render::ScreenSpaceUiTextBatch;

impl ScreenSpaceUiTextBatch {
    /// Creates a native-only overlay for an SDF atlas fallback span.
    ///
    /// The overlay owns a source substring, so it must never retain the source batch's
    /// resolved-layout output or error state. Native text will shape the substring again.
    pub(super) fn native_fallback_overlay(
        &self,
        text: String,
        frame: UiFrame,
        text_direction: UiTextDirection,
    ) -> Self {
        Self {
            route_identity: self.route_identity.clone(),
            command_generation: self.command_generation,
            raster_scale: self.raster_scale,
            text,
            frame,
            clip_frame: self.clip_frame,
            source_range: None,
            is_source_isomorphic_layout_line: false,
            glyph_advances: Vec::new(),
            shaped_glyphs: Vec::new(),
            preserve_shaped_glyphs: false,
            glyph_artifact_line: None,
            layout_error: None,
            color: self.color,
            background_color: self.background_color,
            font: self.font.clone(),
            font_family: self.font_family.clone(),
            language: self.language.clone(),
            font_weight: self.font_weight,
            font_size: self.font_size,
            line_height: self.line_height,
            text_align: UiTextAlign::Left,
            text_direction,
            writing_mode: self.writing_mode,
            wrap: UiTextWrap::None,
            style: self.style.clone(),
            distance_field_mode: self.distance_field_mode,
            text_effects: self.text_effects.clone(),
            text_decorations: self.text_decorations.clone(),
            text_decoration_baseline: self.text_decoration_baseline,
            clip_transform: self.clip_transform,
        }
    }
}
