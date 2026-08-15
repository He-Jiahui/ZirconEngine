#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
use std::cell::Cell;
use std::time::Instant;

use crate::core::framework::text::TextDirection;
use crate::text::{TextRange, TextStyle};
use glyphon::{
    cosmic_text::{BidiParagraphs, FeatureTag, FontFeatures, LineEnding, LineIter},
    Attrs, Buffer, Family, LayoutGlyph, Metrics, Shaping, Weight, Wrap,
};

use crate::text::font::FontDatabase;
use crate::text::{
    BackendShapeRequest, ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun,
    ShapedTextLine, TextOrientation,
};

use super::bidi::BidiParagraph;
use super::horizontal::shape_horizontal_request;
use super::line_break::{ClusterLineBreakFlags, LineBreakOpportunityMap};
use super::normalize::ShapingTextView;
use super::script_segment::{
    script_for_range, script_segments, shaped_script_for_cluster, ScriptSegment,
};
use super::vertical::{apply_vertical_layout, shape_vertical_request};

mod fallback;
mod font_system_cache;
mod hard_lines;

use super::fallback_text_spans;
use fallback::fallback_shape;
use font_system_cache::with_font_system;
use hard_lines::normalize_cosmic_hard_lines;

pub(crate) fn shape_text(request: BackendShapeRequest<'_>) -> ShapedGlyphRun {
    debug_assert!(request.features_are_normalized());
    let text_view = ShapingTextView::v1_disabled(request.text);
    let bidi = BidiParagraph::new(text_view.shaping_text(), request.base_direction);
    if let Some(shaped) = shape_with_cosmic(request, &text_view, &bidi) {
        return shaped;
    }
    let mut shaped = fallback_shape(request, &text_view, &bidi);
    apply_vertical_layout(&mut shaped, request, None);
    shaped
}

