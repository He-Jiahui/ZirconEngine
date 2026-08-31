use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextDirection, UiTextWritingMode};

use crate::core::framework::text::{TextFontFaceHandle, TextGlyph};
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::ui::render::{
    ScreenSpaceUiShapedGlyph, ScreenSpaceUiTextBatch,
};
use crate::graphics::scene::scene_renderer::ui::sdf_render::resolved_horizontal_shaped_glyph_advances;
use crate::graphics::scene::scene_renderer::ui::text_pixel_snap::text_frame_device_origin;
use crate::text::atlas::{
    GlyphAtlasFormat, GlyphHintingMode, GlyphRasterKey, GlyphRasterRequest, GlyphSmoothingMode,
    SyntheticGlyphStyle, render_plan::GlyphAtlasScreenRect,
};
use crate::text::font::resolve_font_handle_batch;
use crate::text::native_bitmap_atlas::{NativeBitmapAtlasGlyph, NativeBitmapAtlasGlyphRun};

use super::font_id_report::{ScreenSpaceUiTextFontIdReport, accumulate_resolved_glyph_faces};

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer::ui) struct NativeBitmapAtlasGlyphRunProjection {
    pub(in crate::graphics::scene::scene_renderer::ui) glyph_runs: Vec<NativeBitmapAtlasGlyphRun>,
    pub(in crate::graphics::scene::scene_renderer::ui) font_ids: ScreenSpaceUiTextFontIdReport,
}

pub(in crate::graphics::scene::scene_renderer::ui) fn native_bitmap_atlas_glyph_runs(
    viewport_size: UVec2,
    texts: &[ScreenSpaceUiTextBatch],
) -> NativeBitmapAtlasGlyphRunProjection {
    let viewport = UiFrame::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let mut glyph_runs = Vec::with_capacity(texts.len());
    let mut font_ids = ScreenSpaceUiTextFontIdReport::default();
    for text in texts {
        if let Some(glyph_run) = native_bitmap_atlas_glyph_run(viewport, text, &mut font_ids) {
            glyph_runs.push(glyph_run);
        }
    }
    NativeBitmapAtlasGlyphRunProjection {
        glyph_runs,
        font_ids,
    }
}

fn native_bitmap_atlas_glyph_run(
    viewport: UiFrame,
    text: &ScreenSpaceUiTextBatch,
    font_ids: &mut ScreenSpaceUiTextFontIdReport,
) -> Option<NativeBitmapAtlasGlyphRun> {
    if !matches!(text.writing_mode, UiTextWritingMode::HorizontalTb) {
        return None;
    }
    let bounds = text
        .clip_frame
        .unwrap_or(viewport)
        .intersection(viewport)
        .map(ui_frame_to_glyph_atlas_rect)?;
    if let Some(artifact_glyphs) = text
        .glyph_artifact_line
        .as_ref()
        .and_then(|line| line.glyphs())
    {
        return native_bitmap_atlas_glyph_run_from_inputs(
            bounds,
            text,
            font_ids,
            artifact_glyphs,
            |_, glyph| glyph.advance,
        );
    }
    let glyph_advances = resolved_horizontal_shaped_glyph_advances(text);
    native_bitmap_atlas_glyph_run_from_inputs(
        bounds,
        text,
        font_ids,
        text.shaped_glyphs.as_slice(),
        |index, _| glyph_advances.get(index).copied().unwrap_or_default(),
    )
}

