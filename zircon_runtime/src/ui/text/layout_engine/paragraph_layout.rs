use crate::text::ParagraphOverride;
use crate::text::SharedTextLayoutSession;
use crate::text::layout::{measure_line_width_with_provider, tab_interval_width};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRange,
};

use crate::text::text_style;
use super::super::rich_text::{UiParsedText, UiTextSourceRun};
use super::candidate_line::CandidateLine;
use super::direction::is_rtl_direction;
use super::wrapping::wrap_source_runs_with_line_widths_provider;

/// Keeps hostile or malformed indentation from collapsing the logical content box.
const MIN_PARAGRAPH_EXTENT: f32 = 1.0;
const MAX_RESOLVED_INDENT_LEVEL: u16 = 32;

#[derive(Clone, Copy, Debug)]
pub(super) struct LineConstraints {
    pub inset: f32,
    pub max_width: f32,
    pub align: UiTextAlign,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ColumnConstraints {
    pub inset: f32,
    pub max_height: f32,
    pub align: UiTextAlign,
}

#[derive(Clone, Debug)]
pub(super) struct ResolvedParagraphColumnConstraints {
    paragraphs: Vec<ResolvedPhysicalParagraphColumns>,
    fallback: ColumnConstraints,
}

#[derive(Clone, Copy, Debug)]
struct ResolvedPhysicalParagraphColumns {
    range: UiTextRange,
    first: ColumnConstraints,
    continuation: ColumnConstraints,
}

impl ResolvedParagraphColumnConstraints {
    pub(super) fn for_source_offset(
        &self,
        source_offset: usize,
        first_physical_column: bool,
    ) -> ColumnConstraints {
        self.paragraphs
            .iter()
            .find(|paragraph| {
                (paragraph.range.start == paragraph.range.end
                    && source_offset == paragraph.range.start)
                    || (paragraph.range.start <= source_offset
                        && source_offset < paragraph.range.end)
            })
            .map(|paragraph| {
                if first_physical_column {
                    paragraph.first
                } else {
                    paragraph.continuation
                }
            })
            .unwrap_or(self.fallback)
    }

