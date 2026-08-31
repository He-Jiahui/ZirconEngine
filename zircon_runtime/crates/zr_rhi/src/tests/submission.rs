use crate::{
    DeviceGeneration, DeviceId, RenderQueueClass, SubmissionHistory, SubmissionLimits,
    SubmissionStatus, SubmissionTicket,
};

fn ticket(sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(41),
        DeviceGeneration::initial(),
        RenderQueueClass::Copy,
        sequence,
    )
}

#[test]
fn submission_ticket_binds_device_generation_queue_and_sequence() {
    let ticket = SubmissionTicket::new(
        DeviceId::new(41),
        DeviceGeneration::initial(),
        RenderQueueClass::Compute,
        17,
    );

    assert_eq!(ticket.device_id(), DeviceId::new(41));
    assert_eq!(ticket.generation(), DeviceGeneration::initial());
    assert_eq!(ticket.queue_class(), RenderQueueClass::Compute);
    assert_eq!(ticket.sequence(), 17);
}

#[test]
fn only_submission_terminal_states_allow_completion_consumers_to_advance() {
    assert!(!SubmissionStatus::Accepted.is_terminal());
    assert!(!SubmissionStatus::Submitted.is_terminal());
    assert!(SubmissionStatus::Completed.is_terminal());
    assert!(SubmissionStatus::Failed.is_terminal());
    assert!(SubmissionStatus::Cancelled.is_terminal());
    assert!(SubmissionStatus::DeviceLost.is_terminal());
}

#[test]
fn submission_history_bounds_observable_statuses_without_losing_retirement_safety() {
    let mut history = SubmissionHistory::new(SubmissionLimits::new(2, 1));
    let first = ticket(1);
    let second = ticket(2);
    let third = ticket(3);

    assert!(history.record_accepted(first));
    assert!(history.record_accepted(second));
    assert!(!history.can_accept());
    history.transition(second, SubmissionStatus::Cancelled);
    assert!(history.record_accepted(third));
    history.transition(third, SubmissionStatus::Cancelled);

    assert_eq!(history.status(second), None);
    assert!(history.is_terminal(second));
    assert!(history.is_terminal(third));
    assert!(!history.is_terminal(first));
    assert_eq!(history.unresolved_count(), 1);

    history.transition(first, SubmissionStatus::Completed);
    assert!(history.is_terminal(first));
    assert_eq!(history.unresolved_count(), 0);
}

#[test]
fn submission_history_terminal_ranges_stay_bounded_by_unresolved_gaps() {
    let mut history = SubmissionHistory::new(SubmissionLimits::new(3, 0));
    let first = ticket(1);
    assert!(history.record_accepted(first));

    for sequence in 2..32 {
        let current = ticket(sequence);
        assert!(history.record_accepted(current));
        history.transition(current, SubmissionStatus::Cancelled);
    }

    assert_eq!(history.unresolved_count(), 1);
    assert_eq!(history.terminal_range_count(), 1);
    assert!(history.is_terminal(ticket(31)));
    assert!(!history.is_terminal(first));
}
