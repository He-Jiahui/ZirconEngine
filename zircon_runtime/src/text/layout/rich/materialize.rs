use std::ops::Range;

use crate::core::math::Vec2;
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome};
use crate::text::{LaidOutLine, LaidOutText, LayoutItem, TextRange, TextStyle};

use super::super::line_metrics_with_provider;
use super::metrics::{inline_box_metrics, inline_origin_y};
use super::{RichAdvanceIndex, RichTextLayoutSource, resolve_rich_run_style};

pub(crate) fn layout_rich_line_with_provider<S, P>(
    source: &S,
    style: &TextStyle,
    provider: &mut P,
) -> TextLayoutOutcome<LaidOutText>
where
    S: RichTextLayoutSource + ?Sized,
    P: TextShapeRunProvider + ?Sized,
{
    HorizontalRichLayoutIndex::new(source, style, provider).and_then(|index| {
        let end = match u32::try_from(source.text().len()) {
            Ok(end) => end,
            Err(_) => {
                return TextShapingOutcome::failed(
                    crate::core::framework::text::TextLayoutError::LayoutFailed,
                );
            }
        };
        TextShapingOutcome::Ready(layout_rich_source_range(
            source,
            (0, end),
            0..source.run_count(),
            &index,
        ))
    })
}

pub(super) fn layout_rich_ranges_with_index<S>(
    source: &S,
    source_ranges: Vec<(u32, u32)>,
    index: &HorizontalRichLayoutIndex,
) -> LaidOutText
where
    S: RichTextLayoutSource + ?Sized,
{
    let mut items = Vec::new();
    let mut lines = Vec::new();
    let mut cursor_y = 0.0;
    let mut max_width = 0.0_f32;
    let mut run_cursor = 0_usize;

    for source_range in source_ranges {
        while source
            .run(run_cursor)
            .is_some_and(|run| run.byte_range.1 <= source_range.0)
        {
            run_cursor = run_cursor.saturating_add(1);
        }
        let mut run_end = run_cursor;
        while source
            .run(run_end)
            .is_some_and(|run| run.byte_range.0 < source_range.1)
        {
            run_end = run_end.saturating_add(1);
        }
        let mut line_layout =
            layout_rich_source_range(source, source_range, run_cursor..run_end, index);
        let item_start = u32::try_from(items.len()).unwrap_or(u32::MAX);
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

fn layout_rich_source_range<S>(
    source: &S,
    source_range: (u32, u32),
    run_range: Range<usize>,
    index: &HorizontalRichLayoutIndex,
) -> LaidOutText
where
    S: RichTextLayoutSource + ?Sized,
{
    let mut text_ascent = index.base_ascent;
    let mut text_descent = index.base_descent;
    for local_run_index in run_range.clone() {
        let Some(run) = source.run(local_run_index) else {
            continue;
        };
        if clipped_run_range(run.byte_range, source_range).is_some() {
            if let Some(metrics) = index
                .run_metrics
                .get(local_run_index)
                .and_then(Option::as_ref)
            {
                text_ascent = text_ascent.max(metrics.ascent);
                text_descent = text_descent.max(metrics.descent);
            }
        }
    }
    let mut ascent = text_ascent;
    let mut descent = text_descent;
    let mut inline_metrics = Vec::with_capacity(run_range.len());

    for local_run_index in run_range.clone() {
        let Some(run) = source.run(local_run_index) else {
            inline_metrics.push(None);
            continue;
        };
        let metrics = clipped_run_range(run.byte_range, source_range)
            .and_then(|_| run.inline)
            .map(|inline| inline_box_metrics(inline, text_ascent, text_descent));
        if let Some(metrics) = metrics {
            ascent = ascent.max(metrics.ascent);
            descent = descent.max(metrics.descent);
        }
        inline_metrics.push(metrics);
    }

    let mut items = Vec::with_capacity(run_range.len());
    let mut cursor_x = 0.0;
    let line_baseline = ascent;
    for (inline_metric_index, local_run_index) in run_range.clone().enumerate() {
        let Some(run) = source.run(local_run_index) else {
            continue;
        };
        let Some(clipped_range) = clipped_run_range(run.byte_range, source_range) else {
            continue;
        };
        let Some(text_range) = ui_range(clipped_range) else {
            continue;
        };
        if let (Some(inline), Some(metrics)) = (run.inline, inline_metrics[inline_metric_index]) {
            let origin_y = inline_origin_y(metrics, line_baseline, ascent + descent);
            items.push(LayoutItem::Inline {
                run_index: run.source_index,
                source_range: clipped_range,
                object: inline.clone(),
                size: metrics.size,
                baseline: metrics.baseline,
                origin: Vec2::new(cursor_x, origin_y),
                advance: metrics.advance,
            });
            cursor_x += metrics.advance;
            continue;
        }
        if source
            .text()
            .get(text_range.start..text_range.end)
            .is_none()
        {
            continue;
        }
        let Some(run_metrics) = index
            .run_metrics
            .get(local_run_index)
            .and_then(Option::as_ref)
        else {
            continue;
        };
        let advance = index.advances.advance(text_range.start, text_range.end);
        items.push(LayoutItem::Text {
            run_index: run.source_index,
            source_range: clipped_range,
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

#[derive(Clone, Debug)]
struct TextRunMetrics {
    style: TextStyle,
    ascent: f32,
    descent: f32,
}

pub(super) struct HorizontalRichLayoutIndex {
    pub(super) advances: RichAdvanceIndex,
    base_ascent: f32,
    base_descent: f32,
    run_metrics: Vec<Option<TextRunMetrics>>,
}

impl HorizontalRichLayoutIndex {
    pub(super) fn new<S, P>(
        source: &S,
        style: &TextStyle,
        provider: &mut P,
    ) -> TextLayoutOutcome<Self>
    where
        S: RichTextLayoutSource + ?Sized,
        P: TextShapeRunProvider + ?Sized,
    {
        let base_line_metrics = match line_metrics_with_provider(style, provider) {
            TextShapingOutcome::Ready(metrics) => metrics,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        let base_ascent = base_line_metrics.baseline.max(0.0);
        let base_descent = (base_line_metrics.line_height - base_ascent).max(0.0);
        let advances = match RichAdvanceIndex::new(source, style, provider, |inline, _| {
            let metrics = inline_box_metrics(inline, base_ascent, base_descent);
            (metrics.advance, metrics.size.y)
        }) {
            TextShapingOutcome::Ready(index) => index,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        let base_metrics = TextRunMetrics {
            style: style.clone(),
            ascent: base_ascent,
            descent: base_descent,
        };
        let mut run_metrics = Vec::with_capacity(source.run_count());
        let mut previous_text_run: Option<((u32, u32), TextRunMetrics)> = None;
        for index in 0..source.run_count() {
            let Some(run) = source.run(index) else {
                run_metrics.push(None);
                previous_text_run = None;
                continue;
            };
            if run.inline.is_some() {
                run_metrics.push(None);
                previous_text_run = None;
                continue;
            }

            let run_style = resolve_rich_run_style(style, run.style);
            let metrics = if run_style == base_metrics.style {
                base_metrics.clone()
            } else if let Some(previous_metrics) =
                previous_text_run
                    .as_ref()
                    .and_then(|(previous_range, previous_metrics)| {
                        (previous_range.1 == run.byte_range.0
                            && previous_metrics.style == run_style)
                            .then_some(previous_metrics)
                    })
            {
                previous_metrics.clone()
            } else {
                let line_metrics = match line_metrics_with_provider(&run_style, provider) {
                    TextShapingOutcome::Ready(metrics) => metrics,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                };
                let ascent = line_metrics.baseline.max(0.0);
                TextRunMetrics {
                    style: run_style,
                    ascent,
                    descent: (line_metrics.line_height - ascent).max(0.0),
                }
            };
            previous_text_run = Some((run.byte_range, metrics.clone()));
            run_metrics.push(Some(metrics));
        }

        TextShapingOutcome::Ready(Self {
            advances,
            base_ascent,
            base_descent,
            run_metrics,
        })
    }
}

fn ui_range(range: (u32, u32)) -> Option<TextRange> {
    Some(TextRange {
        start: usize::try_from(range.0).ok()?,
        end: usize::try_from(range.1).ok()?,
    })
}

fn clipped_run_range(run_range: (u32, u32), source_range: (u32, u32)) -> Option<(u32, u32)> {
    let start = run_range.0.max(source_range.0);
    let end = run_range.1.min(source_range.1);
    (start < end).then_some((start, end))
}