fn native_bitmap_atlas_glyph_run_from_inputs<Input>(
    bounds: GlyphAtlasScreenRect,
    text: &ScreenSpaceUiTextBatch,
    font_ids: &mut ScreenSpaceUiTextFontIdReport,
    inputs: &[Input],
    advance_at: impl Fn(usize, &Input) -> f32,
) -> Option<NativeBitmapAtlasGlyphRun>
where
    Input: NativeBitmapAtlasGlyphInput,
{
    if inputs.is_empty() {
        return None;
    }
    let handles = resolve_font_handle_batch(
        &inputs
            .iter()
            .map(|input| input.font_handles())
            .collect::<Vec<_>>(),
    );
    if handles.len() != inputs.len() {
        return None;
    }
    accumulate_resolved_glyph_faces(font_ids, handles.iter().map(|(face, _)| *face));
    let text_width = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| sanitized_non_negative(advance_at(index, input)))
        .sum::<f32>();
    let positioned_frame = text_frame_device_origin(text.frame);
    let baseline_y = native_bitmap_atlas_baseline(text, positioned_frame);
    let mut cursor_x = aligned_text_start_x(text, text_width);
    let mut glyphs = Vec::with_capacity(inputs.len());

    for (index, (input, (_, instance))) in inputs.iter().zip(handles).enumerate() {
        let advance = sanitized_non_negative(advance_at(index, input));
        let screen_x = cursor_x + sanitized_finite(input.offset_x());
        if input.requires_atlas_slot() {
            if let Some(instance) = instance {
                let mut raster_key = GlyphRasterKey::from_request(GlyphRasterRequest {
                    face: instance,
                    glyph_id: input.glyph_id(),
                    logical_px: text.font_size,
                    scale_factor: text.raster_scale,
                    screen_x,
                    snap_to_pixel: false,
                    format: GlyphAtlasFormat::AlphaMask,
                    hinting: GlyphHintingMode::Full,
                    smoothing: GlyphSmoothingMode::Grayscale,
                    synthetic: SyntheticGlyphStyle {
                        bold: false,
                        oblique: text.style.emphasis,
                    },
                });
                raster_key.vertical_subpixel_bin = vertical_subpixel_bin(baseline_y);
                glyphs.push(NativeBitmapAtlasGlyph {
                    raster_key,
                    screen_x,
                    baseline_y: baseline_y + sanitized_finite(input.offset_y()),
                    placeholder_rect: GlyphAtlasScreenRect::new(
                        screen_x,
                        baseline_y - text.font_size.max(1.0),
                        advance.max(1.0),
                        text.line_height.max(1.0),
                    ),
                    foreground_color: text.color,
                    background_color: text.background_color,
                });
            }
        }
        cursor_x += advance;
    }

    (!glyphs.is_empty()).then(|| NativeBitmapAtlasGlyphRun::new(bounds, glyphs))
}

trait NativeBitmapAtlasGlyphInput {
    fn glyph_id(&self) -> u32;
    fn font_handles(&self) -> (Option<TextFontFaceHandle>, Option<TextFontFaceHandle>);
    fn offset_x(&self) -> f32;
    fn offset_y(&self) -> f32;
    fn requires_atlas_slot(&self) -> bool;
}

impl NativeBitmapAtlasGlyphInput for ScreenSpaceUiShapedGlyph {
    fn glyph_id(&self) -> u32 {
        self.glyph_id
    }

    fn font_handles(&self) -> (Option<TextFontFaceHandle>, Option<TextFontFaceHandle>) {
        (self.font_id, self.font_instance_id)
    }

    fn offset_x(&self) -> f32 {
        self.offset_x
    }

    fn offset_y(&self) -> f32 {
        self.offset_y
    }

    fn requires_atlas_slot(&self) -> bool {
        self.requires_atlas_slot
    }
}

impl NativeBitmapAtlasGlyphInput for TextGlyph {
    fn glyph_id(&self) -> u32 {
        self.glyph_id
    }

    fn font_handles(&self) -> (Option<TextFontFaceHandle>, Option<TextFontFaceHandle>) {
        (self.font_face, self.font_instance)
    }

    fn offset_x(&self) -> f32 {
        self.offset[0]
    }

    fn offset_y(&self) -> f32 {
        self.offset[1]
    }

    fn requires_atlas_slot(&self) -> bool {
        self.requires_rasterization
    }
}

fn native_bitmap_atlas_baseline(text: &ScreenSpaceUiTextBatch, positioned_frame: UiFrame) -> f32 {
    if let Some(baseline) = text
        .glyph_artifact_line
        .as_ref()
        .and_then(|line| line.layout_baseline())
    {
        return positioned_frame.y + baseline;
    }
    positioned_frame.y
        + native_bitmap_atlas_relative_baseline(
            text.text_decoration_baseline,
            text.frame.y,
            text.line_height,
            text.font_size,
        )
}

