use crate::{core::framework::text::TextDirection, text::shaping::analyze_bidi_line};
use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::UiTextRange;

use super::{
    is_secure_text_presentation_artifact, register_secure_text_presentation_artifact,
    UiSecureTextPresentation,
};

#[test]
fn secure_presentation_marker_has_stable_runtime_owner_identity() {
    let first = register_secure_text_presentation_artifact();
    let second = register_secure_text_presentation_artifact();

    assert_eq!(first, second);
    assert!(is_secure_text_presentation_artifact(&first));
}

#[test]
fn masks_each_extended_grapheme_and_preserves_hard_line_separators() {
    let source = "a\u{0301}\u{4E2D}\u{1F9D1}\u{200D}\u{1F4BB}\r\nb";

    let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();

    assert_eq!(
        presentation.display_text(),
        "\u{2022}\u{2022}\u{2022}\r\n\u{2022}"
    );
    assert_eq!(presentation.source_len(), source.len());
    assert_eq!(presentation.lines().len(), 2);
    assert_eq!(
        presentation.lines()[0].source_range,
        UiTextRange { start: 0, end: 17 }
    );
    assert_eq!(
        presentation.lines()[0].display_range,
        UiTextRange { start: 0, end: 9 }
    );
    assert_eq!(presentation.lines()[0].cluster_range, 0..3);
    assert_eq!(
        presentation.lines()[1].source_range,
        UiTextRange { start: 19, end: 20 }
    );
    assert_eq!(
        presentation.lines()[1].display_range,
        UiTextRange { start: 11, end: 14 }
    );
    assert_eq!(
        presentation
            .clusters()
            .iter()
            .map(|cluster| (
                cluster.source_range,
                cluster.display_range,
                cluster.is_hard_line_separator,
            ))
            .collect::<Vec<_>>(),
        vec![
            (
                UiTextRange { start: 0, end: 3 },
                UiTextRange { start: 0, end: 3 },
                false,
            ),
            (
                UiTextRange { start: 3, end: 6 },
                UiTextRange { start: 3, end: 6 },
                false,
            ),
            (
                UiTextRange { start: 6, end: 17 },
                UiTextRange { start: 6, end: 9 },
                false,
            ),
            (
                UiTextRange { start: 17, end: 19 },
                UiTextRange { start: 9, end: 11 },
                true,
            ),
            (
                UiTextRange { start: 19, end: 20 },
                UiTextRange { start: 11, end: 14 },
                false,
            ),
        ]
    );
}

#[test]
fn source_and_display_ranges_round_trip_at_atomic_boundaries_only() {
    let source = "a\u{0301}\u{4E2D}\u{1F9D1}\u{200D}\u{1F4BB}\r\nb";
    let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();

    let source_range = UiTextRange { start: 3, end: 19 };
    let display_range = UiTextRange { start: 3, end: 11 };
    assert_eq!(
        presentation.display_range_for_source_range(source_range),
        Some(display_range)
    );
    assert_eq!(
        presentation.source_range_for_display_range(display_range),
        Some(source_range)
    );
    assert_eq!(presentation.display_offset_for_source_boundary(1), None);
    assert_eq!(presentation.source_offset_for_display_boundary(1), None);
}

#[test]
fn render_editable_state_contains_only_mask_text_and_source_offsets() {
    let source = "a\u{0301}\u{4E2D}";
    let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();
    let source_state = zircon_runtime_interface::ui::surface::UiEditableTextState {
        text: source.to_string(),
        caret: zircon_runtime_interface::ui::surface::UiTextCaret {
            offset: source.len().saturating_add(4),
            affinity: zircon_runtime_interface::ui::surface::UiTextCaretAffinity::Downstream,
        },
        selection: Some(zircon_runtime_interface::ui::surface::UiTextSelection {
            anchor: 0,
            focus: source.len().saturating_add(2),
        }),
        composition: Some(zircon_runtime_interface::ui::surface::UiTextComposition::default()),
        read_only: true,
    };

    let render_state = presentation.render_editable_state(&source_state);

    assert_eq!(render_state.text, "\u{2022}\u{2022}");
    assert_eq!(render_state.caret.offset, source.len());
    assert_eq!(
        render_state.selection,
        Some(zircon_runtime_interface::ui::surface::UiTextSelection {
            anchor: 0,
            focus: source.len(),
        })
    );
    assert_eq!(render_state.composition, None);
    assert!(render_state.read_only);
}

