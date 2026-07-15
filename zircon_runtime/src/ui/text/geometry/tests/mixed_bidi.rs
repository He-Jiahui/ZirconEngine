use super::*;
use zircon_runtime_interface::ui::surface::{UiResolvedTextRun, UiTextDirection, UiTextRunKind};

#[test]
fn mixed_bidi_caret_affinity_and_range_geometry_follow_visual_source_map() {
    let layout = mixed_bidi_layout();
    let upstream = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 4,
            affinity: UiTextCaretAffinity::Upstream,
        },
    )
    .expect("upstream mixed-BiDi caret");
    let downstream = caret_frame_for_text_layout(
        &layout,
        &UiTextCaret {
            offset: 4,
            affinity: UiTextCaretAffinity::Downstream,
        },
    )
    .expect("downstream mixed-BiDi caret");
    let selection = text_range_frames_for_text_layout(&layout, UiTextRange { start: 0, end: 6 });

    assert_eq!(upstream, UiFrame::new(50.0, 20.0, 1.0, 12.0));
    assert_eq!(downstream, UiFrame::new(70.0, 20.0, 1.0, 12.0));
    assert_eq!(
        selection,
        vec![
            UiFrame::new(10.0, 20.0, 40.0, 12.0),
            UiFrame::new(60.0, 20.0, 10.0, 12.0),
        ]
    );
}

fn mixed_bidi_layout() -> UiResolvedTextLayout {
    UiResolvedTextLayout {
        font_size: 10.0,
        line_height: 12.0,
        source_range: UiTextRange { start: 0, end: 8 },
        direction: UiTextDirection::LeftToRight,
        lines: vec![UiResolvedTextLine {
            text: "abc בא".to_string(),
            frame: UiFrame::new(10.0, 20.0, 60.0, 12.0),
            source_range: UiTextRange { start: 0, end: 8 },
            visual_range: UiTextRange { start: 0, end: 8 },
            measured_width: 60.0,
            glyph_advances: vec![10.0; 6],
            baseline: 9.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![
                resolved_run("abc ", 0, 4, 0, 4, UiTextDirection::LeftToRight),
                resolved_run("ב", 6, 8, 4, 6, UiTextDirection::RightToLeft),
                resolved_run("א", 4, 6, 6, 8, UiTextDirection::RightToLeft),
            ],
            ellipsized: false,
        }],
        ..UiResolvedTextLayout::default()
    }
}

fn resolved_run(
    text: &str,
    source_start: usize,
    source_end: usize,
    visual_start: usize,
    visual_end: usize,
    direction: UiTextDirection,
) -> UiResolvedTextRun {
    UiResolvedTextRun {
        kind: UiTextRunKind::Plain,
        text: text.to_string(),
        source_range: UiTextRange {
            start: source_start,
            end: source_end,
        },
        visual_range: UiTextRange {
            start: visual_start,
            end: visual_end,
        },
        direction,
    }
}
