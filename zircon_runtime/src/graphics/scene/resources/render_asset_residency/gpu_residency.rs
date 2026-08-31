use std::collections::{HashMap, VecDeque};

use crate::core::resource::UntypedResourceHandle;
use zr_rhi::{SubmissionLimits, SubmissionPollReceipt, SubmissionStatus, SubmissionTicket};

use super::gpu_maintenance::RenderAssetGpuSubmissionFrontier;
use super::gpu_upload::{
    RenderAssetGpuArtifact, RenderAssetGpuUploadFinalize, RenderAssetGpuUploadLease,
};
use super::{
    RenderAssetDeviceEpoch, RenderAssetResidencyManager, RenderAssetResidencyMutation,
    RenderAssetResidencyTicket, RenderAssetResidencyTransitionError,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RenderAssetGpuResidencyLimits {
    max_tracked_submissions: usize,
    max_ready_retirements: usize,
}

impl RenderAssetGpuResidencyLimits {
    pub(crate) const fn new(max_tracked_submissions: usize, max_ready_retirements: usize) -> Self {
        Self {
            max_tracked_submissions,
            max_ready_retirements,
        }
    }

    pub(crate) const fn max_tracked_submissions(self) -> usize {
        self.max_tracked_submissions
    }

    pub(crate) const fn max_ready_retirements(self) -> usize {
        self.max_ready_retirements
    }
}

impl Default for RenderAssetGpuResidencyLimits {
    fn default() -> Self {
        let submission_limit = SubmissionLimits::default().max_terminal_statuses();
        Self::new(submission_limit, submission_limit)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RenderAssetGpuRetirementBackpressure {
    pub(super) ready_retirements: usize,
    pub(super) requested_retirements: usize,
    pub(super) limit: usize,
}

impl RenderAssetGpuRetirementBackpressure {
    pub(super) fn into_admission_error(self) -> super::RenderAssetResidencyAdmissionError {
        super::RenderAssetResidencyAdmissionError::GpuRetirementBackpressure {
            ready_retirements: self.ready_retirements,
            requested_retirements: self.requested_retirements,
            limit: self.limit,
        }
    }

    pub(super) fn into_transition_error(self) -> RenderAssetResidencyTransitionError {
        RenderAssetResidencyTransitionError::GpuRetirementBackpressure {
            ready_retirements: self.ready_retirements,
            requested_retirements: self.requested_retirements,
            limit: self.limit,
        }
    }
}

fn ensure_retirement_capacity(
    ready_retirements: usize,
    requested_retirements: usize,
    limit: usize,
) -> Result<(), RenderAssetGpuRetirementBackpressure> {
    if ready_retirements
        .checked_add(requested_retirements)
        .is_some_and(|required| required <= limit)
    {
        return Ok(());
    }
    Err(RenderAssetGpuRetirementBackpressure {
        ready_retirements,
        requested_retirements,
        limit,
    })
}

pub(super) struct RenderAssetGpuResidencyState {
    limits: RenderAssetGpuResidencyLimits,
    bound_device_epoch: Option<RenderAssetDeviceEpoch>,
    pending_submissions: HashMap<SubmissionTicket, RenderAssetResidencyTicket>,
    retiring_uploads: HashMap<SubmissionTicket, Vec<RenderAssetGpuUploadLease>>,
    tracked_submissions: RenderAssetGpuSubmissionFrontier,
    last_poll_receipt: Option<SubmissionPollReceipt>,
    observation_scratch: Vec<SubmissionTicket>,
    status_scratch: Vec<Result<SubmissionStatus, zr_rhi::RhiError>>,
    ready_retirements: VecDeque<RenderAssetGpuArtifact>,
    ready_retirement_bytes: u64,
}

impl Default for RenderAssetGpuResidencyState {
    fn default() -> Self {
        Self::new(RenderAssetGpuResidencyLimits::default())
    }
}

impl RenderAssetGpuResidencyState {
    pub(super) fn new(limits: RenderAssetGpuResidencyLimits) -> Self {
        Self {
            limits,
            bound_device_epoch: None,
            pending_submissions: HashMap::new(),
            retiring_uploads: HashMap::new(),
            tracked_submissions: RenderAssetGpuSubmissionFrontier::default(),
            last_poll_receipt: None,
            observation_scratch: Vec::new(),
            status_scratch: Vec::new(),
            ready_retirements: VecDeque::new(),
            ready_retirement_bytes: 0,
        }
    }

    pub(super) fn detach_entry(&mut self, entry: &mut super::manager::RenderAssetResidencyEntry) {
        self.detach_pending_upload(entry);
        if let Some(artifact) = entry.active_artifact.take() {
            self.enqueue_retirement(artifact);
        }
    }

    pub(super) fn detach_pending_upload(
        &mut self,
        entry: &mut super::manager::RenderAssetResidencyEntry,
    ) {
        if let Some(upload) = entry.pending_upload.take() {
            self.pending_submissions.remove(&upload.submission());
            self.retiring_uploads
                .entry(upload.submission())
                .or_default()
                .push(upload);
        }
    }

    pub(super) fn enqueue_retirement(&mut self, artifact: RenderAssetGpuArtifact) {
        debug_assert!(self.ensure_ready_retirement_capacity(1).is_ok());
        self.ready_retirement_bytes = self
            .ready_retirement_bytes
            .saturating_add(artifact.allocation_bytes());
        self.ready_retirements.push_back(artifact);
    }

    fn observe_retiring_upload_status(
        &mut self,
        submission: SubmissionTicket,
        status: SubmissionStatus,
    ) -> (usize, usize) {
        if !status.is_terminal() {
            return (0, 0);
        }
        let available = self.available_ready_retirement_slots();
        let upload_count = self
            .retiring_uploads
            .get(&submission)
            .map(Vec::len)
            .unwrap_or(0);
        let finalize_count = upload_count.min(available);
        let mut finalized = 0_usize;
        for _ in 0..finalize_count {
            let Some(upload) = self
                .retiring_uploads
                .get_mut(&submission)
                .and_then(Vec::pop)
            else {
                break;
            };
            match upload.finalize(status) {
                RenderAssetGpuUploadFinalize::Pending(upload) => {
                    self.retiring_uploads
                        .entry(submission)
                        .or_default()
                        .push(upload);
                    break;
                }
                RenderAssetGpuUploadFinalize::Resident { artifact, .. }
                | RenderAssetGpuUploadFinalize::Failed { artifact, .. } => {
                    self.enqueue_retirement(artifact);
                    finalized = finalized.saturating_add(1);
                }
            }
        }
        let deferred = self
            .retiring_uploads
            .get(&submission)
            .map(Vec::len)
            .unwrap_or(0);
        if deferred == 0 {
            self.retiring_uploads.remove(&submission);
        }
        self.finish_tracking_if_unowned(submission);
        (finalized, deferred)
    }

    fn ensure_can_track(
        &self,
        submission: SubmissionTicket,
    ) -> Result<(), RenderAssetResidencyTransitionError> {
        let actual = RenderAssetDeviceEpoch::new(submission.device_id(), submission.generation());
        if let Some(expected) = self.bound_device_epoch {
            if actual != expected {
                return Err(
                    RenderAssetResidencyTransitionError::SubmissionDeviceMismatch {
                        expected,
                        actual,
                    },
                );
            }
        }
        if self.tracked_submissions.contains(submission) {
            return Err(
                RenderAssetResidencyTransitionError::SubmissionAlreadyTracked { submission },
            );
        }
        let tracked_submissions = self.tracked_submissions.len();
        if tracked_submissions >= self.limits.max_tracked_submissions() {
            return Err(
                RenderAssetResidencyTransitionError::GpuTrackingBackpressure {
                    tracked_submissions,
                    limit: self.limits.max_tracked_submissions(),
                },
            );
        }
        Ok(())
    }

    fn track_pending(&mut self, submission: SubmissionTicket, ticket: RenderAssetResidencyTicket) {
        let epoch = RenderAssetDeviceEpoch::new(submission.device_id(), submission.generation());
        self.bind_device_epoch(epoch);
        let inserted = self.tracked_submissions.insert(submission);
        debug_assert!(inserted);
        let previous = self.pending_submissions.insert(submission, ticket);
        debug_assert!(previous.is_none());
    }

    fn finish_pending_tracking(&mut self, submission: SubmissionTicket) {
        self.pending_submissions.remove(&submission);
        self.finish_tracking_if_unowned(submission);
    }

    fn finish_tracking_if_unowned(&mut self, submission: SubmissionTicket) {
        if !self.pending_submissions.contains_key(&submission)
            && !self.retiring_uploads.contains_key(&submission)
        {
            self.tracked_submissions.remove(submission);
        }
    }

    pub(super) fn take_observation_scratch(&mut self, limit: usize) -> Vec<SubmissionTicket> {
        let mut scratch = std::mem::take(&mut self.observation_scratch);
        scratch.clear();
        self.tracked_submissions
            .append_next_batch(limit, &mut scratch);
        scratch
    }

    pub(super) fn restore_observation_scratch(&mut self, mut scratch: Vec<SubmissionTicket>) {
        scratch.clear();
        self.observation_scratch = scratch;
    }

    pub(super) fn take_status_scratch(
        &mut self,
    ) -> Vec<Result<SubmissionStatus, zr_rhi::RhiError>> {
        let mut scratch = std::mem::take(&mut self.status_scratch);
        scratch.clear();
        scratch
    }

    pub(super) fn restore_status_scratch(
        &mut self,
        mut scratch: Vec<Result<SubmissionStatus, zr_rhi::RhiError>>,
    ) {
        scratch.clear();
        self.status_scratch = scratch;
    }

    pub(super) fn pending_ticket_for_submission(
        &self,
        submission: SubmissionTicket,
    ) -> Option<RenderAssetResidencyTicket> {
        self.pending_submissions.get(&submission).copied()
    }

    pub(super) fn pop_ready_retirement(&mut self) -> Option<RenderAssetGpuArtifact> {
        let artifact = self.ready_retirements.pop_front()?;
        self.ready_retirement_bytes = self
            .ready_retirement_bytes
            .saturating_sub(artifact.allocation_bytes());
        Some(artifact)
    }

    pub(super) fn ensure_ready_retirement_capacity(
        &self,
        requested_retirements: usize,
    ) -> Result<(), RenderAssetGpuRetirementBackpressure> {
        ensure_retirement_capacity(
            self.ready_retirements.len(),
            requested_retirements,
            self.limits.max_ready_retirements(),
        )
    }

    fn available_ready_retirement_slots(&self) -> usize {
        self.limits
            .max_ready_retirements()
            .saturating_sub(self.ready_retirements.len())
    }

    pub(super) fn tracked_submission_count(&self) -> usize {
        self.tracked_submissions.len()
    }

    pub(super) const fn ready_retirement_bytes(&self) -> u64 {
        self.ready_retirement_bytes
    }

    pub(super) const fn last_poll_receipt(&self) -> Option<SubmissionPollReceipt> {
        self.last_poll_receipt
    }

    pub(super) fn record_poll_receipt(&mut self, receipt: SubmissionPollReceipt) {
        let epoch = RenderAssetDeviceEpoch::new(receipt.device_id(), receipt.generation());
        self.bind_device_epoch(epoch);
        self.last_poll_receipt = Some(receipt);
    }

    pub(super) const fn bound_device_epoch(&self) -> Option<RenderAssetDeviceEpoch> {
        self.bound_device_epoch
    }

    pub(super) fn bind_device_epoch(&mut self, epoch: RenderAssetDeviceEpoch) {
        debug_assert!(self.bound_device_epoch.is_none_or(|bound| bound == epoch));
        if self.bound_device_epoch.is_none() {
            self.bound_device_epoch = Some(epoch);
        }
    }

    pub(super) fn abandon_for_device_recovery(
        &mut self,
        replacement: RenderAssetDeviceEpoch,
    ) -> RenderAssetGpuAbandonReport {
        let detached_uploads = self
            .retiring_uploads
            .values()
            .map(Vec::len)
            .fold(0_usize, usize::saturating_add);
        let detached_bytes = self
            .retiring_uploads
            .values()
            .flatten()
            .map(|upload| upload.artifact().allocation_bytes())
            .fold(0_u64, u64::saturating_add);
        let report = RenderAssetGpuAbandonReport {
            tracked_submissions: self.tracked_submissions.len(),
            detached_uploads,
            ready_retirements: self.ready_retirements.len(),
            allocation_bytes: detached_bytes.saturating_add(self.ready_retirement_bytes),
        };

        self.pending_submissions.clear();
        self.retiring_uploads.clear();
        self.tracked_submissions = RenderAssetGpuSubmissionFrontier::default();
        self.last_poll_receipt = None;
        self.observation_scratch.clear();
        self.status_scratch.clear();
        self.ready_retirements.clear();
        self.ready_retirement_bytes = 0;
        self.bound_device_epoch = Some(replacement);
        report
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RenderAssetGpuAbandonReport {
    pub(super) tracked_submissions: usize,
    pub(super) detached_uploads: usize,
    pub(super) ready_retirements: usize,
    pub(super) allocation_bytes: u64,
}

pub(crate) struct RenderAssetGpuUploadBindFailure {
    error: RenderAssetResidencyTransitionError,
    upload: RenderAssetGpuUploadLease,
}

impl RenderAssetGpuUploadBindFailure {
    fn new(error: RenderAssetResidencyTransitionError, upload: RenderAssetGpuUploadLease) -> Self {
        Self { error, upload }
    }

    pub(crate) const fn error(&self) -> &RenderAssetResidencyTransitionError {
        &self.error
    }

    pub(crate) const fn upload(&self) -> &RenderAssetGpuUploadLease {
        &self.upload
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        RenderAssetResidencyTransitionError,
        RenderAssetGpuUploadLease,
    ) {
        (self.error, self.upload)
    }
}

impl RenderAssetResidencyManager {
    pub(crate) fn bind_gpu_upload(
        &mut self,
        upload: RenderAssetGpuUploadLease,
    ) -> Result<(), RenderAssetGpuUploadBindFailure> {
        let ticket = upload.ticket();
        let submission = upload.submission();
        if let Err(error) = self.gpu.ensure_ready_retirement_capacity(1) {
            return Err(RenderAssetGpuUploadBindFailure::new(
                error.into_transition_error(),
                upload,
            ));
        }
        if let Err(error) = self.gpu.ensure_can_track(submission) {
            return Err(RenderAssetGpuUploadBindFailure::new(error, upload));
        }
        let entry = match self.bind_upload_submission_entry(ticket, submission) {
            Ok(entry) => entry,
            Err(error) => return Err(RenderAssetGpuUploadBindFailure::new(error, upload)),
        };
        entry.pending_upload = Some(upload);
        self.gpu.track_pending(submission, ticket);
        Ok(())
    }

    pub(super) fn complete_gpu_upload(
        &mut self,
        ticket: RenderAssetResidencyTicket,
        status: SubmissionStatus,
    ) -> Result<RenderAssetResidencyMutation, RenderAssetResidencyTransitionError> {
        let submission = self
            .entries
            .get(&ticket.resource())
            .and_then(|entry| entry.pending_upload.as_ref())
            .ok_or(RenderAssetResidencyTransitionError::UploadLeaseNotBound {
                ticket: ticket.id(),
            })?
            .submission();
        self.validate_upload_completion(ticket, submission, status)?;
        let requested_retirements = if status == SubmissionStatus::Completed
            && self
                .entries
                .get(&ticket.resource())
                .is_some_and(|entry| entry.active_artifact.is_none())
        {
            0
        } else {
            1
        };
        self.gpu
            .ensure_ready_retirement_capacity(requested_retirements)
            .map_err(RenderAssetGpuRetirementBackpressure::into_transition_error)?;
        let upload = self
            .entries
            .get_mut(&ticket.resource())
            .and_then(|entry| entry.pending_upload.take())
            .ok_or(RenderAssetResidencyTransitionError::UploadLeaseNotBound {
                ticket: ticket.id(),
            })?;
        let mutation = match self.complete_upload(ticket, submission, status) {
            Ok(mutation) => mutation,
            Err(error) => {
                if let Some(entry) = self.entries.get_mut(&ticket.resource()) {
                    entry.pending_upload = Some(upload);
                }
                return Err(error);
            }
        };

        match upload.finalize(status) {
            RenderAssetGpuUploadFinalize::Pending(upload) => {
                if let Some(entry) = self.entries.get_mut(&ticket.resource()) {
                    entry.pending_upload = Some(upload);
                }
                return Err(RenderAssetResidencyTransitionError::SubmissionNotTerminal { status });
            }
            RenderAssetGpuUploadFinalize::Resident { artifact, .. } => {
                let previous = self
                    .entries
                    .get_mut(&ticket.resource())
                    .and_then(|entry| entry.active_artifact.replace(artifact));
                if let Some(previous) = previous {
                    self.gpu.enqueue_retirement(previous);
                }
            }
            RenderAssetGpuUploadFinalize::Failed { artifact, .. } => {
                self.gpu.enqueue_retirement(artifact);
            }
        }
        self.gpu.finish_pending_tracking(submission);
        Ok(mutation)
    }

    pub(super) fn observe_retiring_gpu_upload_status(
        &mut self,
        submission: SubmissionTicket,
        status: SubmissionStatus,
    ) -> (usize, usize) {
        self.gpu.observe_retiring_upload_status(submission, status)
    }

    pub(crate) fn gpu_artifact(
        &self,
        resource: UntypedResourceHandle,
    ) -> Option<&RenderAssetGpuArtifact> {
        self.entries
            .get(&resource)
            .and_then(|entry| entry.active_artifact.as_ref())
    }

    pub(crate) fn retiring_gpu_upload_count(&self) -> usize {
        self.gpu
            .retiring_uploads
            .values()
            .map(Vec::len)
            .fold(0_usize, usize::saturating_add)
    }

    pub(crate) fn ready_gpu_retirement_count(&self) -> usize {
        self.gpu.ready_retirements.len()
    }

    pub(crate) const fn ready_gpu_retirement_bytes(&self) -> u64 {
        self.gpu.ready_retirement_bytes()
    }
}

#[cfg(test)]
mod tests {
    use zr_rhi::{
        DeviceGeneration, DeviceId, RenderQueueClass, SubmissionPollReceipt, SubmissionTicket,
    };

    use super::{
        RenderAssetGpuResidencyLimits, RenderAssetGpuResidencyState,
        RenderAssetGpuRetirementBackpressure, ensure_retirement_capacity,
    };
    use crate::graphics::scene::resources::render_asset_residency::{
        RenderAssetDeviceEpoch, RenderAssetResidencyTransitionError,
    };

    fn ticket(sequence: u64) -> SubmissionTicket {
        SubmissionTicket::new(
            DeviceId::new(31),
            DeviceGeneration::new(4),
            RenderQueueClass::Copy,
            sequence,
        )
    }

    #[test]
    fn gpu_tracking_rejects_duplicates_and_capacity_before_state_mutation() {
        let mut state = RenderAssetGpuResidencyState::new(RenderAssetGpuResidencyLimits::new(1, 1));
        assert_eq!(state.ensure_can_track(ticket(1)), Ok(()));
        assert!(state.tracked_submissions.insert(ticket(1)));

        assert_eq!(
            state.ensure_can_track(ticket(1)),
            Err(
                RenderAssetResidencyTransitionError::SubmissionAlreadyTracked {
                    submission: ticket(1),
                }
            )
        );
        assert_eq!(
            state.ensure_can_track(ticket(2)),
            Err(
                RenderAssetResidencyTransitionError::GpuTrackingBackpressure {
                    tracked_submissions: 1,
                    limit: 1,
                }
            )
        );
        assert_eq!(state.tracked_submissions.len(), 1);
    }

    #[test]
    fn retirement_capacity_rejects_full_and_overflowing_batches_without_saturation() {
        assert_eq!(ensure_retirement_capacity(1, 2, 3), Ok(()));
        let full = RenderAssetGpuRetirementBackpressure {
            ready_retirements: 2,
            requested_retirements: 2,
            limit: 3,
        };
        assert_eq!(ensure_retirement_capacity(2, 2, 3), Err(full));

        let overflow = RenderAssetGpuRetirementBackpressure {
            ready_retirements: usize::MAX,
            requested_retirements: 1,
            limit: usize::MAX,
        };
        assert_eq!(
            ensure_retirement_capacity(usize::MAX, 1, usize::MAX),
            Err(overflow)
        );
    }

    #[test]
    fn device_recovery_resets_completion_stream_and_rejects_implicit_epoch_changes() {
        let old = RenderAssetDeviceEpoch::new(DeviceId::new(31), DeviceGeneration::new(4));
        let replacement = RenderAssetDeviceEpoch::new(DeviceId::new(31), DeviceGeneration::new(5));
        let mut state = RenderAssetGpuResidencyState::default();
        let old_poll = SubmissionPollReceipt::new(old.device_id(), old.generation(), 7);
        state.record_poll_receipt(old_poll);

        let replacement_submission = SubmissionTicket::new(
            replacement.device_id(),
            replacement.generation(),
            RenderQueueClass::Copy,
            1,
        );
        assert!(matches!(
            state.ensure_can_track(replacement_submission),
            Err(RenderAssetResidencyTransitionError::SubmissionDeviceMismatch { .. })
        ));

        let abandoned = state.abandon_for_device_recovery(replacement);
        assert_eq!(abandoned, super::RenderAssetGpuAbandonReport::default());
        assert_eq!(state.bound_device_epoch(), Some(replacement));
        assert_eq!(state.last_poll_receipt(), None);
        assert_eq!(state.ensure_can_track(replacement_submission), Ok(()));
    }
}
