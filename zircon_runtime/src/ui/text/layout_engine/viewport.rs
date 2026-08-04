use std::ops::Range;

use crate::text::{SharedTextLayoutSession, TextDocumentKey};
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiRichTextFormat, UiTextOverflow, UiTextRange, UiTextWrap, UiTextWritingMode,
};

use super::super::{resolved_layout::UiTextViewport, rich_text::UiParsedText};
use super::candidate_line::{CandidateLine, append_segment};

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
    line_height: f32,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> Option<VisibleTextLineWindow> {
    if !matches!(style.rich_text_format, UiRichTextFormat::Plain)
        || !matches!(style.wrap, UiTextWrap::None)
        || !matches!(style.text_overflow, UiTextOverflow::Clip)
        || matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl)
        || parsed.source_offset() != 0
        || !line_height.is_finite()
        || line_height <= 0.0
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

    let requested_window = unbounded_line_window(
        viewport.offset_y,
        viewport.extent_y,
        line_height,
        viewport.overscan_screens,
    )?;
    let (total_line_count, hard_lines) =
        provider.hard_line_count_and_window(text, document_key, requested_window.clone());
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
mod tests {
    use super::{line_window, visible_plain_text_lines};
    use crate::{
        text::{RichTextFormat, SharedTextLayoutSession, TextDocumentKey},
        ui::text::rich_text::parse_source_text,
    };
    use zircon_runtime_interface::ui::surface::{
        UiResolvedStyle, UiTextOverflow, UiTextRange, UiTextWrap, UiTextWritingMode,
    };

    #[test]
    fn line_window_keeps_two_viewports_of_overscan() {
        assert_eq!(line_window(120.0, 20.0, 10.0, 100, 2), Some((8, 16)));
    }

    #[test]
    fn line_window_clamps_at_document_edges() {
        assert_eq!(line_window(0.0, 20.0, 10.0, 5, 2), Some((0, 5)));
        assert_eq!(line_window(1_000.0, 20.0, 10.0, 5, 2), Some((1, 5)));
    }

    #[test]
    fn unbounded_line_window_saturates_before_document_count_is_known() {
        assert_eq!(
            super::unbounded_line_window(f32::MAX, 20.0, 10.0, 2),
            Some(usize::MAX..usize::MAX)
        );
    }

    #[test]
    fn visible_window_rejects_vertical_text() {
        let parsed = parse_source_text("vertical", RichTextFormat::Plain);
        let style = UiResolvedStyle {
            wrap: UiTextWrap::None,
            text_overflow: UiTextOverflow::Clip,
            text_writing_mode: UiTextWritingMode::VerticalRl,
            ..UiResolvedStyle::default()
        };
        let viewport =
            super::super::UiTextViewport::new(0.0, 20.0, 2).expect("finite document viewport");
        let mut provider = SharedTextLayoutSession::new();

        assert!(
            visible_plain_text_lines(&parsed, &style, viewport, 10.0, None, &mut provider)
                .is_none()
        );
    }

    #[test]
    fn visible_window_borrows_only_the_requested_plain_hard_lines() {
        let parsed = parse_source_text("first\nsecond\nthird\nfourth", RichTextFormat::Plain);
        let style = UiResolvedStyle {
            wrap: UiTextWrap::None,
            text_overflow: UiTextOverflow::Clip,
            ..UiResolvedStyle::default()
        };
        let viewport =
            super::super::UiTextViewport::new(11.0, 1.0, 0).expect("finite document viewport");
        let mut provider = SharedTextLayoutSession::new();

        let window = visible_plain_text_lines(&parsed, &style, viewport, 10.0, None, &mut provider)
            .expect("partial plain viewport window");

        assert_eq!(window.first_line, 1);
        assert_eq!(window.total_line_count, 4);
        assert_eq!(
            window
                .lines
                .iter()
                .map(|line| (line.text.as_str(), line.source_range))
                .collect::<Vec<_>>(),
            vec![("second", UiTextRange { start: 6, end: 12 })]
        );
    }

    #[test]
    fn visible_windows_reuse_the_session_hard_line_index() {
        let parsed = parse_source_text("zero\none\ntwo\nthree", RichTextFormat::Plain);
        let style = UiResolvedStyle {
            wrap: UiTextWrap::None,
            text_overflow: UiTextOverflow::Clip,
            ..UiResolvedStyle::default()
        };
        let mut provider = SharedTextLayoutSession::new();
        let first =
            super::super::UiTextViewport::new(0.0, 1.0, 0).expect("finite document viewport");
        let third =
            super::super::UiTextViewport::new(21.0, 1.0, 0).expect("finite document viewport");

        let key = TextDocumentKey::new(7, 1);
        let first_window =
            visible_plain_text_lines(&parsed, &style, first, 10.0, Some(key), &mut provider)
                .expect("first plain viewport window");
        let third_window =
            visible_plain_text_lines(&parsed, &style, third, 10.0, Some(key), &mut provider)
                .expect("third plain viewport window");

        assert_eq!(first_window.lines[0].text, "zero");
        assert_eq!(third_window.lines[0].text, "two");
        let report = provider.hard_line_index_report();
        assert_eq!(report.build_count, 1);
        assert_eq!(report.hit_count, 1);
    }
}
