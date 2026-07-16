use crate::core::math::Vec2;
use crate::text::shaping::TextShapeRunProvider;
use crate::text::{
    InlineBaseline, InlineObjectRef, LaidOutLine, LaidOutText, LayoutItem, RichParseResult,
};
use crate::text::{TextRange, TextStyle};
use unicode_segmentation::UnicodeSegmentation;

use super::{
    line_break_chunks_with_provider, line_metrics_with_provider,
    measure_text_source_range_width_with_provider, trim_leading_wrap_spaces,
    word_smart_line_break_chunks_with_provider,
};

pub(crate) fn layout_rich_line_with_provider<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    provider: &mut P,
) -> LaidOutText
where
    P: TextShapeRunProvider + ?Sized,
{
    let text_metrics = line_metrics_with_provider(style, provider);
    let mut text_ascent = text_metrics.baseline.max(0.0);
    let mut text_descent = (text_metrics.line_height - text_ascent).max(0.0);
    let mut text_run_metrics = Vec::with_capacity(parsed.runs.len());
    for run in &parsed.runs {
        let metrics = run.inline.is_none().then(|| {
            let style = resolve_rich_run_style(style, &run.style);
            let metrics = line_metrics_with_provider(&style, provider);
            let ascent = metrics.baseline.max(0.0);
            let descent = (metrics.line_height - ascent).max(0.0);
            TextRunMetrics {
                style,
                ascent,
                descent,
            }
        });
        if let Some(metrics) = &metrics {
            text_ascent = text_ascent.max(metrics.ascent);
            text_descent = text_descent.max(metrics.descent);
        }
        text_run_metrics.push(metrics);
    }
    let mut ascent = text_ascent;
    let mut descent = text_descent;
    let mut inline_metrics = Vec::with_capacity(parsed.runs.len());

    for run in &parsed.runs {
        let metrics = run
            .inline
            .as_ref()
            .map(|inline| inline_box_metrics(inline, text_ascent, text_descent));
        if let Some(metrics) = metrics {
            ascent = ascent.max(metrics.ascent);
            descent = descent.max(metrics.descent);
        }
        inline_metrics.push(metrics);
    }

    let mut items = Vec::with_capacity(parsed.runs.len());
    let mut cursor_x = 0.0;
    let line_baseline = ascent;
    for (run_index, run) in parsed.runs.iter().enumerate() {
        let Some(source_range) = ui_range(run.byte_range) else {
            continue;
        };
        if let (Some(inline), Some(metrics)) = (&run.inline, inline_metrics[run_index]) {
            let origin_y = inline_origin_y(metrics, line_baseline, ascent + descent);
            items.push(LayoutItem::Inline {
                run_index: u32::try_from(run_index).unwrap_or(u32::MAX),
                source_range: run.byte_range,
                object: inline.clone(),
                size: metrics.size,
                baseline: metrics.baseline,
                origin: Vec2::new(cursor_x, origin_y),
                advance: metrics.advance,
            });
            cursor_x += metrics.advance;
            continue;
        }
        let Some(text) = parsed.text.get(source_range.start..source_range.end) else {
            continue;
        };
        let Some(run_metrics) = text_run_metrics[run_index].as_ref() else {
            continue;
        };
        let advance = measure_text_source_range_width_with_provider(
            text,
            &run_metrics.style,
            TextRange {
                start: 0,
                end: text.len(),
            },
            provider,
        );
        items.push(LayoutItem::Text {
            run_index: u32::try_from(run_index).unwrap_or(u32::MAX),
            source_range: run.byte_range,
            origin: Vec2::new(cursor_x, line_baseline - run_metrics.ascent),
            advance,
        });
        cursor_x += advance;
    }

    let line_height = ascent + descent;
    let item_count = u32::try_from(items.len()).unwrap_or(u32::MAX);
    LaidOutText {
        items,
        lines: vec![LaidOutLine {
            item_range: (0, item_count),
            origin: Vec2::new(0.0, 0.0),
            baseline: line_baseline,
            width: cursor_x,
            ascent,
            descent,
        }],
        size: Vec2::new(cursor_x, line_height),
    }
}

