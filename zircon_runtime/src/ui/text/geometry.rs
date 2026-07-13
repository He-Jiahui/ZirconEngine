use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextAlign,
        UiTextCaret, UiTextCaretAffinity, UiTextDirection, UiTextRange, UiTextWritingMode,
    },
};

use super::measure_text_source_range_width;

const TEXT_CARET_WIDTH: f32 = 1.0;

#[derive(Clone, Copy)]
enum SourceVisualBias {
    Leading,
    Trailing,
}

#[derive(Clone, Copy)]
struct SourceMeasureContext<'a> {
    text: &'a str,
    style: &'a UiResolvedStyle,
}

pub(crate) fn caret_frame_for_text_layout(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
) -> Option<UiFrame> {
    caret_frame_for_text_layout_inner(layout, caret, None)
}

pub(crate) fn caret_frame_for_text_layout_with_source_metrics(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    text: &str,
    style: &UiResolvedStyle,
) -> Option<UiFrame> {
    caret_frame_for_text_layout_inner(layout, caret, Some(SourceMeasureContext { text, style }))
}

fn caret_frame_for_text_layout_inner(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    measure_context: Option<SourceMeasureContext<'_>>,
) -> Option<UiFrame> {
    let offset = caret.offset;
    let line = match caret.affinity {
        UiTextCaretAffinity::Upstream => layout
            .lines
            .iter()
            .find(|line| offset >= line.source_range.start && offset <= line.source_range.end),
        UiTextCaretAffinity::Downstream => layout
            .lines
            .iter()
            .rev()
            .find(|line| offset >= line.source_range.start && offset <= line.source_range.end),
    }
    .or_else(|| {
        layout
            .lines
            .first()
            .filter(|line| offset < line.source_range.start)
    })
    .or_else(|| layout.lines.last())?;
    let bias = match caret.affinity {
        UiTextCaretAffinity::Upstream => SourceVisualBias::Leading,
        UiTextCaretAffinity::Downstream => SourceVisualBias::Trailing,
    };
    let visual_offset = line
        .runs
        .iter()
        .find_map(|run| {
            (offset >= run.source_range.start && offset <= run.source_range.end)
                .then(|| run_visual_offset_for_source_offset(run, offset, bias))
        })
        .unwrap_or(line.visual_range.end);
    if is_vertical_rl(layout) {
        return Some(UiFrame::new(
            line.frame.x,
            visual_y(layout, line, visual_offset, measure_context),
            line.frame.width.max(TEXT_CARET_WIDTH),
            TEXT_CARET_WIDTH,
        ));
    }
    Some(UiFrame::new(
        visual_x(layout, line, visual_offset, measure_context),
        line.frame.y,
        TEXT_CARET_WIDTH,
        line.frame.height.max(TEXT_CARET_WIDTH),
    ))
}

pub(crate) fn text_range_frames_for_text_layout(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
) -> Vec<UiFrame> {
    text_range_frames_for_text_layout_inner(layout, range, None)
}

pub(crate) fn text_range_frames_for_text_layout_with_source_metrics(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    text: &str,
    style: &UiResolvedStyle,
) -> Vec<UiFrame> {
    text_range_frames_for_text_layout_inner(
        layout,
        range,
        Some(SourceMeasureContext { text, style }),
    )
}

fn text_range_frames_for_text_layout_inner(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    measure_context: Option<SourceMeasureContext<'_>>,
) -> Vec<UiFrame> {
    if range.start == range.end {
        return caret_frame_for_text_layout_inner(
            layout,
            &UiTextCaret {
                offset: range.start,
                affinity: UiTextCaretAffinity::Downstream,
            },
            measure_context,
        )
        .into_iter()
        .collect();
    }

    let mut frames = Vec::new();
    for line in &layout.lines {
        for run in &line.runs {
            let start = range.start.max(run.source_range.start);
            let end = range.end.min(run.source_range.end);
            if start >= end {
                continue;
            }
            let visual_start =
                run_visual_offset_for_source_offset(run, start, SourceVisualBias::Leading);
            let visual_end =
                run_visual_offset_for_source_offset(run, end, SourceVisualBias::Trailing);
            let x0 = visual_x(layout, line, visual_start, measure_context);
            let x1 = visual_x(layout, line, visual_end, measure_context);
            if is_vertical_rl(layout) {
                let y0 = visual_y(layout, line, visual_start, measure_context);
                let y1 = visual_y(layout, line, visual_end, measure_context);
                frames.push(UiFrame::new(
                    line.frame.x,
                    y0.min(y1),
                    line.frame.width.max(TEXT_CARET_WIDTH),
                    (y1 - y0).abs().max(TEXT_CARET_WIDTH),
                ));
            } else {
                frames.push(UiFrame::new(
                    x0.min(x1),
                    line.frame.y,
                    (x1 - x0).abs().max(TEXT_CARET_WIDTH),
                    line.frame.height.max(TEXT_CARET_WIDTH),
                ));
            }
        }
    }
    frames
}

