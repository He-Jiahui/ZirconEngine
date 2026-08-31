use std::{ops::Range, sync::Arc};

use super::{
    PreparedTextDocumentReplace, TextDocument, TextDocumentEditError, TextDocumentEditOutcome,
    TextDocumentEditReceipt, TextDocumentLengthDelta,
};
use crate::text::TextDocumentKey;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    text::{UiTextByteSelection, UiTextDocumentRevision, UiTextEditKind, UiTextEditSource},
};

fn changed_receipt(outcome: TextDocumentEditOutcome) -> TextDocumentEditReceipt {
    match outcome {
        TextDocumentEditOutcome::Changed(receipt) => receipt,
        TextDocumentEditOutcome::Unchanged { key } => {
            panic!("expected a changed document receipt, got unchanged key {key:?}")
        }
    }
}

#[test]
fn repeated_snapshot_leases_share_one_revision_bound_source() {
    let document = TextDocument::new(37, "stable snapshot");

    let first = document.snapshot_lease();
    let second = document.snapshot_lease();

    assert_eq!(first.key(), document.key());
    assert_eq!(first.document_id(), document.document_id());
    assert_eq!(first.revision(), UiTextDocumentRevision::new(0));
    assert_eq!(first.as_str(), "stable snapshot");
    assert!(Arc::ptr_eq(first.shared_source(), second.shared_source()));
}

#[test]
fn document_and_snapshot_debug_output_do_not_expose_source_text() {
    let document = TextDocument::new(38, "private document value");
    let lease = document.snapshot_lease();

    let document_debug = format!("{document:?}");
    let lease_debug = format!("{lease:?}");

    assert!(!document_debug.contains("private document value"));
    assert!(!lease_debug.contains("private document value"));
    assert!(document_debug.contains("byte_len"));
    assert!(lease_debug.contains("byte_len"));
}

#[test]
fn storage_report_accounts_for_revision_owned_source_without_exposing_content() {
    let document = TextDocument::new(381, "private source");

    let report = document.storage_report();

    assert_eq!(report.byte_len, "private source".len());
    assert_eq!(report.original_bytes, "private source".len());
    assert_eq!(report.addition_source_count, 0);
    assert_eq!(report.addition_bytes, 0);
    assert_eq!(report.flattened_snapshot_bytes, 0);
    assert!(report.has_flattened_snapshot);
    assert!(report.estimated_retained_bytes_lower_bound >= report.original_bytes);
    assert!(!format!("{report:?}").contains("private source"));
}

#[test]
fn changed_revision_report_accounts_for_lazy_contiguous_snapshot_once() {
    let mut document = TextDocument::new(382, "alpha beta");
    document
        .replace(document.key(), 6..10, "BETA")
        .expect("replacement succeeds");

    let before_lease = document.storage_report();
    assert!(!before_lease.has_flattened_snapshot);
    assert_eq!(before_lease.flattened_snapshot_bytes, 0);
    assert_eq!(before_lease.addition_source_count, 1);
    assert_eq!(before_lease.addition_bytes, "BETA".len());

    let first = document.snapshot_lease();
    let after_first = document.storage_report();
    let second = document.snapshot_lease();
    let after_second = document.storage_report();

    assert_eq!(after_first.flattened_snapshot_bytes, document.len());
    assert_eq!(after_second, after_first);
    assert!(Arc::ptr_eq(first.shared_source(), second.shared_source()));
}

#[test]
fn sequential_tail_inserts_share_one_addition_source_and_piece() {
    let mut document = TextDocument::new(383, "");

    for _ in 0..8 {
        document
            .replace(document.key(), document.len()..document.len(), "x")
            .expect("tail insertion succeeds");
    }

    let report = document.storage_report();
    assert_eq!(report.byte_len, 8);
    assert_eq!(report.addition_source_count, 1);
    assert_eq!(report.addition_bytes, 8);
    assert_eq!(report.piece_count, 1);
}

#[test]
fn changed_document_publishes_a_new_snapshot_without_invalidating_the_old_lease() {
    let mut document = TextDocument::new(39, "alpha beta");
    let old = document.snapshot_lease();

    document
        .replace(document.key(), 6..10, "BETA")
        .expect("replacement succeeds");
    let current = document.snapshot_lease();

    assert_eq!(old.key(), TextDocumentKey::new(39, 0));
    assert_eq!(old.as_str(), "alpha beta");
    assert_eq!(current.key(), TextDocumentKey::new(39, 1));
    assert_eq!(current.as_str(), "alpha BETA");
    assert!(!Arc::ptr_eq(old.shared_source(), current.shared_source()));
}