#[test]
fn preserves_original_rtl_bidi_order_instead_of_reanalyzing_mask_glyphs() {
    let source = "\u{0645}\u{0631}\u{062D}\u{0628}\u{0627} abc";
    let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();
    let source_ranges = source
        .grapheme_indices(true)
        .map(|(start, grapheme)| crate::text::TextRange {
            start,
            end: start + grapheme.len(),
        })
        .collect::<Vec<_>>();
    let expected = analyze_bidi_line(
        source,
        TextDirection::Auto,
        crate::text::TextRange {
            start: 0,
            end: source.len(),
        },
        &source_ranges,
    )
    .unwrap();

    let line = &presentation.lines()[0];
    assert_eq!(
        line.bidi.resolved_base_direction,
        expected.resolved_base_direction
    );
    assert_eq!(line.bidi.logical_levels, expected.logical_levels);
    assert_eq!(line.bidi.visual_indices, expected.visual_indices);
    assert_eq!(
        line.bidi.resolved_base_direction,
        TextDirection::RightToLeft
    );
    assert_ne!(
        line.bidi.visual_indices,
        (0..presentation.clusters().len()).collect::<Vec<_>>()
    );
    let first_visual_logical_index = line.bidi.visual_indices[0];
    assert_eq!(
        presentation.cluster_for_line_visual_index(0, 0),
        presentation
            .clusters()
            .get(first_visual_logical_index)
            .copied()
    );
}

#[test]
fn wrapped_secure_rtl_line_replays_source_l1_instead_of_analyzing_bullets() {
    let source = "\u{05D0}\u{05D1} abc ";
    let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto).unwrap();
    let source_ranges = source
        .grapheme_indices(true)
        .map(|(start, grapheme)| crate::text::TextRange {
            start,
            end: start + grapheme.len(),
        })
        .collect::<Vec<_>>();
    let display_range = UiTextRange {
        start: presentation.clusters()[0].display_range.start,
        end: presentation.clusters()[2].display_range.end,
    };
    let expected = analyze_bidi_line(
        source,
        TextDirection::Auto,
        crate::text::TextRange {
            start: 0,
            end: source_ranges[2].end,
        },
        &source_ranges[..3],
    )
    .unwrap();

    let actual = presentation
        .bidi_for_display_range(display_range)
        .unwrap()
        .unwrap();

    assert_eq!(
        actual.resolved_base_direction,
        expected.resolved_base_direction
    );
    assert_eq!(actual.logical_levels, expected.logical_levels);
    assert_eq!(actual.visual_indices, expected.visual_indices);
    assert_eq!(
        presentation.bidi_for_display_range(UiTextRange { start: 1, end: 3 }),
        Ok(None)
    );
}

#[test]
fn display_range_lookup_reaches_late_hard_lines_without_reinterpreting_mask_text() {
    const HARD_LINE_COUNT: usize = 64;
    let source = (0..HARD_LINE_COUNT)
        .map(|_| "\u{05d0}a")
        .collect::<Vec<_>>()
        .join("\n");
    let presentation = UiSecureTextPresentation::new(&source, TextDirection::Auto).unwrap();
    let line = presentation
        .lines()
        .last()
        .expect("the final hard line must be retained");

    let order = presentation
        .bidi_for_display_range(line.display_range)
        .unwrap()
        .expect("a complete final mask line must replay source-owned bidi");

    assert_eq!(presentation.lines().len(), HARD_LINE_COUNT);
    assert_eq!(order.logical_levels.len(), 2);
    assert_eq!(order.visual_indices.len(), 2);
    assert_eq!(
        presentation.display_offset_for_source_boundary(line.source_range.start),
        Some(line.display_range.start)
    );
    assert_eq!(
        presentation.source_offset_for_display_boundary(line.display_range.end),
        Some(line.source_range.end)
    );
    assert_eq!(
        presentation.bidi_for_display_range(UiTextRange {
            start: line.display_range.start.saturating_add(1),
            end: line.display_range.end,
        }),
        Ok(None),
        "a non-atomic display boundary remains invalid even on a late hard line"
    );
}