fn run_visual_offset_for_source_offset(
    run: &UiResolvedTextRun,
    offset: usize,
    bias: SourceVisualBias,
) -> usize {
    if offset <= run.source_range.start {
        return run.visual_range.start;
    }
    if offset >= run.source_range.end {
        return run.visual_range.end;
    }

    let source_len = run.source_range.end.saturating_sub(run.source_range.start);
    if source_len == 0 {
        return match bias {
            SourceVisualBias::Leading => run.visual_range.start,
            SourceVisualBias::Trailing => run.visual_range.end,
        };
    }

    let run_visual_len = run.visual_range.end.saturating_sub(run.visual_range.start);
    let local_source_offset = offset.saturating_sub(run.source_range.start);
    let local_visual_offset = if source_len == run.text.len() {
        match bias {
            SourceVisualBias::Leading => grapheme_floor(run.text.as_str(), local_source_offset),
            SourceVisualBias::Trailing => grapheme_ceil(run.text.as_str(), local_source_offset),
        }
    } else {
        non_isomorphic_local_visual_offset(run.text.as_str(), local_source_offset, source_len, bias)
    };

    run.visual_range.start + local_visual_offset.min(run_visual_len)
}

fn non_isomorphic_local_visual_offset(
    text: &str,
    local_source_offset: usize,
    source_len: usize,
    bias: SourceVisualBias,
) -> usize {
    let grapheme_count = text.graphemes(true).count();
    if grapheme_count == 0 {
        return 0;
    }

    let progress = local_source_offset.min(source_len) as f32 / source_len as f32;
    let visual_index = match bias {
        SourceVisualBias::Leading => (progress * grapheme_count as f32).floor() as usize,
        SourceVisualBias::Trailing => (progress * grapheme_count as f32).ceil() as usize,
    };
    grapheme_boundary_by_index(text, visual_index.min(grapheme_count))
}

fn grapheme_boundary_by_index(text: &str, index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    text.grapheme_indices(true)
        .nth(index)
        .map(|(start, _)| start)
        .unwrap_or(text.len())
}

fn visual_x(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    visual_offset: usize,
    measure_context: Option<SourceMeasureContext<'_>>,
) -> f32 {
    if let Some(width) = measured_source_prefix_width(layout, line, visual_offset, measure_context)
    {
        return line.frame.x + width;
    }

    let text = line.text.as_str();
    let offset = grapheme_floor(text, visual_offset.min(text.len()));
    let total_units = text.graphemes(true).count();
    let before_units = text[..offset].graphemes(true).count();
    if line.glyph_advances.len() == total_units {
        return line.frame.x
            + line
                .glyph_advances
                .iter()
                .take(before_units)
                .map(|advance| sanitized_advance(*advance))
                .sum::<f32>();
    }

    let total_units = total_units.max(1) as f32;
    let before_units = before_units as f32;
    line.frame.x + (line.frame.width.max(0.0) * before_units / total_units)
}

fn visual_y(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    visual_offset: usize,
    measure_context: Option<SourceMeasureContext<'_>>,
) -> f32 {
    if let Some(width) = measured_source_prefix_width(layout, line, visual_offset, measure_context)
    {
        return line.frame.y + width;
    }

    let text = line.text.as_str();
    let offset = grapheme_floor(text, visual_offset.min(text.len()));
    let total_units = text.graphemes(true).count();
    let before_units = text[..offset].graphemes(true).count();
    if line.glyph_advances.len() == total_units {
        return line.frame.y
            + line
                .glyph_advances
                .iter()
                .take(before_units)
                .map(|advance| sanitized_advance(*advance))
                .sum::<f32>();
    }

    let total_units = total_units.max(1) as f32;
    let before_units = before_units as f32;
    line.frame.y + (line.frame.height.max(0.0) * before_units / total_units)
}