#[test]
fn identical_replacement_preserves_the_current_snapshot_lease() {
    let mut document = TextDocument::new(40, "unchanged");
    let before = document.snapshot_lease();

    let outcome = document
        .replace(document.key(), 0.."unchanged".len(), "unchanged")
        .expect("identical replacement is a no-op");
    let after = document.snapshot_lease();

    assert_eq!(
        outcome,
        TextDocumentEditOutcome::Unchanged {
            key: document.key(),
        }
    );
    assert!(Arc::ptr_eq(before.shared_source(), after.shared_source()));
}

#[test]
fn prepared_replacement_does_not_publish_until_commit() {
    let mut document = TextDocument::new(401, "alpha beta");
    let original_key = document.key();
    let prepared = document
        .prepare_replace(original_key, 6..10, "BETA")
        .expect("replacement preparation succeeds");

    assert!(matches!(&prepared, PreparedTextDocumentReplace::Changed(_)));
    assert_eq!(document.key(), original_key);
    assert_eq!(document.snapshot(), "alpha beta");
    assert_eq!(document.storage_report().addition_source_count, 0);

    let receipt = changed_receipt(
        document
            .commit_replace(prepared)
            .expect("prepared replacement commits"),
    );
    assert_eq!(receipt.previous_key, original_key);
    assert_eq!(document.snapshot(), "alpha BETA");
}

#[test]
fn stale_prepared_replacement_fails_without_overwriting_the_new_revision() {
    let mut document = TextDocument::new(402, "first");
    let stale = document
        .prepare_replace(document.key(), 0..5, "stale")
        .expect("first replacement prepares");
    document
        .replace(document.key(), 0..5, "current")
        .expect("newer replacement commits");
    let current_key = document.key();
    let current_report = document.storage_report();

    let error = document
        .commit_replace(stale)
        .expect_err("a prepared edit cannot overwrite a newer revision");

    assert_eq!(
        error,
        TextDocumentEditError::StaleDocument {
            expected: TextDocumentKey::new(402, 0),
            actual: current_key,
        }
    );
    assert_eq!(document.storage_report(), current_report);
    assert_eq!(document.snapshot(), "current");
}

#[test]
fn prepared_replacement_cannot_cross_documents_with_the_same_internal_key() {
    let source = TextDocument::new(403, "source");
    let mut wrong_target = TextDocument::new(403, "target");
    assert_eq!(source.key(), wrong_target.key());
    assert_ne!(source.document_id(), wrong_target.document_id());
    let prepared = source
        .prepare_replace(source.key(), 0..6, "changed")
        .expect("source replacement prepares");

    let error = wrong_target
        .commit_replace(prepared)
        .expect_err("public document identity prevents cross-store key aliasing");

    assert_eq!(
        error,
        TextDocumentEditError::DocumentIdentityMismatch {
            expected: source.document_id(),
            actual: wrong_target.document_id(),
        }
    );
    assert_eq!(wrong_target.snapshot(), "target");
    assert_eq!(wrong_target.key(), TextDocumentKey::new(403, 0));
}

#[test]
fn changed_receipt_projects_to_a_content_free_public_document_receipt() {
    let mut document = TextDocument::new(43, "alpha beta");
    let changed = changed_receipt(
        document
            .replace(document.key(), 6..10, "BETA")
            .expect("replacement succeeds"),
    );
    let public = changed
        .project_public(
            UiNodeId::new(71),
            UiTextEditSource::Keyboard,
            UiTextEditKind::Replace,
            UiTextByteSelection::collapsed(10),
        )
        .expect("internal receipt projects to the public contract");

    assert_eq!(public.document_id, document.document_id());
    assert_eq!(public.previous_revision, UiTextDocumentRevision::new(0));
    assert_eq!(public.revision, UiTextDocumentRevision::new(1));
    assert_eq!(public.changed.old.start_byte, 6);
    assert_eq!(public.changed.old.end_byte, 10);
    assert_eq!(public.changed.new.start_byte, 6);
    assert_eq!(public.changed.new.end_byte, 10);
    assert!(public.validate().is_ok());
    let serialized = serde_json::to_string(&public).expect("receipt serializes");
    assert!(!serialized.contains("alpha"));
    assert!(!serialized.contains("BETA"));
}

