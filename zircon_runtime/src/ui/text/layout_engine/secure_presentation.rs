use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedTextLayout, UiTextAlign, UiTextDirection, UiTextRange, UiTextRunKind,
        UiTextWritingMode,
    },
};

use crate::ui::text::{UiSecureTextPresentation, UiSecureTextPresentationError};

use super::super::presentation::register_secure_text_presentation_artifact;

use super::candidate_line::{CandidateLine, append_segment};
use super::visual_order::apply_visual_order_from_bidi_order_for_presentation_with_advances;

/// Replaces a mask layout's neutral-glyph provenance with the presentation owner's source map.
///
/// The generic layout pass determines physical wrap opportunities and bullet advances. This pass
/// reconstructs each physical line from complete display graphemes, then applies the UAX#9 order
/// captured from the original source. It intentionally rejects unsupported synthetic/vertical
/// output instead of treating bullet text as an `Auto` bidi paragraph.
pub(crate) fn apply_secure_text_presentation(
    layout: &mut UiResolvedTextLayout,
    presentation: &UiSecureTextPresentation,
) -> Result<(), UiSecureTextPresentationError> {
    if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
        return Err(UiSecureTextPresentationError::UnsupportedLayoutProjection);
    }

    let generic_direction = layout.direction;
    let mut resolved_direction = None;
    let mut source_range = None;
    for line in &mut layout.lines {
        if line.ellipsized {
            return Err(UiSecureTextPresentationError::UnsupportedLayoutProjection);
        }
        let display_range = line.source_range;
        let (mut candidate, order) = secure_candidate_line(presentation, display_range)?;
        let mut glyph_advances = std::mem::take(&mut line.glyph_advances);
        if glyph_advances.len() != candidate.text.chars().count() {
            return Err(UiSecureTextPresentationError::UnsupportedLayoutProjection);
        }
        apply_visual_order_from_bidi_order_for_presentation_with_advances(
            &mut candidate,
            &order,
            &mut glyph_advances,
        )
        .map_err(UiSecureTextPresentationError::Bidi)?;

        let visual_end = candidate.text.len();
        line.text = candidate.text;
        line.source_range = candidate.source_range;
        line.visual_range = UiTextRange {
            start: 0,
            end: visual_end,
        };
        line.glyph_advances = glyph_advances;
        line.direction = order.resolved_base_direction.into();
        line.frame = frame_with_projected_direction(
            line.frame,
            layout.text_align,
            generic_direction,
            line.direction,
        );
        line.runs = candidate.runs;
        resolved_direction.get_or_insert(line.direction);
        source_range = Some(merge_source_ranges(source_range, line.source_range));
    }

    layout.direction = resolved_direction.unwrap_or(layout.direction);
    layout.source_range = source_range.unwrap_or(UiTextRange {
        start: 0,
        end: presentation.source_len(),
    });
    // The generic pass may have built an artifact while it still considered neutral bullets to
    // be source text. Replace it with a no-source marker consumed by command-phase publication.
    layout.rich_text_artifact = Some(register_secure_text_presentation_artifact());
    Ok(())
}

/// `Start` and `End` are direction-relative. The neutral display mask is initially laid out
/// using one generic paragraph direction, then each physical line receives source-owned UAX#9
/// direction. Preserve the already-resolved logical edge while placing that line on its own
/// direction-relative side. Absolute alignment and center/justify are direction-invariant.
fn frame_with_projected_direction(
    frame: UiFrame,
    align: UiTextAlign,
    generic_direction: UiTextDirection,
    projected_direction: UiTextDirection,
) -> UiFrame {
    if generic_direction == projected_direction {
        return frame;
    }
    let x = match align {
        UiTextAlign::Start => {
            let logical_start = if matches!(generic_direction, UiTextDirection::RightToLeft) {
                frame.right()
            } else {
                frame.x
            };
            if matches!(projected_direction, UiTextDirection::RightToLeft) {
                logical_start - frame.width
            } else {
                logical_start
            }
        }
        UiTextAlign::End => {
            let logical_end = if matches!(generic_direction, UiTextDirection::RightToLeft) {
                frame.x
            } else {
                frame.right()
            };
            if matches!(projected_direction, UiTextDirection::RightToLeft) {
                logical_end
            } else {
                logical_end - frame.width
            }
        }
        UiTextAlign::Left | UiTextAlign::Center | UiTextAlign::Right | UiTextAlign::Justify => {
            frame.x
        }
    };
    UiFrame::new(x, frame.y, frame.width, frame.height)
}

