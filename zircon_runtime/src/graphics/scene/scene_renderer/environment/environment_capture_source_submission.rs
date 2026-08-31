use crate::core::framework::render::{
    RenderEnvironmentCaptureHandle, RenderEnvironmentCaptureOutputIdentity,
    RenderEnvironmentCaptureRequest,
};
use crate::core::resource::ResourceId;
use crate::graphics::backend::{SourceCubemapWgpuPendingReadback, SourceCubemapWgpuReadback};
use crate::rhi::SubmissionTicket;
use zr_rhi::SubmissionStatus;

use super::{
    EnvironmentCaptureFilterWgpuRecordReport, EnvironmentCaptureGpuOutput,
    EnvironmentCaptureGpuTarget, EnvironmentCaptureGpuTargetPlan,
    EnvironmentCaptureWgpuRecordReport, ProbeCubemapSlotReservation,
};

/// A probe-array destination reserved by the capture submission.
///
/// The reservation is carried with the source ticket owner and committed only after that ticket
/// completes and the scheduler accepts the generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) struct EnvironmentCaptureProbePublication {
    probe_id: u64,
    cubemap: ResourceId,
    reservation: ProbeCubemapSlotReservation,
}

impl EnvironmentCaptureProbePublication {
    pub(in crate::graphics) const fn new(
        probe_id: u64,
        cubemap: ResourceId,
        reservation: ProbeCubemapSlotReservation,
    ) -> Self {
        Self {
            probe_id,
            cubemap,
            reservation,
        }
    }

    pub(in crate::graphics) const fn probe_id(self) -> u64 {
        self.probe_id
    }

    pub(in crate::graphics) const fn cubemap(self) -> ResourceId {
        self.cubemap
    }

    pub(in crate::graphics) const fn reservation(self) -> ProbeCubemapSlotReservation {
        self.reservation
    }
}

/// Renderer-owned source cubemap and the exact backend work that produced it.
///
/// Keeping the target alive with its submission tickets makes the next source-mip,
/// PMREM and SH9 stages an explicit continuation of the same GPU transaction.
pub(in crate::graphics) struct EnvironmentCaptureSourceSubmission {
    handle: RenderEnvironmentCaptureHandle,
    request: RenderEnvironmentCaptureRequest,
    target: EnvironmentCaptureGpuTarget,
    resource_upload_submission: SubmissionTicket,
    capture_submission: SubmissionTicket,
    record_report: EnvironmentCaptureWgpuRecordReport,
    filter_report: EnvironmentCaptureFilterWgpuRecordReport,
    probe_publication: Option<EnvironmentCaptureProbePublication>,
}

/// Multi-frame source persistence continuation retaining the original capture target.
pub(in crate::graphics) struct EnvironmentCapturePersistenceSubmission {
    source: EnvironmentCaptureSourceSubmission,
    readback: SourceCubemapWgpuPendingReadback,
    batch_submission: Option<SubmissionTicket>,
    submitted_batch_count: u32,
}

