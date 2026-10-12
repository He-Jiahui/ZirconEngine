use super::*;
use crate::rhi::RenderQueueClass;

fn ticket(sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(7),
        DeviceGeneration::initial(),
        RenderQueueClass::Graphics,
        sequence,
    )
}

fn poll(sequence: u64) -> SubmissionPollReceipt {
    SubmissionPollReceipt::new(DeviceId::new(7), DeviceGeneration::initial(), sequence)
}

fn new_journal(capacity: usize) -> SceneSubmissionCompletionJournal {
    SceneSubmissionCompletionJournal::new(DeviceId::new(7), DeviceGeneration::initial(), capacity)
}

#[test]
fn completion_observation_batches_pending_tickets_and_keeps_non_terminal_work() {
    let mut journal = new_journal(4);
    journal.track(10, ticket(1));
    journal.track(11, ticket(2));

    journal
        .observe(poll(1), |tickets, statuses| {
            assert_eq!(tickets, &[ticket(1), ticket(2)]);
            statuses.extend([
                Ok(SubmissionStatus::Completed),
                Ok(SubmissionStatus::Submitted),
            ]);
        })
        .unwrap();
    assert_eq!(journal.pending.len(), 1);
    assert_eq!(journal.last_report().frame_generation, 10);
    assert_eq!(journal.last_report().pending_submission_count, 1);
    assert_eq!(journal.last_report().tracking_capacity, 4);
    assert_eq!(journal.last_report().last_poll_observed_submission_count, 2);
    assert_eq!(journal.last_report().last_poll_terminal_submission_count, 1);
    assert_eq!(
        journal.last_report().status,
        RenderSceneSubmissionCompletionStatus::Completed
    );

    journal
        .observe(poll(2), |tickets, statuses| {
            assert_eq!(tickets, &[ticket(2)]);
            statuses.push(Ok(SubmissionStatus::DeviceLost));
        })
        .unwrap();
    assert!(journal.pending.is_empty());
    assert_eq!(journal.last_report().frame_generation, 11);
    assert_eq!(journal.last_report().pending_submission_count, 0);
    assert_eq!(journal.last_report().last_poll_observed_submission_count, 1);
    assert_eq!(journal.last_report().last_poll_terminal_submission_count, 1);
    assert_eq!(
        journal.last_report().status,
        RenderSceneSubmissionCompletionStatus::DeviceLost
    );
}

#[test]
fn replayed_or_foreign_poll_is_rejected_before_status_observation() {
    let mut journal = new_journal(1);
    journal.observe(poll(2), |_, _| {}).unwrap();

    let replayed = journal.observe(poll(2), |_, _| panic!("must not query statuses"));
    assert!(matches!(
        replayed,
        Err(RenderSceneSubmissionCompletionError::PollSequenceDidNotAdvance { .. })
    ));
    let foreign = SubmissionPollReceipt::new(DeviceId::new(8), DeviceGeneration::initial(), 3);
    assert!(matches!(
        journal.observe(foreign, |_, _| panic!("must not query statuses")),
        Err(RenderSceneSubmissionCompletionError::PollOwnerMismatch { .. })
    ));
}

#[test]
fn empty_journal_advances_receipt_without_taking_the_status_lock() {
    let mut journal = new_journal(1);

    journal
        .observe(poll(1), |_, _| {
            panic!("empty journal must not query statuses")
        })
        .unwrap();
    assert_eq!(journal.last_report().pending_submission_count, 0);
    assert_eq!(journal.last_report().last_poll_observed_submission_count, 0);
    assert_eq!(journal.last_report().last_poll_terminal_submission_count, 0);
    assert!(matches!(
        journal.observe(poll(1), |_, _| {}),
        Err(RenderSceneSubmissionCompletionError::PollSequenceDidNotAdvance { .. })
    ));
}

#[test]
fn malformed_status_batch_does_not_consume_pending_work_or_receipt() {
    let mut journal = new_journal(1);
    journal.track(42, ticket(1));

    assert!(matches!(
        journal.observe(poll(1), |_, _| {}),
        Err(
            RenderSceneSubmissionCompletionError::StatusResultCountMismatch {
                expected: 1,
                actual: 0,
            }
        )
    ));
    assert_eq!(journal.pending.len(), 1);
    journal
        .observe(poll(1), |_, statuses| {
            statuses.push(Ok(SubmissionStatus::Completed));
        })
        .unwrap();
    assert!(journal.pending.is_empty());
    assert_eq!(journal.last_report().last_poll_observed_submission_count, 1);
    assert_eq!(journal.last_report().last_poll_terminal_submission_count, 1);
}

#[test]
fn status_history_miss_fails_closed_and_does_not_leak_pending_work() {
    let mut journal = new_journal(1);
    journal.track(42, ticket(1));
    journal
        .observe(poll(1), |_, statuses| {
            statuses.push(Err(RhiError::UnknownSubmissionTicket(ticket(1))));
        })
        .unwrap();

    assert!(journal.pending.is_empty());
    assert_eq!(journal.last_report().last_poll_observed_submission_count, 1);
    assert_eq!(journal.last_report().last_poll_terminal_submission_count, 0);
    assert_eq!(
        journal.last_report().status,
        RenderSceneSubmissionCompletionStatus::ObservationFailed
    );
    assert_eq!(
        journal.last_report().failure,
        RenderSceneSubmissionCompletionFailure::StatusUnavailable
    );
}

#[test]
fn tracking_capacity_is_bounded_without_evicting_live_work() {
    let mut journal = new_journal(1);
    journal.track(1, ticket(1));
    journal.track(2, ticket(2));

    assert_eq!(journal.pending.len(), 1);
    assert_eq!(journal.pending.front().unwrap().ticket, ticket(1));
    assert_eq!(
        journal.last_report().failure,
        RenderSceneSubmissionCompletionFailure::CapacityExceeded
    );
    assert_eq!(journal.last_report().pending_submission_count, 1);
    assert_eq!(journal.last_report().tracking_capacity, 1);
}

#[test]
fn tracking_rejects_a_non_advancing_submission_sequence_in_constant_time() {
    let mut journal = new_journal(2);
    journal.track(1, ticket(2));
    journal.track(2, ticket(2));

    assert_eq!(journal.pending.len(), 1);
    assert_eq!(
        journal.last_report().failure,
        RenderSceneSubmissionCompletionFailure::SubmissionSequenceDidNotAdvance
    );

    journal
        .observe(poll(1), |_, statuses| {
            statuses.push(Ok(SubmissionStatus::Completed));
        })
        .unwrap();
    assert!(journal.pending.is_empty());
    journal.track(3, ticket(2));
    assert!(journal.pending.is_empty());
    assert_eq!(
        journal.last_report().failure,
        RenderSceneSubmissionCompletionFailure::SubmissionSequenceDidNotAdvance
    );
}
