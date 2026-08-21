use std::cmp::Ordering;
use std::collections::BinaryHeap;

use crate::text::layout::{measure_line_width_with_provider, tab_interval_width};
use crate::text::ParagraphOverride;
use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRange,
};

use super::super::rich_text::{UiParsedText, UiTextSourceRun};
use super::candidate_line::CandidateLine;
use super::direction::is_rtl_direction;
use super::wrapping::wrap_source_run_range_with_line_widths_provider;
use crate::text::text_style;

/// Keeps hostile or malformed indentation from collapsing the logical content box.
const MIN_PARAGRAPH_EXTENT: f32 = 1.0;
const MAX_RESOLVED_INDENT_LEVEL: u16 = 32;

#[derive(Clone, Copy, Debug)]
pub(super) struct ColumnConstraints {
    pub inset: f32,
    pub max_height: f32,
    pub align: UiTextAlign,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct LineConstraints {
    pub inset: f32,
    pub max_width: f32,
    pub align: UiTextAlign,
}

#[derive(Clone, Debug)]
pub(super) struct ResolvedParagraphConstraints<T> {
    paragraphs: Vec<ResolvedPhysicalParagraphConstraints<T>>,
    fallback: T,
}

pub(super) type ResolvedParagraphColumnConstraints =
    ResolvedParagraphConstraints<ColumnConstraints>;
pub(super) type ResolvedParagraphLineConstraints = ResolvedParagraphConstraints<LineConstraints>;

#[derive(Clone, Debug)]
struct ResolvedPhysicalParagraphOverride {
    range: UiTextRange,
    paragraph: ParagraphOverride,
}

#[derive(Clone, Copy, Debug)]
struct ResolvedPhysicalParagraphConstraints<T> {
    range: UiTextRange,
    first: T,
    continuation: T,
}

type ResolvedPhysicalParagraphColumns = ResolvedPhysicalParagraphConstraints<ColumnConstraints>;
type ResolvedPhysicalParagraphLines = ResolvedPhysicalParagraphConstraints<LineConstraints>;

#[derive(Clone, Debug)]
struct ParagraphOverrideSpan {
    range: UiTextRange,
    paragraph: ParagraphOverride,
    list_prefix: Option<UiTextRange>,
    order: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParagraphOwner {
    span_index: usize,
    start: usize,
    end: usize,
    order: usize,
}

impl ParagraphOwner {
    fn from_span(span_index: usize, span: &ParagraphOverrideSpan) -> Self {
        Self {
            span_index,
            start: span.range.start,
            end: span.range.end,
            order: span.order,
        }
    }

    fn span_len(self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

impl Ord for ParagraphOwner {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap is a max-heap: smaller spans are more specific, then later starts win.
        other
            .span_len()
            .cmp(&self.span_len())
            .then_with(|| self.start.cmp(&other.start))
            .then_with(|| self.order.cmp(&other.order))
    }
}

impl PartialOrd for ParagraphOwner {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Copy> ResolvedParagraphConstraints<T> {
    pub(super) fn for_source_offset(&self, source_offset: usize, first_physical_item: bool) -> T {
        self.paragraphs
            .get(
                self.paragraphs
                    .partition_point(|paragraph| paragraph.range.start <= source_offset)
                    .saturating_sub(1),
            )
            .filter(|paragraph| {
                (paragraph.range.start == paragraph.range.end
                    && source_offset == paragraph.range.start)
                    || (paragraph.range.start <= source_offset
                        && source_offset < paragraph.range.end)
            })
            .map(|paragraph| {
                if first_physical_item {
                    paragraph.first
                } else {
                    paragraph.continuation
                }
            })
            .unwrap_or(self.fallback)
    }

