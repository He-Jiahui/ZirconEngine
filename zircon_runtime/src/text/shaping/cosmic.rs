use std::time::Instant;

use crate::core::framework::text::{TextDirection, TextLayoutError};
use crate::text::{TextRange, TextStyle};
use glyphon::{
    Attrs, Buffer, Family, LayoutGlyph, Metrics, Shaping, Weight, Wrap,
    cosmic_text::{
        BidiParagraphs, FeatureTag, FontFeatures, LineEnding, LineIter, Style as CosmicStyle,
    },
};

use crate::text::font::{FontCollectionSnapshot, FontDatabase};
use crate::text::model::TextShapingRequestDiagnostics;
use crate::text::{
    BackendShapeRequest, ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun,
    ShapedHardLine, TextHorizontalCompositionReceipt, TextOrientation,
};

use super::bidi::BidiParagraph;
use super::direct_error::DirectShapeError;
use super::failure_receipt::classify_direct_shape_failure;
use super::horizontal::{HorizontalDirectShapeAttempt, shape_horizontal_request};
use super::line_break::{
    ClusterLineBreakFlags, LineBreakOpportunityMap, contains_mandatory_break_control,
};
use super::normalize::ShapingTextView;
use super::script_segment::ParagraphTextAnalysis;
use super::vertical::{apply_vertical_layout, shape_vertical_request};
use super::{TextShapingCompletion, TextShapingFailure, TextShapingFailureReceipt};

#[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
pub(super) mod direct_profile;
#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
mod fallback_profile;
mod font_system_cache;
mod hard_lines;
mod horizontal_recovery;

use super::{FallbackItemizationError, fallback_primary_face, fallback_text_spans_with_report};
use font_system_cache::with_font_system;
use hard_lines::normalize_cosmic_hard_lines;

enum DirectAttemptFailure {
    Unrecorded(DirectShapeError),
    Recorded(TextShapingFailureReceipt),
}

pub(crate) fn shape_text_in_font_collection(
    request: BackendShapeRequest<'_>,
    font_collection: &FontCollectionSnapshot,
) -> Result<TextShapingCompletion<ShapedGlyphRun>, TextShapingFailure> {
    debug_assert!(request.features_are_normalized());
    let text_view = ShapingTextView::source_preserving(request.text);
    let bidi = BidiParagraph::for_snapshot(
        text_view.shaping_text(),
        request.base_direction,
        request.unicode_data_snapshot(),
    );
    shape_with_cosmic(request, &text_view, &bidi, font_collection)
}

