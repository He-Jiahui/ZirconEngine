#[cfg(test)]
use std::cell::Cell;
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
    BackendShapeRequest, OpenTypeFeature, ShapedGlyph, ShapedGlyphRotation, ShapedGlyphRun,
    TextRange, TextStyle, VerticalMode,
};

// Bound caller-thread shaping during font reload storms; the next frame retries.
const MAX_FONT_GENERATION_SHAPE_ATTEMPTS: usize = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextLayoutGenerationRetryReport {
    pub canonical_shape_count: u64,
    pub neutral_projection_count: u64,
    pub neutral_projection_glyph_count: u64,
    pub neutral_projection_bytes: u64,
    pub restart_count: u64,
    pub deferred_count: u64,
}

#[derive(Default)]
struct TextLayoutGenerationRetryMetrics {
    canonical_shape_count: AtomicU64,
    neutral_projection_count: AtomicU64,
    neutral_projection_glyph_count: AtomicU64,
    neutral_projection_bytes: AtomicU64,
    restart_count: AtomicU64,
    deferred_count: AtomicU64,
}

impl TextLayoutGenerationRetryMetrics {
    fn report(&self) -> TextLayoutGenerationRetryReport {
        TextLayoutGenerationRetryReport {
            canonical_shape_count: self.canonical_shape_count.load(Ordering::Relaxed),
            neutral_projection_count: self.neutral_projection_count.load(Ordering::Relaxed),
            neutral_projection_glyph_count: self
                .neutral_projection_glyph_count
                .load(Ordering::Relaxed),
            neutral_projection_bytes: self.neutral_projection_bytes.load(Ordering::Relaxed),
            restart_count: self.restart_count.load(Ordering::Relaxed),
            deferred_count: self.deferred_count.load(Ordering::Relaxed),
        }
    }
}

fn generation_retry_metrics() -> &'static TextLayoutGenerationRetryMetrics {
    static METRICS: OnceLock<TextLayoutGenerationRetryMetrics> = OnceLock::new();
    METRICS.get_or_init(TextLayoutGenerationRetryMetrics::default)
}

#[cfg(test)]
thread_local! {
    static CURRENT_THREAD_NEUTRAL_PROJECTION_COUNT: Cell<u64> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn current_thread_neutral_projection_count() -> u64 {
    CURRENT_THREAD_NEUTRAL_PROJECTION_COUNT.get()
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
    let features = backend_features(&request);
    fallback_text_spans(
        request.text,
        fallback_backend_request(&request, &style, features.as_slice()),
        font_database,
    )
}

fn fallback_backend_request<'a>(
    request: &'a TextShapeRequest<'a>,
    style: &'a TextStyle,
    features: &'a [OpenTypeFeature],
) -> BackendShapeRequest<'a> {
    BackendShapeRequest::horizontal(
        request.text,
        style,
        request.direction,
        TextRange {
            start: 0,
            end: request.text.len(),
        },
    )
    .with_features(features)
    .with_language(request.language)
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
        let features = backend_features(&request);
        let source_range = TextRange {
            start: 0,
            end: request.text.len(),
        };
        let backend_request = match request.writing_mode {
            TextWritingMode::HorizontalTopToBottom => BackendShapeRequest::horizontal(
                request.text,
                &style,
                request.direction,
                source_range,
            )
            .with_kerning(request.include_kerning),
            TextWritingMode::VerticalRightToLeft => BackendShapeRequest::vertical(
                request.text,
                &style,
                request.direction,
                source_range,
                VerticalMode::Mixed,
            )
            .with_kerning(request.include_kerning),
        }
        .with_features(features.as_slice())
        .with_language(request.language);
        shape_backend_request_at_stable_generation(backend_request, |shaped, generation| {
            let resolved_direction = shaped.direction;
            project_shape_result(shaped, resolved_direction, generation)
        })
    }
}

