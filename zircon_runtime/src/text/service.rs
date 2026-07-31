use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use crate::core::framework::text::{
    TextDirection, TextFontRequest, TextGlyph, TextGlyphFlags, TextGlyphRotation, TextLayoutError,
    TextLayoutMetrics, TextLayoutService, TextRenderMode, TextShapeRequest, TextShapeResult,
    TextShapeRun, TextWritingMode,
};

use super::font::{register_font_handle_batch, shared_font_database_generation, FontDatabase};
use super::shaping::{
    fallback_text_spans, resolve_bidi_base_direction, shape_text, FallbackTextSpan,
};
use super::{
    BackendShapeRequest, ShapedGlyph, ShapedGlyphRotation, ShapedGlyphRun, TextRange, TextStyle,
    VerticalMode,
};

// Bound caller-thread shaping during font reload storms; the next frame retries.
const MAX_FONT_GENERATION_SHAPE_ATTEMPTS: usize = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextLayoutGenerationRetryReport {
    pub canonical_shape_count: u64,
    pub restart_count: u64,
    pub deferred_count: u64,
}

#[derive(Default)]
struct TextLayoutGenerationRetryMetrics {
    canonical_shape_count: AtomicU64,
    restart_count: AtomicU64,
    deferred_count: AtomicU64,
}

impl TextLayoutGenerationRetryMetrics {
    fn report(&self) -> TextLayoutGenerationRetryReport {
        TextLayoutGenerationRetryReport {
            canonical_shape_count: self.canonical_shape_count.load(Ordering::Relaxed),
            restart_count: self.restart_count.load(Ordering::Relaxed),
            deferred_count: self.deferred_count.load(Ordering::Relaxed),
        }
    }
}

fn generation_retry_metrics() -> &'static TextLayoutGenerationRetryMetrics {
    static METRICS: OnceLock<TextLayoutGenerationRetryMetrics> = OnceLock::new();
    METRICS.get_or_init(TextLayoutGenerationRetryMetrics::default)
}

pub(crate) fn shared_text_layout_generation_retry_report() -> TextLayoutGenerationRetryReport {
    generation_retry_metrics().report()
}

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
        shape_backend_request_at_stable_generation(backend_request, |shaped, generation| {
            project_shape_result(shaped, resolved_direction, generation)
        })
    }
}

pub(super) fn shape_backend_request_at_stable_generation<Projected>(
    request: BackendShapeRequest<'_>,
    project: impl FnMut(ShapedGlyphRun, u64) -> Projected,
) -> Result<Projected, TextLayoutError> {
    validate_backend_shape_request(&request)?;
    shape_for_stable_font_generation(
        shared_font_database_generation,
        || shape_text(request),
        project,
    )
}

fn shape_for_stable_font_generation<Shaped, Projected>(
    mut generation: impl FnMut() -> u64,
    mut shape: impl FnMut() -> Shaped,
    mut project: impl FnMut(Shaped, u64) -> Projected,
) -> Result<Projected, TextLayoutError> {
    let metrics = generation_retry_metrics();
    for _ in 0..MAX_FONT_GENERATION_SHAPE_ATTEMPTS {
        let shape_generation = generation();
        let shaped = shape();
        metrics
            .canonical_shape_count
            .fetch_add(1, Ordering::Relaxed);
        if shape_generation != generation() {
            metrics.restart_count.fetch_add(1, Ordering::Relaxed);
            continue;
        }
        let projected = project(shaped, shape_generation);
        if shape_generation == generation() {
            return Ok(projected);
        }
        metrics.restart_count.fetch_add(1, Ordering::Relaxed);
    }
    metrics.deferred_count.fetch_add(1, Ordering::Relaxed);
    Err(TextLayoutError::FontGenerationChanged)
}

fn validate_backend_shape_request(
    request: &BackendShapeRequest<'_>,
) -> Result<(), TextLayoutError> {
    if !request.style.font_size.is_finite() || request.style.font_size <= 0.0 {
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
    let font_handles = register_font_handle_batch(
        &shaped
            .lines
            .iter()
            .flat_map(|line| {
                line.glyphs
                    .iter()
                    .map(|glyph| (glyph.font_id, glyph.font_instance_id))
            })
            .collect::<Vec<_>>(),
        font_database_generation,
    );
    let mut font_handles = font_handles.into_iter();
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
                .map(|glyph| {
                    // A malformed projection batch must not terminate text rendering. Missing
                    // handles take the existing fail-closed raster path for this glyph.
                    project_glyph(glyph, font_handles.next().unwrap_or_default())
                })
                .collect(),
        })
        .collect();
    TextShapeResult {
        runs,
        metrics,
        resolved_direction,
    }
}

fn project_glyph(
    glyph: ShapedGlyph,
    (font_face, font_instance): (
        Option<crate::core::framework::text::TextFontFaceHandle>,
        Option<crate::core::framework::text::TextFontFaceHandle>,
    ),
) -> TextGlyph {
    TextGlyph {
        glyph_id: glyph.glyph_id,
        source_range: glyph.source_range.start..glyph.source_range.end,
        visual_range: glyph.visual_range.start..glyph.visual_range.end,
        advance: glyph.advance,
        position: [glyph.x, glyph.y],
        offset: [glyph.offset_x, glyph.offset_y],
        font_face,
        font_instance,
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
    use crate::text::font::{font_handle_registry_report, shared_font_database_test_serial_guard};

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

    #[test]
    fn service_projects_a_run_with_one_font_handle_batch() {
        let _shared_font_database = shared_font_database_test_serial_guard();
        let before = font_handle_registry_report();
        let font = TextFontRequest {
            size: 16.0,
            ..TextFontRequest::default()
        };

        let result = shared_text_layout_service()
            .shape(TextShapeRequest::new("Batch projection", font))
            .expect("production text service should project a shaped run");
        let after = font_handle_registry_report();
        let glyph_count = result
            .runs
            .iter()
            .map(|run| run.glyphs.len())
            .sum::<usize>();

        assert!(glyph_count > 1);
        assert_eq!(
            after.registration_batch_count,
            before.registration_batch_count + 1
        );
        assert_eq!(
            after.registration_lock_acquire_count,
            before.registration_lock_acquire_count + 1
        );
        assert!(
            after.registration_unique_pair_count - before.registration_unique_pair_count
                <= glyph_count as u64
        );
    }

    #[test]
    fn generation_retry_is_bounded_and_defers_after_the_budget() {
        let before = shared_text_layout_generation_retry_report();
        let mut generations = [10_u64, 11, 12, 13].into_iter();
        let mut shaped_count = 0;

        let result = shape_for_stable_font_generation(
            || generations.next().expect("generation probe"),
            || {
                shaped_count += 1;
                shaped_count
            },
            |shaped_count, _| shaped_count,
        );
        let after = shared_text_layout_generation_retry_report();

        assert_eq!(result, Err(TextLayoutError::FontGenerationChanged));
        assert_eq!(shaped_count, MAX_FONT_GENERATION_SHAPE_ATTEMPTS);
        assert!(
            after.canonical_shape_count
                >= before.canonical_shape_count + MAX_FONT_GENERATION_SHAPE_ATTEMPTS as u64
        );
        assert!(after.restart_count >= before.restart_count + 2);
        assert!(after.deferred_count >= before.deferred_count + 1);
    }
}
