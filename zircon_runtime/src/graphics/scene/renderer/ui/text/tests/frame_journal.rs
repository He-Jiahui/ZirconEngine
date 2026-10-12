use super::*;

fn generation(value: u64) -> ScreenSpaceUiTextFrameProductGeneration {
    let mut counter = value.saturating_sub(1);
    ScreenSpaceUiTextFrameProductGeneration::next(&mut counter)
}

fn font_revision(generation: u64) -> FontCollectionRevision {
    FontCollectionRevision::new(
        crate::text::font::shared_font_collection_handle(),
        generation,
    )
}

fn local_source_journal(
    base_generation: u64,
    changed_segment_indices: &[usize],
    appended_segment_count: usize,
    truncated_segment_count: usize,
) -> ScreenSpaceUiFrameChangeJournal {
    ScreenSpaceUiFrameChangeJournal::for_test_local_delta(
        base_generation,
        changed_segment_indices,
        appended_segment_count,
        truncated_segment_count,
    )
}

fn local_inputs<'a>(
    source: &'a ScreenSpaceUiFrameChangeJournal,
    changed_text_segment_indices: &'a [usize],
) -> ScreenSpaceUiTextFrameJournalInputs<'a> {
    ScreenSpaceUiTextFrameJournalInputs {
        source,
        previous_product_generation: Some(generation(7)),
        previous_source_generation: Some(41),
        previous_segment_count: 3,
        previous_viewport_size: UVec2::new(800, 600),
        previous_font_revision: Some(font_revision(5)),
        retained_state_consistent: true,
        current_generation: generation(8),
        current_segment_count: 3,
        current_viewport_size: UVec2::new(800, 600),
        current_font_revision: font_revision(5),
        changed_text_segment_indices,
        pending_full_rebuild_reason: None,
    }
}

#[test]
fn local_text_journal_publishes_exact_product_generations_and_changes() {
    let source = local_source_journal(41, &[0, 2], 0, 0);
    let journal = ScreenSpaceUiTextFrameChangeJournal::publish(local_inputs(&source, &[2]));

    assert_eq!(journal.base_generation(), Some(generation(7)));
    assert_eq!(journal.current_generation(), generation(8));
    assert_eq!(journal.changed_segment_indices(), [2]);
    assert_eq!(journal.appended_segment_count(), 0);
    assert_eq!(journal.truncated_segment_count(), 0);
    assert!(!journal.is_full_rebuild());
}

#[test]
fn local_text_journal_preserves_append_and_truncate_topology() {
    let append_source = local_source_journal(41, &[], 2, 0);
    let mut append_inputs = local_inputs(&append_source, &[]);
    append_inputs.current_segment_count = 5;
    let append = ScreenSpaceUiTextFrameChangeJournal::publish(append_inputs);
    assert_eq!(append.appended_segment_count(), 2);
    assert_eq!(append.truncated_segment_count(), 0);

    let truncate_source = local_source_journal(41, &[], 0, 2);
    let mut truncate_inputs = local_inputs(&truncate_source, &[]);
    truncate_inputs.previous_segment_count = 5;
    let truncate = ScreenSpaceUiTextFrameChangeJournal::publish(truncate_inputs);
    assert_eq!(truncate.appended_segment_count(), 0);
    assert_eq!(truncate.truncated_segment_count(), 2);
    assert!(!truncate.is_full_rebuild());
}

#[test]
fn malformed_or_recovery_journals_publish_typed_full_rebuilds() {
    let malformed_source = local_source_journal(41, &[3], 0, 0);
    let malformed =
        ScreenSpaceUiTextFrameChangeJournal::publish(local_inputs(&malformed_source, &[3]));
    assert_eq!(
        malformed.full_rebuild_reason(),
        Some(ScreenSpaceUiTextFrameFullRebuildReason::SourceTopologyMismatch)
    );

    let mut recovery_inputs = local_inputs(&malformed_source, &[]);
    recovery_inputs.previous_product_generation = None;
    recovery_inputs.pending_full_rebuild_reason =
        Some(ScreenSpaceUiTextFrameFullRebuildReason::ExplicitRecovery);
    let recovery = ScreenSpaceUiTextFrameChangeJournal::publish(recovery_inputs);
    assert_eq!(
        recovery.full_rebuild_reason(),
        Some(ScreenSpaceUiTextFrameFullRebuildReason::ExplicitRecovery)
    );

    let incomplete_source = local_source_journal(41, &[0], 0, 0);
    let incomplete =
        ScreenSpaceUiTextFrameChangeJournal::publish(local_inputs(&incomplete_source, &[1]));
    assert_eq!(
        incomplete.full_rebuild_reason(),
        Some(ScreenSpaceUiTextFrameFullRebuildReason::SourceChangeSetMismatch)
    );
}
