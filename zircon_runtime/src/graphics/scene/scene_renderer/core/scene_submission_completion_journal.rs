use std::collections::VecDeque;

use crate::core::framework::render::{
    RenderSceneSubmissionCompletionError, RenderSceneSubmissionCompletionFailure,
    RenderSceneSubmissionCompletionReport, RenderSceneSubmissionCompletionStatus,
};
use crate::rhi::{
    DeviceGeneration, DeviceId, RhiError, SubmissionPollReceipt, SubmissionStatus, SubmissionTicket,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PendingSceneSubmission {
    frame_generation: u64,
    ticket: SubmissionTicket,
}

pub(in crate::graphics::scene::scene_renderer::core) struct SceneSubmissionCompletionJournal {
    device_id: DeviceId,
    device_generation: DeviceGeneration,
    capacity: usize,
    pending: VecDeque<PendingSceneSubmission>,
    ticket_scratch: Vec<SubmissionTicket>,
    status_scratch: Vec<Result<SubmissionStatus, RhiError>>,
    last_tracked_submission_sequence: Option<u64>,
    last_poll_sequence: Option<u64>,
    last_report: RenderSceneSubmissionCompletionReport,
}

impl SceneSubmissionCompletionJournal {
    pub(in crate::graphics::scene::scene_renderer::core) fn new(
        device_id: DeviceId,
        device_generation: DeviceGeneration,
        capacity: usize,
    ) -> Self {
        Self {
            device_id,
            device_generation,
            capacity,
            pending: VecDeque::new(),
            ticket_scratch: Vec::new(),
            status_scratch: Vec::new(),
            last_tracked_submission_sequence: None,
            last_poll_sequence: None,
            last_report: RenderSceneSubmissionCompletionReport {
                tracking_capacity: capacity,
                ..RenderSceneSubmissionCompletionReport::default()
            },
        }
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn track(
        &mut self,
        frame_generation: u64,
        ticket: SubmissionTicket,
    ) {
        let failure = if ticket.device_id() != self.device_id
            || ticket.generation() != self.device_generation
        {
            Some(RenderSceneSubmissionCompletionFailure::SubmissionOwnerMismatch)
        } else if self
            .last_tracked_submission_sequence
            .is_some_and(|sequence| ticket.sequence() <= sequence)
        {
            Some(RenderSceneSubmissionCompletionFailure::SubmissionSequenceDidNotAdvance)
        } else if self.pending.len() >= self.capacity {
            Some(RenderSceneSubmissionCompletionFailure::CapacityExceeded)
        } else {
            None
        };

        if let Some(failure) = failure {
            self.last_report = RenderSceneSubmissionCompletionReport {
                status: RenderSceneSubmissionCompletionStatus::TrackingFailed,
                failure,
                frame_generation,
                submission: Some(ticket),
                observed_after_poll: None,
                pending_submission_count: self.pending.len(),
                tracking_capacity: self.capacity,
                last_poll_observed_submission_count: 0,
                last_poll_terminal_submission_count: 0,
            };
            return;
        }

        self.pending.push_back(PendingSceneSubmission {
            frame_generation,
            ticket,
        });
        self.last_tracked_submission_sequence = Some(ticket.sequence());
        self.last_report.pending_submission_count = self.pending.len();
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn observe(
        &mut self,
        poll: SubmissionPollReceipt,
        append_statuses: impl FnOnce(&[SubmissionTicket], &mut Vec<Result<SubmissionStatus, RhiError>>),
    ) -> Result<(), RenderSceneSubmissionCompletionError> {
        self.validate_poll(poll)?;

        if self.pending.is_empty() {
            self.last_poll_sequence = Some(poll.sequence());
            self.last_report.pending_submission_count = 0;
            self.last_report.last_poll_observed_submission_count = 0;
            self.last_report.last_poll_terminal_submission_count = 0;
            return Ok(());
        }

        self.ticket_scratch.clear();
        self.ticket_scratch
            .extend(self.pending.iter().map(|pending| pending.ticket));
        self.status_scratch.clear();
        append_statuses(&self.ticket_scratch, &mut self.status_scratch);
        if self.status_scratch.len() != self.pending.len() {
            return Err(
                RenderSceneSubmissionCompletionError::StatusResultCountMismatch {
                    expected: self.pending.len(),
                    actual: self.status_scratch.len(),
                },
            );
        }

        let observed_count = self.pending.len();
        let mut terminal_count = 0;
        for status in self.status_scratch.drain(..).take(observed_count) {
            let pending = self
                .pending
                .pop_front()
                .expect("status count was validated against the pending queue");
            match status {
                Ok(status) if status.is_terminal() => {
                    self.last_report = terminal_report(pending, poll, status);
                    terminal_count += 1;
                }
                Ok(_) => self.pending.push_back(pending),
                Err(_) => {
                    self.last_report = RenderSceneSubmissionCompletionReport {
                        status: RenderSceneSubmissionCompletionStatus::ObservationFailed,
                        failure: RenderSceneSubmissionCompletionFailure::StatusUnavailable,
                        frame_generation: pending.frame_generation,
                        submission: Some(pending.ticket),
                        observed_after_poll: Some(poll),
                        pending_submission_count: 0,
                        tracking_capacity: self.capacity,
                        last_poll_observed_submission_count: 0,
                        last_poll_terminal_submission_count: 0,
                    };
                }
            }
        }
        self.last_report.pending_submission_count = self.pending.len();
        self.last_report.tracking_capacity = self.capacity;
        self.last_report.last_poll_observed_submission_count = observed_count;
        self.last_report.last_poll_terminal_submission_count = terminal_count;
        self.last_poll_sequence = Some(poll.sequence());
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer::core) const fn last_report(
        &self,
    ) -> RenderSceneSubmissionCompletionReport {
        self.last_report
    }

    fn validate_poll(
        &self,
        poll: SubmissionPollReceipt,
    ) -> Result<(), RenderSceneSubmissionCompletionError> {
        if poll.device_id() != self.device_id || poll.generation() != self.device_generation {
            return Err(RenderSceneSubmissionCompletionError::PollOwnerMismatch {
                poll_device: poll.device_id(),
                poll_generation: poll.generation(),
                journal_device: self.device_id,
                journal_generation: self.device_generation,
            });
        }
        if let Some(previous_sequence) = self.last_poll_sequence {
            if poll.sequence() <= previous_sequence {
                return Err(
                    RenderSceneSubmissionCompletionError::PollSequenceDidNotAdvance {
                        previous_sequence,
                        poll_sequence: poll.sequence(),
                    },
                );
            }
        }
        Ok(())
    }
}

fn terminal_report(
    pending: PendingSceneSubmission,
    poll: SubmissionPollReceipt,
    status: SubmissionStatus,
) -> RenderSceneSubmissionCompletionReport {
    let status = match status {
        SubmissionStatus::Completed => RenderSceneSubmissionCompletionStatus::Completed,
        SubmissionStatus::Failed => RenderSceneSubmissionCompletionStatus::Failed,
        SubmissionStatus::Cancelled => RenderSceneSubmissionCompletionStatus::Cancelled,
        SubmissionStatus::DeviceLost => RenderSceneSubmissionCompletionStatus::DeviceLost,
        SubmissionStatus::Accepted | SubmissionStatus::Submitted => {
            unreachable!("terminal_report requires a terminal submission status")
        }
    };
    RenderSceneSubmissionCompletionReport {
        status,
        failure: RenderSceneSubmissionCompletionFailure::None,
        frame_generation: pending.frame_generation,
        submission: Some(pending.ticket),
        observed_after_poll: Some(poll),
        pending_submission_count: 0,
        tracking_capacity: 0,
        last_poll_observed_submission_count: 0,
        last_poll_terminal_submission_count: 0,
    }
}

#[cfg(test)]
mod tests {
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
        SceneSubmissionCompletionJournal::new(
            DeviceId::new(7),
            DeviceGeneration::initial(),
            capacity,
        )
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
}