fn shape_with_cosmic(
    request: BackendShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    bidi: &BidiParagraph<'_>,
    font_collection: &FontCollectionSnapshot,
) -> Result<TextShapingCompletion<ShapedGlyphRun>, TextShapingFailure> {
    if text_view.shaping_text().is_empty() {
        let mut shaped = empty_run(request, bidi);
        apply_vertical_layout(&mut shaped, request, None);
        return Ok(TextShapingCompletion::new(
            shaped,
            TextShapingRequestDiagnostics::EMPTY,
        ));
    }

    let profile_shape = std::env::var_os("ZR_UI_LAYOUT_PROFILE").is_some();
    let shape_started = Instant::now();
    let mut request_diagnostics = TextShapingRequestDiagnostics::EMPTY;
    let shaped = with_font_system(
        font_collection,
        request.language,
        |font_system, font_database| {
            if font_database.face_count() == 0 {
                return Err(FallbackItemizationError::PrimaryFaceUnavailable.into());
            }
            let line_height = resolved_line_height(request);
            let analysis = ParagraphTextAnalysis::for_snapshot(
                text_view.shaping_text(),
                request.explicit_language_script(),
                request.unicode_data_snapshot(),
            );
            let fallback_started = Instant::now();
            #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
            let fallback_cache_before = fallback_profile::begin(font_database);
            let fallback_result = fallback_text_spans_with_report(
                text_view.shaping_text(),
                request,
                font_database,
                &analysis,
            );
            #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
            fallback_profile::finish(font_database, fallback_cache_before);
            let (fallback_spans, font_resolution) =
                fallback_result.map_err(TextShapingFailure::from)?;
            request_diagnostics.font_resolution.merge(font_resolution);
            emit_slow_cosmic_profile(
                profile_shape,
                "fallback-spans",
                fallback_started,
                request.text,
            );
            #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
            direct_profile::begin();
            let mut horizontal_partial = None;
            let direct_result = if matches!(request.orientation, TextOrientation::Horizontal) {
                match shape_horizontal_request(
                    request,
                    bidi,
                    &fallback_spans,
                    &analysis,
                    font_database,
                ) {
                    Ok(HorizontalDirectShapeAttempt::Complete(shaped)) => Ok(shaped),
                    Ok(HorizontalDirectShapeAttempt::Partial(partial)) => {
                        let pending = horizontal_recovery::PendingHorizontalComposition::classify(
                            partial,
                            request.orientation,
                        )?;
                        let first_failure = pending.first_failure();
                        horizontal_partial = Some(pending);
                        Err(DirectAttemptFailure::Recorded(first_failure))
                    }
                    Err(error) => Err(DirectAttemptFailure::Unrecorded(error)),
                }
            } else {
                shape_vertical_request(request, bidi, &fallback_spans, &analysis, font_database)
                    .map_err(DirectAttemptFailure::Unrecorded)
            };
            let alternate_backend_receipt = match direct_result {
                Ok(shaped) => {
                    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                    direct_profile::record_completed_request(&shaped, request.text);
                    return Ok(shaped);
                }
                Err(DirectAttemptFailure::Unrecorded(error)) => {
                    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                    direct_profile::discard();
                    let receipt = classify_direct_shape_failure(&error, request.orientation);
                    if !receipt.allows_alternate_backend() {
                        return Err(TextShapingFailure::with_receipt(
                            horizontal_recovery::direct_failure_layout_error(receipt),
                            receipt,
                        ));
                    }
                    Some(receipt)
                }
                Err(DirectAttemptFailure::Recorded(receipt)) => Some(receipt),
            };
            if !cosmic_backend_fallback_allowed(request.orientation) {
                return Err(TextShapingFailure::with_optional_receipt(
                    TextLayoutError::ShapingFailed,
                    alternate_backend_receipt,
                ));
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

            let line_breaks = LineBreakOpportunityMap::for_snapshot(
                text_view.shaping_text(),
                request.unicode_data_snapshot(),
            );
            debug_assert_eq!(
                line_breaks.unicode_data_snapshot(),
                request.unicode_data_snapshot(),
                "Cosmic line-break analysis must use the request-bound Unicode snapshot"
            );
            let hard_lines = crate::text::hard_lines(text_view.shaping_text());
            let mut raw_lines = Vec::new();
            for run in buffer.layout_runs() {
                raw_lines.push(
                    line_from_layout_run(
                        request,
                        text_view,
                        &run,
                        line_starts
                            .get(run.line_i)
                            .copied()
                            .unwrap_or(text_view.shaping_text().len()),
                        &line_breaks,
                        &analysis,
                        bidi,
                        &fallback_spans,
                        font_database,
                    )
                    .map_err(|error| {
                        TextShapingFailure::with_optional_receipt(error, alternate_backend_receipt)
                    })?,
                );
            }

            if raw_lines.is_empty() {
                return Err(TextShapingFailure::with_optional_receipt(
                    TextLayoutError::ShapingFailed,
                    alternate_backend_receipt,
                ));
            }
            let mut lines =
                normalize_cosmic_hard_lines(request, bidi, &analysis, &hard_lines, raw_lines)
                    .map_err(|error| {
                        let error = DirectShapeError::from(error);
                        let receipt = classify_direct_shape_failure(&error, request.orientation);
                        TextShapingFailure::with_receipt(
                            horizontal_recovery::direct_failure_layout_error(receipt),
                            receipt,
                        )
                    })?;

            let measured_width = lines
                .iter()
                .map(|line| line.measured_width)
                .fold(0.0_f32, f32::max);
            let measured_height = lines.iter().map(|line| line.line_height).sum::<f32>();
            let mut shaped = ShapedGlyphRun {
                source_text: super::source_profile::materialize_source_text(request),
                source_range: request.source_range,
                unicode_data_snapshot: request.unicode_data_snapshot(),
                primary_face_id: fallback_primary_face(&fallback_spans),
                direction: bidi.resolved_base_direction(),
                orientation: request.orientation,
                vertical_mode: request.vertical_mode,
                include_kerning: request.include_kerning,
                measured_width,
                measured_height,
                horizontal_composition_receipt: None,
                horizontal_line_raw_metrics: Vec::new(),
                horizontal_glyph_metric_spans: Vec::new(),
                lines,
            };
            if horizontal_partial.is_none() {
                if let Some(first_failure) = alternate_backend_receipt {
                    shaped.horizontal_composition_receipt =
                        Some(Box::new(TextHorizontalCompositionReceipt {
                            alternate_ranges: Vec::new(),
                            first_failure,
                        }));
                }
            }
            if let Some(pending) = horizontal_partial {
                shaped = pending.compose_or_retain_alternate(
                    shaped,
                    font_database,
                    request.style.font_size,
                    line_height,
                    request.text.len(),
                );
            }
            apply_vertical_layout(&mut shaped, request, Some(font_database));
            shaped_run_has_raster_faces(&shaped)
                .then_some(shaped)
                .ok_or_else(|| {
                    TextShapingFailure::with_optional_receipt(
                        TextLayoutError::FallbackExhausted,
                        alternate_backend_receipt,
                    )
                })
        },
    );
    emit_slow_cosmic_profile(profile_shape, "shape-total", shape_started, request.text);
    shaped
        .map(|run| TextShapingCompletion::new(run, request_diagnostics))
        .map_err(|failure| failure.with_request_diagnostics(request_diagnostics))
}

fn shaped_run_has_raster_faces(shaped: &ShapedGlyphRun) -> bool {
    shaped
        .lines
        .iter()
        .flat_map(|line| &line.glyphs)
        .all(|glyph| {
            glyph.cluster_flags.virtual_glyph
                || glyph.cluster_flags.whitespace
                || glyph.cluster_flags.space
                || glyph.cluster_flags.tab
                || glyph.font_id.is_some()
        })
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

fn line_from_layout_run(
    request: BackendShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    run: &glyphon::LayoutRun<'_>,
    line_visual_start: usize,
    line_breaks: &LineBreakOpportunityMap,
    analysis: &ParagraphTextAnalysis,
    bidi: &BidiParagraph<'_>,
    fallback_spans: &[super::fallback_spans::FallbackTextSpan],
    font_database: &FontDatabase,
) -> Result<ShapedHardLine, TextLayoutError> {
    let line_visual_end = line_visual_start
        .checked_add(run.text.len())
        .ok_or(TextLayoutError::BidiInvariant)?;
    let shaping_text = text_view.shaping_text();
    if line_visual_start > line_visual_end
        || line_visual_end > shaping_text.len()
        || !shaping_text.is_char_boundary(line_visual_start)
        || !shaping_text.is_char_boundary(line_visual_end)
    {
        return Err(TextLayoutError::BidiInvariant);
    }
    let line_shaping_range = line_visual_start..line_visual_end;
    let line_source_range = text_view.source_range_for_shaping_range(line_shaping_range);
    let line_source_start = request
        .source_range
        .start
        .checked_add(line_source_range.start)
        .ok_or(TextLayoutError::BidiInvariant)?;
    let line_source_end = request
        .source_range
        .start
        .checked_add(line_source_range.end)
        .ok_or(TextLayoutError::BidiInvariant)?;
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
                line_visual_end,
                cluster_start,
                line_breaks,
                analysis,
                bidi,
                fallback_spans,
                font_database,
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| TextLayoutError::BidiInvariant)?;

    Ok(ShapedHardLine {
        line_index: run.line_i,
        source_range: TextRange {
            start: line_source_start,
            end: line_source_end,
        },
        visual_range,
        measured_width: run.line_w.max(0.0),
        baseline: cosmic_line_baseline(run.line_y, run.line_top, run.line_height),
        line_height: run.line_height.max(resolved_line_height(request)),
        glyphs,
    })
}

fn glyph_from_layout_glyph(
    request: BackendShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    glyph: &LayoutGlyph,
    run_rtl: bool,
    line_visual_start: usize,
    line_visual_end: usize,
    cluster_start: bool,
    line_breaks: &LineBreakOpportunityMap,
    analysis: &ParagraphTextAnalysis,
    bidi: &BidiParagraph<'_>,
    fallback_spans: &[super::fallback_spans::FallbackTextSpan],
    font_database: &FontDatabase,
) -> Result<ShapedGlyph, super::bidi::BidiInvariantError> {
    let line_len = line_visual_end.checked_sub(line_visual_start).ok_or(
        super::bidi::BidiInvariantError::InvalidResolvedRange {
            start: line_visual_start,
            end: line_visual_end,
        },
    )?;
    if glyph.start > glyph.end || glyph.end > line_len {
        return Err(super::bidi::BidiInvariantError::InvalidResolvedRange {
            start: glyph.start,
            end: glyph.end,
        });
    }
    let shaping_start = line_visual_start.checked_add(glyph.start).ok_or(
        super::bidi::BidiInvariantError::InvalidResolvedRange {
            start: glyph.start,
            end: glyph.end,
        },
    )?;
    let shaping_end = line_visual_start.checked_add(glyph.end).ok_or(
        super::bidi::BidiInvariantError::InvalidResolvedRange {
            start: glyph.start,
            end: glyph.end,
        },
    )?;
    let shaping_range = shaping_start..shaping_end;
    let shaping_text = text_view.shaping_text();
    if shaping_start > shaping_end
        || shaping_end > shaping_text.len()
        || !shaping_text.is_char_boundary(shaping_start)
        || !shaping_text.is_char_boundary(shaping_end)
    {
        return Err(super::bidi::BidiInvariantError::InvalidResolvedRange {
            start: shaping_start,
            end: shaping_end,
        });
    }
    let projected_source_range = text_view.source_range_for_shaping_range(shaping_range.clone());
    let source_range = absolute_range(
        request.source_range.start,
        projected_source_range.start,
        projected_source_range.end,
    )
    .ok_or(super::bidi::BidiInvariantError::InvalidResolvedRange {
        start: projected_source_range.start,
        end: projected_source_range.end,
    })?;
    let cluster_text = shaping_text.get(shaping_range.clone()).ok_or(
        super::bidi::BidiInvariantError::InvalidResolvedRange {
            start: shaping_range.start,
            end: shaping_range.end,
        },
    )?;
    let local_range = TextRange {
        start: line_visual_start + glyph.start,
        end: line_visual_start + glyph.end,
    };
    let bidi_level = bidi.level_for_range(local_range)?;
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
    let script = analysis.shaped_script_for_range(local_range);

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
            .filter(|span| span.resolution.face() == face)
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
    Ok(ShapedGlyph {
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
    })
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
        source_text: super::source_profile::materialize_source_text(request),
        source_range: request.source_range,
        unicode_data_snapshot: request.unicode_data_snapshot(),
        primary_face_id: None,
        direction: bidi.resolved_base_direction(),
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width: 0.0,
        measured_height: line_height,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines: vec![ShapedHardLine {
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
    let mandatory_control = contains_mandatory_break_control(cluster_text);
    ShapedGlyphClusterFlags {
        cluster_start,
        rtl: matches!(direction, TextDirection::RightToLeft),
        whitespace: cluster_text.chars().any(char::is_whitespace),
        space: cluster_text
            .chars()
            .any(|ch| matches!(ch, ' ' | '\u{00a0}')),
        tab: cluster_text.contains('\t'),
        mandatory_break: line_breaks.mandatory_break || mandatory_control,
        soft_break: line_breaks.soft_break,
        virtual_glyph: cluster_text.chars().any(char::is_control),
        break_safety: Default::default(),
        line_break: line_breaks.receipt_for_cluster(cluster_start, mandatory_control),
        vertical_decision: None,
    }
}

fn attrs_for_style<'a>(request: BackendShapeRequest<'a>) -> Attrs<'a> {
    let attrs = match request
        .style
        .font_family
        .as_deref()
        .map(str::trim)
        .filter(|family| !family.is_empty())
    {
        Some(family) => Attrs::new().family(Family::Name(family)),
        None => Attrs::new(),
    };
    let attrs = attrs.weight(Weight(TextStyle::normalized_font_weight(
        request.style.font_weight,
    )));
    let attrs = if request.style.italic {
        attrs.style(CosmicStyle::Italic)
    } else {
        attrs
    };
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

fn absolute_range(
    source_start: usize,
    visual_start: usize,
    visual_end: usize,
) -> Option<TextRange> {
    Some(TextRange {
        start: source_start.checked_add(visual_start)?,
        end: source_start.checked_add(visual_end.max(visual_start))?,
    })
}

#[cfg(test)]
mod tests;