#[test]
fn secure_layout_projection_keeps_mask_text_and_original_grapheme_ranges() {
    use crate::ui::text::layout_engine::{apply_secure_text_presentation, layout_text};
    use zircon_runtime_interface::ui::{
        layout::UiFrame,
        surface::{
            UiResolvedStyle, UiTextLineSourceMap, UiTextOverflow, UiTextVisualBoundaryBias,
            UiTextWrap,
        },
    };

    let source = "A \u{0645}\u{0631}\u{062d}\u{0628}\u{0627} \u{4e2d}";
    let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto)
        .expect("a valid source must produce a secure presentation");
    let style = UiResolvedStyle {
        wrap: UiTextWrap::None,
        text_overflow: UiTextOverflow::Clip,
        font_size: 16.0,
        line_height: 20.0,
        ..UiResolvedStyle::default()
    };
    let mut layout = layout_text(
        presentation.display_text(),
        &style,
        UiFrame::new(0.0, 0.0, 160.0, 24.0),
        None,
    );

    apply_secure_text_presentation(&mut layout, &presentation)
        .expect("secure projection must preserve valid source cluster ownership");

    let line = layout.lines.first().expect("one resolved line");
    assert_eq!(line.text, presentation.display_text());
    assert!(line.runs.iter().all(|run| run.text == "\u{2022}"));
    assert_eq!(
        line.runs
            .iter()
            .map(|run| run.source_range)
            .collect::<Vec<_>>(),
        presentation.lines()[0]
            .bidi
            .visual_indices
            .iter()
            .map(|&index| presentation.clusters()[index].source_range)
            .collect::<Vec<_>>()
    );
    let source_map = UiTextLineSourceMap::new(line);
    let caret = source_map.caret_for_visual_boundary(
        0,
        UiTextVisualBoundaryBias::LeadingCurrent,
        usize::MAX,
    );
    assert_ne!(caret.offset, usize::MAX);
    assert!(caret.offset <= source.len());
}

#[test]
fn secure_layout_projection_reanchors_start_for_each_source_owned_hard_line_direction() {
    use crate::ui::text::layout_engine::{apply_secure_text_presentation, layout_text};
    use zircon_runtime_interface::ui::{
        layout::UiFrame,
        surface::{UiResolvedStyle, UiTextAlign, UiTextOverflow, UiTextWrap},
    };

    let source = "abc\n\u{05D0}\u{05D1}";
    let presentation = UiSecureTextPresentation::new(source, TextDirection::Auto)
        .expect("a valid source must produce a secure presentation");
    let style = UiResolvedStyle {
        text_align: UiTextAlign::Start,
        wrap: UiTextWrap::None,
        text_overflow: UiTextOverflow::Clip,
        ..UiResolvedStyle::default()
    };
    let frame = UiFrame::new(10.0, 0.0, 120.0, 48.0);
    let mut layout = layout_text(presentation.display_text(), &style, frame, None);

    apply_secure_text_presentation(&mut layout, &presentation)
        .expect("secure projection must preserve each hard-line direction");

    assert_eq!(layout.lines.len(), 2);
    assert!((layout.lines[0].frame.x - frame.x).abs() < 0.01);
    assert_eq!(
        layout.lines[1].direction,
        zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft
    );
    assert!((layout.lines[1].frame.right() - frame.right()).abs() < 0.01);
}