#[test]
fn public_receipt_projection_rejects_owner_revision_range_and_selection_aliases() {
    use super::{TextDocumentDirtySpan, TextDocumentReceiptProjectionError};

    let mut document = TextDocument::new(45, "source");
    let changed = changed_receipt(
        document
            .replace(document.key(), 0..6, "changed")
            .expect("replacement succeeds"),
    );
    let arguments = || {
        (
            UiNodeId::new(73),
            UiTextEditSource::Programmatic,
            UiTextEditKind::Replace,
            UiTextByteSelection::collapsed(7),
        )
    };

    let mut wrong_owner = changed.clone();
    wrong_owner.key = TextDocumentKey::new(46, 1);
    let (node, source, kind, selection) = arguments();
    assert_eq!(
        wrong_owner.project_public(node, source, kind, selection),
        Err(TextDocumentReceiptProjectionError::DocumentOwnerChanged)
    );

    let mut wrong_revision = changed.clone();
    wrong_revision.key = TextDocumentKey::new(45, 2);
    let (node, source, kind, selection) = arguments();
    assert_eq!(
        wrong_revision.project_public(node, source, kind, selection),
        Err(TextDocumentReceiptProjectionError::NonConsecutiveRevision)
    );

    let mut overflowing_range = changed.clone();
    overflowing_range.dirty = TextDocumentDirtySpan {
        old: usize::MAX..usize::MAX,
        new: 0..1,
    };
    let (node, source, kind, selection) = arguments();
    assert_eq!(
        overflowing_range.project_public(node, source, kind, selection),
        Err(TextDocumentReceiptProjectionError::ByteRangeOverflow)
    );

    let mut inconsistent_delta = changed.clone();
    inconsistent_delta.length_delta = TextDocumentLengthDelta::Increased(2);
    let (node, source, kind, selection) = arguments();
    assert_eq!(
        inconsistent_delta.project_public(node, source, kind, selection),
        Err(TextDocumentReceiptProjectionError::InconsistentLengthDelta)
    );

    let mut inconsistent_document_len = changed.clone();
    inconsistent_document_len.byte_len = 8;
    let (node, source, kind, selection) = arguments();
    assert_eq!(
        inconsistent_document_len.project_public(node, source, kind, selection),
        Err(TextDocumentReceiptProjectionError::InconsistentDocumentLength)
    );

    let (node, source, kind, _) = arguments();
    assert_eq!(
        changed.project_public(node, source, kind, UiTextByteSelection::collapsed(8)),
        Err(TextDocumentReceiptProjectionError::SelectionOutOfBounds)
    );
}

#[test]
fn replacement_reuses_original_chunks_and_advances_the_document_revision() {
    let mut document = TextDocument::new(41, "alpha beta gamma");
    let original_chunk = document.original_chunk();
    let expected_key = document.key();

    let receipt = changed_receipt(
        document
            .replace(expected_key, 6..10, "BETA")
            .expect("UTF-8 boundary replacement succeeds"),
    );

    assert_eq!(document.snapshot(), "alpha BETA gamma");
    assert!(document.references_original_chunk(&original_chunk));
    assert_eq!(document.key(), TextDocumentKey::new(41, 1));
    assert_eq!(receipt.previous_key, TextDocumentKey::new(41, 0));
    assert_eq!(receipt.key, TextDocumentKey::new(41, 1));
    assert_eq!(receipt.dirty.old, 6..10);
    assert_eq!(receipt.dirty.new, 6..10);
    assert_eq!(receipt.length_delta, TextDocumentLengthDelta::Unchanged);
    assert_eq!(receipt.previous_byte_len, "alpha beta gamma".len());
    assert_eq!(receipt.byte_len, "alpha BETA gamma".len());
}

