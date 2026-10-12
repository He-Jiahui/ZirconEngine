use super::*;
use crate::ui::surface::{UiResolvedTextRun, UiTextDirection, UiTextRunKind};

#[test]
fn localized_selection_and_preedit_share_one_intersecting_line_source_map() {
    let lines = (0..128)
        .map(|line_index| UiResolvedTextLine {
            text: "x".to_string(),
            placement_frame: UiFrame::default(),
            frame: UiFrame::new(0.0, line_index as f32 * 12.0, 8.0, 12.0),
            source_range: UiTextRange {
                start: line_index,
                end: line_index + 1,
            },
            visual_range: UiTextRange { start: 0, end: 1 },
            measured_width: 8.0,
            glyph_advances: vec![8.0],
            baseline: 9.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "x".to_string(),
                source_range: UiTextRange {
                    start: line_index,
                    end: line_index + 1,
                },
                visual_range: UiTextRange { start: 0, end: 1 },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        })
        .collect::<Vec<_>>();
    let layout = UiResolvedTextLayout {
        lines,
        ..Default::default()
    };
    let range = UiTextRange { start: 64, end: 65 };
    let mut decorations = Vec::new();
    let mut source_maps = TextDecorationLineSourceMaps::new(&layout.lines);

    append_range_decorations_with_source_maps(
        &mut decorations,
        &layout,
        &[
            TextRangeDecoration::composition_highlight(range),
            TextRangeDecoration::selection(range),
            TextRangeDecoration::composition_underline(range, TEXT_COMPOSITION_UNDERLINE_COLOR),
        ],
        &mut source_maps,
    );

    let caret_frame = caret_frame_with_source_maps(
        &layout,
        &UiTextCaret {
            offset: range.start,
            affinity: UiTextCaretAffinity::Downstream,
        },
        &mut source_maps,
    )
    .expect("caret frame");

    assert_eq!(source_maps.initialized_count(), 1);
    assert_eq!(
        decorations
            .iter()
            .map(|decoration| decoration.kind)
            .collect::<Vec<_>>(),
        vec![
            UiTextPaintDecorationKind::CompositionHighlight,
            UiTextPaintDecorationKind::Selection,
            UiTextPaintDecorationKind::CompositionUnderline,
        ]
    );
    assert!(decorations
        .iter()
        .all(|decoration| decoration.range == range));
    assert_eq!(decorations[0].frame, UiFrame::new(0.0, 768.0, 8.0, 12.0));
    assert_eq!(decorations[1].frame, UiFrame::new(0.0, 768.0, 8.0, 12.0));
    assert_eq!(decorations[2].frame, UiFrame::new(0.0, 778.0, 8.0, 2.0));
    assert_eq!(caret_frame, UiFrame::new(0.0, 768.0, 1.0, 12.0));
}