pub(crate) fn layout_rich_text_with_provider<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    provider: &mut P,
) -> LaidOutText
where
    P: TextShapeRunProvider + ?Sized,
{
    layout_rich_ranges_with_provider(
        parsed,
        style,
        rich_forced_line_ranges(&parsed.text),
        provider,
    )
}

pub(crate) fn layout_rich_text_glyph_wrapped_with_provider<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    max_width: f32,
    provider: &mut P,
) -> LaidOutText
where
    P: TextShapeRunProvider + ?Sized,
{
    let ranges = rich_glyph_line_ranges_with_provider(parsed, style, max_width, provider);
    layout_rich_ranges_with_provider(parsed, style, ranges, provider)
}

pub(crate) fn layout_rich_text_word_wrapped_with_provider<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    max_width: f32,
    mode: RichWordWrapMode,
    provider: &mut P,
) -> (LaidOutText, Vec<(u32, u32)>)
where
    P: TextShapeRunProvider + ?Sized,
{
    let ranges = rich_word_line_ranges_with_provider(parsed, style, max_width, mode, provider);
    let layout = layout_rich_ranges_with_provider(parsed, style, ranges.clone(), provider);
    (layout, ranges)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RichWordWrapMode {
    Word,
    WordSmart,
}

fn layout_rich_ranges_with_provider<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    source_ranges: Vec<(u32, u32)>,
    provider: &mut P,
) -> LaidOutText
where
    P: TextShapeRunProvider + ?Sized,
{
    let mut items = Vec::new();
    let mut lines = Vec::new();
    let mut cursor_y = 0.0;
    let mut max_width = 0.0_f32;

    for source_range in source_ranges {
        let mut run_indices = Vec::new();
        let runs = parsed
            .runs
            .iter()
            .enumerate()
            .filter_map(|(run_index, run)| {
                let start = run.byte_range.0.max(source_range.0);
                let end = run.byte_range.1.min(source_range.1);
                (start < end).then(|| {
                    run_indices.push(run_index);
                    let mut run = run.clone();
                    run.byte_range = (start, end);
                    run
                })
            })
            .collect();
        let line_parsed = RichParseResult {
            text: parsed.text.clone(),
            runs,
            paragraphs: Vec::new(),
            tables: Vec::new(),
        };
        let mut line_layout = layout_rich_line_with_provider(&line_parsed, style, provider);
        let item_start = u32::try_from(items.len()).unwrap_or(u32::MAX);
        for item in &mut line_layout.items {
            let local_run_index = match item {
                LayoutItem::Text { run_index, .. } | LayoutItem::Inline { run_index, .. } => {
                    *run_index
                }
            };
            let original_run_index = usize::try_from(local_run_index)
                .ok()
                .and_then(|index| run_indices.get(index))
                .copied()
                .unwrap_or(usize::MAX);
            match item {
                LayoutItem::Text { run_index, .. } | LayoutItem::Inline { run_index, .. } => {
                    *run_index = u32::try_from(original_run_index).unwrap_or(u32::MAX);
                }
            }
        }
        items.append(&mut line_layout.items);
        let item_end = u32::try_from(items.len()).unwrap_or(u32::MAX);
        let line = line_layout.lines.into_iter().next().unwrap_or_default();
        let line_height = line.ascent + line.descent;
        max_width = max_width.max(line.width);
        lines.push(LaidOutLine {
            item_range: (item_start, item_end),
            origin: Vec2::new(0.0, cursor_y),
            ..line
        });
        cursor_y += line_height;
    }

    LaidOutText {
        items,
        lines,
        size: Vec2::new(max_width, cursor_y),
    }
}