fn shape_with_cosmic(
    request: BackendShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    bidi: &BidiParagraph<'_>,
) -> Option<ShapedGlyphRun> {
    if text_view.shaping_text().is_empty() {
        let mut shaped = empty_run(request, bidi);
        apply_vertical_layout(&mut shaped, request, None);
        return Some(shaped);
    }

    let profile_shape = std::env::var_os("ZR_UI_LAYOUT_PROFILE").is_some();
    let shape_started = Instant::now();
    let shaped = with_font_system(request.language, |font_system, font_database| {
        let line_height = resolved_line_height(request);
        let fallback_started = Instant::now();
        let fallback_spans = fallback_text_spans(text_view.shaping_text(), request, font_database);
        emit_slow_cosmic_profile(
            profile_shape,
            "fallback-spans",
            fallback_started,
            request.text,
        );
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        begin_direct_shape_profile_metrics();
        if matches!(request.orientation, TextOrientation::Horizontal) {
            if let Some(shaped) =
                shape_horizontal_request(request, bidi, &fallback_spans, font_database)
            {
                #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                record_direct_shape_profile_metrics(&shaped, request.text);
                return Some(shaped);
            }
        } else if let Some(shaped) =
            shape_vertical_request(request, bidi, &fallback_spans, font_database)
        {
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            record_direct_shape_profile_metrics(&shaped, request.text);
            return Some(shaped);
        }
        #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
        discard_direct_shape_profile_metrics();
        if crate::text::hard_lines(request.text)
            .iter()
            .any(crate::text::HardLine::is_run_cap_break)
        {
            return None;
        }
        if !cosmic_backend_fallback_allowed(request.orientation) {
            return None;
        }

        let metrics = Metrics::new(request.style.font_size.max(1.0), line_height);
        let mut buffer = Buffer::new(font_system, metrics);
        let mut buffer = buffer.borrow_with(font_system);
        buffer.set_size(None, Some(line_height));
        buffer.set_wrap(Wrap::None);
        let default_attrs = attrs_for_style(request);
        let buffer_started = Instant::now();
        let line_starts = if fallback_spans.is_empty() {
            buffer.set_text(
                text_view.shaping_text(),
                &default_attrs,
                Shaping::Advanced,
                None,
            );
            cosmic_plain_line_starts(text_view.shaping_text())
        } else {
            buffer.set_rich_text(
                fallback_spans.iter().map(|span| {
                    let attrs = span
                        .family
                        .as_deref()
                        .map(|family| default_attrs.clone().family(Family::Name(family)))
                        .unwrap_or_else(|| default_attrs.clone());
                    (&text_view.shaping_text()[span.range.clone()], attrs)
                }),
                &default_attrs,
                Shaping::Advanced,
                None,
            );
            cosmic_rich_line_starts(text_view.shaping_text())
        };
        buffer.shape_until_scroll(true);
        emit_slow_cosmic_profile(profile_shape, "buffer-shape", buffer_started, request.text);

        let line_breaks = LineBreakOpportunityMap::new(text_view.shaping_text());
        let scripts = script_segments(text_view.shaping_text());
        let hard_lines = crate::text::hard_lines(text_view.shaping_text());
        let mut raw_lines = Vec::new();
        for run in buffer.layout_runs() {
            raw_lines.push(line_from_layout_run(
                request,
                text_view,
                &run,
                line_starts
                    .get(run.line_i)
                    .copied()
                    .unwrap_or(text_view.shaping_text().len()),
                &line_breaks,
                &scripts,
                bidi,
                &fallback_spans,
                font_database,
            ));
        }

        if raw_lines.is_empty() {
            return None;
        }
        let mut lines =
            normalize_cosmic_hard_lines(request, bidi, &scripts, &hard_lines, raw_lines);

        let measured_width = lines
            .iter()
            .map(|line| line.measured_width)
            .fold(0.0_f32, f32::max);
        let measured_height = lines.iter().map(|line| line.line_height).sum::<f32>();
        let mut shaped = ShapedGlyphRun {
            source_text: request.shared_source_text(),
            source_range: request.source_range,
            direction: bidi.resolved_base_direction(),
            orientation: request.orientation,
            vertical_mode: request.vertical_mode,
            include_kerning: request.include_kerning,
            measured_width,
            measured_height,
            lines,
        };
        apply_vertical_layout(&mut shaped, request, Some(font_database));
        Some(shaped)
    });
    emit_slow_cosmic_profile(profile_shape, "shape-total", shape_started, request.text);
    shaped
}

#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
thread_local! {
    static DIRECT_SHAPE_BACKEND_CALL_COUNT: Cell<Option<usize>> = const { Cell::new(None) };
}

fn emit_slow_cosmic_profile(enabled: bool, stage: &str, started: Instant, text: &str) {
    if !enabled {
        return;
    }
    let elapsed_ms = started.elapsed().as_millis();
    if elapsed_ms < 10 {
        return;
    }
    eprintln!(
        "ui-layout-profile stage=slow-text-{stage} elapsed_ms={elapsed_ms} text_bytes={}",
        text.len(),
    );
}

/// Records only a completed direct request. Fallback shaping deliberately does not contribute to
/// this stream, so the scale harness can reject a regression to a second backend path.
#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
fn record_direct_shape_profile_metrics(shaped: &ShapedGlyphRun, text: &str) {
    let Some(backend_shape_calls) = take_direct_shape_backend_call_count() else {
        return;
    };
    let glyph_count = shaped
        .lines
        .iter()
        .map(|line| line.glyphs.len())
        .sum::<usize>();
    crate::profile_counter!("runtime", "text_direct_shape_request_count", 1);
    crate::profile_counter!("runtime", "text_direct_shape_input_byte_count", text.len());
    crate::profile_counter!(
        "runtime",
        "text_direct_shape_output_glyph_count",
        glyph_count
    );
    crate::profile_counter!(
        "runtime",
        "text_direct_backend_shape_call_count",
        backend_shape_calls
    );
}

