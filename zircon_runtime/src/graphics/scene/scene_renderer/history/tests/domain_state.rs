use super::*;

#[test]
fn history_write_intent_merge_preserves_successful_pass_receipts() {
    let mut frame = SceneHistoryWriteIntent::default();
    let mut taa = SceneHistoryWriteIntent::default();
    taa.record(SceneHistoryDomain::TaaSceneColor, true);
    let mut exposure = SceneHistoryWriteIntent::default();
    exposure.record(SceneHistoryDomain::Exposure, true);

    frame.merge(taa);
    frame.merge(exposure);

    assert!(frame.was_written(SceneHistoryDomain::TaaSceneColor));
    assert!(frame.was_written(SceneHistoryDomain::Exposure));
}

#[test]
fn domain_invalidation_does_not_collapse_unrelated_history() {
    let mut states = SceneHistoryDomainStates::default();
    let mut seed = SceneHistoryFrameTransaction::begin(&states);
    let mut writes = SceneHistoryWriteIntent::default();
    writes.record(SceneHistoryDomain::TaaSceneColor, true);
    writes.record(SceneHistoryDomain::Exposure, true);
    seed.absorb_writes(writes);
    seed.commit(&mut states, 7);

    let mut next = SceneHistoryFrameTransaction::begin(&states);
    next.invalidate(
        SceneHistoryDomain::TaaSceneColor,
        SceneHistoryResetReason::CameraCut,
    );

    assert!(!next
        .availability()
        .is_available(SceneHistoryDomain::TaaSceneColor));
    assert!(next
        .availability()
        .is_available(SceneHistoryDomain::Exposure));
    assert!(states.state(SceneHistoryDomain::TaaSceneColor).is_valid());
}

#[test]
fn writes_become_valid_only_when_the_frame_transaction_commits() {
    let mut states = SceneHistoryDomainStates::default();
    let mut frame = SceneHistoryFrameTransaction::begin(&states);
    let mut writes = SceneHistoryWriteIntent::default();
    writes.record(SceneHistoryDomain::AmbientOcclusion, true);
    frame.absorb_writes(writes);

    assert!(!states
        .state(SceneHistoryDomain::AmbientOcclusion)
        .is_valid());
    frame.commit(&mut states, 41);

    let state = states.state(SceneHistoryDomain::AmbientOcclusion);
    assert!(state.is_valid());
    assert_eq!(state.generation(), 1);
    assert_eq!(state.last_successful_frame(), Some(41));
    assert_eq!(state.reset_reason(), None);
}

#[test]
fn requested_copy_without_a_source_invalidates_only_that_domain() {
    let mut states = SceneHistoryDomainStates::default();
    let mut seed = SceneHistoryFrameTransaction::begin(&states);
    let mut seed_writes = SceneHistoryWriteIntent::default();
    seed_writes.record(SceneHistoryDomain::ScreenSpaceReflection, true);
    seed_writes.record(SceneHistoryDomain::HzbFurthest, true);
    seed.absorb_writes(seed_writes);
    seed.commit(&mut states, 3);

    let mut frame = SceneHistoryFrameTransaction::begin(&states);
    let mut writes = SceneHistoryWriteIntent::default();
    writes.record(SceneHistoryDomain::ScreenSpaceReflection, false);
    frame.absorb_writes(writes);
    frame.commit(&mut states, 4);

    assert_eq!(
        states
            .state(SceneHistoryDomain::ScreenSpaceReflection)
            .reset_reason(),
        Some(SceneHistoryResetReason::SourceUnavailable)
    );
    assert!(states.state(SceneHistoryDomain::HzbFurthest).is_valid());
}

#[test]
fn frame_transaction_merges_write_intents_without_losing_success() {
    let mut states = SceneHistoryDomainStates::default();
    let mut frame = SceneHistoryFrameTransaction::begin(&states);
    let mut first = SceneHistoryWriteIntent::default();
    first.record(SceneHistoryDomain::TaaSceneColor, true);
    frame.absorb_writes(first);
    let mut second = SceneHistoryWriteIntent::default();
    second.record(SceneHistoryDomain::TaaSceneColor, false);
    second.record(SceneHistoryDomain::Exposure, true);
    frame.absorb_writes(second);

    let report = frame.commit(&mut states, 8);

    assert!(report.state(SceneHistoryDomain::TaaSceneColor).valid);
    assert!(report.state(SceneHistoryDomain::Exposure).valid);
    assert_eq!(
        report
            .state(SceneHistoryDomain::TaaSceneColor)
            .last_successful_frame,
        Some(8)
    );
}

#[test]
fn successful_reseed_wins_over_the_current_frame_reset() {
    let mut states = SceneHistoryDomainStates::default();
    let mut frame = SceneHistoryFrameTransaction::begin(&states);
    frame.invalidate_spatial(SceneHistoryResetReason::PreviousFrameUnavailable);
    let mut writes = SceneHistoryWriteIntent::default();
    writes.record(SceneHistoryDomain::HybridGlobalIllumination, true);
    frame.absorb_writes(writes);
    let report = frame.commit(&mut states, 9);

    assert!(states
        .state(SceneHistoryDomain::HybridGlobalIllumination)
        .is_valid());
    assert_eq!(
        states
            .state(SceneHistoryDomain::HybridGlobalIllumination)
            .last_successful_frame(),
        Some(9)
    );
    let report_state = report.state(SceneHistoryDomain::HybridGlobalIllumination);
    assert!(report_state.valid);
    assert_eq!(report_state.active_reset_reason, None);
    assert_eq!(
        report_state.frame_reset_reason,
        Some(SceneHistoryResetReason::PreviousFrameUnavailable)
    );
}
