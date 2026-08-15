use super::{
    physical_paragraph_ranges, physical_paragraph_start, resolve_physical_paragraph_override_spans,
    CandidateLine, ColumnConstraints, LineConstraints, ParagraphOverrideSpan,
    ResolvedParagraphColumnConstraints, ResolvedParagraphLineConstraints,
    ResolvedPhysicalParagraphColumns, ResolvedPhysicalParagraphLines,
};
use crate::text::ParagraphOverride;
use zircon_runtime_interface::ui::surface::UiTextAlign;
use zircon_runtime_interface::ui::surface::UiTextRange;

#[test]
fn physical_paragraphs_follow_canonical_hard_line_separators() {
    let text = "one\u{2028}two\r\nthree\u{0085}four\u{2029}five\u{000b}six\u{000c}seven\reight";
    let starts = [
        0,
        "one\u{2028}".len(),
        "one\u{2028}two\r\n".len(),
        "one\u{2028}two\r\nthree\u{0085}".len(),
        "one\u{2028}two\r\nthree\u{0085}four\u{2029}".len(),
        "one\u{2028}two\r\nthree\u{0085}four\u{2029}five\u{000b}".len(),
        "one\u{2028}two\r\nthree\u{0085}four\u{2029}five\u{000b}six\u{000c}".len(),
    ];
    let ends = [
        "one".len(),
        "one\u{2028}two".len(),
        "one\u{2028}two\r\nthree".len(),
        "one\u{2028}two\r\nthree\u{0085}four".len(),
        "one\u{2028}two\r\nthree\u{0085}four\u{2029}five".len(),
        "one\u{2028}two\r\nthree\u{0085}four\u{2029}five\u{000b}six".len(),
        text.len(),
    ];

    assert_eq!(
        physical_paragraph_ranges(text),
        starts
            .into_iter()
            .zip(ends)
            .map(|(start, end)| UiTextRange { start, end })
            .collect::<Vec<_>>()
    );
    assert_eq!(physical_paragraph_start(text, starts[4] + 1), starts[4]);
    assert_eq!(physical_paragraph_start(text, starts[6] + 1), starts[6]);
}

#[test]
fn paragraph_constraint_index_resolves_terminal_and_nonterminal_ranges() {
    let fallback = ColumnConstraints {
        inset: 0.0,
        max_height: 10.0,
        align: UiTextAlign::Left,
    };
    let constraints = ResolvedParagraphColumnConstraints {
        paragraphs: vec![
            ResolvedPhysicalParagraphColumns {
                range: UiTextRange { start: 0, end: 3 },
                first: ColumnConstraints {
                    inset: 1.0,
                    max_height: 9.0,
                    align: UiTextAlign::Center,
                },
                continuation: ColumnConstraints {
                    inset: 2.0,
                    max_height: 8.0,
                    align: UiTextAlign::Center,
                },
            },
            ResolvedPhysicalParagraphColumns {
                range: UiTextRange { start: 4, end: 6 },
                first: ColumnConstraints {
                    inset: 3.0,
                    max_height: 7.0,
                    align: UiTextAlign::Right,
                },
                continuation: ColumnConstraints {
                    inset: 4.0,
                    max_height: 6.0,
                    align: UiTextAlign::Right,
                },
            },
            ResolvedPhysicalParagraphColumns {
                range: UiTextRange { start: 7, end: 7 },
                first: ColumnConstraints {
                    inset: 5.0,
                    max_height: 5.0,
                    align: UiTextAlign::End,
                },
                continuation: ColumnConstraints {
                    inset: 6.0,
                    max_height: 4.0,
                    align: UiTextAlign::End,
                },
            },
        ],
        fallback,
    };

    let first = constraints.for_source_offset(1, true);
    let continuation = constraints.for_source_offset(1, false);
    let second = constraints.for_source_offset(4, true);
    let terminal = constraints.for_source_offset(7, true);
    let separator = constraints.for_source_offset(3, true);
    let mut columns = vec![CandidateLine::empty(); 4];
    columns[0].source_range = UiTextRange { start: 0, end: 1 };
    columns[1].source_range = UiTextRange { start: 1, end: 3 };
    columns[2].source_range = UiTextRange { start: 4, end: 6 };
    columns[3].source_range = UiTextRange { start: 7, end: 7 };
    let projected = constraints.for_candidates(&columns);

    assert_eq!(
        (first.inset, first.max_height, first.align),
        (1.0, 9.0, UiTextAlign::Center)
    );
    assert_eq!(
        (
            continuation.inset,
            continuation.max_height,
            continuation.align
        ),
        (2.0, 8.0, UiTextAlign::Center)
    );
    assert_eq!(
        (second.inset, second.max_height, second.align),
        (3.0, 7.0, UiTextAlign::Right)
    );
    assert_eq!(
        (terminal.inset, terminal.max_height, terminal.align),
        (5.0, 5.0, UiTextAlign::End)
    );
    assert_eq!(
        (separator.inset, separator.max_height, separator.align),
        (fallback.inset, fallback.max_height, fallback.align)
    );
    assert_eq!(
        projected
            .iter()
            .map(|constraint| (constraint.inset, constraint.max_height, constraint.align))
            .collect::<Vec<_>>(),
        vec![
            (1.0, 9.0, UiTextAlign::Center),
            (2.0, 8.0, UiTextAlign::Center),
            (3.0, 7.0, UiTextAlign::Right),
            (5.0, 5.0, UiTextAlign::End),
        ]
    );
}