/// Starts a request-local counter only while a managed capture is active. The backend leafs then
/// increment this TLS value so a long text run does not lock the profiler once per segment.
#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
fn begin_direct_shape_profile_metrics() {
    DIRECT_SHAPE_BACKEND_CALL_COUNT.with(|count| {
        count.set(direct_shape_profile_metrics_enabled().then_some(0));
    });
}

#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
fn discard_direct_shape_profile_metrics() {
    DIRECT_SHAPE_BACKEND_CALL_COUNT.with(|count| count.set(None));
}

#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
pub(super) fn record_direct_backend_shape_call() {
    DIRECT_SHAPE_BACKEND_CALL_COUNT.with(|count| {
        if let Some(call_count) = count.get() {
            count.set(Some(call_count.saturating_add(1)));
        }
    });
}

#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
fn take_direct_shape_backend_call_count() -> Option<usize> {
    DIRECT_SHAPE_BACKEND_CALL_COUNT.with(|count| count.replace(None))
}

#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
fn direct_shape_profile_metrics_enabled() -> bool {
    #[cfg(feature = "profiling-tracy")]
    {
        return true;
    }
    #[cfg(all(feature = "profiling", not(feature = "profiling-tracy")))]
    {
        return crate::core::diagnostics::profiling::capture_active();
    }
    #[cfg(not(any(feature = "profiling", feature = "profiling-tracy")))]
    {
        false
    }
}

fn line_from_layout_run(
    request: BackendShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    run: &glyphon::LayoutRun<'_>,
    line_visual_start: usize,
    line_breaks: &LineBreakOpportunityMap,
    scripts: &[ScriptSegment],
    bidi: &BidiParagraph<'_>,
    fallback_spans: &[super::fallback_spans::FallbackTextSpan],
    font_database: &FontDatabase,
) -> ShapedTextLine {
    let line_shaping_range = line_visual_start..line_visual_start + run.text.len();
    let line_source_range = text_view.source_range_for_shaping_range(line_shaping_range);
    let line_source_start = request.source_range.start + line_source_range.start;
    let visual_range = TextRange {
        start: 0,
        end: run.text.len(),
    };
    let mut previous_range = None;
    let glyphs = run
        .glyphs
        .iter()
        .map(|glyph| {
            let current_range = (glyph.start, glyph.end);
            let cluster_start = previous_range != Some(current_range);
            previous_range = Some(current_range);
            glyph_from_layout_glyph(
                request,
                text_view,
                glyph,
                run.rtl,
                line_visual_start,
                cluster_start,
                line_breaks,
                scripts,
                bidi,
                fallback_spans,
                font_database,
            )
        })
        .collect::<Vec<_>>();

    ShapedTextLine {
        line_index: run.line_i,
        source_range: TextRange {
            start: line_source_start,
            end: request.source_range.start + line_source_range.end,
        },
        visual_range,
        measured_width: run.line_w.max(0.0),
        baseline: cosmic_line_baseline(run.line_y, run.line_top, run.line_height),
        line_height: run.line_height.max(resolved_line_height(request)),
        glyphs,
    }
}