fn measured_source_prefix_width(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    visual_offset: usize,
    measure_context: Option<SourceMeasureContext<'_>>,
) -> Option<f32> {
    let measure_context = measure_context?;
    if !line_accepts_source_measure(layout, line, measure_context.text) {
        return None;
    }

    let range = source_prefix_range_for_visual_offset(line, visual_offset);
    let width = measure_text_source_range_width(measure_context.text, measure_context.style, range);
    width.is_finite().then_some(width.max(0.0))
}

fn source_prefix_range_for_visual_offset(
    line: &UiResolvedTextLine,
    visual_offset: usize,
) -> UiTextRange {
    let local_end = grapheme_floor(line.text.as_str(), visual_offset.min(line.text.len()));
    UiTextRange {
        start: line.source_range.start,
        end: line
            .source_range
            .start
            .saturating_add(local_end)
            .min(line.source_range.end),
    }
}

fn line_accepts_source_measure(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    source_text: &str,
) -> bool {
    if matches!(layout.text_align, UiTextAlign::Justify)
        || !matches!(layout.writing_mode, UiTextWritingMode::HorizontalTb)
        || !matches!(layout.direction, UiTextDirection::LeftToRight)
        || !matches!(line.direction, UiTextDirection::LeftToRight)
        || line.ellipsized
        || line.text.contains('\t')
    {
        return false;
    }

    let Some(source_slice) = source_text.get(line.source_range.start..line.source_range.end) else {
        return false;
    };
    if source_slice != line.text {
        return false;
    }

    let [run] = line.runs.as_slice() else {
        return false;
    };
    run.source_range == line.source_range
        && run.visual_range == line.visual_range
        && run.text == line.text
        && matches!(run.direction, UiTextDirection::LeftToRight)
}

fn is_vertical_rl(layout: &UiResolvedTextLayout) -> bool {
    matches!(layout.writing_mode, UiTextWritingMode::VerticalRl)
}

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}

fn grapheme_floor(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return start;
        }
        if start >= offset {
            break;
        }
    }
    offset
}