pub(in crate::graphics) enum EnvironmentCaptureSubmission {
    Capturing(EnvironmentCaptureSourceSubmission),
    Persisting(EnvironmentCapturePersistenceSubmission),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) enum EnvironmentCaptureSourceSubmissionStatus {
    Pending,
    Completed,
    Failed {
        resource_upload: SubmissionStatus,
        capture: SubmissionStatus,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::graphics) enum EnvironmentCapturePersistenceSubmissionStatus {
    Pending,
    ReadyForNextBatch,
    Completed,
    Failed { submission: SubmissionStatus },
}

pub(in crate::graphics) struct EnvironmentCaptureResidentOutput {
    identity: RenderEnvironmentCaptureOutputIdentity,
    gpu_output: EnvironmentCaptureGpuOutput,
    record_report: EnvironmentCaptureWgpuRecordReport,
    filter_report: EnvironmentCaptureFilterWgpuRecordReport,
    probe_publication: Option<EnvironmentCaptureProbePublication>,
}

impl EnvironmentCaptureSourceSubmission {
    pub(in crate::graphics) fn new(
        handle: RenderEnvironmentCaptureHandle,
        request: RenderEnvironmentCaptureRequest,
        target: EnvironmentCaptureGpuTarget,
        resource_upload_submission: SubmissionTicket,
        capture_submission: SubmissionTicket,
        record_report: EnvironmentCaptureWgpuRecordReport,
        filter_report: EnvironmentCaptureFilterWgpuRecordReport,
        probe_publication: Option<EnvironmentCaptureProbePublication>,
    ) -> Self {
        Self {
            handle,
            request,
            target,
            resource_upload_submission,
            capture_submission,
            record_report,
            filter_report,
            probe_publication,
        }
    }

    pub(in crate::graphics) const fn handle(&self) -> RenderEnvironmentCaptureHandle {
        self.handle
    }

    pub(in crate::graphics) fn request(&self) -> &RenderEnvironmentCaptureRequest {
        &self.request
    }

    pub(in crate::graphics) fn target(&self) -> &EnvironmentCaptureGpuTarget {
        &self.target
    }

    pub(in crate::graphics) fn target_plan(&self) -> EnvironmentCaptureGpuTargetPlan {
        self.target.plan()
    }

    pub(in crate::graphics) const fn resource_upload_submission(&self) -> SubmissionTicket {
        self.resource_upload_submission
    }

    pub(in crate::graphics) const fn capture_submission(&self) -> SubmissionTicket {
        self.capture_submission
    }

    pub(in crate::graphics) const fn record_report(&self) -> EnvironmentCaptureWgpuRecordReport {
        self.record_report
    }

    pub(in crate::graphics) const fn filter_report(
        &self,
    ) -> EnvironmentCaptureFilterWgpuRecordReport {
        self.filter_report
    }

    pub(in crate::graphics) const fn probe_publication(
        &self,
    ) -> Option<EnvironmentCaptureProbePublication> {
        self.probe_publication
    }

    pub(in crate::graphics) fn into_resident_output(self) -> EnvironmentCaptureResidentOutput {
        EnvironmentCaptureResidentOutput {
            identity: RenderEnvironmentCaptureOutputIdentity::from_request(&self.request),
            gpu_output: self.target.into_filtered_output(),
            record_report: self.record_report,
            filter_report: self.filter_report,
            probe_publication: self.probe_publication,
        }
    }
}

impl EnvironmentCapturePersistenceSubmission {
    pub(in crate::graphics) fn new(
        source: EnvironmentCaptureSourceSubmission,
        readback: SourceCubemapWgpuPendingReadback,
    ) -> Self {
        Self {
            source,
            readback,
            batch_submission: None,
            submitted_batch_count: 0,
        }
    }

    pub(in crate::graphics) const fn handle(&self) -> RenderEnvironmentCaptureHandle {
        self.source.handle()
    }

    pub(in crate::graphics) fn request(&self) -> &RenderEnvironmentCaptureRequest {
        self.source.request()
    }

    pub(in crate::graphics) fn source(&self) -> &EnvironmentCaptureSourceSubmission {
        &self.source
    }

    pub(in crate::graphics) fn readback(&self) -> &SourceCubemapWgpuPendingReadback {
        &self.readback
    }

    pub(in crate::graphics) const fn batch_submission(&self) -> Option<SubmissionTicket> {
        self.batch_submission
    }

    pub(in crate::graphics) const fn submitted_batch_count(&self) -> u32 {
        self.submitted_batch_count
    }

    pub(in crate::graphics) fn commit_batch_submission(
        &mut self,
        submission: Option<SubmissionTicket>,
    ) {
        self.batch_submission = submission;
        self.submitted_batch_count = self.submitted_batch_count.saturating_add(1);
    }

    pub(in crate::graphics) const fn probe_publication(
        &self,
    ) -> Option<EnvironmentCaptureProbePublication> {
        self.source.probe_publication()
    }

    pub(in crate::graphics) fn into_parts(
        self,
    ) -> (
        EnvironmentCaptureSourceSubmission,
        Result<SourceCubemapWgpuReadback, crate::graphics::types::GraphicsError>,
    ) {
        (self.source, self.readback.finish())
    }
}

impl EnvironmentCaptureSubmission {
    pub(in crate::graphics) const fn handle(&self) -> RenderEnvironmentCaptureHandle {
        match self {
            Self::Capturing(submission) => submission.handle(),
            Self::Persisting(submission) => submission.handle(),
        }
    }

    pub(in crate::graphics) const fn probe_publication(
        &self,
    ) -> Option<EnvironmentCaptureProbePublication> {
        match self {
            Self::Capturing(submission) => submission.probe_publication(),
            Self::Persisting(submission) => submission.probe_publication(),
        }
    }
}

impl EnvironmentCaptureSourceSubmissionStatus {
    pub(in crate::graphics) const fn from_statuses(
        resource_upload: SubmissionStatus,
        capture: SubmissionStatus,
    ) -> Self {
        if matches!(resource_upload, SubmissionStatus::Completed)
            && matches!(capture, SubmissionStatus::Completed)
        {
            return Self::Completed;
        }
        if matches!(
            resource_upload,
            SubmissionStatus::Failed | SubmissionStatus::Cancelled | SubmissionStatus::DeviceLost
        ) || matches!(
            capture,
            SubmissionStatus::Failed | SubmissionStatus::Cancelled | SubmissionStatus::DeviceLost
        ) {
            return Self::Failed {
                resource_upload,
                capture,
            };
        }
        Self::Pending
    }

    pub(in crate::graphics) fn failure_diagnostic(self) -> Option<String> {
        match self {
            Self::Failed {
                resource_upload,
                capture,
            } => Some(format!(
                "environment capture GPU transaction failed: upload={resource_upload:?}, capture={capture:?}"
            )),
            Self::Pending | Self::Completed => None,
        }
    }
}

impl EnvironmentCaptureResidentOutput {
    pub(in crate::graphics) fn identity(&self) -> &RenderEnvironmentCaptureOutputIdentity {
        &self.identity
    }

    pub(in crate::graphics) fn gpu_bytes(&self) -> u64 {
        self.gpu_output.gpu_bytes()
    }

    pub(in crate::graphics) fn gpu_output(&self) -> &EnvironmentCaptureGpuOutput {
        &self.gpu_output
    }

    pub(in crate::graphics) const fn record_report(&self) -> EnvironmentCaptureWgpuRecordReport {
        self.record_report
    }

    pub(in crate::graphics) const fn filter_report(
        &self,
    ) -> EnvironmentCaptureFilterWgpuRecordReport {
        self.filter_report
    }

    pub(in crate::graphics) const fn probe_publication(
        &self,
    ) -> Option<EnvironmentCaptureProbePublication> {
        self.probe_publication
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("environment_capture_source_submission.rs");

    #[test]
    fn source_submission_retains_target_and_both_backend_tickets() {
        for field in [
            "target: EnvironmentCaptureGpuTarget",
            "resource_upload_submission: SubmissionTicket",
            "capture_submission: SubmissionTicket",
            "record_report: EnvironmentCaptureWgpuRecordReport",
            "filter_report: EnvironmentCaptureFilterWgpuRecordReport",
            "probe_publication: Option<EnvironmentCaptureProbePublication>",
        ] {
            assert!(
                SOURCE.contains(field),
                "missing source owner field: {field}"
            );
        }
    }

    #[test]
    fn submission_status_requires_both_backend_tickets_to_complete() {
        use zr_rhi::SubmissionStatus::{Completed, DeviceLost, Submitted};

        assert_eq!(
            EnvironmentCaptureSourceSubmissionStatus::from_statuses(Completed, Completed),
            EnvironmentCaptureSourceSubmissionStatus::Completed
        );
        assert_eq!(
            EnvironmentCaptureSourceSubmissionStatus::from_statuses(Completed, Submitted),
            EnvironmentCaptureSourceSubmissionStatus::Pending
        );
        assert!(matches!(
            EnvironmentCaptureSourceSubmissionStatus::from_statuses(Completed, DeviceLost),
            EnvironmentCaptureSourceSubmissionStatus::Failed { .. }
        ));
    }
}
