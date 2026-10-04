use std::ops::Range;

use crate::text::{SharedTextLayoutSession, TextDocumentKey};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiRichTextFormat, UiTextOverflow, UiTextRange, UiTextWrap, UiTextWritingMode,
};

use super::super::{resolved_layout::UiTextViewport, rich_text::UiParsedText};
use super::candidate_line::{append_segment, CandidateLine};
use super::measurement::certified_plain_viewport_line_height;

pub(super) struct VisibleTextLineWindow {
    pub(super) first_line: usize,
    pub(super) total_line_count: usize,
    pub(super) lines: Vec<CandidateLine>,
}

/// Returns a bounded candidate-line slice only for the simple path whose physical line height is
/// known before shaping. Rich, wrapped, vertical, and editable text retain their complete layout
/// path until they have equivalent paragraph-height and scroll-anchor contracts.
pub(super) fn visible_plain_text_lines(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    viewport: UiTextViewport,
    sample_line_height: f32,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> Option<VisibleTextLineWindow> {
    if !matches!(style.rich_text_format, UiRichTextFormat::Plain)
        || !matches!(style.wrap, UiTextWrap::None)
        || !matches!(style.text_overflow, UiTextOverflow::Clip)
        || matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl)
        || parsed.source_offset() != 0
        || !sample_line_height.is_finite()
        || sample_line_height <= 0.0
    {
        return None;
    }

    let text = parsed.text();
    let run = parsed.runs.first()?;
    if parsed.runs.len() != 1
        || run.source_range
            != (UiTextRange {
                start: 0,
                end: text.len(),
            })
    {
        return None;
    }

    let line_height =
        certified_plain_viewport_line_height(text, style, sample_line_height, provider)?;
    visible_plain_text_lines_from_certified_height(
        parsed,
        viewport,
        Some(line_height),
        document_key,
        provider,
    )
}

fn visible_plain_text_lines_from_certified_height(
    parsed: &UiParsedText,
    viewport: UiTextViewport,
    certified_uniform_line_height: Option<f32>,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> Option<VisibleTextLineWindow> {
    let line_height = certified_uniform_line_height?;
    if !line_height.is_finite() || line_height <= 0.0 {
        return None;
    }
    let text = parsed.text();
    let run = parsed.runs.first()?;

    crate::profile_scope!("runtime", "text.layout", "select_visible_plain_lines");
    let requested_window = unbounded_line_window(
        viewport.offset_y,
        viewport.extent_y,
        line_height,
        viewport.overscan_screens,
    )?;
    let (total_line_count, hard_lines) = match document_key {
        Some(document_key) => provider.retained_hard_line_count_and_window(
            parsed.rich.shared_text(),
            document_key,
            requested_window.clone(),
        ),
        None => provider.unretained_hard_line_count_and_window(text, requested_window.clone()),
    };
    let first_line = requested_window.start.min(total_line_count);
    let last_line_exclusive = requested_window.end.min(total_line_count);
    if first_line == 0 && last_line_exclusive == total_line_count {
        return None;
    }

    let lines = hard_lines
        .into_iter()
        .map(|hard_line| {
            let mut candidate = CandidateLine::empty();
            let source_range = UiTextRange {
                start: hard_line.content.start,
                end: hard_line.content.end,
            };
            candidate.source_range = source_range;
            append_segment(
                &mut candidate,
                run.kind,
                &text[hard_line.content],
                source_range,
            );
            candidate
        })
        .collect();

    Some(VisibleTextLineWindow {
        first_line,
        total_line_count,
        lines,
    })
}

fn line_window(
    offset_y: f32,
    extent_y: f32,
    line_height: f32,
    line_count: usize,
    overscan_screens: usize,
) -> Option<(usize, usize)> {
    if line_count == 0 {
        return None;
    }

    let requested = unbounded_line_window(offset_y, extent_y, line_height, overscan_screens)?;

    Some((
        requested.start.min(line_count),
        requested.end.min(line_count),
    ))
}

fn unbounded_line_window(
    offset_y: f32,
    extent_y: f32,
    line_height: f32,
    overscan_screens: usize,
) -> Option<Range<usize>> {
    if !offset_y.is_finite()
        || !extent_y.is_finite()
        || extent_y <= 0.0
        || !line_height.is_finite()
        || line_height <= 0.0
    {
        return None;
    }

    let maximum_line_index = usize::MAX as f32;
    let first_visible = (offset_y.max(0.0) / line_height)
        .floor()
        .min(maximum_line_index) as usize;
    let last_visible_exclusive = ((offset_y.max(0.0) + extent_y) / line_height)
        .ceil()
        .max(first_visible as f32)
        .min(maximum_line_index) as usize;
    let lines_per_screen = (extent_y / line_height)
        .ceil()
        .max(1.0)
        .min(maximum_line_index) as usize;
    let overscan = lines_per_screen.saturating_mul(overscan_screens);

    Some(first_visible.saturating_sub(overscan)..last_visible_exclusive.saturating_add(overscan))
}

#[cfg(test)]
#[path = "tests/viewport_unit.rs"]
mod tests;
