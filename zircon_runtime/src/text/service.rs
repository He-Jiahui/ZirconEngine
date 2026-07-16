use crate::core::framework::text::{
    TextDirection, TextFontRequest, TextGlyph, TextGlyphFlags, TextGlyphRotation, TextLayoutError,
    TextLayoutMetrics, TextLayoutService, TextRenderMode, TextShapeRequest, TextShapeResult,
    TextShapeRun, TextWritingMode,
};

use super::font::{
    register_font_face_handle, register_font_instance_handle, shared_font_database_generation,
    FontDatabase,
};
use super::shaping::{
    fallback_text_spans, resolve_bidi_base_direction, shape_text, FallbackTextSpan,
};
use super::{
    BackendShapeRequest, ShapedGlyph, ShapedGlyphRotation, ShapedGlyphRun, TextRange, TextStyle,
    VerticalMode,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct SharedTextLayoutService;

pub fn shared_text_layout_service() -> &'static dyn TextLayoutService {
    static SERVICE: SharedTextLayoutService = SharedTextLayoutService;
    &SERVICE
}

pub(crate) fn fallback_spans_for_request(
    request: TextShapeRequest<'_>,
    font_database: &FontDatabase,
) -> Vec<FallbackTextSpan> {
    let style = backend_style(&request);
    let direction = resolve_bidi_base_direction(request.text, request.direction);
    fallback_text_spans(
        request.text,
        BackendShapeRequest::horizontal(
            request.text,
            &style,
            direction,
            TextRange {
                start: 0,
                end: request.text.len(),
            },
        )
        .with_language(request.language),
        font_database,
    )
}

impl TextLayoutService for SharedTextLayoutService {
    fn resolve_render_mode(&self, request: &TextFontRequest<'_>) -> TextRenderMode {
        match request.render_mode {
            TextRenderMode::Auto => TextRenderMode::Native,
            mode => mode,
        }
    }

    fn resolve_direction(&self, text: &str, requested: TextDirection) -> TextDirection {
        resolve_bidi_base_direction(text, requested)
    }

    fn shape(&self, request: TextShapeRequest<'_>) -> Result<TextShapeResult, TextLayoutError> {
        validate_shape_request(&request)?;
        let style = backend_style(&request);
        let source_range = TextRange {
            start: 0,
            end: request.text.len(),
        };
        let resolved_direction = self.resolve_direction(request.text, request.direction);
        let backend_request = match request.writing_mode {
            TextWritingMode::HorizontalTopToBottom => BackendShapeRequest::horizontal(
                request.text,
                &style,
                resolved_direction,
                source_range,
            )
            .with_kerning(request.include_kerning),
            TextWritingMode::VerticalRightToLeft => BackendShapeRequest::vertical(
                request.text,
                &style,
                resolved_direction,
                source_range,
                VerticalMode::Mixed,
            )
            .with_kerning(request.include_kerning),
        }
        .with_language(request.language);
        loop {
            let generation = shared_font_database_generation();
            let shaped = shape_text(backend_request);
            if generation != shared_font_database_generation() {
                continue;
            }
            let projected = project_shape_result(shaped, resolved_direction, generation);
            if generation == shared_font_database_generation() {
                return Ok(projected);
            }
        }
    }
}

fn validate_shape_request(request: &TextShapeRequest<'_>) -> Result<(), TextLayoutError> {
    if !request.font.size.is_finite() || request.font.size <= 0.0 {
        return Err(TextLayoutError::InvalidFontSize);
    }
    if request
        .language
        .is_some_and(|language| language.trim().is_empty())
    {
        return Err(TextLayoutError::InvalidLanguage);
    }
    Ok(())
}

fn backend_style(request: &TextShapeRequest<'_>) -> TextStyle {
    TextStyle {
        font: request.font.asset.map(str::to_string),
        font_family: request
            .font
            .families
            .first()
            .map(|family| (*family).to_string()),
        language: request.language.map(str::to_string),
        font_weight: request.font.weight,
        font_size: request.font.size,
        line_height: request.line_height,
        tab_size: request.tab_size,
        ..TextStyle::default()
    }
}

fn project_shape_result(
    shaped: ShapedGlyphRun,
    resolved_direction: TextDirection,
    font_database_generation: u64,
) -> TextShapeResult {
    let metrics = TextLayoutMetrics {
        width: shaped.measured_width,
        height: shaped.measured_height,
        ascent: shaped.lines.first().map_or(0.0, |line| line.baseline),
        descent: shaped
            .lines
            .first()
            .map_or(0.0, |line| (line.line_height - line.baseline).max(0.0)),
        line_gap: 0.0,
        baseline: shaped.lines.first().map_or(0.0, |line| line.baseline),
    };
    let runs = shaped
        .lines
        .into_iter()
        .map(|line| TextShapeRun {
            source_range: line.source_range.start..line.source_range.end,
            direction: line
                .glyphs
                .first()
                .map_or(resolved_direction, |glyph| glyph.direction),
            glyphs: line
                .glyphs
                .into_iter()
                .map(|glyph| project_glyph(glyph, font_database_generation))
                .collect(),
        })
        .collect();
    TextShapeResult {
        runs,
        metrics,
        resolved_direction,
    }
}

fn project_glyph(glyph: ShapedGlyph, font_database_generation: u64) -> TextGlyph {
    TextGlyph {
        glyph_id: glyph.glyph_id,
        source_range: glyph.source_range.start..glyph.source_range.end,
        visual_range: glyph.visual_range.start..glyph.visual_range.end,
        advance: glyph.advance,
        position: [glyph.x, glyph.y],
        offset: [glyph.offset_x, glyph.offset_y],
        font_face: glyph
            .font_id
            .and_then(|face| register_font_face_handle(face, font_database_generation)),
        font_instance: glyph
            .font_instance_id
            .and_then(|instance| register_font_instance_handle(instance, font_database_generation)),
        rotation: match glyph.rotation {
            ShapedGlyphRotation::None => TextGlyphRotation::None,
            ShapedGlyphRotation::Cw90 => TextGlyphRotation::Clockwise90,
        },
        bidi_level: glyph.bidi_level,
        flags: TextGlyphFlags {
            cluster_start: glyph.cluster_flags.cluster_start,
            right_to_left: glyph.cluster_flags.rtl,
            whitespace: glyph.cluster_flags.whitespace,
            space: glyph.cluster_flags.space,
            tab: glyph.cluster_flags.tab,
            mandatory_break: glyph.cluster_flags.mandatory_break,
            soft_break: glyph.cluster_flags.soft_break,
            virtual_glyph: glyph.cluster_flags.virtual_glyph,
        },
        requires_rasterization: !glyph.cluster_flags.virtual_glyph
            && !glyph.cluster_flags.whitespace
            && !glyph.cluster_flags.space
            && !glyph.cluster_flags.tab,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_text_layout_service_shapes_through_neutral_contract() {
        let font = TextFontRequest {
            size: 16.0,
            ..TextFontRequest::default()
        };
        let result = shared_text_layout_service()
            .shape(TextShapeRequest::new("Zircon", font))
            .expect("production text service should shape a neutral request");

        assert!(!result.runs.is_empty());
        assert!(result.metrics.width > 0.0);
        assert!(result.runs.iter().any(|run| !run.glyphs.is_empty()));
    }
}