pub(crate) fn rich_glyph_line_ranges_with_provider<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    max_width: f32,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let text_metrics = line_metrics_with_provider(style, provider);
    let text_ascent = text_metrics.baseline.max(0.0);
    let text_descent = (text_metrics.line_height - text_ascent).max(0.0);
    let mut ranges = Vec::new();

    for forced_range in rich_forced_line_ranges(&parsed.text) {
        let start = usize::try_from(forced_range.0).unwrap_or(usize::MAX);
        let end = usize::try_from(forced_range.1).unwrap_or(usize::MAX);
        let Some(text) = parsed.text.get(start..end) else {
            continue;
        };
        let mut line_start = start;
        let mut line_width = 0.0_f32;
        for (offset, grapheme) in text.grapheme_indices(true) {
            let grapheme_start = start + offset;
            let grapheme_end = grapheme_start + grapheme.len();
            let advance = rich_grapheme_advance_with_provider(
                parsed,
                style,
                grapheme,
                grapheme_start,
                text_ascent,
                text_descent,
                provider,
            );
            if grapheme_start > line_start && line_width + advance > max_width {
                ranges.push((
                    u32::try_from(line_start).unwrap_or(u32::MAX),
                    u32::try_from(grapheme_start).unwrap_or(u32::MAX),
                ));
                line_start = grapheme_start;
                line_width = 0.0;
            }
            line_width += advance;
            if grapheme_end == end {
                ranges.push((
                    u32::try_from(line_start).unwrap_or(u32::MAX),
                    u32::try_from(end).unwrap_or(u32::MAX),
                ));
            }
        }
        if text.is_empty() {
            ranges.push(forced_range);
        }
    }
    ranges
}

pub(crate) fn rich_word_line_ranges_with_provider<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    max_width: f32,
    mode: RichWordWrapMode,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    P: TextShapeRunProvider + ?Sized,
{
    let max_width = finite_non_negative(max_width);
    let text_metrics = line_metrics_with_provider(style, provider);
    let text_ascent = text_metrics.baseline.max(0.0);
    let text_descent = (text_metrics.line_height - text_ascent).max(0.0);
    let mut ranges = Vec::new();

    for forced_range in rich_forced_line_ranges(&parsed.text) {
        let start = usize::try_from(forced_range.0).unwrap_or(usize::MAX);
        let end = usize::try_from(forced_range.1).unwrap_or(usize::MAX);
        let Some(text) = parsed.text.get(start..end) else {
            continue;
        };
        let chunks = match mode {
            RichWordWrapMode::Word => line_break_chunks_with_provider(text, style, provider),
            RichWordWrapMode::WordSmart => {
                word_smart_line_break_chunks_with_provider(text, style, provider)
            }
        };
        if chunks.is_empty() {
            ranges.push(forced_range);
            continue;
        }

        let mut line_start = start;
        let mut line_end = start;
        let mut line_width = 0.0_f32;
        for chunk in chunks {
            let mut chunk_start = start + chunk.source_range.start;
            let chunk_end = start + chunk.source_range.end;
            if line_end == line_start {
                chunk_start = trim_rich_leading_spaces(&parsed.text, chunk_start, chunk_end);
                line_start = chunk_start;
                line_end = chunk_start;
            }
            if chunk_start >= chunk_end {
                continue;
            }
            let chunk_width = rich_source_range_advance_with_provider(
                parsed,
                style,
                chunk_start,
                chunk_end,
                text_ascent,
                text_descent,
                provider,
            );
            if line_end > line_start && line_width + chunk_width > max_width {
                ranges.push((
                    u32::try_from(line_start).unwrap_or(u32::MAX),
                    u32::try_from(line_end).unwrap_or(u32::MAX),
                ));
                chunk_start = trim_rich_leading_spaces(&parsed.text, chunk_start, chunk_end);
                line_start = chunk_start;
                line_end = chunk_start;
                line_width = 0.0;
            }
            if chunk_start >= chunk_end {
                continue;
            }

            let chunk_width = rich_source_range_advance_with_provider(
                parsed,
                style,
                chunk_start,
                chunk_end,
                text_ascent,
                text_descent,
                provider,
            );
            if line_end == line_start && chunk_width > max_width && chunk.allow_glyph_fallback {
                let fallback_ranges = rich_glyph_ranges_for_source_range_with_provider(
                    parsed,
                    style,
                    chunk_start,
                    chunk_end,
                    max_width,
                    text_ascent,
                    text_descent,
                    provider,
                );
                let fallback_count = fallback_ranges.len();
                for (index, range) in fallback_ranges.into_iter().enumerate() {
                    if index + 1 == fallback_count {
                        line_start = usize::try_from(range.0).unwrap_or(chunk_start);
                        line_end = usize::try_from(range.1).unwrap_or(chunk_end);
                        line_width = rich_source_range_advance_with_provider(
                            parsed,
                            style,
                            line_start,
                            line_end,
                            text_ascent,
                            text_descent,
                            provider,
                        );
                    } else {
                        ranges.push(range);
                    }
                }
                continue;
            }
            line_end = chunk_end;
            line_width += chunk_width;
        }
        if line_end > line_start {
            ranges.push((
                u32::try_from(line_start).unwrap_or(u32::MAX),
                u32::try_from(line_end).unwrap_or(u32::MAX),
            ));
        }
    }
    ranges
}