    /// Projects consecutive layout items with one forward paragraph cursor.
    pub(super) fn for_candidates(&self, candidates: &[CandidateLine]) -> Vec<T> {
        let mut paragraph_index: usize = 0;
        let mut previous_paragraph = None;
        let mut constraints = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            while paragraph_index.saturating_add(1) < self.paragraphs.len()
                && self.paragraphs[paragraph_index + 1].range.start <= candidate.source_range.start
            {
                paragraph_index = paragraph_index.saturating_add(1);
            }
            let current_paragraph = self
                .paragraphs
                .get(paragraph_index)
                .filter(|paragraph| {
                    (paragraph.range.start == paragraph.range.end
                        && candidate.source_range.start == paragraph.range.start)
                        || (paragraph.range.start <= candidate.source_range.start
                            && candidate.source_range.start < paragraph.range.end)
                })
                .map(|_| paragraph_index);
            let first_physical_item = previous_paragraph != current_paragraph;
            previous_paragraph = current_paragraph;
            constraints.push(
                current_paragraph
                    .map(|index| {
                        let paragraph = &self.paragraphs[index];
                        if first_physical_item {
                            paragraph.first
                        } else {
                            paragraph.continuation
                        }
                    })
                    .unwrap_or(self.fallback),
            );
        }
        constraints
    }
}

pub(super) fn has_block_layout(parsed: &UiParsedText) -> bool {
    parsed.paragraphs().any(|(_, paragraph, _)| {
        paragraph.indent.is_some()
            || paragraph.indent_level.is_some()
            || paragraph.list_prefix.is_some()
            || paragraph.align.is_some()
    })
}

pub(super) fn resolve_paragraph_column_constraints_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_height: f32,
    provider: &mut SharedTextLayoutSession,
) -> ResolvedParagraphColumnConstraints {
    let paragraphs = resolved_physical_paragraph_overrides(parsed)
        .into_iter()
        .map(|paragraph| ResolvedPhysicalParagraphColumns {
            range: paragraph.range,
            first: column_constraints_for_paragraph_with_provider(
                parsed.text(),
                style,
                frame_height,
                &paragraph.paragraph,
                true,
                provider,
            ),
            continuation: column_constraints_for_paragraph_with_provider(
                parsed.text(),
                style,
                frame_height,
                &paragraph.paragraph,
                false,
                provider,
            ),
        })
        .collect();
    ResolvedParagraphColumnConstraints {
        paragraphs,
        fallback: ColumnConstraints {
            inset: 0.0,
            max_height: frame_height.max(0.0),
            align: style.text_align,
        },
    }
}

pub(super) fn resolve_paragraph_line_constraints_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_width: f32,
    provider: &mut SharedTextLayoutSession,
) -> ResolvedParagraphLineConstraints {
    let paragraphs = resolved_physical_paragraph_overrides(parsed)
        .into_iter()
        .map(|paragraph| ResolvedPhysicalParagraphLines {
            range: paragraph.range,
            first: line_constraints_for_paragraph_with_provider(
                parsed.text(),
                style,
                frame_width,
                &paragraph.paragraph,
                true,
                provider,
            ),
            continuation: line_constraints_for_paragraph_with_provider(
                parsed.text(),
                style,
                frame_width,
                &paragraph.paragraph,
                false,
                provider,
            ),
        })
        .collect();
    ResolvedParagraphLineConstraints {
        paragraphs,
        fallback: LineConstraints {
            inset: 0.0,
            max_width: frame_width.max(0.0),
            align: style.text_align,
        },
    }
}

pub(super) fn aligned_column_y(
    frame: UiFrame,
    column_height: f32,
    constraints: ColumnConstraints,
) -> f32 {
    let remaining = (constraints.max_height - column_height).max(0.0);
    let alignment_offset = match constraints.align {
        UiTextAlign::Center => remaining * 0.5,
        UiTextAlign::Right | UiTextAlign::End => remaining,
        UiTextAlign::Left | UiTextAlign::Start | UiTextAlign::Justify => 0.0,
    };
    frame.y + constraints.inset + alignment_offset
}

pub(super) fn wrap_block_paragraphs_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_width: f32,
    provider: &mut SharedTextLayoutSession,
) -> Vec<CandidateLine> {
    let mut lines = Vec::new();
    let mut run_cursor = 0;
    for ResolvedPhysicalParagraphOverride { range, paragraph } in
        resolved_physical_paragraph_overrides(parsed)
    {
        let (first_inset, continuation_inset) =
            paragraph_insets(parsed.text(), &paragraph, style, provider);
        let mut paragraph_lines = wrap_source_run_range_with_line_widths_provider(
            &parsed.runs,
            range,
            &mut run_cursor,
            style.wrap,
            available_width(frame_width, first_inset),
            available_width(frame_width, continuation_inset),
            style,
            provider,
        );
        if paragraph_lines.is_empty() {
            paragraph_lines.push(CandidateLine::empty());
        }
        if let Some(first) = paragraph_lines.first_mut() {
            if first.text.is_empty() {
                first.source_range = range;
            }
        }
        lines.extend(paragraph_lines);
    }
    lines
}

