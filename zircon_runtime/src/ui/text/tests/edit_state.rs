use super::*;

fn legacy_replace_selection_or_range(
    state: &mut UiEditableTextState,
    text: &str,
) -> Option<CommittedTextEditIntent> {
    let tracks_committed_source = state.composition.is_none();
    if let Some(selection) = state.selection.take() {
        let range = selection.range();
        let start = clamp_grapheme_boundary(&state.text, range.start);
        let end = clamp_grapheme_boundary(&state.text, range.end).max(start);
        let unchanged = state.text.get(start..end) == Some(text);
        replace_range_preserving_composition(state, start, end, text);
        state.composition = None;
        (tracks_committed_source && !unchanged)
            .then(|| committed_edit_intent(start..end, text.len()))
    } else {
        let offset = clamp_grapheme_boundary(&state.text, state.caret.offset);
        replace_range_preserving_composition(state, offset, offset, text);
        state.composition = None;
        (tracks_committed_source && !text.is_empty())
            .then(|| committed_edit_intent(offset..offset, text.len()))
    }
}

#[test]
fn unicode_selection_and_caret_replacement_match_the_legacy_clamped_path() {
    let text = "\u{0600}a\u{0301}b\r\nc";
    for selection in [
        Some(UiTextSelection {
            anchor: 3,
            focus: 5,
        }),
        None,
    ] {
        let mut expected = editable_text_state(text, 4, None);
        expected.selection = selection.clone();
        let mut actual = expected.clone();

        let expected_intent = legacy_replace_selection_or_range(&mut expected, "X");
        let actual_intent = replace_selection_or_range(&mut actual, "X");
        assert_eq!(actual, expected);
        assert_eq!(actual_intent, expected_intent);
    }
}

#[test]
fn unicode_backspace_and_delete_keep_exact_grapheme_ranges() {
    let text = "a\u{0301}b\r\nc";
    let backspaced = apply_text_edit_action_with_intent(
        editable_text_state(text, "a\u{0301}".len(), None),
        UiTextEditAction::Backspace,
    );
    assert_eq!(backspaced.state.text, "b\r\nc");
    assert_eq!(backspaced.committed.unwrap().old, 0.."a\u{0301}".len());

    let deleted = apply_text_edit_action_with_intent(
        editable_text_state(text, 0, None),
        UiTextEditAction::Delete,
    );
    assert_eq!(deleted.state.text, "b\r\nc");
    assert_eq!(deleted.committed.unwrap().old, 0.."a\u{0301}".len());
}

