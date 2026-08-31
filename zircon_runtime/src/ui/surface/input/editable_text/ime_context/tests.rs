use std::borrow::Cow;

use super::{
    FontMetrics, committed_text_for_input_method, line_height_for_metadata, range_rects_for_text,
    surrounding_text_for_state, visual_line_column_for_offset,
};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiEditableTextState, UiResolvedStyle, UiTextCaret, UiTextCaretAffinity, UiTextComposition,
        UiTextRange, UiTextSelection,
    },
};

#[test]
fn ime_fallback_line_height_uses_the_resolved_text_style_default() {
    let font_size = 15.0;

    assert_eq!(
        line_height_for_metadata(None, font_size),
        UiResolvedStyle::default_line_height(font_size)
    );
}

#[test]
fn ime_fallback_geometry_counts_combining_graphemes_once() {
    let text = "e\u{301}x";
    let combining_end = "e\u{301}".len();
    let metrics = FontMetrics {
        char_advance: 7.0,
        line_height: 16.0,
    };

    assert_eq!(
        visual_line_column_for_offset(text, combining_end, None),
        (0, 1)
    );

    let rects = range_rects_for_text(
        text,
        UiTextRange {
            start: 0,
            end: combining_end,
        },
        UiFrame::new(0.0, 0.0, 140.0, 24.0),
        metrics,
        None,
    );
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].width, metrics.char_advance);
}

#[test]
fn ime_fallback_geometry_expands_a_range_start_inside_a_combining_grapheme() {
    let text = "e\u{301}x";
    let metrics = FontMetrics {
        char_advance: 7.0,
        line_height: 16.0,
    };

    assert_eq!(visual_line_column_for_offset(text, 1, None), (0, 0));

    let rects = range_rects_for_text(
        text,
        UiTextRange { start: 1, end: 3 },
        UiFrame::new(0.0, 0.0, 140.0, 24.0),
        metrics,
        None,
    );
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].x, 0.0);
    assert_eq!(rects[0].width, metrics.char_advance);
}

#[test]
fn ime_fallback_geometry_treats_crlf_as_one_line_break() {
    let text = "left\r\nright";
    let after_line_break = "left\r\n".len();

    assert_eq!(
        visual_line_column_for_offset(text, after_line_break, None),
        (1, 0)
    );
}

#[test]
fn surrounding_text_omits_a_selection_that_exceeds_the_window() {
    let grapheme = "e\u{301}";
    let text = grapheme.repeat(600);
    let caret_offset = grapheme.len() * 300;
    let state = UiEditableTextState {
        text,
        caret: UiTextCaret {
            offset: caret_offset,
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection: Some(UiTextSelection {
            anchor: 0,
            focus: caret_offset,
        }),
        ..Default::default()
    };

    assert!(surrounding_text_for_state(&state).is_none());
}

#[test]
fn input_method_text_borrows_the_committed_text_without_a_composition() {
    let state = UiEditableTextState {
        text: "committed text".to_owned(),
        ..Default::default()
    };

    let (text, source_range, source_replacement_len) = committed_text_for_input_method(&state);

    assert!(matches!(text, Cow::Borrowed(_)));
    assert_eq!(text, "committed text");
    assert_eq!(source_range, UiTextRange::default());
    assert_eq!(source_replacement_len, 0);
}

#[test]
fn paint_only_composition_keeps_the_visible_text_and_mapping_for_ime_context() {
    let state = UiEditableTextState {
        text: "aXb".to_owned(),
        caret: UiTextCaret {
            offset: 2,
            affinity: UiTextCaretAffinity::Downstream,
        },
        composition: Some(UiTextComposition {
            range: UiTextRange { start: 1, end: 2 },
            preedit_clauses: Vec::new(),
            text: "X".to_owned(),
            restore_text: None,
        }),
        ..Default::default()
    };

    let surrounding = surrounding_text_for_state(&state).expect("visible surrounding text");

    assert_eq!(surrounding.text, "aXb");
    assert_eq!(surrounding.cursor_byte, 1);
    assert_eq!(surrounding.anchor_byte, 1);
}

#[test]
fn surrounding_text_rebases_the_restored_composition_range() {
    let state = UiEditableTextState {
        text: "aWXYZQf".to_owned(),
        caret: UiTextCaret {
            offset: 6,
            affinity: UiTextCaretAffinity::Downstream,
        },
        composition: Some(UiTextComposition {
            range: UiTextRange { start: 1, end: 6 },
            preedit_clauses: Vec::new(),
            text: "WXYZQ".to_owned(),
            restore_text: Some("bcde".to_owned()),
        }),
        ..Default::default()
    };

    let surrounding = surrounding_text_for_state(&state).expect("committed surrounding text");

    assert_eq!(surrounding.text, "abcdef");
    assert_eq!(surrounding.cursor_byte, 5);
    assert_eq!(surrounding.anchor_byte, 5);
    assert_eq!(
        surrounding.composition_range,
        Some(zircon_runtime_interface::ui::dispatch::UiTextByteRange::new(1, 5))
    );
}