fn native_bitmap_atlas_relative_baseline(
    decoration_baseline: Option<f32>,
    logical_frame_y: f32,
    line_height: f32,
    font_size: f32,
) -> f32 {
    decoration_baseline
        .map(|baseline| baseline - logical_frame_y)
        .filter(|baseline| baseline.is_finite())
        .unwrap_or_else(|| {
            (line_height.max(font_size) - font_size.max(1.0)).max(0.0) * 0.5
                + font_size.max(1.0) * 0.8
        })
}

fn aligned_text_start_x(text: &ScreenSpaceUiTextBatch, text_width: f32) -> f32 {
    let positioned_frame = text_frame_device_origin(text.frame);
    aligned_device_text_start_x(
        positioned_frame,
        text.text_align,
        text.text_direction,
        text_width,
    )
}

fn aligned_device_text_start_x(
    positioned_frame: UiFrame,
    text_align: UiTextAlign,
    text_direction: UiTextDirection,
    text_width: f32,
) -> f32 {
    let free_width = (positioned_frame.width - sanitized_non_negative(text_width)).max(0.0);
    let offset = match text_align {
        UiTextAlign::Left => 0.0,
        UiTextAlign::Center => free_width * 0.5,
        UiTextAlign::Right => free_width,
        UiTextAlign::Start if matches!(text_direction, UiTextDirection::RightToLeft) => free_width,
        UiTextAlign::Start => 0.0,
        UiTextAlign::End if matches!(text_direction, UiTextDirection::RightToLeft) => 0.0,
        UiTextAlign::End => free_width,
        UiTextAlign::Justify => 0.0,
    };
    positioned_frame.x + offset
}

fn ui_frame_to_glyph_atlas_rect(frame: UiFrame) -> GlyphAtlasScreenRect {
    GlyphAtlasScreenRect::new(
        frame.x,
        frame.y,
        frame.width.max(0.0),
        frame.height.max(0.0),
    )
}

fn vertical_subpixel_bin(value: f32) -> u8 {
    if !value.is_finite() {
        return 0;
    }
    ((value.rem_euclid(1.0) * 4.0).floor() as u8).min(3)
}

fn sanitized_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn sanitized_finite(value: f32) -> f32 {
    value.is_finite().then_some(value).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::{
        UiFrame, UiTextAlign, UiTextDirection, aligned_device_text_start_x,
        native_bitmap_atlas_relative_baseline, vertical_subpixel_bin,
    };

    #[test]
    fn native_bitmap_glyph_run_retains_four_vertical_raster_phases() {
        assert_eq!(vertical_subpixel_bin(12.0), 0);
        assert_eq!(vertical_subpixel_bin(12.26), 1);
        assert_eq!(vertical_subpixel_bin(12.51), 2);
        assert_eq!(vertical_subpixel_bin(12.76), 3);
    }

    #[test]
    fn native_bitmap_glyph_run_keeps_logical_start_end_alignment_after_device_origin_snap() {
        let frame = UiFrame::new(10.0, 7.0, 100.0, 24.0);

        assert_eq!(
            aligned_device_text_start_x(
                frame,
                UiTextAlign::Start,
                UiTextDirection::LeftToRight,
                30.0,
            ),
            10.0
        );
        assert_eq!(
            aligned_device_text_start_x(
                frame,
                UiTextAlign::Center,
                UiTextDirection::LeftToRight,
                f32::NAN,
            ),
            60.0
        );
        assert_eq!(
            aligned_device_text_start_x(
                frame,
                UiTextAlign::Start,
                UiTextDirection::RightToLeft,
                30.0,
            ),
            80.0
        );
        assert_eq!(
            aligned_device_text_start_x(
                frame,
                UiTextAlign::End,
                UiTextDirection::LeftToRight,
                30.0,
            ),
            80.0
        );
        assert_eq!(
            aligned_device_text_start_x(
                frame,
                UiTextAlign::End,
                UiTextDirection::RightToLeft,
                30.0,
            ),
            10.0
        );
    }

    #[test]
    fn native_bitmap_glyph_run_prefers_layout_decoration_baseline_and_sanitizes_invalid_input() {
        assert_eq!(
            native_bitmap_atlas_relative_baseline(Some(35.0), 20.0, 30.0, 20.0),
            15.0
        );
        assert_eq!(
            native_bitmap_atlas_relative_baseline(Some(f32::NAN), 20.0, 30.0, 20.0),
            21.0
        );
    }
}