#[test]
fn replacement_reports_the_local_dirty_span_without_rewriting_the_suffix() {
    let mut document = TextDocument::new(7, "first\nsecond\nthird");
    let original_chunk = document.original_chunk();
    let expected_key = document.key();

    let receipt = changed_receipt(
        document
            .replace(expected_key, 6..12, "second\nfourth")
            .expect("replacement succeeds"),
    );

    assert_eq!(document.snapshot(), "first\nsecond\nfourth\nthird");
    assert!(document.references_original_chunk(&original_chunk));
    assert_eq!(receipt.dirty.old, 6..12);
    assert_eq!(receipt.dirty.new, 6..19);
    assert_eq!(receipt.length_delta, TextDocumentLengthDelta::Increased(7));
    assert_eq!(document.key(), TextDocumentKey::new(7, 1));
}

#[test]
fn replacement_rejects_an_exhausted_revision_without_mutating_the_document() {
    let mut document = TextDocument::new(17, "stable");
    document.revision = u64::MAX;
    let expected_key = document.key();

    let error = document
        .replace(expected_key, 0..6, "changed")
        .expect_err("an exhausted revision cannot publish an aliased key");

    assert_eq!(error, TextDocumentEditError::RevisionExhausted);
    assert_eq!(document.snapshot(), "stable");
    assert_eq!(document.key(), TextDocumentKey::new(17, u64::MAX));
}

#[test]
fn replacement_rejects_a_stale_document_key_without_mutating_the_document() {
    let mut document = TextDocument::new(19, "first");
    let stale_key = document.key();
    document
        .replace(stale_key, 0..5, "second")
        .expect("the current revision succeeds");

    let error = document
        .replace(stale_key, 0..6, "third")
        .expect_err("a stale revision cannot overwrite newer content");

    assert_eq!(
        error,
        TextDocumentEditError::StaleDocument {
            expected: stale_key,
            actual: TextDocumentKey::new(19, 1),
        }
    );
    assert_eq!(document.snapshot(), "second");
    assert_eq!(document.key(), TextDocumentKey::new(19, 1));
}

#[test]
fn identical_replacement_across_pieces_is_a_typed_no_op_without_invalidating_indexes() {
    let mut document = TextDocument::new(59, "alpha beta gamma");
    let first = changed_receipt(
        document
            .replace(document.key(), 6..10, "BETA")
            .expect("first replacement creates an addition piece"),
    );
    assert_eq!(first.key, TextDocumentKey::new(59, 1));
    let expected_key = document.key();
    let pieces_before = document.pieces.clone();
    let addition_count_before = document.additions.len();
    let hard_line_ids_before = document
        .hard_line_models()
        .iter()
        .map(|line| line.id())
        .collect::<Vec<_>>();
    let index_before = {
        let index = document.source_index();
        (index.revision(), index.grapheme_boundaries().to_vec())
    };

    let outcome = document
        .replace(
            expected_key,
            0.."alpha BETA gamma".len(),
            "alpha BETA gamma",
        )
        .expect("an identical replacement is a valid no-op");

    assert_eq!(
        outcome,
        TextDocumentEditOutcome::Unchanged { key: expected_key }
    );
    assert_eq!(document.key(), expected_key);
    assert_eq!(document.snapshot(), "alpha BETA gamma");
    assert_eq!(document.pieces, pieces_before);
    assert_eq!(document.additions.len(), addition_count_before);
    assert_eq!(
        document
            .hard_line_models()
            .iter()
            .map(|line| line.id())
            .collect::<Vec<_>>(),
        hard_line_ids_before
    );
    let index_after = document.source_index();
    assert_eq!(
        (index_after.revision(), index_after.grapheme_boundaries()),
        (index_before.0, index_before.1.as_slice())
    );
}

#[test]
fn identical_replacement_does_not_require_another_revision_after_max() {
    let mut document = TextDocument::new(61, "stable");
    document.revision = u64::MAX;
    let expected_key = document.key();

    let outcome = document
        .replace(expected_key, 0.."stable".len(), "stable")
        .expect("unchanged source does not need a publishable next revision");

    assert_eq!(
        outcome,
        TextDocumentEditOutcome::Unchanged { key: expected_key }
    );
    assert_eq!(document.key(), expected_key);
    assert_eq!(document.snapshot(), "stable");
}

#[test]
fn replacement_rejects_ranges_that_split_utf8_code_points() {
    let mut document = TextDocument::new(9, "a\u{4e16}b");
    let expected_key = document.key();

    let error = document
        .replace(expected_key, 2..2, "x")
        .expect_err("a byte inside a code point is not an edit boundary");

    assert_eq!(error, TextDocumentEditError::InvalidUtf8Boundary);
    assert_eq!(document.snapshot(), "a\u{4e16}b");
    assert_eq!(document.key(), TextDocumentKey::new(9, 0));
}

