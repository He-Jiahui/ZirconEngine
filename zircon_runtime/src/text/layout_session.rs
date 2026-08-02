use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, OnceLock};

use crate::core::framework::text::{TextDirection, TextLayoutError};
use crate::core::runtime::tasks::TaskPool;

use super::cache::{
    ShapedRunCache, ShapedRunCacheLookupKey, ShapedRunCacheReport,
    DEFAULT_SHAPED_RUN_CACHE_CAPACITY, DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
};
use super::parallel::shape_pool::{
    shape_paragraphs_with_cache, TextParallelShapeBatchReport, TextShapeParagraph,
};
use super::service::shape_backend_request_at_stable_generation;
use super::shaping::TextShapeRunProvider;
use super::{BackendShapeRequest, ShapedGlyphRun, TextRange, TextStyle, VerticalMode};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextLayoutFallbackReport {
    pub fallback_count: u64,
    pub generation_deferred_count: u64,
    pub invalid_font_size_count: u64,
    pub invalid_language_count: u64,
    pub other_error_count: u64,
}

impl TextLayoutFallbackReport {
    pub(crate) fn record(&mut self, error: &TextLayoutError) {
        if matches!(error, TextLayoutError::FontGenerationChanged) {
            self.record_generation_deferred();
            return;
        }
        self.fallback_count = self.fallback_count.saturating_add(1);
        match error {
            TextLayoutError::InvalidFontSize => {
                self.invalid_font_size_count = self.invalid_font_size_count.saturating_add(1);
            }
            TextLayoutError::InvalidLanguage => {
                self.invalid_language_count = self.invalid_language_count.saturating_add(1);
            }
            _ => {
                self.other_error_count = self.other_error_count.saturating_add(1);
            }
        }
    }

    pub(crate) fn record_generation_deferred(&mut self) {
        self.generation_deferred_count = self.generation_deferred_count.saturating_add(1);
    }
}

fn shared_fallback_report() -> &'static Mutex<TextLayoutFallbackReport> {
    static REPORT: OnceLock<Mutex<TextLayoutFallbackReport>> = OnceLock::new();
    REPORT.get_or_init(|| Mutex::new(TextLayoutFallbackReport::default()))
}