#[test]
#[ignore = "managed Windows Release Unicode edit-path performance evidence"]
fn unicode_selection_replace_single_clamp_release_p95() {
    use std::{hint::black_box, time::Instant};

    const SAMPLES: usize = 31;
    const MARKER: &str = "RUNTIME82_UNICODE_SELECTION_REPLACE_SINGLE_CLAMP_BENCH_V1";

    fn measure(
        replace: fn(&mut UiEditableTextState, &str) -> Option<CommittedTextEditIntent>,
        template: &UiEditableTextState,
        offset: usize,
        iterations: usize,
    ) -> u128 {
        let mut state = template.clone();
        let start = Instant::now();
        for iteration in 0..iterations {
            state.selection = Some(UiTextSelection {
                anchor: offset,
                focus: offset + "a\u{0301}".len(),
            });
            let replacement = if iteration % 2 == 0 {
                "b\u{0301}"
            } else {
                "a\u{0301}"
            };
            black_box(replace(black_box(&mut state), black_box(replacement)));
        }
        let elapsed = start.elapsed().as_nanos();
        assert_eq!(&state.text[..offset], &template.text[..offset]);
        assert_eq!(&state.text[offset..], "b\u{0301}");
        assert_eq!(state.caret.offset, offset + "b\u{0301}".len());
        black_box(&state);
        elapsed
    }

    for (graphemes, iterations) in [(128, 255), (1_024, 63), (8_192, 7)] {
        let text = "a\u{0301}".repeat(graphemes);
        let offset = text.len() - "a\u{0301}".len();
        let template = editable_text_state(&text, offset, None);
        let mut expected = template.clone();
        let mut actual = template.clone();
        expected.selection = Some(UiTextSelection {
            anchor: offset,
            focus: text.len(),
        });
        actual.selection = expected.selection.clone();
        assert_eq!(
            legacy_replace_selection_or_range(&mut expected, "b\u{0301}"),
            replace_selection_or_range(&mut actual, "b\u{0301}")
        );
        assert_eq!(actual, expected);
        assert_eq!(&actual.text[offset..], "b\u{0301}");

        for _ in 0..5 {
            black_box(measure(
                legacy_replace_selection_or_range,
                &template,
                offset,
                iterations,
            ));
            black_box(measure(
                replace_selection_or_range,
                &template,
                offset,
                iterations,
            ));
        }
        let mut legacy = Vec::with_capacity(SAMPLES);
        let mut optimized = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            if sample % 2 == 0 {
                legacy.push(measure(
                    legacy_replace_selection_or_range,
                    &template,
                    offset,
                    iterations,
                ));
                optimized.push(measure(
                    replace_selection_or_range,
                    &template,
                    offset,
                    iterations,
                ));
            } else {
                optimized.push(measure(
                    replace_selection_or_range,
                    &template,
                    offset,
                    iterations,
                ));
                legacy.push(measure(
                    legacy_replace_selection_or_range,
                    &template,
                    offset,
                    iterations,
                ));
            }
        }
        let mut legacy_sorted = legacy.clone();
        let mut optimized_sorted = optimized.clone();
        legacy_sorted.sort_unstable();
        optimized_sorted.sort_unstable();
        let rank = |samples: &[u128], percentile: usize| {
            samples[(samples.len() * percentile).div_ceil(100) - 1]
        };
        let legacy_p95 = rank(&legacy_sorted, 95);
        let optimized_p95 = rank(&optimized_sorted, 95);
        println!(
            "{MARKER} graphemes={graphemes} text_bytes={} iterations={iterations} warmups=5 samples={SAMPLES} legacy_p50_ns={} legacy_p95_ns={legacy_p95} legacy_p99_ns={} optimized_p50_ns={} optimized_p95_ns={optimized_p95} optimized_p99_ns={} legacy_raw_ns={legacy:?} optimized_raw_ns={optimized:?} os={} arch={} package_version={}",
            text.len(),
            rank(&legacy_sorted, 50),
            rank(&legacy_sorted, 99),
            rank(&optimized_sorted, 50),
            rank(&optimized_sorted, 99),
            std::env::consts::OS,
            std::env::consts::ARCH,
            env!("CARGO_PKG_VERSION")
        );
        if graphemes >= 1_024 {
            assert!(
                optimized_p95.saturating_mul(4) <= legacy_p95.saturating_mul(3),
                "Unicode selection replace P95 must be at most 75% of the old path"
            );
        }
    }
}

#[test]
fn set_composition_replaces_explicit_range_without_extending_to_preedit_len() {
    let state = editable_text_state("abcdef", 5, Some(UiTextRange { start: 1, end: 5 }));

    let next = apply_text_edit_action(
        state,
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 5 },
            text: "WXYZQ".to_string(),
        },
    );

    let composition = next.composition.as_ref().expect("composition");
    assert_eq!(next.text, "aWXYZQf");
    assert_eq!(next.caret.offset, 6);
    assert_eq!(composition.range, UiTextRange { start: 1, end: 6 });
    assert_eq!(composition.restore_text.as_deref(), Some("bcde"));
}

