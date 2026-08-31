use crate::text::layout::measure_text_size_with_provider as measure_backend_text_size_with_provider;
use zircon_runtime_interface::ui::layout::UiSize;

use super::*;

pub(super) fn layout_parsed_text_without_tables_with_retained_fragments(
    parsed: &super::super::rich_text::UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    viewport: Option<UiTextViewport>,
    document_key: Option<TextDocumentKey>,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<LayoutWithoutArtifact> {
    let visible_text = parsed.text();
    let effective_style =
        match resolve_overflow_style_with_provider(visible_text, style, frame, provider) {
            TextShapingOutcome::Ready(style) => style,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
    let style = &effective_style;
    let font_size = style.font_size.max(MIN_TEXT_FONT_SIZE);
    let metrics: TextLineMetrics = match line_metrics_with_provider(&text_style(style), provider) {
        TextShapingOutcome::Ready(metrics) => metrics,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let sample_line_height = metrics.line_height;
    if matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl) {
        return vertical::layout_vertical_text_with_provider(
            &parsed, style, frame, clip_frame, font_size, metrics, provider,
        );
    }

    let direction = resolve_text_direction(visible_text, style.text_direction);
    match rich_layout::layout_rich_text_with_provider(
        &parsed, style, frame, clip_frame, font_size, direction, provider,
    ) {
        TextShapingOutcome::Ready(Some(layout)) => {
            return TextShapingOutcome::Ready(layout);
        }
        TextShapingOutcome::Ready(None) => {}
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    }
    let source_runs = &parsed.runs;
    let max_width = available_wrap_extent(frame.width);
    let block_layout = paragraph_layout::has_block_layout(parsed);
    let (mut lines, line_index_offset, total_line_count, virtualized) = match viewport
        .and_then(|viewport| {
            visible_plain_text_lines(
                &parsed,
                style,
                viewport,
                sample_line_height,
                document_key,
                provider,
            )
        })
        .map(|window| {
            (
                window.lines,
                window.first_line,
                window.total_line_count,
                true,
            )
        })
        .map(TextShapingOutcome::Ready)
        .unwrap_or_else(|| {
            crate::profile_scope!("runtime", "text.layout", "materialize_full_document_lines");
            let lines = if block_layout {
                paragraph_layout::wrap_block_paragraphs_with_provider(
                    &parsed, style, max_width, provider,
                )
            } else {
                wrap_source_runs_with_provider(source_runs, style.wrap, max_width, style, provider)
            };
            lines.map(|lines| {
                let total_line_count = lines.len();
                (lines, 0, total_line_count, false)
            })
        }) {
        TextShapingOutcome::Ready(lines) => lines,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let line_constraints = if block_layout {
        crate::profile_scope!(
            "runtime",
            "text.layout",
            "resolve_paragraph_line_constraints"
        );
        match paragraph_layout::resolve_paragraph_line_constraints_with_provider(
            &parsed,
            style,
            frame.width,
            provider,
        ) {
            TextShapingOutcome::Ready(constraints) => constraints.for_candidates(&lines),
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
    } else {
        vec![
            paragraph_layout::LineConstraints {
                inset: 0.0,
                max_width: frame.width.max(0.0),
                align: style.text_align,
            };
            lines.len()
        ]
    };
    let clip = clip_frame.unwrap_or(frame);
    let mut physical_line_fragments = if virtualized {
        None
    } else {
        match physical_line_metrics::PhysicalLineFragments::shape_with_provider(
            visible_text,
            parsed.source_offset(),
            &lines,
            style,
            direction,
            provider,
        ) {
            TextShapingOutcome::Ready(source) => Some(source),
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
    };
    let provisional_metrics = physical_line_fragments.as_ref().map_or_else(
        || vec![metrics; lines.len()],
        |fragments| fragments.metrics(metrics),
    );
    let line_capacity =
        physical_line_metrics::visible_line_capacity(&provisional_metrics, frame.height);
    let mut overflow_clipped = total_line_count > line_capacity;
    if is_ellipsis_overflow(style.text_overflow) && overflow_clipped {
        if matches!(
            style.text_overflow,
            UiTextOverflow::EllipsisStart | UiTextOverflow::EllipsisMiddle
        ) {
            merge_clipped_lines_for_tail_preserving_ellipsis(&mut lines, line_capacity);
        }
        lines.truncate(line_capacity);
        let last_index = lines.len().saturating_sub(1);
        let available_width = line_constraints[last_index].max_width;
        if let Some(last) = lines.last_mut() {
            match ellipsize_line_with_provider(
                last,
                available_width,
                style,
                style.text_overflow,
                provider,
            ) {
                TextShapingOutcome::Ready(()) => {}
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            }
        }
    }
    if is_ellipsis_overflow(style.text_overflow) {
        for index in 0..lines.len() {
            let available_width = line_constraints[index].max_width;
            let line = &mut lines[index];
            let overflows = match line_overflows_horizontally_with_provider(
                line,
                available_width,
                style,
                provider,
            ) {
                TextShapingOutcome::Ready(overflows) => overflows,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            if !line.ellipsized && overflows {
                match ellipsize_line_with_provider(
                    line,
                    available_width,
                    style,
                    style.text_overflow,
                    provider,
                ) {
                    TextShapingOutcome::Ready(()) => {}
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                }
                overflow_clipped = true;
            }
        }
    }
    for index in 0..lines.len() {
        let line_index = line_index_offset.saturating_add(index);
        let is_last_line = line_index.saturating_add(1) == total_line_count;
        let constraints = line_constraints[index];
        let mut line_style = style.clone();
        line_style.text_align = constraints.align;
        match materialize_arabic_tatweels_for_justified_line(
            &mut lines[index],
            &line_style,
            constraints.max_width,
            is_last_line,
            provider,
        ) {
            TextShapingOutcome::Ready(()) => {}
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
    }
    if let Some(fragments) = &mut physical_line_fragments {
        match fragments.refresh_with_provider(
            visible_text,
            parsed.source_offset(),
            &lines,
            style,
            direction,
            provider,
        ) {
            TextShapingOutcome::Ready(()) => {}
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
    }
    let mut physical_metrics = physical_line_fragments.as_ref().map_or_else(
        || {
            // Viewport selection still indexes a uniform-height window. It must stay on the
            // sample metric until the Text03 prefix-metrics cache can supply preceding heights.
            vec![metrics; lines.len()]
        },
        |fragments| fragments.metrics(metrics),
    );
    let mut visual_fragment_advances = physical_line_fragments.as_ref().map(|fragments| {
        (0..lines.len())
            .map(|index| fragments.grapheme_advances_for_layout(index, &lines[index]))
            .collect::<Vec<_>>()
    });
    let shaped_style = text_style(style);
    let logical_virtual_line_sequences =
        match virtual_fragment_sequence::shape_and_apply_visual_order_with_sequences(
            &mut lines,
            visible_text,
            direction,
            &shaped_style,
            provider,
            &mut visual_fragment_advances,
            &mut physical_metrics,
            physical_line_fragments.as_ref(),
        ) {
            TextShapingOutcome::Ready(sequences) => sequences,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
    let line_height =
        physical_line_metrics::maximum_line_height(&physical_metrics, sample_line_height);

    let mut resolved_lines = Vec::new();
    let mut resolved_line_fragments = physical_line_fragments
        .as_ref()
        .map(|_| Vec::with_capacity(lines.len()));
    let mut resolved_virtual_line_sequences = logical_virtual_line_sequences
        .as_ref()
        .map(|_| Vec::with_capacity(lines.len()));
    let mut unclipped_measured_width = 0.0_f32;
    let mut y = frame.y
        + if virtualized {
            line_index_offset as f32 * sample_line_height
        } else {
            0.0
        };
    for (index, line) in lines.iter().enumerate() {
        let line_index = line_index_offset.saturating_add(index);
        let physical_metrics = physical_metrics[index];
        let is_last_line = line_index.saturating_add(1) == total_line_count;
        let constraints = line_constraints[index];
        let line_align = constraints.align;
        let mut line_style = style.clone();
        line_style.text_align = line_align;
        let (measured_width, glyph_advances, line_width) = match resolve_line_widths_with_provider(
            line,
            &line_style,
            constraints.max_width,
            is_last_line,
            visual_fragment_advances
                .as_mut()
                .and_then(|advances| advances.get_mut(index))
                .and_then(Option::take),
            provider,
        ) {
            TextShapingOutcome::Ready(widths) => widths,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        let content_frame =
            paragraph_layout::inset_logical_start(frame, constraints.inset, direction);
        let placement_frame = UiFrame::new(
            content_frame.x,
            y,
            content_frame.width,
            physical_metrics.line_height,
        );
        let line_frame = UiFrame::new(
            aligned_x(content_frame, line_width, line_align, direction),
            y,
            measured_width,
            physical_metrics.line_height,
        );
        if !virtualized {
            unclipped_measured_width = unclipped_measured_width.max(measured_width);
        }
        if placement_frame.intersection(clip).is_some() {
            resolved_lines.push(UiResolvedTextLine {
                text: line.text.clone(),
                frame: line_frame,
                placement_frame,
                source_range: line.source_range,
                visual_range: UiTextRange {
                    start: 0,
                    end: line.text.len(),
                },
                measured_width,
                glyph_advances,
                baseline: physical_metrics.baseline,
                direction,
                runs: line.runs.clone(),
                ellipsized: line.ellipsized,
            });
            if let Some(fragment_handles) = &mut resolved_line_fragments {
                fragment_handles.push(
                    physical_line_fragments
                        .as_ref()
                        .and_then(|fragments| fragments.fragment_handle_at(index)),
                );
            }
            if let Some(virtual_sequences) = &mut resolved_virtual_line_sequences {
                virtual_sequences.push(
                    logical_virtual_line_sequences
                        .as_ref()
                        .and_then(|sequences| sequences.get(index))
                        .cloned()
                        .flatten(),
                );
            }
        } else {
            overflow_clipped = true;
        }
        y += physical_metrics.line_height;
    }

    let measured_width = if virtualized {
        resolved_lines
            .iter()
            .map(|line| line.measured_width)
            .fold(0.0_f32, f32::max)
    } else {
        unclipped_measured_width
    };
    let measured_height = if virtualized {
        total_line_count as f32 * sample_line_height
    } else {
        physical_line_metrics::total_line_height(&physical_metrics)
    };
    TextShapingOutcome::Ready(LayoutWithoutArtifact {
        layout: UiResolvedTextLayout {
            text_align: style.text_align,
            wrap: style.wrap,
            direction,
            writing_mode: style.text_writing_mode,
            overflow: style.text_overflow,
            font_size,
            line_height,
            measured_width,
            measured_height,
            source_range: UiTextRange {
                start: 0,
                end: visible_text.len(),
            },
            lines: resolved_lines,
            boxes: Vec::new(),
            overflow_clipped,
            editable: None,
            rich_text_artifact: None,
        },
        retained_line_fragments: resolved_line_fragments,
        retained_virtual_line_sequences: resolved_virtual_line_sequences,
    })
}

fn resolve_overflow_style_with_provider(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<UiResolvedStyle> {
    let max_extent = if matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl) {
        frame.height
    } else {
        frame.width
    };
    overflow_style::resolve(text, style, max_extent, |text, style| {
        measure_backend_text_size_with_provider(text, &text_style(style), provider)
            .map(UiSize::from)
    })
}
