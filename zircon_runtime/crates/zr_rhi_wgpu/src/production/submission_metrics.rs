use std::time::Duration;

use zr_rhi::{DeviceGeneration, DeviceId};

/// Monotonic, device-generation-local facts emitted by the native submission owner.
///
/// Consumers sample two snapshots and compute their own frame interval. The service never resets
/// these counters, so concurrent diagnostics cannot race to consume a shared measurement window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WgpuSubmissionMetricsSnapshot {
    device_id: Option<DeviceId>,
    generation: Option<DeviceGeneration>,
    buffer_upload_batch_count: u64,
    texture_upload_batch_count: u64,
    buffer_write_count: u64,
    texture_write_count: u64,
    upload_payload_bytes: u64,
    native_submission_count: u64,
    submitted_ticket_count: u64,
    completed_ticket_count: u64,
    completed_latency_total_ns: u64,
    completed_latency_max_ns: u64,
    pending_upload_bytes: u64,
    peak_pending_upload_bytes: u64,
    buffer_upload_rejection_count: u64,
    texture_upload_rejection_count: u64,
}

/// One sampling interval derived from two snapshots of the same device-generation owner.
///
/// Counter fields are interval values. Retained bytes and high-water marks are observations at
/// the later sample, because neither can be derived correctly by subtracting cumulative gauges.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WgpuSubmissionMetricsDelta {
    buffer_upload_batch_count: u64,
    texture_upload_batch_count: u64,
    buffer_write_count: u64,
    texture_write_count: u64,
    upload_payload_bytes: u64,
    native_submission_count: u64,
    submitted_ticket_count: u64,
    completed_ticket_count: u64,
    completed_latency_total_ns: u64,
    pending_upload_bytes: u64,
    lifetime_peak_pending_upload_bytes: u64,
    lifetime_max_completion_latency_ns: u64,
    buffer_upload_rejection_count: u64,
    texture_upload_rejection_count: u64,
}

impl WgpuSubmissionMetricsSnapshot {
    pub const fn device_id(self) -> Option<DeviceId> {
        self.device_id
    }

    pub const fn generation(self) -> Option<DeviceGeneration> {
        self.generation
    }

    pub const fn buffer_upload_batch_count(self) -> u64 {
        self.buffer_upload_batch_count
    }

    pub const fn texture_upload_batch_count(self) -> u64 {
        self.texture_upload_batch_count
    }

    pub const fn buffer_write_count(self) -> u64 {
        self.buffer_write_count
    }

    pub const fn texture_write_count(self) -> u64 {
        self.texture_write_count
    }

    pub const fn upload_payload_bytes(self) -> u64 {
        self.upload_payload_bytes
    }

    pub const fn native_submission_count(self) -> u64 {
        self.native_submission_count
    }

    pub const fn submitted_ticket_count(self) -> u64 {
        self.submitted_ticket_count
    }

    pub const fn completed_ticket_count(self) -> u64 {
        self.completed_ticket_count
    }

    pub const fn completed_latency_total_ns(self) -> u64 {
        self.completed_latency_total_ns
    }

    pub const fn completed_latency_max_ns(self) -> u64 {
        self.completed_latency_max_ns
    }

    /// Upload bytes currently retained by accepted or flushing submissions.
    pub const fn pending_upload_bytes(self) -> u64 {
        self.pending_upload_bytes
    }

    pub const fn peak_pending_upload_bytes(self) -> u64 {
        self.peak_pending_upload_bytes
    }

    pub const fn buffer_upload_rejection_count(self) -> u64 {
        self.buffer_upload_rejection_count
    }

    pub const fn texture_upload_rejection_count(self) -> u64 {
        self.texture_upload_rejection_count
    }

