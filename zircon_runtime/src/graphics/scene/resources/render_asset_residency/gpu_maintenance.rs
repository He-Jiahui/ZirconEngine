use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};

use zr_rhi::{
    RenderDevice, RenderQueueClass, RhiError, SubmissionPollReceipt, SubmissionStatus,
    SubmissionTicket,
};

use super::gpu_upload::RenderAssetGpuArtifactKind;
use super::{
    RenderAssetDeviceEpoch, RenderAssetResidencyManager, RenderAssetResidencyRelease,
    RenderAssetResidencyTicket, RenderAssetResidencyTransitionError,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderAssetGpuMaintenanceBudget {
    max_submission_status_checks: usize,
    max_artifact_retirements: usize,
}

impl RenderAssetGpuMaintenanceBudget {
    pub(crate) const fn new(
        max_submission_status_checks: usize,
        max_artifact_retirements: usize,
    ) -> Self {
        Self {
            max_submission_status_checks,
            max_artifact_retirements,
        }
    }

    pub(crate) const fn max_submission_status_checks(self) -> usize {
        self.max_submission_status_checks
    }

    pub(crate) const fn max_artifact_retirements(self) -> usize {
        self.max_artifact_retirements
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetGpuMaintenanceFailure {
    PollReceipt(RenderAssetGpuPollReceiptError),
    SubmissionStatus {
        submission: SubmissionTicket,
        error: RhiError,
    },
    ResidencyTransition {
        ticket: RenderAssetResidencyTicket,
        error: RenderAssetResidencyTransitionError,
    },
    ArtifactRetirement {
        kind: RenderAssetGpuArtifactKind,
        allocation_bytes: u64,
        error: RhiError,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RenderAssetGpuPollReceiptError {
    DeviceMismatch {
        expected: RenderAssetDeviceEpoch,
        received: SubmissionPollReceipt,
    },
    BoundEpochMismatch {
        bound: RenderAssetDeviceEpoch,
        received: SubmissionPollReceipt,
    },
    StreamChanged {
        previous: SubmissionPollReceipt,
        received: SubmissionPollReceipt,
    },
    NotAdvanced {
        previous: SubmissionPollReceipt,
        received: SubmissionPollReceipt,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RenderAssetGpuMaintenanceReport {
    submission_status_checks: usize,
    terminal_submissions: usize,
    published_artifacts: usize,
    failed_uploads: usize,
    detached_uploads_finalized: usize,
    deferred_terminal_uploads: usize,
    artifact_retirement_attempts: usize,
    retired_artifacts: usize,
    retired_bytes: u64,
    remaining_tracked_submissions: usize,
    remaining_ready_retirements: usize,
    remaining_ready_retirement_bytes: u64,
    releases: Vec<RenderAssetResidencyRelease>,
    failures: Vec<RenderAssetGpuMaintenanceFailure>,
}

impl RenderAssetGpuMaintenanceReport {
    pub(crate) const fn submission_status_checks(&self) -> usize {
        self.submission_status_checks
    }

    pub(crate) const fn terminal_submissions(&self) -> usize {
        self.terminal_submissions
    }

    pub(crate) const fn published_artifacts(&self) -> usize {
        self.published_artifacts
    }

    pub(crate) const fn failed_uploads(&self) -> usize {
        self.failed_uploads
    }

    pub(crate) const fn detached_uploads_finalized(&self) -> usize {
        self.detached_uploads_finalized
    }

    pub(crate) const fn deferred_terminal_uploads(&self) -> usize {
        self.deferred_terminal_uploads
    }

    pub(crate) const fn artifact_retirement_attempts(&self) -> usize {
        self.artifact_retirement_attempts
    }

    pub(crate) const fn retired_artifacts(&self) -> usize {
        self.retired_artifacts
    }

    pub(crate) const fn retired_bytes(&self) -> u64 {
        self.retired_bytes
    }

    pub(crate) const fn remaining_tracked_submissions(&self) -> usize {
        self.remaining_tracked_submissions
    }

    pub(crate) const fn remaining_ready_retirements(&self) -> usize {
        self.remaining_ready_retirements
    }

    pub(crate) const fn remaining_ready_retirement_bytes(&self) -> u64 {
        self.remaining_ready_retirement_bytes
    }

    pub(crate) fn releases(&self) -> &[RenderAssetResidencyRelease] {
        &self.releases
    }

    pub(crate) fn failures(&self) -> &[RenderAssetGpuMaintenanceFailure] {
        &self.failures
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct RenderAssetGpuSubmissionKey {
    device: u64,
    generation: u64,
    sequence: u64,
    queue: u8,
}

impl From<SubmissionTicket> for RenderAssetGpuSubmissionKey {
    fn from(ticket: SubmissionTicket) -> Self {
        Self {
            device: ticket.device_id().raw(),
            generation: ticket.generation().raw(),
            sequence: ticket.sequence(),
            queue: queue_order(ticket.queue_class()),
        }
    }
}

const fn queue_order(queue: RenderQueueClass) -> u8 {
    match queue {
        RenderQueueClass::Graphics => 0,
        RenderQueueClass::Compute => 1,
        RenderQueueClass::Copy => 2,
    }
}

#[derive(Default)]
pub(super) struct RenderAssetGpuSubmissionFrontier {
    tracked: BTreeMap<RenderAssetGpuSubmissionKey, SubmissionTicket>,
    cursor: Option<RenderAssetGpuSubmissionKey>,
}

impl RenderAssetGpuSubmissionFrontier {
    pub(super) fn insert(&mut self, ticket: SubmissionTicket) -> bool {
        self.tracked.insert(ticket.into(), ticket).is_none()
    }

    pub(super) fn remove(&mut self, ticket: SubmissionTicket) -> bool {
        self.tracked.remove(&ticket.into()).is_some()
    }

    pub(super) fn contains(&self, ticket: SubmissionTicket) -> bool {
        self.tracked.contains_key(&ticket.into())
    }

    pub(super) fn len(&self) -> usize {
        self.tracked.len()
    }

    pub(super) fn append_next_batch(
        &mut self,
        limit: usize,
        batch: &mut Vec<SubmissionTicket>,
    ) -> usize {
        let count = limit.min(self.tracked.len());
        let initial_len = batch.len();
        batch.reserve(count);
        for _ in 0..count {
            let Some((key, ticket)) = self.next_entry() else {
                break;
            };
            self.cursor = Some(key);
            batch.push(ticket);
        }
        batch.len().saturating_sub(initial_len)
    }

    fn next_entry(&self) -> Option<(RenderAssetGpuSubmissionKey, SubmissionTicket)> {
        let after_cursor = self.cursor.and_then(|cursor| {
            self.tracked
                .range((Excluded(cursor), Unbounded))
                .next()
                .map(|(key, ticket)| (*key, *ticket))
        });
        after_cursor.or_else(|| {
            self.tracked
                .first_key_value()
                .map(|(key, ticket)| (*key, *ticket))
        })
    }
}

impl RenderAssetResidencyManager {
    /// Advances only this manager's bounded submission frontier after the
    /// product RHI owner has performed its single nonblocking device poll.
    pub(crate) fn maintain_gpu_after_rhi_poll(
        &mut self,
        device: &dyn RenderDevice,
        poll: SubmissionPollReceipt,
        budget: RenderAssetGpuMaintenanceBudget,
    ) -> RenderAssetGpuMaintenanceReport {
        let mut report = RenderAssetGpuMaintenanceReport::default();
        let expected = RenderAssetDeviceEpoch::new(device.device_id(), device.generation());
        if let Err(error) = validate_poll_receipt(
            expected,
            self.gpu.bound_device_epoch(),
            self.gpu.last_poll_receipt(),
            poll,
        ) {
            report
                .failures
                .push(RenderAssetGpuMaintenanceFailure::PollReceipt(error));
            report.remaining_tracked_submissions = self.gpu.tracked_submission_count();
            report.remaining_ready_retirements = self.ready_gpu_retirement_count();
            report.remaining_ready_retirement_bytes = self.ready_gpu_retirement_bytes();
            return report;
        }
        self.gpu.record_poll_receipt(poll);

        let retirement_attempts = budget
            .max_artifact_retirements()
            .min(self.ready_gpu_retirement_count());
        for _ in 0..retirement_attempts {
            let Some(artifact) = self.gpu.pop_ready_retirement() else {
                break;
            };
            let kind = artifact.kind();
            let allocation_bytes = artifact.allocation_bytes();
            report.artifact_retirement_attempts =
                report.artifact_retirement_attempts.saturating_add(1);
            match artifact.retire(device) {
                Ok(()) => {
                    report.retired_artifacts = report.retired_artifacts.saturating_add(1);
                    report.retired_bytes = report.retired_bytes.saturating_add(allocation_bytes);
                }
                Err((artifact, error)) => {
                    report
                        .failures
                        .push(RenderAssetGpuMaintenanceFailure::ArtifactRetirement {
                            kind,
                            allocation_bytes,
                            error,
                        });
                    self.gpu.enqueue_retirement(artifact);
                }
            }
        }

        let observations = self
            .gpu
            .take_observation_scratch(budget.max_submission_status_checks());
        let mut statuses = self.gpu.take_status_scratch();
        device.append_submission_statuses(&observations, &mut statuses);
        debug_assert_eq!(observations.len(), statuses.len());
        report.submission_status_checks = statuses.len();
        for (submission, status) in observations.iter().copied().zip(statuses.drain(..)) {
            let status = match status {
                Ok(status) => status,
                Err(error) => {
                    report
                        .failures
                        .push(RenderAssetGpuMaintenanceFailure::SubmissionStatus {
                            submission,
                            error,
                        });
                    SubmissionStatus::Failed
                }
            };
            if !status.is_terminal() {
                continue;
            }
            report.terminal_submissions = report.terminal_submissions.saturating_add(1);
            if let Some(ticket) = self.gpu.pending_ticket_for_submission(submission) {
                match self.complete_gpu_upload(&ticket, status) {
                    Ok(mutation) => {
                        report.releases.extend_from_slice(mutation.releases());
                        if status == SubmissionStatus::Completed {
                            report.published_artifacts =
                                report.published_artifacts.saturating_add(1);
                        } else {
                            report.failed_uploads = report.failed_uploads.saturating_add(1);
                        }
                    }
                    Err(error) => report.failures.push(
                        RenderAssetGpuMaintenanceFailure::ResidencyTransition { ticket, error },
                    ),
                }
            }
            let (finalized, deferred) = self.observe_retiring_gpu_upload_status(submission, status);
            report.detached_uploads_finalized =
                report.detached_uploads_finalized.saturating_add(finalized);
            report.deferred_terminal_uploads =
                report.deferred_terminal_uploads.saturating_add(deferred);
        }
        self.gpu.restore_observation_scratch(observations);
        self.gpu.restore_status_scratch(statuses);
        report.remaining_tracked_submissions = self.gpu.tracked_submission_count();
        report.remaining_ready_retirements = self.ready_gpu_retirement_count();
        report.remaining_ready_retirement_bytes = self.ready_gpu_retirement_bytes();
        report
    }
}

fn validate_poll_receipt(
    expected: RenderAssetDeviceEpoch,
    bound: Option<RenderAssetDeviceEpoch>,
    previous: Option<SubmissionPollReceipt>,
    received: SubmissionPollReceipt,
) -> Result<(), RenderAssetGpuPollReceiptError> {
    let actual = RenderAssetDeviceEpoch::new(received.device_id(), received.generation());
    if actual != expected {
        return Err(RenderAssetGpuPollReceiptError::DeviceMismatch { expected, received });
    }
    if let Some(bound) = bound {
        if actual != bound {
            return Err(RenderAssetGpuPollReceiptError::BoundEpochMismatch { bound, received });
        }
    }
    let Some(previous) = previous else {
        return Ok(());
    };
    if previous.device_id() != received.device_id()
        || previous.generation() != received.generation()
    {
        return Err(RenderAssetGpuPollReceiptError::StreamChanged { previous, received });
    }
    if received.sequence() <= previous.sequence() {
        return Err(RenderAssetGpuPollReceiptError::NotAdvanced { previous, received });
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/gpu_maintenance.rs"]
mod tests;