fn glyph_from_layout_glyph(
    request: BackendShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    glyph: &LayoutGlyph,
    run_rtl: bool,
    line_visual_start: usize,
    cluster_start: bool,
    line_breaks: &LineBreakOpportunityMap,
    scripts: &[ScriptSegment],
    bidi: &BidiParagraph<'_>,
    fallback_spans: &[super::fallback_spans::FallbackTextSpan],
    font_database: &FontDatabase,
) -> ShapedGlyph {
    let shaping_range = line_visual_start + glyph.start..line_visual_start + glyph.end;
    let projected_source_range = text_view.source_range_for_shaping_range(shaping_range.clone());
    let source_range = absolute_range(
        request.source_range.start,
        projected_source_range.start,
        projected_source_range.end,
    );
    let cluster_text = text_view
        .shaping_text()
        .get(
            shaping_range.start.min(text_view.shaping_text().len())
                ..shaping_range.end.min(text_view.shaping_text().len()),
        )
        .unwrap_or_default();
    let local_range = TextRange {
        start: line_visual_start + glyph.start,
        end: line_visual_start + glyph.end,
    };
    let bidi_level = bidi.level_for_range(local_range);
    let direction = if bidi_level % 2 == 1 || glyph.level.is_rtl() || run_rtl {
        TextDirection::RightToLeft
    } else {
        TextDirection::LeftToRight
    };
    let cluster_line_breaks = if cluster_start {
        line_breaks.flags_for_cluster(
            line_visual_start + glyph.start,
            line_visual_start + glyph.end,
        )
    } else {
        ClusterLineBreakFlags::default()
    };
    let script = shaped_script_for_cluster(cluster_text, script_for_range(scripts, local_range));

    let (offset_x, offset_y) =
        glyph_layout_offset_px(glyph.font_size, glyph.x_offset, glyph.y_offset);
    let resolved_span = fallback_spans
        .get(fallback_spans.partition_point(|span| span.range.end <= shaping_range.start))
        .filter(|span| {
            span.range.start <= shaping_range.start && span.range.end >= shaping_range.end
        });
    let font_id = font_database.font_face_id(glyph.font_id);
    let font_instance_id = font_id.and_then(|face| {
        resolved_span
            .filter(|span| span.face == Some(face))
            .and_then(|span| span.instance)
            .or_else(|| {
                font_database
                    .effective_instance_id(
                        face,
                        TextStyle::normalized_font_weight(request.style.font_weight),
                    )
                    .ok()
            })
    });
    ShapedGlyph {
        glyph_id: glyph.glyph_id as u32,
        font_id,
        font_instance_id,
        source_range,
        visual_range: TextRange {
            start: glyph.start,
            end: glyph.end,
        },
        advance: glyph.w.max(0.0),
        x: glyph.x,
        y: glyph.y,
        offset_x,
        offset_y,
        direction,
        bidi_level,
        cluster_flags: cluster_flags(cluster_text, direction, cluster_start, cluster_line_breaks),
        rotation: ShapedGlyphRotation::None,
        script,
    }
}

fn glyph_layout_offset_px(font_size: f32, x_offset: f32, y_offset: f32) -> (f32, f32) {
    let font_size = font_size.max(1.0);
    (
        finite_offset_px(font_size, x_offset),
        finite_offset_px(font_size, y_offset),
    )
}

fn finite_offset_px(font_size: f32, offset: f32) -> f32 {
    if offset.is_finite() {
        font_size * offset
    } else {
        0.0
    }
}

const fn cosmic_backend_fallback_allowed(orientation: TextOrientation) -> bool {
    matches!(orientation, TextOrientation::Horizontal)
}

fn cosmic_plain_line_starts(text: &str) -> Vec<usize> {
    let mut starts = Vec::new();
    let mut last_ending = None;
    for (range, ending) in LineIter::new(text) {
        starts.push(range.start);
        last_ending = Some(ending);
    }
    if !matches!(last_ending, Some(LineEnding::None)) {
        starts.push(text.len());
    }
    starts
}

fn cosmic_rich_line_starts(text: &str) -> Vec<usize> {
    let text_start = text.as_ptr() as usize;
    let mut starts = BidiParagraphs::new(text)
        .map(|paragraph| paragraph.as_ptr() as usize - text_start)
        .collect::<Vec<_>>();
    if starts.is_empty() {
        starts.push(0);
    }
    starts
}

fn cosmic_line_baseline(line_y: f32, line_top: f32, line_height: f32) -> f32 {
    (line_y - line_top).clamp(0.0, line_height.max(0.0))
}