    /// Produces one interval when both snapshots belong to the same monotonic metrics owner.
    ///
    /// A counter regression means the device-generation owner was replaced or reset; callers must
    /// discard that sample and establish a new baseline instead of reporting a fabricated delta.
    pub fn delta_since(self, baseline: Self) -> Option<WgpuSubmissionMetricsDelta> {
        (self.has_same_owner_as(baseline) && !self.counters_regressed_from(baseline)).then(|| {
            WgpuSubmissionMetricsDelta {
                buffer_upload_batch_count: self
                    .buffer_upload_batch_count
                    .saturating_sub(baseline.buffer_upload_batch_count),
                texture_upload_batch_count: self
                    .texture_upload_batch_count
                    .saturating_sub(baseline.texture_upload_batch_count),
                buffer_write_count: self
                    .buffer_write_count
                    .saturating_sub(baseline.buffer_write_count),
                texture_write_count: self
                    .texture_write_count
                    .saturating_sub(baseline.texture_write_count),
                upload_payload_bytes: self
                    .upload_payload_bytes
                    .saturating_sub(baseline.upload_payload_bytes),
                native_submission_count: self
                    .native_submission_count
                    .saturating_sub(baseline.native_submission_count),
                submitted_ticket_count: self
                    .submitted_ticket_count
                    .saturating_sub(baseline.submitted_ticket_count),
                completed_ticket_count: self
                    .completed_ticket_count
                    .saturating_sub(baseline.completed_ticket_count),
                completed_latency_total_ns: self
                    .completed_latency_total_ns
                    .saturating_sub(baseline.completed_latency_total_ns),
                pending_upload_bytes: self.pending_upload_bytes,
                lifetime_peak_pending_upload_bytes: self.peak_pending_upload_bytes,
                lifetime_max_completion_latency_ns: self.completed_latency_max_ns,
                buffer_upload_rejection_count: self
                    .buffer_upload_rejection_count
                    .saturating_sub(baseline.buffer_upload_rejection_count),
                texture_upload_rejection_count: self
                    .texture_upload_rejection_count
                    .saturating_sub(baseline.texture_upload_rejection_count),
            }
        })
    }

    fn has_same_owner_as(self, baseline: Self) -> bool {
        self.device_id.is_some()
            && self.device_id == baseline.device_id
            && self.generation == baseline.generation
    }

    fn counters_regressed_from(self, baseline: Self) -> bool {
        self.buffer_upload_batch_count < baseline.buffer_upload_batch_count
            || self.texture_upload_batch_count < baseline.texture_upload_batch_count
            || self.buffer_write_count < baseline.buffer_write_count
            || self.texture_write_count < baseline.texture_write_count
            || self.upload_payload_bytes < baseline.upload_payload_bytes
            || self.native_submission_count < baseline.native_submission_count
            || self.submitted_ticket_count < baseline.submitted_ticket_count
            || self.completed_ticket_count < baseline.completed_ticket_count
            || self.completed_latency_total_ns < baseline.completed_latency_total_ns
            || self.completed_latency_max_ns < baseline.completed_latency_max_ns
            || self.peak_pending_upload_bytes < baseline.peak_pending_upload_bytes
            || self.buffer_upload_rejection_count < baseline.buffer_upload_rejection_count
            || self.texture_upload_rejection_count < baseline.texture_upload_rejection_count
    }
}

impl WgpuSubmissionMetricsDelta {
    pub const fn buffer_upload_batch_count(self) -> u64 {
        self.buffer_upload_batch_count
    }

    pub const fn texture_upload_batch_count(self) -> u64 {
        self.texture_upload_batch_count
    }

    pub const fn buffer_write_count(self) -> u64 {
        self.buffer_write_count
    }

    pub const fn texture_write_count(self) -> u64 {
        self.texture_write_count
    }

    pub const fn upload_payload_bytes(self) -> u64 {
        self.upload_payload_bytes
    }

    pub const fn native_submission_count(self) -> u64 {
        self.native_submission_count
    }

    pub const fn submitted_ticket_count(self) -> u64 {
        self.submitted_ticket_count
    }