#[test]
fn source_index_is_revision_bound_and_distinguishes_hard_lines_from_graphemes() {
    let mut document = TextDocument::new(13, "a\u{0301}\n\u{4e16}\u{754c}");

    let first = document.source_index();
    assert_eq!(first.revision(), 0);
    assert_eq!(first.grapheme_boundaries(), &[0, 3, 4, 7, 10]);
    assert_eq!(document.hard_line_models().len(), 2);

    document
        .replace(document.key(), 4..10, "\u{4e16}\u{754c}\nthird")
        .expect("replacement at indexed boundaries succeeds");
    let second = document.source_index();

    assert_eq!(second.revision(), 1);
    assert_eq!(second.grapheme_boundaries().last(), Some(&16));
    assert_eq!(document.hard_line_models().len(), 3);
}

#[test]
fn source_index_counts_graphemes_outside_an_exact_replacement_range() {
    let mut document = TextDocument::new(14, "a\u{0301}b\u{1f469}\u{200d}\u{1f4bb}");
    let combining_end = "a\u{0301}".len();
    let emoji_start = combining_end + "b".len();

    assert_eq!(
        document.retained_grapheme_count(combining_end..emoji_start),
        Ok(2)
    );
    assert_eq!(document.retained_grapheme_count(0..document.len()), Ok(0));
}

#[test]
fn source_index_rejects_ranges_that_split_a_grapheme_cluster() {
    let mut document = TextDocument::new(15, "a\u{0301}b");

    assert_eq!(
        document.retained_grapheme_count(1..3),
        Err(TextDocumentEditError::InvalidGraphemeBoundary)
    );
}

#[test]
fn source_index_incrementally_updates_an_ascii_edit_and_shifts_suffix_boundaries() {
    let mut document = TextDocument::new(16, "abc def ghi");
    let initial = document.source_index().grapheme_boundaries().to_vec();
    assert_eq!(initial, (0..=document.len()).collect::<Vec<_>>());

    document
        .replace(document.key(), 4..7, "WXYZ")
        .expect("an ASCII edit at grapheme boundaries succeeds");

    assert_eq!(document.snapshot(), "abc WXYZ ghi");
    assert!(document.source_index.matches_revision(document.revision));
    let expected = (0..=document.len()).collect::<Vec<_>>();
    assert_eq!(
        document.source_index().grapheme_boundaries(),
        expected.as_slice()
    );
}

#[test]
fn source_index_incrementally_updates_empty_insert_and_delete_edits() {
    let mut document = TextDocument::new(19, "abc");
    assert_eq!(document.source_index().grapheme_boundaries(), &[0, 1, 2, 3]);

    document
        .replace(document.key(), 1..1, "x")
        .expect("ASCII insertion into an indexed document succeeds");
    assert!(document.source_index.matches_revision(document.revision));
    assert_eq!(document.snapshot(), "axbc");
    assert_eq!(
        document.source_index().grapheme_boundaries(),
        &[0, 1, 2, 3, 4]
    );

    let mut empty_document = TextDocument::new(22, "");
    assert_eq!(empty_document.source_index().grapheme_boundaries(), &[0]);
    empty_document
        .replace(empty_document.key(), 0..0, "abc")
        .expect("ASCII insertion into an empty indexed document succeeds");
    assert!(
        empty_document
            .source_index
            .matches_revision(empty_document.revision)
    );
    assert_eq!(
        empty_document.source_index().grapheme_boundaries(),
        &[0, 1, 2, 3]
    );

    document
        .replace(document.key(), 1..2, "")
        .expect("ASCII deletion from an indexed document succeeds");
    assert_eq!(document.snapshot(), "abc");
    assert!(document.source_index.matches_revision(document.revision));
    assert_eq!(document.source_index().grapheme_boundaries(), &[0, 1, 2, 3]);
}