#[test]
fn set_composition_update_reuses_restored_source_range() {
    let state = editable_text_state("abcdef", 5, Some(UiTextRange { start: 1, end: 5 }));
    let first = apply_text_edit_action(
        state,
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 5 },
            text: "WXYZQ".to_string(),
        },
    );
    let visible_composition_range = first.composition.as_ref().unwrap().range;

    let next = apply_text_edit_action(
        first,
        UiTextEditAction::SetComposition {
            range: visible_composition_range,
            text: "UV".to_string(),
        },
    );

    let composition = next.composition.as_ref().expect("composition");
    assert_eq!(next.text, "aUVf");
    assert_eq!(next.caret.offset, 3);
    assert_eq!(composition.range, UiTextRange { start: 1, end: 3 });
    assert_eq!(composition.restore_text.as_deref(), Some("bcde"));
}

#[test]
fn external_selection_and_composition_offsets_do_not_split_a_grapheme() {
    let selected = apply_text_edit_action(
        editable_text_state("a\u{0301}b", 0, None),
        UiTextEditAction::SetSelection {
            anchor: 1,
            focus: 2,
        },
    );
    assert_eq!(
        selected.selection,
        Some(UiTextSelection {
            anchor: 0,
            focus: 0,
        })
    );
    assert_eq!(selected.caret.offset, 0);

    let composed = apply_text_edit_action(
        editable_text_state("a\u{0301}b", 0, None),
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 2 },
            text: "x".to_string(),
        },
    );
    assert_eq!(composed.text, "xa\u{0301}b");
    assert_eq!(
        composed
            .composition
            .as_ref()
            .map(|composition| composition.range),
        Some(UiTextRange { start: 0, end: 1 })
    );
}

#[test]
fn committed_insert_reports_exact_old_new_ranges_without_another_text_copy() {
    let selected = apply_text_edit_action(
        editable_text_state("alpha", 5, None),
        UiTextEditAction::SetSelection {
            anchor: 1,
            focus: 4,
        },
    );

    let transition = apply_text_edit_action_with_intent(
        selected,
        UiTextEditAction::Insert {
            text: "XYZ".to_string(),
        },
    );

    let intent = transition.committed.as_ref().expect("committed intent");
    assert_eq!(transition.state.text, "aXYZa");
    assert_eq!(intent.old, 1..4);
    assert_eq!(intent.new, 1..4);
    assert_eq!(intent.kind, UiTextEditKind::Replace);
    assert_eq!(intent.replacement(&transition.state), Some("XYZ"));
}

#[test]
fn grapheme_delete_and_caret_move_distinguish_committed_and_state_only_changes() {
    let deleted = apply_text_edit_action_with_intent(
        editable_text_state("a\u{0301}b", 3, None),
        UiTextEditAction::Backspace,
    );
    let intent = deleted.committed.as_ref().expect("delete intent");
    assert_eq!(deleted.state.text, "b");
    assert_eq!(intent.old, 0..3);
    assert_eq!(intent.new, 0..0);
    assert_eq!(intent.kind, UiTextEditKind::Delete);

    let moved = apply_text_edit_action_with_intent(
        deleted.state,
        UiTextEditAction::MoveCaret {
            offset: 1,
            extend_selection: false,
        },
    );
    assert!(moved.committed.is_none());
}

#[test]
fn composition_preedit_is_transient_and_commit_reports_the_original_source_range() {
    let preedit = apply_text_edit_action_with_intent(
        editable_text_state("abcdef", 5, Some(UiTextRange { start: 1, end: 5 })),
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 5 },
            text: "XY".to_string(),
        },
    );
    assert!(preedit.committed.is_none());

    let committed =
        apply_text_edit_action_with_intent(preedit.state, UiTextEditAction::CommitComposition);
    let intent = committed.committed.as_ref().expect("composition commit");
    assert_eq!(committed.state.text, "aXYf");
    assert_eq!(intent.old, 1..5);
    assert_eq!(intent.new, 1..3);
    assert_eq!(intent.kind, UiTextEditKind::CompositionCommit);
    assert_eq!(intent.replacement(&committed.state), Some("XY"));
}

