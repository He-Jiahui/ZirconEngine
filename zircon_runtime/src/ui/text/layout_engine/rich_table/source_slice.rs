use crate::core::framework::render::{RichParseResult, StyledRun};
use crate::graphics::text::shaping::TextShapeRunProvider;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiResolvedTextLayout, UiTextRange},
};

use super::super::super::rich_text::{UiParsedText, UiTextSourceRun};
use super::super::layout_parsed_text_with_provider;

pub(super) fn layout_range_with_provider<P>(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip: UiFrame,
    provider: &mut P,
) -> UiResolvedTextLayout
where
    P: TextShapeRunProvider + ?Sized,
{
    let start = range.start.min(parsed.text.len());
    let end = range.end.min(parsed.text.len()).max(start);
    let local = slice_parsed(parsed, start..end);
    let mut layout = layout_parsed_text_with_provider(&local, style, frame, Some(clip), provider);
    shift_layout_source_ranges(&mut layout, start);
    layout
}

pub(super) fn layout_cell_range_with_provider<P>(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
    parent_table_depth: u16,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip: UiFrame,
    provider: &mut P,
) -> UiResolvedTextLayout
where
    P: TextShapeRunProvider + ?Sized,
{
    let start = range.start.min(parsed.text.len());
    let end = range.end.min(parsed.text.len()).max(start);
    let local = slice_parsed_with_table_depth(parsed, start..end, Some(parent_table_depth));
    let mut layout = layout_parsed_text_with_provider(&local, style, frame, Some(clip), provider);
    shift_layout_source_ranges(&mut layout, start);
    layout
}

pub(super) fn slice_parsed(parsed: &UiParsedText, range: std::ops::Range<usize>) -> UiParsedText {
    slice_parsed_with_table_depth(parsed, range, None)
}

fn slice_parsed_with_table_depth(
    parsed: &UiParsedText,
    range: std::ops::Range<usize>,
    parent_table_depth: Option<u16>,
) -> UiParsedText {
    let start = range.start.min(parsed.text.len());
    let end = range.end.min(parsed.text.len()).max(start);
    let text = parsed.text[start..end].to_string();
    let runs = parsed
        .runs
        .iter()
        .filter_map(|run| slice_ui_run(run, start, end))
        .collect();
    let rich_runs = parsed
        .rich
        .runs
        .iter()
        .filter_map(|run| slice_rich_run(run, start, end))
        .collect();
    let paragraphs = parsed
        .paragraphs
        .iter()
        .filter_map(|(paragraph_range, paragraph)| {
            intersect_u32_range(*paragraph_range, start, end)
                .map(|range| (range, paragraph.clone()))
        })
        .collect::<Vec<_>>();
    let tables = parsed
        .rich
        .tables
        .iter()
        .filter(|table| {
            table.byte_range.0 as usize >= start
                && table.byte_range.1 as usize <= end
                && parent_table_depth.is_none_or(|depth| table.depth > depth)
        })
        .cloned()
        .map(|mut table| {
            table.byte_range = shift_u32_range(table.byte_range, start);
            table.depth = table.depth.saturating_sub(
                parent_table_depth
                    .map(|depth| depth.saturating_add(1))
                    .unwrap_or(1),
            );
            for cell in &mut table.cells {
                cell.byte_range = shift_u32_range(cell.byte_range, start);
            }
            table
        })
        .collect();
    let rich = RichParseResult {
        text: text.clone(),
        runs: rich_runs,
        paragraphs: paragraphs.clone(),
        tables,
    };
    UiParsedText {
        text,
        runs,
        paragraphs,
        rich,
    }
}

fn slice_ui_run(run: &UiTextSourceRun, start: usize, end: usize) -> Option<UiTextSourceRun> {
    let range_start = run.source_range.start.max(start);
    let range_end = run.source_range.end.min(end);
    (range_start < range_end).then(|| UiTextSourceRun {
        kind: run.kind,
        text: parsed_run_text(&run.text, run.source_range.start, range_start, range_end),
        source_range: UiTextRange {
            start: range_start - start,
            end: range_end - start,
        },
        style: run.style.clone(),
        inline: run.inline.clone(),
        link: run.link.clone(),
    })
}

fn slice_rich_run(run: &StyledRun, start: usize, end: usize) -> Option<StyledRun> {
    let run_start = run.byte_range.0 as usize;
    let run_end = run.byte_range.1 as usize;
    let range_start = run_start.max(start);
    let range_end = run_end.min(end);
    (range_start < range_end).then(|| StyledRun {
        byte_range: ((range_start - start) as u32, (range_end - start) as u32),
        style: run.style.clone(),
        inline: run.inline.clone(),
        link: run.link.clone(),
    })
}

fn parsed_run_text(
    text: &str,
    source_start: usize,
    range_start: usize,
    range_end: usize,
) -> String {
    text[range_start - source_start..range_end - source_start].to_string()
}

fn intersect_u32_range(range: (u32, u32), start: usize, end: usize) -> Option<(u32, u32)> {
    let range_start = range.0 as usize;
    let range_end = range.1 as usize;
    let intersection_start = range_start.max(start);
    let intersection_end = range_end.min(end);
    (intersection_start < intersection_end).then(|| {
        (
            (intersection_start - start) as u32,
            (intersection_end - start) as u32,
        )
    })
}

fn shift_u32_range(range: (u32, u32), start: usize) -> (u32, u32) {
    (
        range.0.saturating_sub(start as u32),
        range.1.saturating_sub(start as u32),
    )
}

fn shift_layout_source_ranges(layout: &mut UiResolvedTextLayout, offset: usize) {
    layout.source_range.start += offset;
    layout.source_range.end += offset;
    for text_box in &mut layout.boxes {
        text_box.range.start += offset;
        text_box.range.end += offset;
    }
    for line in &mut layout.lines {
        line.source_range.start += offset;
        line.source_range.end += offset;
        for run in &mut line.runs {
            run.source_range.start += offset;
            run.source_range.end += offset;
        }
    }
}
