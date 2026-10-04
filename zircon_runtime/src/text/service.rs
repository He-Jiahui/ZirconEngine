#[cfg(test)]
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use crate::core::framework::text::{
    TextDirection, TextFontRequest, TextLayoutError, TextLayoutService, TextRenderMode,
    TextShapeRequest, TextShapeResult, TextWritingMode,
};

use super::font::{
    shared_font_collection_service, FontCollectionService, FontCollectionSnapshot, FontDatabase,
};
use super::model::TextShapingRequestDiagnostics;
use super::shaping::{
    fallback_text_spans, resolve_bidi_base_direction,
    shape_text_with_diagnostics_in_font_collection, FallbackTextSpan, ParagraphTextAnalysis,
    TextShapingCompletion, TextShapingFailure,
};
use super::{
    BackendShapeRequest, OpenTypeFeature, ShapedGlyphRun, TextRange, TextStyle, VerticalMode,
};

mod projection;

pub(crate) use projection::project_glyph;
use projection::project_shape_result;

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
) -> Result<Vec<FallbackTextSpan>, TextLayoutError> {
    let style = backend_style(&request);
    let features = backend_features(&request);
    let backend_request = fallback_backend_request(&request, &style, features.as_slice());
    let canonical_request = backend_request.canonicalized()?;
    let backend_request = canonical_request.request();
    let analysis = ParagraphTextAnalysis::for_snapshot(
        backend_request.text,
        backend_request.explicit_language_script(),
        backend_request.unicode_data_snapshot(),
    );
    fallback_text_spans(
        backend_request.text,
        backend_request,
        font_database,
        &analysis,
    )
    .map_err(|_| TextLayoutError::FontUnavailable)
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
        shape_text_request_in_font_collection(request, &shared_font_collection_service())
    }
}

pub(crate) fn shape_text_request_in_font_collection(
    request: TextShapeRequest<'_>,
    font_collection: &Arc<FontCollectionService>,
) -> Result<TextShapeResult, TextLayoutError> {
    let style = backend_style(&request);
    let features = backend_features(&request);
    let source_range = TextRange {
        start: 0,
        end: request.text.len(),
    };
    let backend_request = match request.writing_mode {
        TextWritingMode::HorizontalTopToBottom => {
            BackendShapeRequest::horizontal(request.text, &style, request.direction, source_range)
                .with_kerning(request.include_kerning)
        }
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
    shape_backend_request_at_stable_generation_in_font_collection(
        backend_request,
        font_collection,
        |shaped, font_collection, _, _| {
            let resolved_direction = shaped.direction;
            project_shape_result(shaped, resolved_direction, font_collection)
        },
    )
    .map_err(TextShapingFailure::into_error)
}

pub(super) fn shape_backend_request_at_stable_generation<Projected>(
    request: BackendShapeRequest<'_>,
    project: impl FnMut(
        ShapedGlyphRun,
        &FontCollectionSnapshot,
        u64,
        TextShapingRequestDiagnostics,
    ) -> Projected,
) -> Result<Projected, TextShapingFailure> {
    let font_collection = shared_font_collection_service();
    shape_backend_request_at_stable_generation_in_font_collection(
        request,
        &font_collection,
        project,
    )
}

pub(super) fn shape_backend_request_at_stable_generation_in_font_collection<Projected>(
    request: BackendShapeRequest<'_>,
    font_collection: &Arc<FontCollectionService>,
    project: impl FnMut(
        ShapedGlyphRun,
        &FontCollectionSnapshot,
        u64,
        TextShapingRequestDiagnostics,
    ) -> Projected,
) -> Result<Projected, TextShapingFailure> {
    let canonical_request = request.canonicalized().map_err(TextShapingFailure::from)?;
    let request = canonical_request.request();
    validate_backend_shape_request(&request).map_err(TextShapingFailure::from)?;
    shape_for_stable_font_generation(
        || {
            let snapshot = font_collection.collection_snapshot();
            (snapshot.generation(), snapshot)
        },
        || font_collection.generation(),
        |snapshot: &FontCollectionSnapshot| {
            shape_text_with_diagnostics_in_font_collection(request, snapshot)
        },
        project,
    )
}

fn shape_for_stable_font_generation<Snapshot, Shaped, Projected>(
    mut snapshot: impl FnMut() -> (u64, Snapshot),
    mut generation: impl FnMut() -> u64,
    mut shape: impl FnMut(&Snapshot) -> Result<TextShapingCompletion<Shaped>, TextShapingFailure>,
    mut project: impl FnMut(Shaped, &Snapshot, u64, TextShapingRequestDiagnostics) -> Projected,
) -> Result<Projected, TextShapingFailure> {
    let metrics = generation_retry_metrics();
    let mut request_diagnostics = TextShapingRequestDiagnostics::EMPTY;
    for _ in 0..MAX_FONT_GENERATION_SHAPE_ATTEMPTS {
        let (shape_generation, font_snapshot) = snapshot();
        let completion = match shape(&font_snapshot) {
            Ok(completion) => completion,
            Err(failure) => {
                let mut failure_diagnostics = failure.request_diagnostics();
                failure_diagnostics.shaping_attempt_count =
                    failure_diagnostics.shaping_attempt_count.saturating_add(1);
                request_diagnostics.merge(failure_diagnostics);
                return Err(failure.replace_request_diagnostics(request_diagnostics));
            }
        };
        let (shaped, mut attempt_diagnostics) = completion.into_parts();
        attempt_diagnostics.shaping_attempt_count =
            attempt_diagnostics.shaping_attempt_count.saturating_add(1);
        request_diagnostics.merge(attempt_diagnostics);
        metrics
            .canonical_shape_count
            .fetch_add(1, Ordering::Relaxed);
        if shape_generation != generation() {
            metrics.restart_count.fetch_add(1, Ordering::Relaxed);
            request_diagnostics.font_generation_restart_count = request_diagnostics
                .font_generation_restart_count
                .saturating_add(1);
            continue;
        }
        let projected = project(
            shaped,
            &font_snapshot,
            shape_generation,
            request_diagnostics,
        );
        if shape_generation == generation() {
            return Ok(projected);
        }
        metrics.restart_count.fetch_add(1, Ordering::Relaxed);
        request_diagnostics.font_generation_restart_count = request_diagnostics
            .font_generation_restart_count
            .saturating_add(1);
    }
    metrics.deferred_count.fetch_add(1, Ordering::Relaxed);
    Err(TextShapingFailure::font_generation_changed().with_request_diagnostics(request_diagnostics))
}

fn validate_backend_shape_request(
    request: &BackendShapeRequest<'_>,
) -> Result<(), TextLayoutError> {
    if !request.style.font_size.is_finite() || request.style.font_size <= 0.0 {
        return Err(TextLayoutError::InvalidFontSize);
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
        italic: request.font.italic,
        font_size: request.font.size,
        line_height: request.line_height,
        tab_size: request.tab_size,
        ..TextStyle::default()
    }
}

#[cfg(test)]
#[path = "tests/service.rs"]
mod tests;