pub(super) fn shape_backend_request_at_stable_generation<Projected>(
    request: BackendShapeRequest<'_>,
    project: impl FnMut(ShapedGlyphRun, u64) -> Projected,
) -> Result<Projected, TextLayoutError> {
    let canonical_request = request.canonicalized();
    let request = canonical_request.request();
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

fn backend_features(request: &TextShapeRequest<'_>) -> Vec<OpenTypeFeature> {
    // Keep framework DTOs independent from the implementation-owned, normalized cache key type.
    request
        .features
        .iter()
        .map(|feature| OpenTypeFeature::new(feature.tag, feature.value))
        .collect()
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
    record_neutral_projection(&shaped);
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
                    project_glyph(&glyph, font_handles.next().unwrap_or_default())
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

fn record_neutral_projection(shaped: &ShapedGlyphRun) {
    #[cfg(test)]
    CURRENT_THREAD_NEUTRAL_PROJECTION_COUNT.set(
        CURRENT_THREAD_NEUTRAL_PROJECTION_COUNT
            .get()
            .saturating_add(1),
    );
    let glyph_count = shaped
        .lines
        .iter()
        .map(|line| line.glyphs.len())
        .sum::<usize>();
    let projected_bytes = shaped
        .lines
        .len()
        .saturating_mul(std::mem::size_of::<TextShapeRun>())
        .saturating_add(glyph_count.saturating_mul(std::mem::size_of::<TextGlyph>()));
    let metrics = generation_retry_metrics();
    metrics
        .neutral_projection_count
        .fetch_add(1, Ordering::Relaxed);
    metrics
        .neutral_projection_glyph_count
        .fetch_add(glyph_count as u64, Ordering::Relaxed);
    metrics
        .neutral_projection_bytes
        .fetch_add(projected_bytes as u64, Ordering::Relaxed);
}

pub(super) fn project_glyph(
    glyph: &ShapedGlyph,
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
    use std::cell::Cell;
    use std::path::Path;

    use super::*;
    use crate::core::framework::text::TextOpenTypeFeature;
    use crate::text::font::{
        font_handle_registry_report, force_publish_shared_font_database,
        shared_font_database_snapshot, shared_font_database_test_serial_guard, FontDatabase,
    };

    struct SharedFontDatabaseRestore(FontDatabase);

    impl Drop for SharedFontDatabaseRestore {
        fn drop(&mut self) {
            force_publish_shared_font_database(&self.0);
        }
    }

    #[test]
    fn production_text_layout_service_shapes_through_neutral_contract() {
        let before = shared_text_layout_generation_retry_report();
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
        let after = shared_text_layout_generation_retry_report();
        let glyph_count = result
            .runs
            .iter()
            .map(|run| run.glyphs.len())
            .sum::<usize>() as u64;
        assert!(after.neutral_projection_count > before.neutral_projection_count);
        assert!(
            after.neutral_projection_glyph_count
                >= before.neutral_projection_glyph_count + glyph_count
        );
        assert!(after.neutral_projection_bytes > before.neutral_projection_bytes);
    }

    #[test]
    fn production_shape_resolves_auto_direction_from_the_canonical_run() {
        let font = TextFontRequest {
            size: 16.0,
            ..TextFontRequest::default()
        };

        let result = shared_text_layout_service()
            .shape(TextShapeRequest::new("مرحبا", font))
            .expect("automatic RTL request shapes");

        assert_eq!(result.resolved_direction, TextDirection::RightToLeft);
    }

    #[test]
    fn native_fallback_request_defers_auto_direction_to_canonical_shaping() {
        let request = TextShapeRequest::new("مرحبا", TextFontRequest::default());
        let style = backend_style(&request);
        let features = backend_features(&request);

        let backend = fallback_backend_request(&request, &style, features.as_slice());

        assert_eq!(backend.base_direction, TextDirection::Auto);
    }

    #[test]
    fn neutral_request_features_map_to_backend_shape_features() {
        let requested = [
            TextOpenTypeFeature::new(*b"liga", 0),
            TextOpenTypeFeature::new(*b"tnum", 1),
        ];
        let request =
            TextShapeRequest::new("0123", TextFontRequest::default()).with_features(&requested);

        assert_eq!(
            backend_features(&request),
            vec![
                OpenTypeFeature::new(*b"liga", 0),
                OpenTypeFeature::new(*b"tnum", 1),
            ]
        );
    }

    #[test]
    fn neutral_service_features_change_final_ligature_glyph_count() {
        let _shared_font_database = shared_font_database_test_serial_guard();
        let (_, original_database) = shared_font_database_snapshot();
        let _restore_database = SharedFontDatabaseRestore(original_database);
        let mut feature_database = FontDatabase::with_default_fallbacks();
        let font_source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
        feature_database
            .register_font_file(&font_source, Some("Fira Sans"), 0)
            .expect("register deterministic ligature fixture");
        force_publish_shared_font_database(&feature_database);

        let families = ["Fira Sans"];
        let font = TextFontRequest {
            families: &families,
            size: 24.0,
            ..TextFontRequest::default()
        };
        let requested = [TextOpenTypeFeature::new(*b"liga", 0)];
        let default_shape = shared_text_layout_service()
            .shape(TextShapeRequest::new("fi", font))
            .expect("default ligature request should shape");
        let disabled_ligature_shape = shared_text_layout_service()
            .shape(TextShapeRequest::new("fi", font).with_features(&requested))
            .expect("feature-bearing ligature request should shape");
        let default_glyph_count = default_shape
            .runs
            .iter()
            .map(|run| run.glyphs.len())
            .sum::<usize>();
        let disabled_ligature_glyph_count = disabled_ligature_shape
            .runs
            .iter()
            .map(|run| run.glyphs.len())
            .sum::<usize>();

        assert!(
            disabled_ligature_glyph_count > default_glyph_count,
            "liga=0 must reach SharedTextLayoutService::shape and suppress the Fira Sans fi ligature: default={default_glyph_count}, disabled={disabled_ligature_glyph_count}"
        );
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

    #[test]
    fn generation_retry_restarts_when_projection_observes_a_font_publish() {
        let generation = Cell::new(10_u64);
        let shape_count = Cell::new(0_u64);
        let projection_count = Cell::new(0_u64);

        let result = shape_for_stable_font_generation(
            || generation.get(),
            || {
                shape_count.set(shape_count.get() + 1);
                shape_count.get()
            },
            |shaped_count, _| {
                projection_count.set(projection_count.get() + 1);
                if projection_count.get() == 1 {
                    generation.set(11);
                }
                shaped_count
            },
        );

        assert_eq!(result, Ok(2));
        assert_eq!(shape_count.get(), 2);
        assert_eq!(projection_count.get(), 2);
    }
}