#[test]
fn ascii_incremental_preflight_checks_piece_bytes_without_crossing_line_breaks() {
    let mut document = TextDocument::new(20, "prefix suffix");
    document
        .replace(document.key(), 7..13, "middle")
        .expect("replacement creates an addition piece");

    assert!(document.range_is_ascii_grapheme_edit(6..8));
    let unicode_document = TextDocument::new(21, "é");
    assert!(!unicode_document.range_is_ascii_grapheme_edit(0.."é".len()));

    document
        .replace(document.key(), 6..6, "\n")
        .expect("separator replacement succeeds");
    assert!(!document.range_is_ascii_grapheme_edit(5..8));
}

#[test]
fn source_index_rejects_incremental_edits_next_to_unicode_context() {
    let mut document = TextDocument::new(17, "é abc");
    let expected = document.source_index().grapheme_boundaries().to_vec();
    let insertion = "x";
    let unicode_end = "é".len();

    document
        .replace(document.key(), unicode_end..unicode_end, insertion)
        .expect("insertion at the Unicode boundary succeeds");

    assert_eq!(document.snapshot(), "éx abc");
    assert!(!document.source_index.matches_revision(document.revision));
    let rebuilt = document.source_index().grapheme_boundaries().to_vec();
    assert_ne!(rebuilt, expected);
    assert_eq!(
        rebuilt,
        vec![
            0,
            "é".len(),
            "éx".len(),
            "éx ".len(),
            "éx a".len(),
            "éx ab".len(),
            "éx abc".len()
        ]
    );
}

#[test]
fn source_index_rejects_incremental_edits_next_to_crlf_context() {
    let mut document = TextDocument::new(18, "ab\r\ncd");
    let _ = document.source_index().grapheme_boundaries().to_vec();

    document
        .replace(document.key(), 2..2, "x")
        .expect("insertion before CRLF succeeds");

    assert_eq!(document.snapshot(), "abx\r\ncd");
    assert!(!document.source_index.matches_revision(document.revision));
    assert_eq!(
        document.source_index().grapheme_boundaries(),
        &[0, 1, 2, 3, 5, 6, 7]
    );
}

#[test]
fn hard_line_models_preserve_canonical_separator_lengths() {
    let document = TextDocument::new(23, "a\r\nb\u{2028}c");

    let lengths = document
        .hard_line_models()
        .iter()
        .map(|line| (line.content_len(), line.separator_len()))
        .collect::<Vec<_>>();
    let ids = document
        .hard_line_models()
        .iter()
        .map(|line| line.id())
        .collect::<Vec<_>>();

    assert_eq!(lengths, vec![(1, 2), (1, 3), (1, 0)]);
    assert!(ids.windows(2).all(|ids| ids[0] != ids[1]));
}

#[test]
fn line_local_edit_preserves_stable_hard_line_ids() {
    let mut document = TextDocument::new(29, "zero\nfirst\nsecond\nthird\nfourth");
    let ids_before = document
        .hard_line_models()
        .iter()
        .map(|line| line.id())
        .collect::<Vec<_>>();
    let second_start = "zero\nfirst\n".len();

    let receipt = changed_receipt(
        document
            .replace(
                document.key(),
                second_start..second_start + "second".len(),
                "changed",
            )
            .expect("line-local replacement succeeds"),
    );
    let ids_after = document
        .hard_line_models()
        .iter()
        .map(|line| line.id())
        .collect::<Vec<_>>();

    assert_eq!(ids_after, ids_before);
    assert_eq!(receipt.reanalyzed_hard_lines.old, 2..3);
    assert_eq!(receipt.reanalyzed_hard_lines.new, 2..3);
}

#[test]
fn insertion_inside_crlf_uses_structural_separator_reanalysis() {
    let mut document = TextDocument::new(30, "a\r\nb");

    document
        .replace(document.key(), 2..2, "x")
        .expect("insertion between CR and LF succeeds");

    assert_eq!(document.snapshot(), "a\rx\nb");
    assert_eq!(
        document
            .hard_line_models()
            .iter()
            .map(|line| (line.content_len(), line.separator_len()))
            .collect::<Vec<_>>(),
        vec![(1, 1), (1, 1), (1, 0)]
    );
}