pub fn shared_text_layout_fallback_report() -> TextLayoutFallbackReport {
    *shared_fallback_report()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn record_text_layout_fallback(error: &TextLayoutError) {
    shared_fallback_report()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .record(error);
}

fn record_text_layout_generation_deferred() {
    shared_fallback_report()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .record_generation_deferred();
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SharedTextLayoutSession {
    shaped_runs: ShapedRunCache,
    vertical_mode: Option<VerticalMode>,
}

impl Default for SharedTextLayoutSession {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedTextLayoutSession {
    pub(crate) fn new() -> Self {
        Self {
            shaped_runs: ShapedRunCache::with_limits(
                DEFAULT_SHAPED_RUN_CACHE_CAPACITY,
                DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
            ),
            vertical_mode: None,
        }
    }

    pub(crate) fn begin_frame(&mut self, frame_index: u64) {
        self.shaped_runs.begin_frame(frame_index);
    }

    pub(crate) fn finish_frame(&mut self) {
        self.shaped_runs.finish_frame();
    }

    pub(crate) fn clear(&mut self) {
        self.shaped_runs.clear();
    }

    pub(crate) fn cache_report(&self) -> ShapedRunCacheReport {
        self.shaped_runs.report()
    }

    pub(crate) fn prewarm_horizontal_paragraphs(
        &mut self,
        pool: &TaskPool,
        paragraphs: &[TextShapeParagraph],
        chunk_size: usize,
    ) -> TextParallelShapeBatchReport {
        shape_paragraphs_with_cache(pool, &mut self.shaped_runs, paragraphs, chunk_size).report
    }

    pub(crate) fn shape_horizontal_line(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
    ) -> Arc<ShapedGlyphRun> {
        self.resolve_or_shape(BackendShapeRequest::horizontal(
            text,
            style,
            direction,
            source_range,
        ))
    }

    pub(crate) fn shape_vertical_line(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
    ) -> Arc<ShapedGlyphRun> {
        self.resolve_or_shape(BackendShapeRequest::vertical_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            true,
        ))
    }

    pub(crate) fn vertical_scope(
        &mut self,
        vertical_mode: VerticalMode,
    ) -> VerticalTextLayoutScope<'_> {
        let previous_mode = self.vertical_mode.replace(vertical_mode);
        VerticalTextLayoutScope {
            session: self,
            previous_mode,
        }
    }

    fn resolve_or_shape(&mut self, request: BackendShapeRequest<'_>) -> Arc<ShapedGlyphRun> {
        let canonical_request = request.canonicalized();
        let request = canonical_request.request();
        if !request.style.font_size.is_finite() || request.style.font_size <= 0.0 {
            return Arc::new(shape_fallback_for_error(
                request,
                &TextLayoutError::InvalidFontSize,
            ));
        }
        let lookup = ShapedRunCacheLookupKey::from_request(&request);
        let lookup_generation = lookup.font_database_generation();
        if let Some(run) = self.shaped_runs.get_with_lookup(&lookup, request.text) {
            return run;
        }
        let key = self.shaped_runs.own_lookup_key(&lookup);
        match try_shape_request_through_canonical_service(request) {
            Ok(shaped) if lookup_generation == shared_font_database_generation() => {
                self.shaped_runs.insert(key, shaped)
            }
            Ok(shaped) => Arc::new(shaped),
            Err(error @ TextLayoutError::FontGenerationChanged) => {
                Arc::new(shape_fallback_for_error(request, &error))
            }
            Err(error) if lookup_generation == shared_font_database_generation() => self
                .shaped_runs
                .insert(key, shape_fallback_for_error(request, &error)),
            Err(error) => Arc::new(shape_fallback_for_error(request, &error)),
        }
    }
}

pub(super) fn shape_request_through_canonical_service(
    request: BackendShapeRequest<'_>,
) -> ShapedGlyphRun {
    match try_shape_request_through_canonical_service(request) {
        Ok(shaped) => shaped,
        Err(error) => shape_fallback_for_error(request, &error),
    }
}

pub(super) fn try_shape_request_through_canonical_service(
    request: BackendShapeRequest<'_>,
) -> Result<ShapedGlyphRun, TextLayoutError> {
    shape_canonical(request)
}

pub(super) fn shape_fallback_for_error(
    request: BackendShapeRequest<'_>,
    error: &TextLayoutError,
) -> ShapedGlyphRun {
    match error {
        TextLayoutError::FontGenerationChanged => record_text_layout_generation_deferred(),
        _ => record_text_layout_fallback(error),
    }
    explicit_empty_fallback(request)
}

fn shape_canonical(request: BackendShapeRequest<'_>) -> Result<ShapedGlyphRun, TextLayoutError> {
    shape_backend_request_at_stable_generation(request, |shaped, _| shaped)
}

impl TextShapeRunProvider for SharedTextLayoutSession {
    fn shape_horizontal_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        match self.vertical_mode {
            Some(vertical_mode) => {
                self.resolve_or_shape(BackendShapeRequest::vertical_with_kerning(
                    text,
                    style,
                    direction,
                    source_range,
                    vertical_mode,
                    include_kerning,
                ))
            }
            None => self.resolve_or_shape(BackendShapeRequest::horizontal_with_kerning(
                text,
                style,
                direction,
                source_range,
                include_kerning,
            )),
        }
    }

    fn shape_vertical_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        vertical_mode: VerticalMode,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        self.resolve_or_shape(BackendShapeRequest::vertical_with_kerning(
            text,
            style,
            direction,
            source_range,
            vertical_mode,
            include_kerning,
        ))
    }
}

pub(crate) struct VerticalTextLayoutScope<'a> {
    session: &'a mut SharedTextLayoutSession,
    previous_mode: Option<VerticalMode>,
}

impl Deref for VerticalTextLayoutScope<'_> {
    type Target = SharedTextLayoutSession;

    fn deref(&self) -> &Self::Target {
        self.session
    }
}

impl DerefMut for VerticalTextLayoutScope<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.session
    }
}

impl Drop for VerticalTextLayoutScope<'_> {
    fn drop(&mut self) {
        self.session.vertical_mode = self.previous_mode;
    }
}

