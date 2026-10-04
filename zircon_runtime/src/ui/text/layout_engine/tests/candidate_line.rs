use super::*;

#[test]
fn virtual_text_keeps_later_candidate_offsets_and_source_anchors() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "سلام",
        UiTextRange { start: 0, end: 8 },
    );

    assert!(insert_virtual_text(&mut line, 2, "ـ"));
    assert!(insert_virtual_text(&mut line, 6, "ـ"));

    assert_eq!(line.text, "سـلـام");
    assert_eq!(line.source_range, UiTextRange { start: 0, end: 8 });
    assert_eq!(line.virtual_source_receipts.len(), 2);
    assert_eq!(
        line.virtual_source_receipts[0].style_source_range,
        UiTextRange { start: 0, end: 8 }
    );
    assert_eq!(
        line.virtual_source_receipts[1].style_source_range,
        UiTextRange { start: 0, end: 8 }
    );
    assert!(line
        .virtual_source_receipts
        .iter()
        .all(|receipt| receipt.replaced_source_range.is_none()));
    assert!(line
        .virtual_source_receipts
        .iter()
        .all(|receipt| { receipt.virtual_role == LogicalVirtualFragmentRole::Justification }));
    assert_eq!(
        line.runs
            .iter()
            .filter(|run| run.source_range.start == run.source_range.end)
            .map(|run| run.source_range.start)
            .collect::<Vec<_>>(),
        vec![2, 4]
    );
}

#[test]
fn discretionary_hyphen_records_a_typed_replacement_at_the_visual_anchor() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "pre",
        UiTextRange { start: 0, end: 3 },
    );
    let decision = crate::text::layout::soft_hyphen_break_suffix_at("pre\u{00ad}", 3)
        .expect("soft hyphen break decision");

    append_virtual_discretionary_hyphen(&mut line, UiTextRunKind::Plain, decision);

    assert_eq!(line.text, "pre-");
    let run = line.runs.last().expect("display-owned hyphen run");
    assert_eq!(run.source_range, UiTextRange { start: 5, end: 5 });
    assert_eq!(line.virtual_source_receipts.len(), 1);
    assert_eq!(
        line.virtual_source_receipts[0].virtual_role,
        LogicalVirtualFragmentRole::DiscretionaryHyphen
    );
    assert_eq!(
        line.virtual_source_receipts[0].replaced_source_range,
        Some(UiTextRange { start: 3, end: 5 })
    );
}