#[test]
fn inserting_a_separator_preserves_neighbors_and_allocates_only_the_split_line_id() {
    let mut document = TextDocument::new(31, "before\nmiddle\nafter");
    let ids_before = document
        .hard_line_models()
        .iter()
        .map(|line| line.id())
        .collect::<Vec<_>>();
    let insertion = "before\nmid".len();

    let receipt = changed_receipt(
        document
            .replace(document.key(), insertion..insertion, "\n")
            .expect("separator insertion succeeds"),
    );
    let ids_after = document
        .hard_line_models()
        .iter()
        .map(|line| line.id())
        .collect::<Vec<_>>();

    assert_eq!(document.snapshot(), "before\nmid\ndle\nafter");
    assert_eq!(ids_after[0], ids_before[0]);
    assert_eq!(ids_after[1], ids_before[1]);
    assert_eq!(ids_after[3], ids_before[2]);
    assert!(!ids_before.contains(&ids_after[2]));
    assert_eq!(receipt.reanalyzed_hard_lines.old, 0..3);
    assert_eq!(receipt.reanalyzed_hard_lines.new, 0..4);
}

#[test]
fn deleting_a_separator_keeps_the_left_line_id_and_removes_the_merged_right_id() {
    let mut document = TextDocument::new(37, "before\nleft\nright\nafter");
    let ids_before = document
        .hard_line_models()
        .iter()
        .map(|line| line.id())
        .collect::<Vec<_>>();
    let separator = "before\nleft".len();

    let receipt = changed_receipt(
        document
            .replace(document.key(), separator..separator + 1, "")
            .expect("separator deletion succeeds"),
    );
    let ids_after = document
        .hard_line_models()
        .iter()
        .map(|line| line.id())
        .collect::<Vec<_>>();

    assert_eq!(document.snapshot(), "before\nleftright\nafter");
    assert_eq!(ids_after, vec![ids_before[0], ids_before[1], ids_before[3]]);
    assert!(!ids_after.contains(&ids_before[2]));
    assert_eq!(receipt.reanalyzed_hard_lines.old, 0..3);
    assert_eq!(receipt.reanalyzed_hard_lines.new, 0..2);
}

#[test]
fn crlf_completion_reuses_both_existing_hard_line_ids() {
    let mut document = TextDocument::new(43, "a\rb");
    let ids_before = document
        .hard_line_models()
        .iter()
        .map(|line| line.id())
        .collect::<Vec<_>>();

    document
        .replace(document.key(), 2..2, "\n")
        .expect("LF completes the CRLF separator");

    let lines = document.hard_line_models();
    assert_eq!(
        lines.iter().map(|line| line.id()).collect::<Vec<_>>(),
        ids_before
    );
    assert_eq!(
        lines
            .iter()
            .map(|line| (line.content_len(), line.separator_len()))
            .collect::<Vec<_>>(),
        vec![(1, 2), (1, 0)]
    );
}

#[test]
fn empty_document_retains_one_stable_terminal_line_through_insertion() {
    let mut document = TextDocument::new(47, "");
    let terminal_id = document.hard_line_models()[0].id();

    let receipt = changed_receipt(
        document
            .replace(document.key(), 0..0, "text")
            .expect("empty-document insertion succeeds"),
    );

    assert_eq!(document.hard_line_models().len(), 1);
    assert_eq!(document.hard_line_models()[0].id(), terminal_id);
    assert_eq!(document.hard_line_models()[0].content_len(), 4);
    assert_eq!(receipt.reanalyzed_hard_lines.old, 0..1);
    assert_eq!(receipt.reanalyzed_hard_lines.new, 0..1);
}

#[test]
fn trailing_separator_insertion_retains_the_existing_line_and_adds_a_terminal_line() {
    let mut document = TextDocument::new(53, "tail");
    let line_id = document.hard_line_models()[0].id();

    document
        .replace(document.key(), 4..4, "\n")
        .expect("trailing separator insertion succeeds");

    let lines = document.hard_line_models();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0].id(), line_id);
    assert_eq!((lines[0].content_len(), lines[0].separator_len()), (4, 1));
    assert_eq!((lines[1].content_len(), lines[1].separator_len()), (0, 0));
    assert_ne!(lines[1].id(), line_id);
}

#[test]
fn source_range_snapshot_only_materializes_the_requested_piece_window() {
    let mut document = TextDocument::new(15, "prefix middle suffix");
    document
        .replace(document.key(), 7..13, "replacement")
        .expect("replacement succeeds");

    assert_eq!(
        document
            .snapshot_range(Range { start: 7, end: 18 })
            .expect("range crosses the appended chunk"),
        "replacement"
    );
    assert_eq!(document.snapshot_range(2..4), Ok("ef".to_string()));
}
