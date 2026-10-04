use crate::core::framework::render::RenderEnvironmentCaptureReport;

use super::EnvironmentCaptureScheduler;

impl EnvironmentCaptureScheduler {
    pub(in crate::graphics::runtime::render_framework) fn report(
        &self,
    ) -> RenderEnvironmentCaptureReport {
        let telemetry = self.telemetry();
        let pending = self.pending.front();
        let active = self.active.as_ref();
        let ready_payload = self.ready_source_payload.as_ref();

        RenderEnvironmentCaptureReport {
            observation_epoch: self.observation_epoch,
            pending_count: telemetry.pending_capture_count as u32,
            active_count: telemetry.active_capture_count as u32,
            terminal_status_count: telemetry.terminal_status_count as u32,
            accepted_request_count: telemetry.accepted_request_count,
            duplicate_request_count: telemetry.duplicate_request_count,
            capacity_rejection_count: telemetry.capacity_rejection_count,
            stale_generation_rejection_count: telemetry.stale_generation_rejection_count,
            superseded_capture_count: telemetry.superseded_capture_count,
            cancellation_request_count: telemetry.cancellation_request_count,
            succeeded_capture_count: telemetry.succeeded_capture_count,
            failed_capture_count: telemetry.failed_capture_count,
            terminal_status_eviction_count: telemetry.terminal_status_eviction_count,
            source_payload_backpressure_count: telemetry.source_payload_backpressure_count,
            source_payload_take_count: telemetry.source_payload_take_count,
            source_payload_publish_count: telemetry.source_payload_publish_count,
            ready_source_payload_bytes: ready_payload.map_or(0, |ready| ready.bytes),
            peak_ready_source_payload_bytes: telemetry.peak_ready_source_payload_bytes,
            cumulative_source_payload_bytes: telemetry.cumulative_source_payload_bytes,
            pending_handle: pending.map(|job| job.handle),
            pending_bake_key: pending.map(|job| job.bake_key),
            pending_output_generation: pending.map(|job| job.request.output_generation()),
            active_handle: active.map(|capture| capture.handle),
            active_phase: active.map(|capture| capture.phase),
            active_completed_work_items: active.map_or(0, |capture| capture.completed_work_items),
            active_bake_key: active.map(|capture| capture.bake_key),
            active_output_generation: active.map(|capture| capture.request.output_generation()),
            ready_source_payload_handle: ready_payload.map(|ready| ready.handle()),
            ready_source_payload_bake_key: ready_payload.map(|ready| ready.bake_key),
        }
    }
}