#[test]
fn horizontal_paragraph_constraints_project_first_and_continuation_line_widths() {
    let constraints = ResolvedParagraphLineConstraints {
        paragraphs: vec![ResolvedPhysicalParagraphLines {
            range: UiTextRange { start: 2, end: 6 },
            first: LineConstraints {
                inset: 3.0,
                max_width: 17.0,
                align: UiTextAlign::Center,
            },
            continuation: LineConstraints {
                inset: 5.0,
                max_width: 15.0,
                align: UiTextAlign::Center,
            },
        }],
        fallback: LineConstraints {
            inset: 0.0,
            max_width: 20.0,
            align: UiTextAlign::Left,
        },
    };
    let mut lines = vec![CandidateLine::empty(); 3];
    lines[0].source_range = UiTextRange { start: 2, end: 3 };
    lines[1].source_range = UiTextRange { start: 3, end: 6 };
    lines[2].source_range = UiTextRange { start: 6, end: 6 };

    assert_eq!(
        constraints
            .for_candidates(&lines)
            .iter()
            .map(|constraint| (constraint.inset, constraint.max_width, constraint.align))
            .collect::<Vec<_>>(),
        vec![
            (3.0, 17.0, UiTextAlign::Center),
            (5.0, 15.0, UiTextAlign::Center),
            (0.0, 20.0, UiTextAlign::Left),
        ]
    );
}

#[test]
fn paragraph_override_sweep_restores_the_outer_override_after_a_nested_range() {
    let mut outer = ParagraphOverride::default();
    outer.indent_level = Some(2);
    outer.indent = Some(3.0);
    outer.align = Some(UiTextAlign::Left.into());

    let mut nested = ParagraphOverride::default();
    nested.indent_level = Some(3);
    nested.indent = Some(4.0);
    nested.align = Some(UiTextAlign::Right.into());

    let resolved = resolve_physical_paragraph_override_spans(
        vec![
            UiTextRange { start: 0, end: 3 },
            UiTextRange { start: 4, end: 6 },
            UiTextRange { start: 7, end: 9 },
        ],
        vec![
            ParagraphOverrideSpan {
                range: UiTextRange { start: 0, end: 9 },
                paragraph: outer,
                list_prefix: None,
                order: 0,
            },
            ParagraphOverrideSpan {
                range: UiTextRange { start: 4, end: 6 },
                paragraph: nested,
                list_prefix: None,
                order: 1,
            },
        ],
    );

    assert_eq!(resolved[0].paragraph.indent_level, Some(2));
    assert_eq!(resolved[0].paragraph.indent, Some(3.0));
    assert_eq!(resolved[0].paragraph.align, Some(UiTextAlign::Left.into()));
    assert_eq!(resolved[1].paragraph.indent_level, Some(5));
    assert_eq!(resolved[1].paragraph.indent, Some(7.0));
    assert_eq!(resolved[1].paragraph.align, Some(UiTextAlign::Right.into()));
    assert_eq!(resolved[2].paragraph.indent_level, Some(2));
    assert_eq!(resolved[2].paragraph.indent, Some(3.0));
    assert_eq!(resolved[2].paragraph.align, Some(UiTextAlign::Left.into()));
}

#[test]
fn paragraph_override_sweep_prefers_later_starts_and_later_orders_for_equal_spans() {
    let mut earlier = ParagraphOverride::default();
    earlier.align = Some(UiTextAlign::Left.into());
    let mut later_start = ParagraphOverride::default();
    later_start.align = Some(UiTextAlign::Right.into());
    let mut later_order = ParagraphOverride::default();
    later_order.align = Some(UiTextAlign::Center.into());

    let later_start_resolved = resolve_physical_paragraph_override_spans(
        vec![UiTextRange { start: 3, end: 4 }],
        vec![
            ParagraphOverrideSpan {
                range: UiTextRange { start: 0, end: 8 },
                paragraph: earlier.clone(),
                list_prefix: None,
                order: 0,
            },
            ParagraphOverrideSpan {
                range: UiTextRange { start: 2, end: 10 },
                paragraph: later_start,
                list_prefix: None,
                order: 1,
            },
        ],
    );
    let later_order_resolved = resolve_physical_paragraph_override_spans(
        vec![UiTextRange { start: 3, end: 4 }],
        vec![
            ParagraphOverrideSpan {
                range: UiTextRange { start: 2, end: 10 },
                paragraph: earlier,
                list_prefix: None,
                order: 2,
            },
            ParagraphOverrideSpan {
                range: UiTextRange { start: 2, end: 10 },
                paragraph: later_order,
                list_prefix: None,
                order: 3,
            },
        ],
    );

    assert_eq!(
        later_start_resolved[0].paragraph.align,
        Some(UiTextAlign::Right.into())
    );
    assert_eq!(
        later_order_resolved[0].paragraph.align,
        Some(UiTextAlign::Center.into())
    );
}