fn empty_run(request: BackendShapeRequest<'_>, bidi: &BidiParagraph<'_>) -> ShapedGlyphRun {
    let line_height = resolved_line_height(request);
    ShapedGlyphRun {
        source_text: request.shared_source_text(),
        source_range: request.source_range,
        direction: bidi.resolved_base_direction(),
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width: 0.0,
        measured_height: line_height,
        lines: vec![ShapedTextLine {
            line_index: 0,
            source_range: request.source_range,
            visual_range: TextRange::default(),
            measured_width: 0.0,
            baseline: request.style.font_size.max(1.0) * 0.8,
            line_height,
            glyphs: Vec::new(),
        }],
    }
}

pub(super) fn cluster_flags(
    cluster_text: &str,
    direction: TextDirection,
    cluster_start: bool,
    line_breaks: ClusterLineBreakFlags,
) -> ShapedGlyphClusterFlags {
    ShapedGlyphClusterFlags {
        cluster_start,
        rtl: matches!(direction, TextDirection::RightToLeft),
        whitespace: cluster_text.chars().any(char::is_whitespace),
        space: cluster_text
            .chars()
            .any(|ch| matches!(ch, ' ' | '\u{00a0}')),
        tab: cluster_text.contains('\t'),
        mandatory_break: line_breaks.mandatory_break
            || cluster_text.chars().any(|ch| matches!(ch, '\n' | '\r')),
        soft_break: line_breaks.soft_break,
        virtual_glyph: cluster_text.chars().any(char::is_control),
    }
}

fn attrs_for_style<'a>(request: BackendShapeRequest<'a>) -> Attrs<'a> {
    let attrs = match request
        .style
        .font_family
        .as_deref()
        .or(request.style.font.as_deref())
        .map(str::trim)
        .filter(|family| !family.is_empty())
    {
        Some(family) => Attrs::new().family(Family::Name(family)),
        None => Attrs::new(),
    };
    let attrs = attrs.weight(Weight(TextStyle::normalized_font_weight(
        request.style.font_weight,
    )));
    let uses_vertical_features = matches!(request.orientation, TextOrientation::Vertical)
        && !matches!(request.vertical_mode, crate::text::VerticalMode::Sideways);
    if request.include_kerning && request.features().is_empty() && !uses_vertical_features {
        return attrs;
    }

    let mut features = FontFeatures::new();
    if !request.include_kerning {
        features.disable(FeatureTag::KERNING);
    }
    if uses_vertical_features {
        if !request
            .features()
            .iter()
            .any(|feature| feature.tag == *b"vert")
        {
            features.set(FeatureTag::new(b"vert"), 1);
        }
        if !request
            .features()
            .iter()
            .any(|feature| feature.tag == *b"vrt2")
        {
            features.set(FeatureTag::new(b"vrt2"), 1);
        }
    }
    for feature in request.features() {
        features.set(FeatureTag::new(&feature.tag), feature.value);
    }
    attrs.font_features(features)
}

pub(super) fn resolved_line_height(request: BackendShapeRequest<'_>) -> f32 {
    request
        .style
        .line_height
        .max(request.style.font_size.max(1.0))
}