fn secure_candidate_line(
    presentation: &UiSecureTextPresentation,
    display_range: UiTextRange,
) -> Result<(CandidateLine, crate::text::shaping::BidiLineOrder), UiSecureTextPresentationError> {
    let clusters = presentation
        .clusters_for_display_range(display_range)
        .ok_or(UiSecureTextPresentationError::UnsupportedLayoutProjection)?;
    let presentation_order = presentation
        .bidi_for_display_range(display_range)?
        .ok_or(UiSecureTextPresentationError::UnsupportedLayoutProjection)?;
    if presentation_order.visual_indices.len() != clusters.len() {
        return Err(UiSecureTextPresentationError::UnsupportedLayoutProjection);
    }
    let order = crate::text::shaping::BidiLineOrder {
        resolved_base_direction: presentation_order.resolved_base_direction,
        logical_levels: presentation_order.logical_levels,
        visual_indices: presentation_order.visual_indices,
        unicode_data_snapshot: presentation_order.unicode_data_snapshot,
    };

    let mut candidate = CandidateLine::empty();
    for cluster in clusters {
        let Some(text) = presentation
            .display_text()
            .get(cluster.display_range.start..cluster.display_range.end)
        else {
            return Err(UiSecureTextPresentationError::UnsupportedLayoutProjection);
        };
        append_segment(
            &mut candidate,
            UiTextRunKind::Plain,
            text,
            cluster.source_range,
        );
    }
    Ok((candidate, order))
}

fn merge_source_ranges(current: Option<UiTextRange>, next: UiTextRange) -> UiTextRange {
    let Some(current) = current else {
        return next;
    };
    UiTextRange {
        start: current.start.min(next.start),
        end: current.end.max(next.end),
    }
}

#[cfg(test)]
mod tests {
    use super::apply_secure_text_presentation;
    use crate::{core::framework::text::TextDirection, ui::text::UiSecureTextPresentation};
    use zircon_runtime_interface::ui::{
        layout::UiFrame,
        surface::{UiResolvedStyle, UiTextOverflow, UiTextRange, UiTextWrap},
    };

    #[test]
    fn wrapped_rtl_secure_rows_replay_each_rows_source_owned_bidi_order() {
        let source = "\u{05d0}\u{05d1}\u{05d2}\u{05d3}\u{05d4}\u{05d5}\u{05d6}\u{05d7}";
        let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto)
            .expect("a valid RTL source must produce a secure presentation");
        let style = UiResolvedStyle {
            wrap: UiTextWrap::Glyph,
            text_overflow: UiTextOverflow::Clip,
            font_size: 18.0,
            line_height: 22.0,
            ..UiResolvedStyle::default()
        };
        let unwrapped_style = UiResolvedStyle {
            wrap: UiTextWrap::None,
            ..style.clone()
        };
        let unwrapped = super::super::layout_text(
            presentation.display_text(),
            &unwrapped_style,
            UiFrame::new(0.0, 0.0, f32::INFINITY, 64.0),
            None,
        );
        let mut layout = super::super::layout_text(
            presentation.display_text(),
            &style,
            UiFrame::new(0.0, 0.0, (unwrapped.measured_width * 0.6).max(1.0), 256.0),
            None,
        );
        let physical_display_ranges = layout
            .lines
            .iter()
            .map(|line| line.source_range)
            .collect::<Vec<UiTextRange>>();

        assert!(
            physical_display_ranges.len() > 1,
            "the measured mask must soft-wrap before projection"
        );
        apply_secure_text_presentation(&mut layout, &presentation)
            .expect("each wrapped row must map through its own source signature");

        for (line, display_range) in layout.lines.iter().zip(physical_display_ranges) {
            let clusters = presentation
                .clusters_for_display_range(display_range)
                .expect("a physical row must contain complete mask graphemes");
            let bidi = presentation
                .bidi_for_display_range(display_range)
                .expect("source-owned bidi replay must remain valid")
                .expect("a non-empty physical row must have bidi metadata");
            let expected_ranges = bidi
                .visual_indices
                .iter()
                .map(|&index| clusters[index].source_range)
                .collect::<Vec<_>>();

            assert_eq!(line.direction, bidi.resolved_base_direction.into());
            assert_eq!(
                line.runs
                    .iter()
                    .map(|run| run.source_range)
                    .collect::<Vec<_>>(),
                expected_ranges,
                "a wrapped row must not reuse the full hard-line visual order"
            );
            assert_eq!(
                line.source_range,
                UiTextRange {
                    start: expected_ranges
                        .iter()
                        .map(|range| range.start)
                        .min()
                        .expect("a physical row has source ranges"),
                    end: expected_ranges
                        .iter()
                        .map(|range| range.end)
                        .max()
                        .expect("a physical row has source ranges"),
                }
            );
        }
    }
}
