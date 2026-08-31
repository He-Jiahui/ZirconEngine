use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::{
    BindGroupLayoutDesc, BufferDesc, BufferUpload, BufferUploadBatch, DeviceGeneration, DeviceId,
    DiagnosticQueryPlan, GpuMemorySnapshot, PipelineDesc, PipelineLayoutDesc, RenderBackendCaps,
    RenderDebugInstrumentationStatus, RenderOperation, RenderOperationSupport, RenderQueueClass,
    RenderSurfaceDescriptor, RhiSubmissionPacket, SamplerDesc, ShaderModuleDesc,
    SubmissionPollReceipt, SubmissionStatus, SubmissionTicket, SurfaceAcquireOutcome,
    SurfaceFrameLease, SurfacePresentReceipt, SurfaceSession, SurfaceSessionCreateOutcome,
    SwapchainDesc, TextureCopyRegion, TextureDesc, TextureUpload, TextureUploadBatch,
    TextureViewDesc,
};

use super::{
    BindGroupDesc, BindGroupHandle, BindGroupLayoutHandle, BufferHandle, CommandList,
    CommandListCommand, PipelineHandle, PipelineLayoutHandle, RhiError, SamplerHandle,
    ShaderModuleHandle, TextureHandle, TextureViewHandle, TransientAllocatorStats,
};

pub trait RenderDevice: Send + Sync {
    fn caps(&self) -> &RenderBackendCaps;

    /// Immutable identity for every resource and submission issued by this
    /// device owner. Submission packets must carry the same pair.
    fn device_id(&self) -> DeviceId;

    fn generation(&self) -> DeviceGeneration;

    fn backend_name(&self) -> &str {
        &self.caps().backend_name
    }

    fn require_operation(
        &self,
        operation: RenderOperation,
    ) -> Result<RenderOperationSupport, RhiError> {
        self.caps()
            .require_operation(operation)
            .map_err(RhiError::from)
    }

    /// Checks that every executable command in a recorded list is represented
    /// by an admitted neutral operation. Concrete submitters must call this
    /// before backend-specific validation or execution.
    fn require_recorded_command_operations(
        &self,
        commands: &[CommandListCommand],
    ) -> Result<(), RhiError> {
        for command in commands {
            if let Some(operation) = command.required_operation() {
                self.require_operation(operation)?;
            }
        }
        Ok(())
    }

    fn debug_instrumentation_status(&self) -> RenderDebugInstrumentationStatus {
        RenderDebugInstrumentationStatus::from_caps(self.caps())
    }

    fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle, RhiError>;
    fn buffer_desc(&self, handle: BufferHandle) -> Result<BufferDesc, RhiError>;
    fn destroy_buffer(&self, handle: BufferHandle) -> Result<(), RhiError>;
    fn create_texture(&self, desc: &TextureDesc) -> Result<TextureHandle, RhiError>;
    fn texture_desc(&self, handle: TextureHandle) -> Result<TextureDesc, RhiError>;
    fn destroy_texture(&self, handle: TextureHandle) -> Result<(), RhiError>;
    fn create_texture_view(&self, desc: &TextureViewDesc) -> Result<TextureViewHandle, RhiError>;
    fn texture_view_desc(&self, handle: TextureViewHandle) -> Result<TextureViewDesc, RhiError>;
    fn destroy_texture_view(&self, handle: TextureViewHandle) -> Result<(), RhiError>;
    fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerHandle, RhiError>;
    fn sampler_desc(&self, handle: SamplerHandle) -> Result<SamplerDesc, RhiError>;
    fn destroy_sampler(&self, handle: SamplerHandle) -> Result<(), RhiError>;
    fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDesc,
    ) -> Result<BindGroupLayoutHandle, RhiError>;
    fn bind_group_layout_desc(
        &self,
        handle: BindGroupLayoutHandle,
    ) -> Result<BindGroupLayoutDesc, RhiError>;
    fn destroy_bind_group_layout(&self, handle: BindGroupLayoutHandle) -> Result<(), RhiError>;
    fn create_bind_group(&self, desc: &BindGroupDesc) -> Result<BindGroupHandle, RhiError>;
    fn bind_group_desc(&self, handle: BindGroupHandle) -> Result<BindGroupDesc, RhiError>;
    fn destroy_bind_group(&self, handle: BindGroupHandle) -> Result<(), RhiError>;
    fn create_shader_module(&self, desc: &ShaderModuleDesc)
        -> Result<ShaderModuleHandle, RhiError>;
    fn shader_module_desc(&self, handle: ShaderModuleHandle) -> Result<ShaderModuleDesc, RhiError>;
    fn destroy_shader_module(&self, handle: ShaderModuleHandle) -> Result<(), RhiError>;
    fn create_pipeline_layout(
        &self,
        desc: &PipelineLayoutDesc,
    ) -> Result<PipelineLayoutHandle, RhiError>;
    fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<PipelineLayoutDesc, RhiError>;
    fn destroy_pipeline_layout(&self, handle: PipelineLayoutHandle) -> Result<(), RhiError>;
    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError>;
    fn pipeline_desc(&self, handle: PipelineHandle) -> Result<PipelineDesc, RhiError>;
    fn destroy_pipeline(&self, handle: PipelineHandle) -> Result<(), RhiError>;
    /// Creates one device-generation-local native surface session. A zero
    /// extent returns a typed non-renderable session rather than a clamped
    /// offscreen target.
    fn create_surface_session(
        &self,
        descriptor: &RenderSurfaceDescriptor,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError>;
    /// Invalidates the old session and every outstanding lease before returning
    /// a replacement session with its negotiated swapchain receipt.
    fn reconfigure_surface_session(
        &self,
        session: SurfaceSession,
        swapchain: &SwapchainDesc,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError>;
    /// Acquires at most one short-lived target lease from a live surface session.
    fn acquire_surface_frame(
        &self,
        session: SurfaceSession,
    ) -> Result<SurfaceAcquireOutcome, RhiError>;
    /// Consumes an acquired frame after a same-device submission which
    /// referenced its target has reached Submitted or Completed. This
    /// operation must not issue a queue submit.
    fn present_surface_frame(
        &self,
        frame: SurfaceFrameLease,
        submission: SubmissionTicket,
    ) -> Result<SurfacePresentReceipt, RhiError>;
    /// Consumes an acquired frame without presentation after graph cull,
    /// cancellation, device loss, or a pre-submit error.
    fn discard_surface_frame(&self, frame: SurfaceFrameLease) -> Result<(), RhiError>;
    /// Tears down a session and terminalizes any outstanding lease identities.
    fn destroy_surface_session(&self, session: SurfaceSession) -> Result<(), RhiError>;
    fn create_command_list(
        &self,
        queue_class: RenderQueueClass,
        label: &str,
    ) -> Result<Box<dyn CommandList>, RhiError>;

    /// Creates one immutable, device-qualified packet. Every command list in
    /// the packet must target the same logical queue class.
    fn create_submission_packet(
        &self,
        queue_class: RenderQueueClass,
        command_lists: Vec<Box<dyn CommandList>>,
    ) -> Result<RhiSubmissionPacket, RhiError> {
        RhiSubmissionPacket::new(
            self.device_id(),
            self.generation(),
            queue_class,
            command_lists,
        )
    }

    /// Creates a packet whose pass-local diagnostics are backed by one bounded
    /// graph-frame query plan and one eventual submission ticket.
    fn create_submission_packet_with_diagnostic_query_plan(
        &self,
        queue_class: RenderQueueClass,
        command_lists: Vec<Box<dyn CommandList>>,
        diagnostic_query_plan: DiagnosticQueryPlan,
    ) -> Result<RhiSubmissionPacket, RhiError> {
        RhiSubmissionPacket::new_with_diagnostic_query_plan(
            self.device_id(),
            self.generation(),
            queue_class,
            command_lists,
            diagnostic_query_plan,
        )
    }

    /// Accepts one immutable packet into the backend's unique submission
    /// service. All command lists in this packet receive one logical ticket.
    fn enqueue_submission_packet(
        &self,
        packet: RhiSubmissionPacket,
    ) -> Result<SubmissionTicket, RhiError>;

    /// Accepts one command list through the same immutable packet path used
    /// by multi-list product frames.
    fn enqueue_command_list(
        &self,
        command_list: Box<dyn CommandList>,
    ) -> Result<SubmissionTicket, RhiError> {
        let packet =
            self.create_submission_packet(command_list.queue_class(), vec![command_list])?;
        self.enqueue_submission_packet(packet)
    }

    /// Submits all currently accepted packets in service order.
    fn flush_submissions(&self) -> Result<usize, RhiError>;

    /// One-shot convenience entry point that still traverses the backend's
    /// unique submission service rather than directly reaching a native queue.
    fn submit(&self, command_list: Box<dyn CommandList>) -> Result<SubmissionTicket, RhiError> {
        let ticket = self.enqueue_command_list(command_list)?;
        self.flush_submissions()?;
        self.poll_submissions()?;
        Ok(ticket)
    }

    /// One-shot packet convenience entry point. It preserves a shared ticket
    /// for all command lists rather than synthesizing one ticket per list.
    fn submit_packet(&self, packet: RhiSubmissionPacket) -> Result<SubmissionTicket, RhiError> {
        let ticket = self.enqueue_submission_packet(packet)?;
        self.flush_submissions()?;
        self.poll_submissions()?;
        Ok(ticket)
    }

    /// Returns a ticket's last service-observed lifecycle state without
    /// performing a native-device poll.
    fn submission_status(&self, ticket: SubmissionTicket) -> Result<SubmissionStatus, RhiError>;

    /// Appends exactly one status result per input ticket in input order.
    /// Backends should override this to hold their submission-state lock once.
    fn append_submission_statuses(
        &self,
        tickets: &[SubmissionTicket],
        statuses: &mut Vec<Result<SubmissionStatus, RhiError>>,
    ) {
        statuses.reserve(tickets.len());
        statuses.extend(
            tickets
                .iter()
                .copied()
                .map(|ticket| self.submission_status(ticket)),
        );
    }

    /// Cancels only work still accepted by the submission service. Native work
    /// already handed to a queue cannot be treated as cancelled.
    fn cancel_submission(&self, ticket: SubmissionTicket) -> Result<SubmissionStatus, RhiError>;

    /// Performs one nonblocking backend completion pump.
    /// Performs one nonblocking native completion pump and returns a
    /// generation-qualified, monotonically increasing observation receipt.
    fn poll_submissions(&self) -> Result<SubmissionPollReceipt, RhiError>;

    /// Waits only for one ticket and always applies the caller's timeout.
    fn wait_for_submission(
        &self,
        ticket: SubmissionTicket,
        timeout: Duration,
    ) -> Result<SubmissionStatus, RhiError> {
        let started = Instant::now();
        loop {
            self.poll_submissions()?;
            let status = self.submission_status(ticket)?;
            if status.is_terminal() {
                return Ok(status);
            }
            if started.elapsed() >= timeout {
                return Err(RhiError::SubmissionWaitTimedOut { ticket, timeout });
            }
            std::thread::yield_now();
        }
    }

    fn transient_allocator_stats(&self) -> TransientAllocatorStats;
    /// Returns physical resource retention and CPU upload staging separately.
    fn memory_snapshot(&self) -> GpuMemorySnapshot;
    /// Queues a `COPY_DST` buffer upload and returns the receipt that
    /// qualifies its lifetime, completion, cancellation, and diagnostics.
    fn write_buffer_batch(&self, batch: BufferUploadBatch) -> Result<SubmissionTicket, RhiError>;
    fn write_buffer(
        &self,
        handle: BufferHandle,
        offset: u64,
        data: &[u8],
    ) -> Result<SubmissionTicket, RhiError> {
        let payload: Arc<[u8]> = Arc::from(data);
        let upload = BufferUpload::from_payload(handle, offset, payload);
        self.write_buffer_batch(BufferUploadBatch::from(upload))
    }
    /// Queues one logical CPU-to-GPU texture batch under one submission ticket.
    /// Each write is restricted to one layer or depth slice; depth/stencil
    /// updates require a future aspect-qualified contract.
    fn write_texture_batch(&self, batch: TextureUploadBatch) -> Result<SubmissionTicket, RhiError>;
    fn write_texture(
        &self,
        handle: TextureHandle,
        region: TextureCopyRegion,
        bytes_per_row: u64,
        data: &[u8],
    ) -> Result<SubmissionTicket, RhiError> {
        let payload: Arc<[u8]> = Arc::from(data);
        let upload = TextureUpload::from_payload(handle, region, bytes_per_row, payload);
        self.write_texture_batch(TextureUploadBatch::from(upload))
    }
    fn read_buffer(
        &self,
        handle: BufferHandle,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, RhiError>;
    fn read_texture(&self, handle: TextureHandle) -> Result<Vec<u8>, RhiError>;
}