fn grapheme_ceil(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset < text.len() && !text.is_char_boundary(offset) {
        offset += 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return end;
        }
        if start >= offset {
            break;
        }
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::surface::{
        UiResolvedTextRun, UiTextDirection, UiTextRunKind,
    };

    #[test]
    fn source_geometry_uses_resolved_glyph_advances() {
        let layout = layout_with_advances("a\tb", vec![6.0, 18.0, 6.0]);

        let caret = caret_frame_for_text_layout(
            &layout,
            &UiTextCaret {
                offset: 2,
                affinity: UiTextCaretAffinity::Downstream,
            },
        )
        .expect("caret frame");
        let frames = text_range_frames_for_text_layout(&layout, UiTextRange { start: 1, end: 3 });

        assert_eq!(caret, UiFrame::new(34.0, 20.0, 1.0, 12.0));
        assert_eq!(frames, vec![UiFrame::new(16.0, 20.0, 24.0, 12.0)]);
    }

    #[test]
    fn source_geometry_uses_vertical_writing_mode_advances() {
        let mut layout = layout_with_advances("abc", vec![6.0, 18.0, 6.0]);
        layout.writing_mode = zircon_runtime_interface::ui::surface::UiTextWritingMode::VerticalRl;
        let line = layout.lines.first_mut().expect("line");
        line.frame = UiFrame::new(20.0, 10.0, 10.0, 30.0);
        line.measured_width = 30.0;

        let caret = caret_frame_for_text_layout(
            &layout,
            &UiTextCaret {
                offset: 2,
                affinity: UiTextCaretAffinity::Downstream,
            },
        )
        .expect("caret frame");
        let frames = text_range_frames_for_text_layout(&layout, UiTextRange { start: 1, end: 3 });

        assert_eq!(caret, UiFrame::new(20.0, 34.0, 10.0, 1.0));
        assert_eq!(frames, vec![UiFrame::new(20.0, 16.0, 10.0, 24.0)]);
    }

    #[test]
    fn text_caret_affinity_soft_wrap_boundary() {
        let mut layout = layout_with_advances("ab", vec![6.0, 6.0]);
        let mut second_line = layout.lines[0].clone();
        second_line.text = "cd".to_string();
        second_line.frame = UiFrame::new(10.0, 32.0, 12.0, 12.0);
        second_line.source_range = UiTextRange { start: 2, end: 4 };
        second_line.visual_range = UiTextRange { start: 0, end: 2 };
        second_line.runs[0].text = second_line.text.clone();
        second_line.runs[0].source_range = second_line.source_range;
        second_line.runs[0].visual_range = second_line.visual_range;
        layout.lines.push(second_line);
        layout.source_range = UiTextRange { start: 0, end: 4 };

        let upstream = caret_frame_for_text_layout(
            &layout,
            &UiTextCaret {
                offset: 2,
                affinity: UiTextCaretAffinity::Upstream,
            },
        )
        .expect("upstream caret frame");
        let downstream = caret_frame_for_text_layout(
            &layout,
            &UiTextCaret {
                offset: 2,
                affinity: UiTextCaretAffinity::Downstream,
            },
        )
        .expect("downstream caret frame");

        assert_eq!(upstream, UiFrame::new(22.0, 20.0, 1.0, 12.0));
        assert_eq!(downstream, UiFrame::new(10.0, 32.0, 1.0, 12.0));
    }

    #[test]
    fn source_geometry_with_source_metrics_uses_shaped_source_range_width() {
        let style = UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        };
        let text = "Wi";
        let layout = layout_with_advances(text, vec![1.0, 1.0]);
        let caret = UiTextCaret {
            offset: "W".len(),
            affinity: UiTextCaretAffinity::Downstream,
        };
        let shaped_prefix = measure_text_source_range_width(
            text,
            &style,
            UiTextRange {
                start: 0,
                end: "W".len(),
            },
        );

        let stale = caret_frame_for_text_layout(&layout, &caret).expect("stale caret");
        let measured =
            caret_frame_for_text_layout_with_source_metrics(&layout, &caret, text, &style)
                .expect("measured caret");

        assert_eq!(stale, UiFrame::new(11.0, 20.0, 1.0, 12.0));
        assert!((measured.x - (10.0 + shaped_prefix)).abs() < 0.1);
        assert!(
            (measured.x - stale.x).abs() > 0.5,
            "source metrics should not reuse stale per-grapheme advances"
        );
    }

    #[test]
    fn source_geometry_with_source_metrics_keeps_tab_aligned_advances() {
        let style = UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        };
        let text = "a\tb";
        let layout = layout_with_advances(text, vec![6.0, 18.0, 6.0]);
        let caret = UiTextCaret {
            offset: 2,
            affinity: UiTextCaretAffinity::Downstream,
        };

        let measured =
            caret_frame_for_text_layout_with_source_metrics(&layout, &caret, text, &style)
                .expect("measured caret");

        assert_eq!(measured, UiFrame::new(34.0, 20.0, 1.0, 12.0));
    }

    #[test]
    fn source_geometry_with_source_metrics_keeps_vertical_advances() {
        let style = UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        };
        let text = "Wi";
        let mut layout = layout_with_advances(text, vec![2.0, 20.0]);
        layout.writing_mode = zircon_runtime_interface::ui::surface::UiTextWritingMode::VerticalRl;
        let line = layout.lines.first_mut().expect("line");
        line.frame = UiFrame::new(20.0, 10.0, 10.0, 30.0);
        let caret = UiTextCaret {
            offset: "W".len(),
            affinity: UiTextCaretAffinity::Downstream,
        };
        let horizontal_source_width = measure_text_source_range_width(
            text,
            &style,
            UiTextRange {
                start: 0,
                end: "W".len(),
            },
        );

        let fallback = caret_frame_for_text_layout(&layout, &caret).expect("fallback caret");
        let measured =
            caret_frame_for_text_layout_with_source_metrics(&layout, &caret, text, &style)
                .expect("measured caret");

        assert_eq!(fallback, UiFrame::new(20.0, 12.0, 10.0, 1.0));
        assert_eq!(measured, fallback);
        assert!(
            (horizontal_source_width - 2.0).abs() > 0.5,
            "test must prove the vertical path did not consume horizontal source width"
        );
    }

    #[test]
    fn source_geometry_with_source_metrics_rejects_unresolved_auto_direction() {
        let style = UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        };
        let text = "Wi";
        let mut layout = layout_with_advances(text, vec![2.0, 20.0]);
        layout.direction = UiTextDirection::Auto;
        let caret = UiTextCaret {
            offset: "W".len(),
            affinity: UiTextCaretAffinity::Downstream,
        };
        let horizontal_source_width = measure_text_source_range_width(
            text,
            &style,
            UiTextRange {
                start: 0,
                end: "W".len(),
            },
        );

        let fallback = caret_frame_for_text_layout(&layout, &caret).expect("fallback caret");
        let measured =
            caret_frame_for_text_layout_with_source_metrics(&layout, &caret, text, &style)
                .expect("measured caret");

        assert_eq!(fallback, UiFrame::new(12.0, 20.0, 1.0, 12.0));
        assert_eq!(measured, fallback);
        assert!(!line_accepts_source_measure(
            &layout,
            layout.lines.first().expect("line"),
            text
        ));
        assert!(
            (horizontal_source_width - 2.0).abs() > 0.5,
            "test must prove unresolved Auto did not consume horizontal source width"
        );
    }

    #[test]
    fn source_geometry_with_source_metrics_requires_ltr_line_and_run_direction() {
        let style = UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        };
        let text = "Wi";
        let mut layout = layout_with_advances(text, vec![2.0, 20.0]);
        assert!(line_accepts_source_measure(
            &layout,
            layout.lines.first().expect("line"),
            text
        ));

        layout.lines[0].direction = UiTextDirection::Auto;
        assert!(!line_accepts_source_measure(
            &layout,
            layout.lines.first().expect("line"),
            text
        ));

        layout.lines[0].direction = UiTextDirection::LeftToRight;
        layout.lines[0].runs[0].direction = UiTextDirection::Auto;
        assert!(!line_accepts_source_measure(
            &layout,
            layout.lines.first().expect("line"),
            text
        ));

        let measured = caret_frame_for_text_layout_with_source_metrics(
            &layout,
            &UiTextCaret {
                offset: "W".len(),
                affinity: UiTextCaretAffinity::Downstream,
            },
            text,
            &style,
        )
        .expect("measured caret");

        assert_eq!(measured, UiFrame::new(12.0, 20.0, 1.0, 12.0));
    }

    #[test]
    fn source_geometry_with_source_metrics_uses_absolute_source_prefix_ranges() {
        let style = UiResolvedStyle {
            font_size: 10.0,
            line_height: 12.0,
            ..UiResolvedStyle::default()
        };
        let source = "sample base.zui";
        let line_text = "base.zui";
        let line_start = "sample ".len();
        let caret_offset = line_start + "base".len();
        let mut layout = layout_with_advances(line_text, vec![1.0; line_text.len()]);
        {
            let line = layout.lines.first_mut().expect("line");
            line.source_range = UiTextRange {
                start: line_start,
                end: source.len(),
            };
            line.runs[0].source_range = line.source_range;
        }

        let measured = caret_frame_for_text_layout_with_source_metrics(
            &layout,
            &UiTextCaret {
                offset: caret_offset,
                affinity: UiTextCaretAffinity::Downstream,
            },
            source,
            &style,
        )
        .expect("measured caret");
        let expected_prefix = UiTextRange {
            start: line_start,
            end: caret_offset,
        };
        let expected_width = measure_text_source_range_width(source, &style, expected_prefix);
        let line = layout.lines.first().expect("line");

        assert_eq!(
            source_prefix_range_for_visual_offset(line, "base".len()),
            expected_prefix
        );
        assert!((measured.x - (10.0 + expected_width)).abs() < 0.1);
    }

    fn layout_with_advances(text: &str, glyph_advances: Vec<f32>) -> UiResolvedTextLayout {
        UiResolvedTextLayout {
            font_size: 10.0,
            line_height: 12.0,
            source_range: UiTextRange {
                start: 0,
                end: text.len(),
            },
            direction: UiTextDirection::LeftToRight,
            lines: vec![UiResolvedTextLine {
                text: text.to_string(),
                frame: UiFrame::new(10.0, 20.0, 30.0, 12.0),
                source_range: UiTextRange {
                    start: 0,
                    end: text.len(),
                },
                visual_range: UiTextRange {
                    start: 0,
                    end: text.len(),
                },
                measured_width: 30.0,
                glyph_advances,
                baseline: 9.0,
                direction: UiTextDirection::LeftToRight,
                runs: vec![UiResolvedTextRun {
                    kind: UiTextRunKind::Plain,
                    text: text.to_string(),
                    source_range: UiTextRange {
                        start: 0,
                        end: text.len(),
                    },
                    visual_range: UiTextRange {
                        start: 0,
                        end: text.len(),
                    },
                    direction: UiTextDirection::LeftToRight,
                }],
                ellipsized: false,
            }],
            ..UiResolvedTextLayout::default()
        }
    }
}
