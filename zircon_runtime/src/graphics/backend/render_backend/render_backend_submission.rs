use std::time::Instant;

use crate::core::framework::render::{
    RenderFrameSubmissionBoundaryReason, RenderFrameSubmissionMetrics,
    RenderFrameSubmissionProducer, RenderFrameSubmissionTransaction,
};
use crate::graphics::types::GraphicsError;
use crate::rhi::{
    RenderDevice, RenderQueueClass, SubmissionPollReceipt, SubmissionStatus, SubmissionTicket,
};
use zr_rhi_wgpu::{
    WgpuBufferUploadBatch, WgpuResourceUploadBatch, WgpuSubmissionMetricsSnapshot,
    WgpuTextureUploadBatch,
};
use zr_rhi_wgpu::{
    WgpuNativeDiagnosticQueryFrame, WgpuNativeDiagnosticReadbackFrame, WgpuNativeSurfaceFrameTarget,
};

use super::render_backend::RenderBackend;
use super::render_backend_diagnostics::PRODUCT_DIAGNOSTIC_CAPTURE_TIMEOUT;

impl RenderBackend {
    /// Records a pre-scene producer after it has been accepted by the device.
    ///
    /// Ledger validation normally cannot fail because both the ticket and the
    /// transaction originate from this backend. If it does, settle the just
    /// accepted ticket immediately so the frame failure path cannot orphan it.
    pub(crate) fn record_pre_scene_submission(
        &self,
        transaction: &mut RenderFrameSubmissionTransaction,
        producer: RenderFrameSubmissionProducer,
        ticket: SubmissionTicket,
    ) -> Result<(), GraphicsError> {
        match transaction.record_pre_scene_submission(producer, ticket) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.settle_rejected_pre_scene_submission(ticket, error)),
        }
    }

    /// Resource-qualified variant of [`Self::record_pre_scene_submission`].
    pub(crate) fn record_pre_scene_resource_submission(
        &self,
        transaction: &mut RenderFrameSubmissionTransaction,
        producer: RenderFrameSubmissionProducer,
        resource_id: crate::core::resource::ResourceId,
        ticket: SubmissionTicket,
    ) -> Result<(), GraphicsError> {
        match transaction.record_pre_scene_resource_submission(producer, resource_id, ticket) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.settle_rejected_pre_scene_submission(ticket, error)),
        }
    }

    /// Resource-qualified producer that must terminate at a typed physical ordering boundary.
    pub(crate) fn record_pre_scene_resource_submission_with_boundary(
        &self,
        transaction: &mut RenderFrameSubmissionTransaction,
        producer: RenderFrameSubmissionProducer,
        resource_id: crate::core::resource::ResourceId,
        boundary_reason: RenderFrameSubmissionBoundaryReason,
        ticket: SubmissionTicket,
    ) -> Result<(), GraphicsError> {
        match transaction.record_pre_scene_resource_submission_with_boundary(
            producer,
            resource_id,
            boundary_reason,
            ticket,
        ) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.settle_rejected_pre_scene_submission(ticket, error)),
        }
    }

    fn settle_rejected_pre_scene_submission(
        &self,
        ticket: SubmissionTicket,
        error: crate::core::framework::render::RenderFrameSubmissionReceiptError,
    ) -> GraphicsError {
        let source = GraphicsError::FrameSubmissionReceipt(error);
        match self.settle_abandoned_submissions(&[ticket]) {
            Ok(statuses) => match statuses.as_slice() {
                [status] => GraphicsError::FrameProducerRegistrationFailed {
                    ticket,
                    status: *status,
                    source: Box::new(source),
                },
                _ => GraphicsError::FrameSubmissionSettlement {
                    settlement: format!(
                        "single rejected producer returned {} settlement statuses",
                        statuses.len()
                    ),
                    source: Box::new(source),
                },
            },
            Err(settlement) => GraphicsError::FrameSubmissionSettlement {
                settlement: settlement.to_string(),
                source: Box::new(source),
            },
        }
    }

    /// Queues one per-frame buffer batch without opening another native submission boundary.
    pub(crate) fn enqueue_copy_buffer_upload_batch(
        &self,
        batch: WgpuBufferUploadBatch,
    ) -> Result<SubmissionTicket, GraphicsError> {
        self.render_device
            .enqueue_native_buffer_upload_batch(batch)
            .map_err(GraphicsError::from)
    }

    /// Queues one asset upload without opening another native submission boundary.
    pub(crate) fn enqueue_copy_texture_upload_batch(
        &self,
        batch: WgpuTextureUploadBatch,
    ) -> Result<SubmissionTicket, GraphicsError> {
        self.render_device
            .enqueue_native_texture_upload_batch(batch)
            .map_err(GraphicsError::from)
    }

    /// Queues one frame resource packet without splitting buffer and texture setup into two tickets.
    pub(crate) fn enqueue_copy_resource_upload_batch(
        &self,
        batch: WgpuResourceUploadBatch,
    ) -> Result<SubmissionTicket, GraphicsError> {
        self.render_device
            .enqueue_native_resource_upload_batch(batch)
            .map_err(GraphicsError::from)
    }

    /// Queues producer-owned setup work so the frame submission remains the flush authority.
    pub(crate) fn enqueue_graphics_command_buffers(
        &self,
        command_buffers: Vec<wgpu::CommandBuffer>,
    ) -> Result<SubmissionTicket, GraphicsError> {
        let mut recorder = self
            .render_device
            .begin_native_recording(RenderQueueClass::Graphics)?;
        recorder.extend_recorded_command_buffers(command_buffers);
        self.render_device
            .enqueue_native_recording_packet(recorder.finish()?)
            .map_err(GraphicsError::from)
    }

    /// Submits one graphics batch through the generation-qualified device owner.
    pub(crate) fn submit_graphics_command_buffers(
        &self,
        command_buffers: Vec<wgpu::CommandBuffer>,
    ) -> Result<SubmissionTicket, GraphicsError> {
        let mut recorder = self
            .render_device
            .begin_native_recording(RenderQueueClass::Graphics)?;
        recorder.extend_recorded_command_buffers(command_buffers);
        self.render_device
            .submit_native_recording_packet(recorder.finish()?)
            .map_err(GraphicsError::from)
    }

    /// Submits one scene packet whose diagnostic tail shares the scene submission ticket.
    pub(crate) fn submit_graphics_command_buffers_with_diagnostics(
        &self,
        command_buffers: Vec<wgpu::CommandBuffer>,
        diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
    ) -> Result<SubmissionTicket, GraphicsError> {
        self.submit_graphics_command_buffers_with_frame_diagnostics(
            command_buffers,
            diagnostic_frame,
            None,
        )
    }

    /// Submits one scene packet with copy and typed-query tails on the same ticket.
    pub(crate) fn submit_graphics_command_buffers_with_frame_diagnostics(
        &self,
        command_buffers: Vec<wgpu::CommandBuffer>,
        diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
        query_frame: Option<WgpuNativeDiagnosticQueryFrame>,
    ) -> Result<SubmissionTicket, GraphicsError> {
        self.submit_graphics_command_buffers_with_frame_diagnostics_and_surface(
            command_buffers,
            diagnostic_frame,
            query_frame,
            None,
        )
    }

    /// Submits one scene packet and retains an acquired surface frame on the same ticket.
    pub(crate) fn submit_graphics_command_buffers_with_frame_diagnostics_and_surface(
        &self,
        command_buffers: Vec<wgpu::CommandBuffer>,
        diagnostic_frame: Option<WgpuNativeDiagnosticReadbackFrame>,
        query_frame: Option<WgpuNativeDiagnosticQueryFrame>,
        surface_target: Option<&WgpuNativeSurfaceFrameTarget>,
    ) -> Result<SubmissionTicket, GraphicsError> {
        let mut recorder = self
            .render_device
            .begin_native_recording(RenderQueueClass::Graphics)?;
        recorder.extend_recorded_command_buffers(command_buffers);
        self.render_device
            .submit_native_recording_packet_with_frame_diagnostics_and_surface(
                recorder.finish()?,
                diagnostic_frame,
                query_frame,
                surface_target,
            )
            .map_err(GraphicsError::from)
    }

    /// Polls the one backend-owned submission timeline before frame resource reuse.
    pub(crate) fn poll_submission_completions(
        &self,
    ) -> Result<SubmissionPollReceipt, GraphicsError> {
        let receipt = self
            .render_device
            .poll_submissions()
            .map_err(GraphicsError::from)?;
        self.dispatch_product_diagnostic_deliveries();
        Ok(receipt)
    }

    /// Explicit bounded capture boundary. Normal rendering uses exactly one completion poll at
    /// frame begin; this loop continues only until the already-submitted diagnostic work drains.
    pub(crate) fn wait_for_product_diagnostic_deliveries(
        &self,
        mut observe_poll: impl FnMut(SubmissionPollReceipt) -> Result<(), GraphicsError>,
    ) -> Result<(), GraphicsError> {
        let started = Instant::now();
        loop {
            let poll_receipt = self.poll_submission_completions()?;
            observe_poll(poll_receipt)?;
            let metrics = self.product_diagnostic_readback_metrics();
            if metrics.active_request_count() == 0
                && metrics.in_flight_request_count() == 0
                && metrics.retained_delivery_count() == 0
            {
                return Ok(());
            }
            if started.elapsed() >= PRODUCT_DIAGNOSTIC_CAPTURE_TIMEOUT {
                return Err(GraphicsError::DiagnosticReadbackTimedOut {
                    timeout: PRODUCT_DIAGNOSTIC_CAPTURE_TIMEOUT,
                });
            }
            std::thread::yield_now();
        }
    }

    /// Closes one frame-owned metrics interval without flushing or polling the device timeline.
    pub(crate) fn frame_submission_metrics_since(
        &self,
        baseline: WgpuSubmissionMetricsSnapshot,
        admitted_logical_packet_count: u64,
    ) -> Option<RenderFrameSubmissionMetrics> {
        let delta = self.submission_metrics().delta_since(baseline)?;
        Some(RenderFrameSubmissionMetrics::new(
            admitted_logical_packet_count,
            delta.submitted_ticket_count(),
            delta.native_submission_count(),
            delta.buffer_upload_batch_count(),
            delta.texture_upload_batch_count(),
            delta.buffer_write_count(),
            delta.texture_write_count(),
            delta.upload_payload_bytes(),
        ))
    }

    pub(crate) fn submission_status(
        &self,
        ticket: SubmissionTicket,
    ) -> Result<SubmissionStatus, crate::rhi::RhiError> {
        self.render_device.submission_status(ticket)
    }

    /// Observes a ticket batch without polling or repeatedly locking submission state.
    pub(crate) fn append_submission_statuses(
        &self,
        tickets: &[SubmissionTicket],
        statuses: &mut Vec<Result<SubmissionStatus, crate::rhi::RhiError>>,
    ) {
        self.render_device
            .append_submission_statuses(tickets, statuses);
    }

    pub(crate) fn settle_abandoned_submissions(
        &self,
        tickets: &[SubmissionTicket],
    ) -> Result<Vec<SubmissionStatus>, GraphicsError> {
        self.render_device
            .settle_abandoned_native_submissions(tickets)
            .map_err(GraphicsError::from)
    }

    /// Returns monotonic submission facts; frame instrumentation derives intervals from snapshots.
    pub(crate) fn submission_metrics(&self) -> WgpuSubmissionMetricsSnapshot {
        self.render_device.submission_metrics()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn raw_backend_submission_routes_only_through_the_wgpu_render_device() {
        let source = include_str!("render_backend_submission.rs");

        assert!(source.contains("submit_native_recording_packet(recorder.finish()?)"));
        assert!(source.contains("enqueue_native_buffer_upload_batch(batch)"));
        assert!(source.contains("enqueue_native_texture_upload_batch(batch)"));
        assert!(source.contains("enqueue_native_recording_packet(recorder.finish()?)"));
        assert!(source.contains(".poll_submissions()"));
        assert!(source.contains("self.render_device.submission_metrics()"));
        assert!(source.contains(".append_submission_statuses(tickets, statuses)"));
        assert!(source.contains(".settle_abandoned_native_submissions(tickets)"));
        assert!(!source.contains("submission_coordinator"));
        assert!(!source.contains("queue.submit"));
    }

    #[test]
    fn frame_submission_metrics_are_derived_without_flushing_or_polling() {
        let source = include_str!("render_backend_submission.rs");
        let sampler = source
            .split("pub(crate) fn frame_submission_metrics_since")
            .nth(1)
            .and_then(|source| source.split("pub(crate) fn submission_status").next())
            .expect("frame submission metrics sampler");

        assert!(sampler.contains("self.submission_metrics().delta_since(baseline)"));
        assert!(sampler.contains("RenderFrameSubmissionMetrics::new("));
        assert!(!sampler.contains("flush_submissions"));
        assert!(!sampler.contains("poll_submissions"));
        assert!(!sampler.contains("queue.submit"));
    }

    #[test]
    fn explicit_diagnostic_drain_is_bounded_and_uses_the_single_completion_pump() {
        let source = include_str!("render_backend_submission.rs")
            .split("\n#[cfg(test)]")
            .next()
            .unwrap_or_default();

        assert!(source.contains("PRODUCT_DIAGNOSTIC_CAPTURE_TIMEOUT"));
        assert!(source.contains("let poll_receipt = self.poll_submission_completions()?"));
        assert!(source.contains("observe_poll(poll_receipt)?"));
        assert!(source.contains("metrics.in_flight_request_count() == 0"));
        assert!(source.contains("metrics.retained_delivery_count() == 0"));
        assert!(source.contains("DiagnosticReadbackTimedOut"));
        assert!(!source.contains("wait_indefinitely"));
        assert!(!source.contains("self.device.poll("));
    }

    #[test]
    fn surface_submission_uses_the_same_device_owned_scene_packet() {
        let source = include_str!("render_backend_submission.rs");

        assert!(
            source.contains(".submit_native_recording_packet_with_frame_diagnostics_and_surface(")
        );
        assert!(source.contains("surface_target,"));
        assert!(!source.contains("queue.submit"));
    }

    #[test]
    fn rejected_frame_producer_tickets_are_settled_at_the_backend_boundary() {
        let source = include_str!("render_backend_submission.rs");
        let record = source
            .find("transaction.record_pre_scene_submission(producer, ticket)")
            .expect("backend helper must delegate producer recording");
        let settle = source
            .find("self.settle_rejected_pre_scene_submission(ticket, error)")
            .expect("backend helper must settle a rejected ticket");
        let settle_impl = source
            .find("self.settle_abandoned_submissions(&[ticket])")
            .expect("rejected ticket settlement must use the backend owner");

        assert!(record < settle);
        assert!(settle < settle_impl);
        assert!(source.contains("record_pre_scene_resource_submission("));
        assert!(source.contains("FrameProducerRegistrationFailed"));
    }
}