fn column_constraints_for_paragraph_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    frame_height: f32,
    paragraph: &ParagraphOverride,
    first_physical_column: bool,
    provider: &mut SharedTextLayoutSession,
) -> ColumnConstraints {
    let (first_inset, continuation_inset) = paragraph_insets(text, paragraph, style, provider);
    let inset = if first_physical_column {
        first_inset
    } else {
        continuation_inset
    };
    let inset = resolved_inset(frame_height, inset);
    ColumnConstraints {
        inset,
        max_height: available_width(frame_height, inset),
        align: paragraph.align.map(Into::into).unwrap_or(style.text_align),
    }
}

fn line_constraints_for_paragraph_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    frame_width: f32,
    paragraph: &ParagraphOverride,
    first_physical_line: bool,
    provider: &mut SharedTextLayoutSession,
) -> LineConstraints {
    let (first_inset, continuation_inset) = paragraph_insets(text, paragraph, style, provider);
    let inset = if first_physical_line {
        first_inset
    } else {
        continuation_inset
    };
    let inset = resolved_inset(frame_width, inset);
    LineConstraints {
        inset,
        max_width: available_width(frame_width, inset),
        align: paragraph.align.map(Into::into).unwrap_or(style.text_align),
    }
}

pub(super) fn conservative_rich_width_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_width: f32,
    provider: &mut SharedTextLayoutSession,
) -> f32 {
    let spans = paragraph_override_spans(parsed);
    let ranges = spans.iter().map(|span| span.range).collect();
    let maximum_inset = resolve_physical_paragraph_override_spans(ranges, spans)
        .into_iter()
        .map(|paragraph| {
            let (first, continuation) =
                paragraph_insets(parsed.text(), &paragraph.paragraph, style, provider);
            first.max(continuation)
        })
        .fold(0.0_f32, f32::max);
    available_width(frame_width, maximum_inset)
}

#[cfg(test)]
fn physical_paragraph_start(text: &str, offset: usize) -> usize {
    crate::text::hard_line_start(text, offset)
}

pub(super) fn inset_logical_start(
    frame: UiFrame,
    inset: f32,
    direction: UiTextDirection,
) -> UiFrame {
    let inset = resolved_inset(frame.width, inset);
    let x = if is_rtl_direction(direction) {
        frame.x
    } else {
        frame.x + inset
    };
    UiFrame::new(x, frame.y, (frame.width - inset).max(0.0), frame.height)
}

fn paragraph_insets(
    text: &str,
    paragraph: &ParagraphOverride,
    style: &UiResolvedStyle,
    provider: &mut SharedTextLayoutSession,
) -> (f32, f32) {
    let indent_level = paragraph.indent_level.unwrap_or_default();
    let first_indent = paragraph.indent.unwrap_or_default().max(0.0);
    if indent_level == 0 && paragraph.list_prefix.is_none() {
        return (first_indent, 0.0);
    }

    let neutral_style = text_style(style);
    let level_indent = if indent_level == 0 {
        0.0
    } else {
        let space_width = measure_line_width_with_provider(" ", &neutral_style, provider);
        f32::from(indent_level) * tab_interval_width(&neutral_style, space_width)
    };
    let prefix_width = paragraph
        .list_prefix
        .and_then(|range| text.get(range.0 as usize..range.1 as usize))
        .map(|prefix| measure_line_width_with_provider(prefix, &neutral_style, provider))
        .unwrap_or_default();
    (level_indent + first_indent, level_indent + prefix_width)
}

fn resolved_physical_paragraph_overrides(
    parsed: &UiParsedText,
) -> Vec<ResolvedPhysicalParagraphOverride> {
    resolve_physical_paragraph_override_spans(
        physical_paragraph_ranges(parsed.text()),
        paragraph_override_spans(parsed),
    )
}

fn paragraph_override_spans(parsed: &UiParsedText) -> Vec<ParagraphOverrideSpan> {
    parsed
        .paragraphs()
        .enumerate()
        .filter_map(|(order, (range, paragraph, list_prefix))| {
            (range.start < range.end).then(|| ParagraphOverrideSpan {
                range,
                paragraph: paragraph.clone(),
                list_prefix,
                order,
            })
        })
        .collect()
}

