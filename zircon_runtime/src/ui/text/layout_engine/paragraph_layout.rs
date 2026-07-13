use crate::core::framework::render::ParagraphOverride;
use crate::graphics::text::layout::{measure_line_width_with_provider, tab_interval_width};
use crate::graphics::text::shaping::TextShapeRunProvider;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiTextAlign, UiTextDirection, UiTextRange,
};

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

pub(super) fn has_block_layout(parsed: &UiParsedText) -> bool {
    parsed.paragraphs.iter().any(|(_, paragraph)| {
        paragraph.indent.is_some()
            || paragraph.indent_level.is_some()
            || paragraph.list_prefix.is_some()
            || paragraph.align.is_some()
    })
}

pub(super) fn column_constraints_with_provider<P>(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_height: f32,
    source_offset: usize,
    first_physical_column: bool,
    provider: &mut P,
) -> ColumnConstraints
where
    P: TextShapeRunProvider + ?Sized,
{
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

pub(super) fn wrap_block_paragraphs_with_provider<P>(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_width: f32,
    provider: &mut P,
) -> Vec<CandidateLine>
where
    P: TextShapeRunProvider + ?Sized,
{
    let mut lines = Vec::new();
    for range in physical_paragraph_ranges(&parsed.text) {
        let paragraph = merged_override(&parsed.paragraphs, range.start);
        let (first_inset, continuation_inset) =
            paragraph_insets(&parsed.text, &paragraph, style, provider);
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

pub(super) fn line_constraints_with_provider<P>(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_width: f32,
    source_offset: usize,
    first_physical_line: bool,
    provider: &mut P,
) -> LineConstraints
where
    P: TextShapeRunProvider + ?Sized,
{
    let paragraph = merged_override(&parsed.paragraphs, source_offset);
    let (first_inset, continuation_inset) =
        paragraph_insets(&parsed.text, &paragraph, style, provider);
    let inset = if first_physical_line {
        first_inset
    } else {
        continuation_inset
    };
    let inset = resolved_inset(frame_width, inset);
    LineConstraints {
        inset,
        max_width: available_width(frame_width, inset),
        align: paragraph.align.unwrap_or(style.text_align),
    }
}

pub(super) fn conservative_rich_width_with_provider<P>(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_width: f32,
    provider: &mut P,
) -> f32
where
    P: TextShapeRunProvider + ?Sized,
{
    let maximum_inset = parsed
        .paragraphs
        .iter()
        .map(|(range, _)| {
            let paragraph = merged_override(&parsed.paragraphs, range.0 as usize);
            let (first, continuation) = paragraph_insets(&parsed.text, &paragraph, style, provider);
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

fn paragraph_insets<P>(
    text: &str,
    paragraph: &ParagraphOverride,
    style: &UiResolvedStyle,
    provider: &mut P,
) -> (f32, f32)
where
    P: TextShapeRunProvider + ?Sized,
{
    let space_width = measure_line_width_with_provider(" ", style, provider);
    let level_width = tab_interval_width(style, space_width);
    let level_indent = f32::from(paragraph.indent_level.unwrap_or_default()) * level_width;
    let first_indent = paragraph.indent.unwrap_or_default().max(0.0);
    let prefix_width = paragraph
        .list_prefix
        .and_then(|range| text.get(range.0 as usize..range.1 as usize))
        .map(|prefix| measure_line_width_with_provider(prefix, style, provider))
        .unwrap_or_default();
    (level_indent + first_indent, level_indent + prefix_width)
}

fn merged_override(
    paragraphs: &[((u32, u32), ParagraphOverride)],
    offset: usize,
) -> ParagraphOverride {
    let offset = u32::try_from(offset).unwrap_or(u32::MAX);
    let mut merged = ParagraphOverride::default();
    let mut level = 0_u16;
    let mut first_indent = 0.0_f32;
    let mut align_owner = None;
    let mut prefix_owner = None;
    for (range, paragraph) in paragraphs
        .iter()
        .filter(|(range, _)| range.0 <= offset && offset < range.1)
    {
        if let Some(align) = paragraph.align {
            if is_more_specific(*range, align_owner) {
                align_owner = Some(*range);
                merged.align = Some(align);
            }
        }
        level = level
            .saturating_add(paragraph.indent_level.unwrap_or_default())
            .min(MAX_RESOLVED_INDENT_LEVEL);
        first_indent += paragraph.indent.unwrap_or_default().max(0.0);
        if paragraph.list_prefix.is_some() && is_more_specific(*range, prefix_owner) {
            prefix_owner = Some(*range);
            merged.list_prefix = paragraph.list_prefix;
        }
    }
    merged.indent_level = (level > 0).then_some(level);
    merged.indent = (first_indent > 0.0).then_some(first_indent);
    merged
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
        .filter_map(|run| {
            let start = run.source_range.start.max(range.start);
            let end = run.source_range.end.min(range.end);
            (start < end).then(|| UiTextSourceRun {
                kind: run.kind,
                text: run.text[start - run.source_range.start..end - run.source_range.start]
                    .to_string(),
                source_range: UiTextRange { start, end },
                style: run.style.clone(),
                inline: run.inline.clone(),
                link: run.link.clone(),
            })
        })
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