fn explicit_empty_fallback(request: BackendShapeRequest<'_>) -> ShapedGlyphRun {
    ShapedGlyphRun {
        source_text: request.shared_source_text(),
        source_range: request.source_range,
        direction: request.base_direction,
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width: 0.0,
        measured_height: request.style.line_height.max(0.0),
        lines: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::font::{font_handle_registry_report, shared_font_database_test_serial_guard};

    #[test]
    fn session_routes_detailed_runs_through_canonical_service() {
        let mut session = SharedTextLayoutSession::new();
        let style = TextStyle::default();
        let run = session.shape_horizontal_line_with_kerning(
            "Canonical",
            &style,
            TextDirection::LeftToRight,
            TextRange { start: 11, end: 20 },
            true,
        );

        assert_eq!(run.source_range.start, 11);
        assert!(run.measured_width > 0.0);
        assert!(run.lines.iter().any(|line| !line.glyphs.is_empty()));
        assert!(run
            .lines
            .iter()
            .flat_map(|line| &line.glyphs)
            .all(|glyph| { glyph.source_range.start >= 11 && glyph.source_range.end <= 20 }));
    }

    #[test]
    fn font_handle_batch_session_keeps_canonical_run_without_framework_roundtrip() {
        let _shared_font_database = shared_font_database_test_serial_guard();
        let before = font_handle_registry_report();
        let mut session = SharedTextLayoutSession::new();
        let style = TextStyle::default();

        let run = session.shape_horizontal_line_with_kerning(
            "Batch resolution",
            &style,
            TextDirection::LeftToRight,
            TextRange { start: 0, end: 16 },
            true,
        );
        let after = font_handle_registry_report();
        let glyph_count = run
            .lines
            .iter()
            .map(|line| line.glyphs.len())
            .sum::<usize>();

        assert!(glyph_count > 1);
        assert_eq!(
            after.registration_batch_count, before.registration_batch_count,
            "the internal session must not project backend identities into framework handles"
        );
        assert_eq!(
            after.resolution_batch_count, before.resolution_batch_count,
            "the internal session must not resolve framework handles back into backend identities"
        );
    }

    #[test]
    fn session_records_typed_fallback_instead_of_silently_swallowing_service_error() {
        let before = shared_text_layout_fallback_report();
        let mut session = SharedTextLayoutSession::new();
        let style = TextStyle {
            font_size: 0.0,
            ..TextStyle::default()
        };

        let run = session.shape_horizontal_line(
            "invalid",
            &style,
            TextDirection::LeftToRight,
            TextRange { start: 0, end: 7 },
        );
        let after = shared_text_layout_fallback_report();

        assert!(run.lines.is_empty());
        assert!(after.fallback_count > before.fallback_count);
        assert!(after.invalid_font_size_count > before.invalid_font_size_count);
    }

    #[test]
    fn invalid_font_size_fallback_cannot_alias_a_valid_one_pixel_shape() {
        let mut session = SharedTextLayoutSession::new();
        let invalid_style = TextStyle {
            font_size: 0.0,
            ..TextStyle::default()
        };
        let valid_style = TextStyle {
            font_size: 1.0,
            ..TextStyle::default()
        };
        let range = TextRange { start: 0, end: 5 };

        let invalid = session.shape_horizontal_line(
            "alias",
            &invalid_style,
            TextDirection::LeftToRight,
            range,
        );
        let valid =
            session.shape_horizontal_line("alias", &valid_style, TextDirection::LeftToRight, range);

        assert!(invalid.lines.is_empty());
        assert!(valid.lines.iter().any(|line| !line.glyphs.is_empty()));
    }

    #[test]
    fn prewarm_routes_through_canonical_validation_and_records_typed_fallback() {
        let before = shared_text_layout_fallback_report();
        let mut session = SharedTextLayoutSession::new();
        let pool = TaskPool::new(crate::core::runtime::tasks::TaskPoolDescriptor::compute());
        let paragraph = TextShapeParagraph::horizontal(
            "invalid prewarm",
            TextStyle {
                font_size: 0.0,
                ..TextStyle::default()
            },
            TextDirection::LeftToRight,
            TextRange { start: 0, end: 15 },
        );

        let report = session.prewarm_horizontal_paragraphs(&pool, &[paragraph], 1);
        let cached = session.shape_horizontal_line(
            "invalid prewarm",
            &TextStyle {
                font_size: 0.0,
                ..TextStyle::default()
            },
            TextDirection::LeftToRight,
            TextRange { start: 0, end: 15 },
        );
        let after = shared_text_layout_fallback_report();

        assert_eq!(report.shaped_count, 1);
        assert!(cached.lines.is_empty());
        assert!(after.invalid_font_size_count > before.invalid_font_size_count);
    }

    #[test]
    fn session_source_uses_canonical_runs_without_framework_roundtrip() {
        let source = include_str!("layout_session.rs");

        assert!(!source.contains(concat!("TextShape", "Result")));
        assert!(!source.contains(concat!("resolve_font_handle", "_batch")));
        assert!(!source.contains(concat!("project_shape", "_result")));
        assert!(source.contains("shape_backend_request_at_stable_generation"));
    }

    #[test]
    fn fallback_report_tracks_generation_defer_without_counting_a_fallback() {
        let mut report = TextLayoutFallbackReport::default();

        report.record(&TextLayoutError::FontGenerationChanged);

        assert_eq!(report.generation_deferred_count, 1);
        assert_eq!(report.fallback_count, 0);
    }
}
