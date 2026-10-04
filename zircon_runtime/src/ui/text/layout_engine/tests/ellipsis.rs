use super::*;
use crate::ui::text::layout_engine::candidate_line::append_segment;

#[test]
fn extreme_finite_advances_do_not_trigger_overflow_ellipsis() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "ab",
        UiTextRange { start: 0, end: 2 },
    );
    let mut advances = vec![f32::MAX, f32::MAX];

    ellipsize_line_with_advances(
        &mut line,
        &mut advances,
        f32::MAX,
        1.0,
        UiTextOverflow::Ellipsis,
    );

    assert_eq!(line.text, "ab");
    assert_eq!(advances, vec![f32::MAX, f32::MAX]);
    assert!(!line.ellipsized);
}

#[test]
fn start_ellipsis_keeps_the_following_rich_run_as_its_style_owner() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "ab",
        UiTextRange { start: 0, end: 2 },
    );
    append_segment(
        &mut line,
        UiTextRunKind::Strong,
        "cd",
        UiTextRange { start: 2, end: 4 },
    );
    let mut advances = vec![1.0; 4];
    let style_owner =
        ellipsis_style_owner_source_range(&line, &advances, 2.0, UiTextOverflow::EllipsisStart);

    ellipsize_line_with_advances_and_style_owner(
        &mut line,
        &mut advances,
        2.0,
        1.0,
        UiTextOverflow::EllipsisStart,
        style_owner,
    );

    assert_eq!(line.text, "\u{2026}d");
    assert_eq!(style_owner, Some(UiTextRange { start: 2, end: 4 }));
    assert_eq!(line.runs[0].kind, UiTextRunKind::Strong);
    assert_eq!(
        line.virtual_source_receipts
            .iter()
            .copied()
            .find(|receipt| receipt.visual_range == line.runs[0].visual_range),
        Some(VirtualTextSourceReceipt {
            visual_range: line.runs[0].visual_range,
            style_source_range: style_owner.expect("style owner"),
            replaced_source_range: Some(UiTextRange { start: 0, end: 3 }),
            virtual_role: LogicalVirtualFragmentRole::Ellipsis,
        })
    );
}

#[test]
fn ellipsis_receipts_cover_the_single_omitted_source_interval() {
    for (overflow, expected_text, expected_omitted) in [
        (
            UiTextOverflow::Ellipsis,
            "ab\u{2026}",
            UiTextRange { start: 2, end: 6 },
        ),
        (
            UiTextOverflow::EllipsisMiddle,
            "a\u{2026}f",
            UiTextRange { start: 1, end: 5 },
        ),
        (
            UiTextOverflow::EllipsisStart,
            "\u{2026}ef",
            UiTextRange { start: 0, end: 4 },
        ),
    ] {
        let mut line = CandidateLine::empty();
        append_segment(
            &mut line,
            UiTextRunKind::Plain,
            "abcdef",
            UiTextRange { start: 0, end: 6 },
        );
        let mut advances = vec![1.0; 6];

        ellipsize_line_with_advances(&mut line, &mut advances, 3.0, 1.0, overflow);

        let marker = line
            .runs
            .iter()
            .find(|run| run.text == ELLIPSIS)
            .expect("ellipsis run");
        assert_eq!(line.text, expected_text);
        assert_eq!(
            line.virtual_source_receipts
                .iter()
                .copied()
                .find(|receipt| receipt.visual_range == marker.visual_range)
                .and_then(|receipt| receipt.replaced_source_range),
            Some(expected_omitted)
        );
    }
}

#[test]
fn omitted_source_receipt_fails_closed_for_multiple_disjoint_gaps() {
    let retained_runs = [
        UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "a".to_string(),
            source_range: UiTextRange { start: 0, end: 1 },
            visual_range: UiTextRange { start: 0, end: 1 },
            direction: UiTextDirection::LeftToRight,
        },
        UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "c".to_string(),
            source_range: UiTextRange { start: 2, end: 3 },
            visual_range: UiTextRange { start: 1, end: 2 },
            direction: UiTextDirection::LeftToRight,
        },
        UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "f".to_string(),
            source_range: UiTextRange { start: 5, end: 6 },
            visual_range: UiTextRange { start: 2, end: 3 },
            direction: UiTextDirection::LeftToRight,
        },
    ];

    assert_eq!(
        single_omitted_source_range(UiTextRange { start: 0, end: 6 }, &retained_runs),
        None
    );
}