fn resolve_physical_paragraph_override_spans(
    physical_ranges: Vec<UiTextRange>,
    mut spans: Vec<ParagraphOverrideSpan>,
) -> Vec<ResolvedPhysicalParagraphOverride> {
    spans.sort_unstable_by_key(|span| (span.range.start, span.range.end, span.order));

    let mut end_order = (0..spans.len()).collect::<Vec<_>>();
    end_order.sort_unstable_by_key(|&index| (spans[index].range.end, spans[index].order));
    let mut active = vec![false; spans.len()];
    let mut next_span = 0;
    let mut next_end = 0;
    let mut indent_level = 0_u32;
    let mut first_indent = 0.0_f32;
    let mut align_owners = BinaryHeap::new();
    let mut prefix_owners = BinaryHeap::new();
    let mut resolved = Vec::with_capacity(physical_ranges.len());

    for range in physical_ranges {
        while next_span < spans.len() && spans[next_span].range.start <= range.start {
            let span = &spans[next_span];
            active[next_span] = true;
            indent_level = indent_level
                .saturating_add(u32::from(span.paragraph.indent_level.unwrap_or_default()));
            first_indent += span.paragraph.indent.unwrap_or_default().max(0.0);
            if span.paragraph.align.is_some() {
                align_owners.push(ParagraphOwner::from_span(next_span, span));
            }
            if span.list_prefix.is_some() {
                prefix_owners.push(ParagraphOwner::from_span(next_span, span));
            }
            next_span = next_span.saturating_add(1);
        }
        while next_end < end_order.len() && spans[end_order[next_end]].range.end <= range.start {
            let span_index = end_order[next_end];
            if active[span_index] {
                let span = &spans[span_index];
                indent_level = indent_level
                    .saturating_sub(u32::from(span.paragraph.indent_level.unwrap_or_default()));
                first_indent -= span.paragraph.indent.unwrap_or_default().max(0.0);
                active[span_index] = false;
            }
            next_end = next_end.saturating_add(1);
        }
        discard_expired_owners(&mut align_owners, &spans, range.start);
        discard_expired_owners(&mut prefix_owners, &spans, range.start);

        let mut paragraph = ParagraphOverride::default();
        paragraph.indent_level = (indent_level > 0).then_some(
            u16::try_from(indent_level)
                .unwrap_or(MAX_RESOLVED_INDENT_LEVEL)
                .min(MAX_RESOLVED_INDENT_LEVEL),
        );
        paragraph.indent = (first_indent > 0.0).then_some(first_indent);
        paragraph.align = align_owners
            .peek()
            .and_then(|owner| spans[owner.span_index].paragraph.align);
        paragraph.list_prefix = prefix_owners.peek().and_then(|owner| {
            spans[owner.span_index]
                .list_prefix
                .map(|prefix| (to_u32(prefix.start), to_u32(prefix.end)))
        });
        resolved.push(ResolvedPhysicalParagraphOverride { range, paragraph });
    }
    resolved
}

fn discard_expired_owners(
    owners: &mut BinaryHeap<ParagraphOwner>,
    spans: &[ParagraphOverrideSpan],
    source_offset: usize,
) {
    while owners
        .peek()
        .is_some_and(|owner| spans[owner.span_index].range.end <= source_offset)
    {
        owners.pop();
    }
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn physical_paragraph_ranges(text: &str) -> Vec<UiTextRange> {
    let mut ranges = Vec::new();
    let mut start = 0;
    crate::text::visit_hard_lines(text, |line| {
        ranges.push(UiTextRange {
            start,
            end: line.content.end,
        });
        start = line.separator.end;
    });
    ranges
}

fn available_width(frame_width: f32, inset: f32) -> f32 {
    (frame_width.max(0.0) - resolved_inset(frame_width, inset)).max(0.0)
}

fn resolved_inset(frame_width: f32, inset: f32) -> f32 {
    let frame_width = frame_width.max(0.0);
    let minimum_extent = MIN_PARAGRAPH_EXTENT.min(frame_width);
    inset.max(0.0).min((frame_width - minimum_extent).max(0.0))
}

#[cfg(test)]
mod tests;
