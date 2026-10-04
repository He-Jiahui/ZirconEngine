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

/// 只追踪本设备世代的场景提交票据；由 SceneRenderer 的单次 RHI poll 驱动完成观察。
/// 容量有界，不能通过丢弃活跃票据腾出空间。
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

    /// 在场景提交后登记严格递增的 ticket；失败只写入 last_report，调用方可读取失败原因。
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

    /// 使用唯一 poll receipt 批量读取状态；错误批次不消费队列，终态与历史缺失的处理由报告承载。
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
#[path = "tests/scene_submission_completion_journal.rs"]
mod tests;