fn absolute_range(source_start: usize, visual_start: usize, visual_end: usize) -> TextRange {
    TextRange {
        start: source_start + visual_start,
        end: source_start + visual_end.max(visual_start),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::text::TextDirection;
    use crate::text::{TextRange, TextStyle};
    use glyphon::cosmic_text::FeatureTag;

    use super::{
        attrs_for_style, cosmic_backend_fallback_allowed, cosmic_line_baseline,
        cosmic_plain_line_starts, cosmic_rich_line_starts, glyph_layout_offset_px,
    };
    use crate::text::{BackendShapeRequest, OpenTypeFeature, TextOrientation};

    #[test]
    fn glyph_layout_offsets_are_projected_to_pixels() {
        let (x, y) = glyph_layout_offset_px(13.0, 0.25, -0.125);

        assert!((x - 3.25).abs() < 0.001);
        assert!((y + 1.625).abs() < 0.001);
    }

    #[test]
    fn glyph_layout_offsets_drop_non_finite_values() {
        let (x, y) = glyph_layout_offset_px(13.0, f32::NAN, f32::INFINITY);

        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn attrs_disable_kerning_when_requested() {
        let style = TextStyle::default();
        let attrs = attrs_for_style(BackendShapeRequest::horizontal_with_kerning(
            "AV",
            &style,
            TextDirection::LeftToRight,
            TextRange { start: 0, end: 2 },
            false,
        ));

        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::KERNING && feature.value == 0));
    }

    #[test]
    fn attrs_apply_normalized_open_type_features() {
        let style = TextStyle::default();
        let features = [
            OpenTypeFeature::new(*b"tnum", 1),
            OpenTypeFeature::new(*b"liga", 0),
        ];
        let request = BackendShapeRequest::horizontal(
            "0123",
            &style,
            TextDirection::LeftToRight,
            TextRange { start: 0, end: 4 },
        )
        .with_features(&features)
        .canonicalized();
        let attrs = attrs_for_style(request.request());

        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"tnum") && feature.value == 1));
        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"liga") && feature.value == 0));
    }

    #[test]
    fn attrs_enable_vertical_substitution_features_for_upright_glyphs() {
        let style = TextStyle::default();
        let attrs = attrs_for_style(BackendShapeRequest::vertical(
            "本文。",
            &style,
            TextDirection::LeftToRight,
            TextRange {
                start: 0,
                end: "本文。".len(),
            },
            crate::text::VerticalMode::Mixed,
        ));

        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"vert") && feature.value == 1));
        assert!(attrs
            .font_features
            .features
            .iter()
            .any(|feature| feature.tag == FeatureTag::new(b"vrt2") && feature.value == 1));
    }

    #[test]
    fn cosmic_plain_line_starts_follow_line_iter_endings() {
        assert_eq!(cosmic_plain_line_starts(""), vec![0]);
        assert_eq!(cosmic_plain_line_starts("one"), vec![0]);
        assert_eq!(cosmic_plain_line_starts("one\ntwo\n"), vec![0, 4, 8]);
        assert_eq!(cosmic_plain_line_starts("a\rb"), vec![0, 2]);
        assert_eq!(cosmic_plain_line_starts("a\r\nb\n\r"), vec![0, 3, 6]);
        assert_eq!(cosmic_plain_line_starts("a\u{0085}b\u{2029}c"), vec![0]);
    }

    #[test]
    fn cosmic_rich_line_starts_follow_backend_bidi_paragraphs() {
        assert_eq!(cosmic_rich_line_starts(""), vec![0]);
        assert_eq!(cosmic_rich_line_starts("one"), vec![0]);
        assert_eq!(cosmic_rich_line_starts("one\ntwo\n"), vec![0, 4]);
        assert_eq!(cosmic_rich_line_starts("a\rb"), vec![0]);
        assert_eq!(cosmic_rich_line_starts("本\rb"), vec![0, 4]);
        assert_eq!(
            cosmic_rich_line_starts("a\u{0085}b\u{2029}c"),
            vec![0, 3, 7]
        );
        assert_eq!(cosmic_rich_line_starts("a\u{2028}b"), vec![0]);
    }

    #[test]
    fn cosmic_fallback_is_horizontal_only() {
        assert!(cosmic_backend_fallback_allowed(TextOrientation::Horizontal));
        assert!(!cosmic_backend_fallback_allowed(TextOrientation::Vertical));
    }

    #[test]
    fn cosmic_baseline_is_relative_to_each_layout_line() {
        assert_eq!(cosmic_line_baseline(18.0, 10.0, 12.0), 8.0);
        assert_eq!(cosmic_line_baseline(40.0, 24.0, 12.0), 12.0);
        assert_eq!(cosmic_line_baseline(5.0, 9.0, 12.0), 0.0);
    }
}
