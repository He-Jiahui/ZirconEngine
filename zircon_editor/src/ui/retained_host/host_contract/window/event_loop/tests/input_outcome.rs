use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::dispatch::UiInputSequence;

use super::*;

#[test]
fn damaged_inputs_coalesce_into_one_bounded_present_batch() {
    let mut tracker = UiInputOutcomeTracker::default();
    let started_at = Instant::now();

    tracker.begin(UiInputSequence::new(1), started_at);
    let first = tracker
        .finish_damaged(started_at + Duration::from_micros(10))
        .expect("first input outcome");
    tracker.begin(UiInputSequence::new(2), started_at);
    let quiet = tracker
        .finish_intentionally_no_damage()
        .expect("quiet input outcome");
    tracker.begin(UiInputSequence::new(3), started_at);
    let third = tracker
        .finish_damaged(started_at + Duration::from_micros(30))
        .expect("third input outcome");

    assert_eq!(first.kind(), UiInputOutcomeKind::Damaged);
    assert_eq!(first.input_to_damage(), Some(Duration::from_micros(10)));
    assert_eq!(quiet.kind(), UiInputOutcomeKind::IntentionallyNoDamage);
    assert_eq!(third.kind(), UiInputOutcomeKind::Damaged);
    let batch = tracker.take_presented_batch().expect("present batch");
    assert_eq!(batch.first_sequence(), UiInputSequence::new(1));
    assert_eq!(batch.last_sequence(), UiInputSequence::new(3));
    assert_eq!(batch.damaged_count(), 2);
    assert_eq!(
        batch.first_damage_started_at(),
        started_at + Duration::from_micros(10)
    );
    assert!(tracker.take_presented_batch().is_none());
}

#[test]
fn retry_does_not_consume_the_pending_present_batch() {
    let mut tracker = UiInputOutcomeTracker::default();
    let started_at = Instant::now();
    tracker.begin(UiInputSequence::new(8), started_at);
    tracker.finish_damaged(started_at).expect("input outcome");

    let before_retry = tracker.pending_present_batch().copied();
    let after_retry = tracker.pending_present_batch().copied();

    assert_eq!(before_retry, after_retry);
    assert_eq!(
        tracker
            .take_presented_batch()
            .expect("retry retained batch")
            .damaged_count(),
        1
    );
}

#[test]
fn rejected_and_quiet_inputs_never_enter_the_present_batch() {
    let mut tracker = UiInputOutcomeTracker::default();
    let started_at = Instant::now();
    tracker.begin(UiInputSequence::new(11), started_at);
    let quiet = tracker
        .finish_intentionally_no_damage()
        .expect("quiet outcome");
    tracker.begin(UiInputSequence::new(12), started_at);
    let rejected = tracker.reject().expect("rejected outcome");

    assert_eq!(quiet.sequence(), UiInputSequence::new(11));
    assert_eq!(rejected.sequence(), UiInputSequence::new(12));
    assert_eq!(rejected.kind(), UiInputOutcomeKind::Rejected);
    assert!(tracker.pending_present_batch().is_none());
}

#[test]
fn beginning_an_input_rejects_an_unfinished_prior_input() {
    let mut tracker = UiInputOutcomeTracker::default();
    let started_at = Instant::now();
    assert!(tracker
        .begin(UiInputSequence::new(21), started_at)
        .is_none());

    let interrupted = tracker
        .begin(UiInputSequence::new(22), started_at)
        .expect("unfinished input must fail closed");

    assert_eq!(interrupted.sequence(), UiInputSequence::new(21));
    assert_eq!(interrupted.kind(), UiInputOutcomeKind::Rejected);
    assert_eq!(
        tracker.reject().expect("current input").sequence(),
        UiInputSequence::new(22)
    );
}

#[test]
fn input_outcome_authority_has_no_unbounded_sequence_collection() {
    let production = include_str!("../input_outcome.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production input outcome source");

    assert!(!production.contains("Vec<"));
    assert!(!production.contains("VecDeque<"));
}

#[test]
fn frame_update_wake_is_not_misclassified_as_present_damage() {
    let frame_update = HostRedrawRequest::FrameUpdate {
        scenario: UiPerfScenario::Click,
        interactive_frame_update: false,
    };
    let present = HostRedrawRequest::Full {
        frame_update: true,
        interactive_frame_update: false,
        scenario: UiPerfScenario::Click,
    };

    assert_eq!(
        redraw_outcome_kind(&frame_update),
        UiInputOutcomeKind::IntentionallyNoDamage
    );
    assert_eq!(redraw_outcome_kind(&present), UiInputOutcomeKind::Damaged);
}