#[allow(clippy::too_many_arguments)]
fn rich_glyph_ranges_for_source_range_with_provider<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    start: usize,
    end: usize,
    max_width: f32,
    text_ascent: f32,
    text_descent: f32,
    provider: &mut P,
) -> Vec<(u32, u32)>
where
    P: TextShapeRunProvider + ?Sized,
{
    let Some(text) = parsed.text.get(start..end) else {
        return Vec::new();
    };
    let mut ranges = Vec::new();
    let mut line_start = start;
    let mut line_width = 0.0_f32;
    for (offset, grapheme) in text.grapheme_indices(true) {
        let grapheme_start = start + offset;
        let grapheme_end = grapheme_start + grapheme.len();
        let advance = rich_grapheme_advance_with_provider(
            parsed,
            style,
            grapheme,
            grapheme_start,
            text_ascent,
            text_descent,
            provider,
        );
        if grapheme_start > line_start && line_width + advance > max_width {
            ranges.push((
                u32::try_from(line_start).unwrap_or(u32::MAX),
                u32::try_from(grapheme_start).unwrap_or(u32::MAX),
            ));
            line_start = grapheme_start;
            line_width = 0.0;
        }
        line_width += advance;
        if grapheme_end == end {
            ranges.push((
                u32::try_from(line_start).unwrap_or(u32::MAX),
                u32::try_from(end).unwrap_or(u32::MAX),
            ));
        }
    }
    ranges
}

#[allow(clippy::too_many_arguments)]
fn rich_source_range_advance_with_provider<P>(
    parsed: &RichParseResult,
    style: &TextStyle,
    start: usize,
    end: usize,
    text_ascent: f32,
    text_descent: f32,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    parsed
        .text
        .get(start..end)
        .map(|text| {
            text.grapheme_indices(true)
                .map(|(offset, grapheme)| {
                    rich_grapheme_advance_with_provider(
                        parsed,
                        style,
                        grapheme,
                        start + offset,
                        text_ascent,
                        text_descent,
                        provider,
                    )
                })
                .sum()
        })
        .unwrap_or(0.0)
}

fn trim_rich_leading_spaces(text: &str, start: usize, end: usize) -> usize {
    text.get(start..end)
        .map(|candidate| trim_leading_wrap_spaces(candidate, start).1)
        .unwrap_or(start)
}