#[test]
fn state_only_selection_followed_by_delete_produces_one_committed_intent() {
    let transition = apply_text_edit_actions_with_intent(
        editable_text_state("alpha beta", 10, None),
        [
            UiTextEditAction::SetSelection {
                anchor: 6,
                focus: 10,
            },
            UiTextEditAction::Delete,
        ],
    )
    .expect("selection plus delete contains one committed edit");

    let intent = transition.committed.as_ref().expect("delete intent");
    assert_eq!(transition.state.text, "alpha ");
    assert_eq!(intent.old, 6..10);
    assert_eq!(intent.new, 6..6);
    assert_eq!(intent.kind, UiTextEditKind::Delete);
}

#[test]
fn multiple_committed_actions_are_rejected_instead_of_losing_an_intent() {
    let error = apply_text_edit_actions_with_intent(
        editable_text_state("ab", 2, None),
        [UiTextEditAction::Backspace, UiTextEditAction::Backspace],
    )
    .expect_err("two committed edits cannot be represented by one intent");

    assert_eq!(error, TextEditActionSequenceError::MultipleCommittedEdits);
}

#[test]
fn unchanged_replacements_do_not_create_document_edit_intents() {
    let empty_insert = apply_text_edit_action_with_intent(
        editable_text_state("alpha", 2, None),
        UiTextEditAction::Insert {
            text: String::new(),
        },
    );
    assert!(empty_insert.committed.is_none());

    let selected = apply_text_edit_action(
        editable_text_state("alpha", 4, None),
        UiTextEditAction::SetSelection {
            anchor: 1,
            focus: 4,
        },
    );
    let identical = apply_text_edit_action_with_intent(
        selected,
        UiTextEditAction::Insert {
            text: "lph".to_string(),
        },
    );
    assert_eq!(identical.state.text, "alpha");
    assert!(identical.committed.is_none());
}

#[test]
fn unchanged_composition_commit_is_state_only() {
    let preedit = apply_text_edit_action_with_intent(
        editable_text_state("alpha", 4, Some(UiTextRange { start: 1, end: 4 })),
        UiTextEditAction::SetComposition {
            range: UiTextRange { start: 1, end: 4 },
            text: "lph".to_string(),
        },
    );
    let committed =
        apply_text_edit_action_with_intent(preedit.state, UiTextEditAction::CommitComposition);

    assert_eq!(committed.state.text, "alpha");
    assert!(committed.state.composition.is_none());
    assert!(committed.committed.is_none());
}

#[test]
fn committed_intent_validation_rejects_malformed_or_state_only_ranges() {
    let state = editable_text_state("a\u{0301}b", 3, None);
    let cases = [
        CommittedTextEditIntent {
            old: 3..2,
            new: 3..3,
            kind: UiTextEditKind::Delete,
        },
        CommittedTextEditIntent {
            old: 0..0,
            new: 3..4,
            kind: UiTextEditKind::Insert,
        },
        CommittedTextEditIntent {
            old: 1..1,
            new: 1..1,
            kind: UiTextEditKind::Replace,
        },
        CommittedTextEditIntent {
            old: 3..3,
            new: 3..3,
            kind: UiTextEditKind::Insert,
        },
    ];

    for intent in cases {
        assert!(!intent.is_valid_for_state(&state));
    }
}

#[test]
fn committed_intent_validation_accepts_exact_grapheme_bounded_replacement() {
    let state = editable_text_state("a\u{0301}XYb", 5, None);
    let intent = CommittedTextEditIntent {
        old: 3..4,
        new: 3..5,
        kind: UiTextEditKind::Replace,
    };

    assert!(intent.is_valid_for_state(&state));
    assert_eq!(intent.replacement(&state), Some("XY"));
}

fn editable_text_state(
    text: &str,
    caret_offset: usize,
    selection_range: Option<UiTextRange>,
) -> UiEditableTextState {
    UiEditableTextState {
        text: text.to_string(),
        caret: UiTextCaret {
            offset: caret_offset,
            affinity: UiTextCaretAffinity::Downstream,
        },
        selection: selection_range.map(|range| UiTextSelection {
            anchor: range.start,
            focus: range.end,
        }),
        composition: None,
        read_only: false,
    }
}