    pub const fn completed_ticket_count(self) -> u64 {
        self.completed_ticket_count
    }

    pub const fn completed_latency_total_ns(self) -> u64 {
        self.completed_latency_total_ns
    }

    pub const fn pending_upload_bytes(self) -> u64 {
        self.pending_upload_bytes
    }

    pub const fn lifetime_peak_pending_upload_bytes(self) -> u64 {
        self.lifetime_peak_pending_upload_bytes
    }

    pub const fn lifetime_max_completion_latency_ns(self) -> u64 {
        self.lifetime_max_completion_latency_ns
    }

    pub const fn buffer_upload_rejection_count(self) -> u64 {
        self.buffer_upload_rejection_count
    }

    pub const fn texture_upload_rejection_count(self) -> u64 {
        self.texture_upload_rejection_count
    }
}

#[derive(Default)]
pub(super) struct WgpuSubmissionMetrics {
    snapshot: WgpuSubmissionMetricsSnapshot,
}

impl WgpuSubmissionMetrics {
    pub(super) const fn snapshot(
        &self,
        device_id: DeviceId,
        generation: DeviceGeneration,
        pending_upload_bytes: u64,
    ) -> WgpuSubmissionMetricsSnapshot {
        WgpuSubmissionMetricsSnapshot {
            device_id: Some(device_id),
            generation: Some(generation),
            pending_upload_bytes,
            ..self.snapshot
        }
    }

    pub(super) fn record_resource_upload_admitted(
        &mut self,
        buffer_write_count: usize,
        texture_write_count: usize,
        payload_bytes: u64,
        pending_upload_bytes: u64,
    ) {
        if buffer_write_count > 0 {
            self.snapshot.buffer_upload_batch_count =
                self.snapshot.buffer_upload_batch_count.saturating_add(1);
            self.snapshot.buffer_write_count = self
                .snapshot
                .buffer_write_count
                .saturating_add(usize_as_u64(buffer_write_count));
        }
        if texture_write_count > 0 {
            self.snapshot.texture_upload_batch_count =
                self.snapshot.texture_upload_batch_count.saturating_add(1);
            self.snapshot.texture_write_count = self
                .snapshot
                .texture_write_count
                .saturating_add(usize_as_u64(texture_write_count));
        }
        self.snapshot.upload_payload_bytes = self
            .snapshot
            .upload_payload_bytes
            .saturating_add(payload_bytes);
        self.snapshot.peak_pending_upload_bytes = self
            .snapshot
            .peak_pending_upload_bytes
            .max(pending_upload_bytes);
    }

    pub(super) fn record_resource_upload_rejected(
        &mut self,
        has_buffer_uploads: bool,
        has_texture_uploads: bool,
    ) {
        if has_buffer_uploads {
            self.snapshot.buffer_upload_rejection_count = self
                .snapshot
                .buffer_upload_rejection_count
                .saturating_add(1);
        }
        if has_texture_uploads {
            self.snapshot.texture_upload_rejection_count = self
                .snapshot
                .texture_upload_rejection_count
                .saturating_add(1);
        }
    }

    pub(super) fn record_native_submission(&mut self, ticket_count: usize) {
        self.snapshot.native_submission_count =
            self.snapshot.native_submission_count.saturating_add(1);
        self.snapshot.submitted_ticket_count = self
            .snapshot
            .submitted_ticket_count
            .saturating_add(usize_as_u64(ticket_count));
    }

    pub(super) fn record_completion(&mut self, latency: Duration) {
        let latency_ns = duration_as_nanos(latency);
        self.snapshot.completed_ticket_count =
            self.snapshot.completed_ticket_count.saturating_add(1);
        self.snapshot.completed_latency_total_ns = self
            .snapshot
            .completed_latency_total_ns
            .saturating_add(latency_ns);
        self.snapshot.completed_latency_max_ns =
            self.snapshot.completed_latency_max_ns.max(latency_ns);
    }
}

fn usize_as_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

fn duration_as_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}
