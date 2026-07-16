use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, OnceLock};

use crate::core::framework::text::{
    TextDirection, TextFontRequest, TextGlyphFlags, TextGlyphRotation, TextLayoutError,
    TextRenderMode, TextShapeRequest, TextShapeResult, TextWritingMode,
};
use crate::core::runtime::tasks::TaskPool;

use super::cache::{
    ShapedRunCache, ShapedRunCacheKey, ShapedRunCacheReport, DEFAULT_SHAPED_RUN_CACHE_CAPACITY,
    DEFAULT_SHAPED_RUN_CACHE_MAX_BYTES,
};
use super::font::{resolve_font_face_handle, resolve_font_instance_handle};
use super::parallel::shape_pool::{
    shape_paragraphs_with_cache, TextParallelShapeBatchReport, TextShapeParagraph,
};
use super::shaping::TextShapeRunProvider;
use super::{
    shared_text_layout_service, BackendShapeRequest, ShapedGlyph, ShapedGlyphClusterFlags,
    ShapedGlyphRotation, ShapedGlyphRun, ShapedGlyphScript, ShapedTextLine, TextOrientation,
    TextRange, TextStyle, VerticalMode,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextLayoutFallbackReport {
    pub fallback_count: u64,
    pub invalid_font_size_count: u64,
    pub invalid_language_count: u64,
    pub other_error_count: u64,
}

impl TextLayoutFallbackReport {
    pub(crate) fn record(&mut self, error: &TextLayoutError) {
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
        let key = ShapedRunCacheKey::from_request(&request);
        if let Some(run) = self.shaped_runs.get(&key, request.text) {
            return run;
        }
        let shaped = shape_request_through_canonical_service(request);
        self.shaped_runs.insert(key, request.text, shaped)
    }
}

pub(super) fn shape_request_through_canonical_service(
    request: BackendShapeRequest<'_>,
) -> ShapedGlyphRun {
    match shape_canonical(request) {
        Ok(shaped) => shaped,
        Err(error) => {
            record_text_layout_fallback(&error);
            explicit_empty_fallback(request)
        }
    }
}

fn shape_canonical(request: BackendShapeRequest<'_>) -> Result<ShapedGlyphRun, TextLayoutError> {
    let family_storage = request.style.font_family.as_deref().map(|family| [family]);
    let families = family_storage
        .as_ref()
        .map_or(&[][..], |families| &families[..]);
    let font = TextFontRequest {
        families,
        asset: request.style.font.as_deref(),
        size: request.style.font_size,
        weight: request.style.font_weight,
        stretch: 100,
        italic: false,
        render_mode: TextRenderMode::Auto,
    };
    let mut canonical_request = TextShapeRequest::new(request.text, font);
    canonical_request.language = request.language;
    canonical_request.direction = request.base_direction;
    canonical_request.writing_mode = match request.orientation {
        TextOrientation::Horizontal => TextWritingMode::HorizontalTopToBottom,
        TextOrientation::Vertical => TextWritingMode::VerticalRightToLeft,
    };
    canonical_request.line_height = request.style.line_height;
    canonical_request.tab_size = request.style.tab_size;
    canonical_request.include_kerning = request.include_kerning;
    let result = shared_text_layout_service().shape(canonical_request)?;
    Ok(detailed_run(
        request.text,
        request.style,
        request.source_range,
        request.orientation,
        request.vertical_mode,
        request.include_kerning,
        result,
    ))
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

fn detailed_run(
    text: &str,
    style: &TextStyle,
    source_range: TextRange,
    orientation: TextOrientation,
    vertical_mode: VerticalMode,
    include_kerning: bool,
    shaped: TextShapeResult,
) -> ShapedGlyphRun {
    let lines = shaped
        .runs
        .into_iter()
        .enumerate()
        .map(|(line_index, run)| {
            let local_start = run.source_range.start.min(text.len());
            let local_end = run.source_range.end.min(text.len()).max(local_start);
            let line_text = text
                .get(local_start..local_end)
                .unwrap_or_default()
                .to_string();
            let glyphs = run
                .glyphs
                .into_iter()
                .map(|glyph| detailed_glyph(glyph, run.direction, source_range.start))
                .collect::<Vec<_>>();
            ShapedTextLine {
                line_index,
                text: line_text.clone(),
                source_range: TextRange {
                    start: source_range.start + local_start,
                    end: source_range.start + local_end,
                },
                visual_range: TextRange {
                    start: 0,
                    end: line_text.len(),
                },
                measured_width: glyphs.iter().map(|glyph| glyph.advance).sum(),
                baseline: shaped.metrics.baseline,
                line_height: style.line_height,
                glyphs,
            }
        })
        .collect();
    ShapedGlyphRun {
        source_text: text.to_string(),
        source_range,
        direction: shaped.resolved_direction,
        orientation,
        vertical_mode,
        include_kerning,
        measured_width: shaped.metrics.width,
        measured_height: shaped.metrics.height,
        lines,
    }
}

fn detailed_glyph(
    glyph: crate::core::framework::text::TextGlyph,
    direction: TextDirection,
    source_offset: usize,
) -> ShapedGlyph {
    ShapedGlyph {
        glyph_id: glyph.glyph_id,
        font_id: glyph.font_face.and_then(resolve_font_face_handle),
        font_instance_id: glyph.font_instance.and_then(resolve_font_instance_handle),
        source_range: TextRange {
            start: source_offset + glyph.source_range.start,
            end: source_offset + glyph.source_range.end,
        },
        visual_range: TextRange {
            start: glyph.visual_range.start,
            end: glyph.visual_range.end,
        },
        advance: glyph.advance,
        x: glyph.position[0],
        y: glyph.position[1],
        offset_x: glyph.offset[0],
        offset_y: glyph.offset[1],
        direction,
        bidi_level: glyph.bidi_level,
        cluster_flags: detailed_flags(glyph.flags),
        rotation: match glyph.rotation {
            TextGlyphRotation::None => ShapedGlyphRotation::None,
            TextGlyphRotation::Clockwise90 => ShapedGlyphRotation::Cw90,
        },
        script: ShapedGlyphScript::default(),
    }
}

fn detailed_flags(flags: TextGlyphFlags) -> ShapedGlyphClusterFlags {
    ShapedGlyphClusterFlags {
        cluster_start: flags.cluster_start,
        rtl: flags.right_to_left,
        whitespace: flags.whitespace,
        space: flags.space,
        tab: flags.tab,
        mandatory_break: flags.mandatory_break,
        soft_break: flags.soft_break,
        virtual_glyph: flags.virtual_glyph,
    }
}

fn explicit_empty_fallback(request: BackendShapeRequest<'_>) -> ShapedGlyphRun {
    ShapedGlyphRun {
        source_text: request.text.to_string(),
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
}