#[allow(clippy::too_many_arguments)]
fn rich_grapheme_advance_with_provider<P>(
    parsed: &RichParseResult,
    base_style: &TextStyle,
    grapheme: &str,
    source_start: usize,
    text_ascent: f32,
    text_descent: f32,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    let run = parsed.runs.iter().find(|run| {
        usize::try_from(run.byte_range.0)
            .ok()
            .is_some_and(|start| start <= source_start)
            && usize::try_from(run.byte_range.1)
                .ok()
                .is_some_and(|end| source_start < end)
    });
    if let Some(inline) = run.and_then(|run| run.inline.as_ref()) {
        return inline_box_metrics(inline, text_ascent, text_descent).advance;
    }
    let style = run
        .map(|run| resolve_rich_run_style(base_style, &run.style))
        .unwrap_or_else(|| base_style.clone());
    measure_text_source_range_width_with_provider(
        grapheme,
        &style,
        TextRange {
            start: 0,
            end: grapheme.len(),
        },
        provider,
    )
}

pub(crate) fn rich_forced_line_ranges(text: &str) -> Vec<(u32, u32)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            ranges.push((
                u32::try_from(start).unwrap_or(u32::MAX),
                u32::try_from(index).unwrap_or(u32::MAX),
            ));
            start = index + ch.len_utf8();
        }
    }
    ranges.push((
        u32::try_from(start).unwrap_or(u32::MAX),
        u32::try_from(text.len()).unwrap_or(u32::MAX),
    ));
    ranges
}

#[derive(Clone, Debug)]
struct TextRunMetrics {
    style: TextStyle,
    ascent: f32,
    descent: f32,
}

pub(crate) fn resolve_rich_run_style(
    base: &TextStyle,
    override_style: &crate::text::StyleOverride,
) -> TextStyle {
    let mut style = base.clone();
    if let Some(weight) = override_style.weight {
        style.font_weight = TextStyle::normalized_font_weight(weight);
    }
    if let Some(font_size) = override_style
        .font_size
        .filter(|size| size.is_finite() && *size > 0.0)
    {
        let line_height_scale = base.line_height / base.font_size.max(1.0);
        style.font_size = font_size;
        style.line_height = font_size * line_height_scale;
    }
    if let Some(family) = override_style
        .family
        .as_ref()
        .filter(|family| !family.is_empty())
    {
        style.font_family = Some(family.as_str().to_string());
    }
    style
}

#[derive(Clone, Copy, Debug)]
struct InlineBoxMetrics {
    advance: f32,
    size: Vec2,
    ascent: f32,
    descent: f32,
    baseline: InlineBaseline,
}

fn inline_box_metrics(
    inline: &InlineObjectRef,
    text_ascent: f32,
    text_descent: f32,
) -> InlineBoxMetrics {
    let (size, baseline) = match inline {
        InlineObjectRef::Image { size, baseline, .. } => (*size, *baseline),
        InlineObjectRef::Widget { size, .. } => (*size, InlineBaseline::Baseline),
        InlineObjectRef::Icon { .. } => (
            Vec2::new(text_ascent + text_descent, text_ascent + text_descent),
            InlineBaseline::Baseline,
        ),
    };
    let size = Vec2::new(finite_non_negative(size.x), finite_non_negative(size.y));
    let (ascent, descent) = match baseline {
        InlineBaseline::Baseline => (size.y, 0.0),
        InlineBaseline::Center => (size.y * 0.5, size.y * 0.5),
        InlineBaseline::Top => (text_ascent, (size.y - text_ascent).max(0.0)),
        InlineBaseline::Bottom => ((size.y - text_descent).max(0.0), text_descent),
    };
    InlineBoxMetrics {
        advance: size.x,
        size,
        ascent,
        descent,
        baseline,
    }
}

fn inline_origin_y(metrics: InlineBoxMetrics, baseline: f32, line_height: f32) -> f32 {
    match metrics.baseline {
        InlineBaseline::Baseline => baseline - metrics.size.y,
        InlineBaseline::Center => (line_height - metrics.size.y) * 0.5,
        InlineBaseline::Top => 0.0,
        InlineBaseline::Bottom => line_height - metrics.size.y,
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn ui_range(range: (u32, u32)) -> Option<TextRange> {
    Some(TextRange {
        start: usize::try_from(range.0).ok()?,
        end: usize::try_from(range.1).ok()?,
    })
}

#[cfg(test)]
mod tests;