    pub(super) fn for_column(
        &self,
        text: &str,
        columns: &[CandidateLine],
        index: usize,
    ) -> ColumnConstraints {
        let column = &columns[index];
        let paragraph_start = physical_paragraph_start(text, column.source_range.start);
        let first_physical_column = index == 0
            || physical_paragraph_start(text, columns[index - 1].source_range.start)
                != paragraph_start;
        self.for_source_offset(column.source_range.start, first_physical_column)
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

pub(super) fn column_constraints_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_height: f32,
    source_offset: usize,
    first_physical_column: bool,
    provider: &mut SharedTextLayoutSession,
) -> ColumnConstraints {
    let constraints = line_constraints_with_provider(
        parsed,
        style,
        frame_height,
        source_offset,
        first_physical_column,
        provider,
    );
    ColumnConstraints {
        inset: constraints.inset,
        max_height: constraints.max_width,
        align: constraints.align,
    }
}

pub(super) fn resolve_paragraph_column_constraints_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_height: f32,
    provider: &mut SharedTextLayoutSession,
) -> ResolvedParagraphColumnConstraints {
    let paragraphs = physical_paragraph_ranges(parsed.text())
        .into_iter()
        .map(|range| ResolvedPhysicalParagraphColumns {
            range,
            first: column_constraints_with_provider(
                parsed,
                style,
                frame_height,
                range.start,
                true,
                provider,
            ),
            continuation: column_constraints_with_provider(
                parsed,
                style,
                frame_height,
                range.start,
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

pub(super) fn column_constraints_for_candidate_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_height: f32,
    columns: &[CandidateLine],
    index: usize,
    provider: &mut SharedTextLayoutSession,
) -> ColumnConstraints {
    let column = &columns[index];
    let paragraph_start = physical_paragraph_start(parsed.text(), column.source_range.start);
    let first_physical_column = index == 0
        || physical_paragraph_start(parsed.text(), columns[index - 1].source_range.start)
            != paragraph_start;
    column_constraints_with_provider(
        parsed,
        style,
        frame_height,
        column.source_range.start,
        first_physical_column,
        provider,
    )
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
    for range in physical_paragraph_ranges(parsed.text()) {
        let paragraph = merged_override(parsed, range.start);
        let (first_inset, continuation_inset) =
            paragraph_insets(parsed.text(), &paragraph, style, provider);
        let runs = slice_runs(&parsed.runs, range);
        let mut paragraph_lines = wrap_source_runs_with_line_widths_provider(
            &runs,
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

pub(super) fn line_constraints_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_width: f32,
    source_offset: usize,
    first_physical_line: bool,
    provider: &mut SharedTextLayoutSession,
) -> LineConstraints {
    let paragraph = merged_override(parsed, source_offset);
    let (first_inset, continuation_inset) =
        paragraph_insets(parsed.text(), &paragraph, style, provider);
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
    let maximum_inset = parsed
        .paragraphs()
        .map(|(range, _, _)| {
            let paragraph = merged_override(parsed, range.start);
            let (first, continuation) =
                paragraph_insets(parsed.text(), &paragraph, style, provider);
            first.max(continuation)
        })
        .fold(0.0_f32, f32::max);
    available_width(frame_width, maximum_inset)
}

pub(super) fn physical_paragraph_start(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    text[..offset].rfind('\n').map_or(0, |index| index + 1)
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

fn merged_override(parsed: &UiParsedText, offset: usize) -> ParagraphOverride {
    let mut merged = ParagraphOverride::default();
    let mut level = 0_u16;
    let mut first_indent = 0.0_f32;
    let mut align_owner = None;
    let mut prefix_owner = None;
    for (range, paragraph, list_prefix) in parsed
        .paragraphs()
        .filter(|(range, _, _)| range.start <= offset && offset < range.end)
    {
        let range = (to_u32(range.start), to_u32(range.end));
        if let Some(align) = paragraph.align {
            if is_more_specific(range, align_owner) {
                align_owner = Some(range);
                merged.align = Some(align);
            }
        }
        level = level
            .saturating_add(paragraph.indent_level.unwrap_or_default())
            .min(MAX_RESOLVED_INDENT_LEVEL);
        first_indent += paragraph.indent.unwrap_or_default().max(0.0);
        if let Some(list_prefix) = list_prefix {
            if is_more_specific(range, prefix_owner) {
                prefix_owner = Some(range);
                merged.list_prefix = Some((to_u32(list_prefix.start), to_u32(list_prefix.end)));
            }
        }
    }
    merged.indent_level = (level > 0).then_some(level);
    merged.indent = (first_indent > 0.0).then_some(first_indent);
    merged
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn is_more_specific(candidate: (u32, u32), current: Option<(u32, u32)>) -> bool {
    let Some(current) = current else {
        return true;
    };
    let candidate_span = candidate.1.saturating_sub(candidate.0);
    let current_span = current.1.saturating_sub(current.0);
    candidate_span < current_span || (candidate_span == current_span && candidate.0 >= current.0)
}

fn physical_paragraph_ranges(text: &str) -> Vec<UiTextRange> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (index, character) in text.char_indices() {
        if character == '\n' {
            ranges.push(UiTextRange { start, end: index });
            start = index + character.len_utf8();
        }
    }
    if start <= text.len() {
        ranges.push(UiTextRange {
            start,
            end: text.len(),
        });
    }
    ranges
}

fn slice_runs(runs: &[UiTextSourceRun], range: UiTextRange) -> Vec<UiTextSourceRun> {
    runs.iter()
        .filter_map(|run| run.subrange(range.start, range.end))
        .collect()
}

fn available_width(frame_width: f32, inset: f32) -> f32 {
    (frame_width.max(0.0) - resolved_inset(frame_width, inset)).max(0.0)
}

fn resolved_inset(frame_width: f32, inset: f32) -> f32 {
    let frame_width = frame_width.max(0.0);
    let minimum_extent = MIN_PARAGRAPH_EXTENT.min(frame_width);
    inset.max(0.0).min((frame_width - minimum_extent).max(0.0))
}
