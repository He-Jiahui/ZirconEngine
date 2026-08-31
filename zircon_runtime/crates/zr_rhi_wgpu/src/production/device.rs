use std::sync::{Arc, Mutex, MutexGuard};

use zr_rhi::{
    BindGroupDesc, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutHandle, BufferDesc,
    BufferHandle, BufferUploadBatch, BufferUsage, CommandList, DeviceAdmissionError,
    DeviceFaultGate, DeviceFaultKind, DiagnosticReadbackTerminal, GpuMemorySnapshot, PipelineDesc,
    PipelineHandle, PipelineLayoutDesc, PipelineLayoutHandle, RenderBackendCaps, RenderDevice,
    RenderDeviceProfile, RenderQueueClass, RenderSurfaceDescriptor, RhiError, RhiSubmissionPacket,
    SamplerDesc, SamplerHandle, ShaderModuleDesc, ShaderModuleHandle, SubmissionPollReceipt,
    SubmissionStatus, SubmissionTicket, SurfaceAcquireOutcome, SurfaceFrameLease,
    SurfacePresentReceipt, SurfaceSession, SurfaceSessionCreateOutcome, SwapchainDesc,
    TextureCopyRegion, TextureDesc, TextureHandle, TextureUploadBatch, TextureUsage,
    TextureViewDesc, TextureViewHandle, TransientAllocatorStats,
};

use crate::{
    texture_copy::{
        texture_upload_byte_len, texture_upload_layout, texture_write_out_of_range,
        validate_texture_copy_destination_aspect,
    },
    WgpuDeviceErrorSupervisor, WgpuUiSharedImageRegistry, WgpuUiSurfaceContext,
};

use super::diagnostics::WgpuDiagnosticReadbackService;
use super::fault_terminal::{diagnostic_terminal_status, submission_terminal_status};
use super::{
    encode_command_list, WgpuBufferUpload, WgpuBufferUploadBatch, WgpuCommandList,
    WgpuResourceRegistry, WgpuSubmissionService, WgpuSurfaceService, WgpuTextureUpload,
    WgpuTextureUploadBatch,
};

pub(super) mod capabilities;
mod context;
mod diagnostics;
mod native_recording;
mod native_submission;
mod native_surface_recording;
mod surface_lifecycle;

use capabilities::production_caps;
pub use context::WgpuRenderDeviceContext;
use context::{
    validate_context_adapter, validate_context_device_limits, validate_context_queue_topology,
    validate_context_requested_features,
};
pub use native_recording::{
    WgpuNativeDiagnosticReadbackFrame, WgpuNativeRecorderLease, WgpuNativeSubmissionPacket,
};
pub use native_surface_recording::WgpuNativeSurfaceFrameTarget;

/// The production WGPU owner for one neutral RHI device generation.
///
/// This is intentionally an owning boundary, not a facade over product WGPU
/// access. Resource handles are allocated by its registry and command lists
/// are encoded only at submission, while this device generation is admitted.
pub struct WgpuRenderDevice {
    // Rust drops fields in declaration order. Native dependents must disappear before their queue
    // and device, and the adapter/instance must outlive the complete device generation.
    submissions: WgpuSubmissionService,
    diagnostics: Mutex<WgpuDiagnosticReadbackService>,
    surfaces: Mutex<WgpuSurfaceService>,
    registry: Mutex<WgpuResourceRegistry>,
    ui_image_registry: Arc<WgpuUiSharedImageRegistry>,
    _error_supervisor: WgpuDeviceErrorSupervisor,
    fault_gate: Arc<DeviceFaultGate>,
    caps: RenderBackendCaps,
    profile: RenderDeviceProfile,
    timestamp_period_ns: f32,
    queue: wgpu::Queue,
    device: wgpu::Device,
    adapter: wgpu::Adapter,
    instance: wgpu::Instance,
}

impl WgpuRenderDevice {
    /// Accepts one native ownership handoff and installs this generation's sole
    /// WGPU error supervisor.
    pub fn new(
        context: WgpuRenderDeviceContext,
        profile: RenderDeviceProfile,
    ) -> Result<Self, RhiError> {
        let WgpuRenderDeviceContext {
            instance,
            adapter,
            device,
            queue,
            ui_image_registry,
        } = context;
        validate_context_adapter(&adapter, &profile)?;
        validate_context_device_limits(&device, &profile)?;
        validate_context_requested_features(&device, &profile)?;
        validate_context_queue_topology(&profile)?;
        let fault_gate = Arc::new(DeviceFaultGate::new(
            profile.device_id(),
            profile.generation(),
        ));
        let error_supervisor = WgpuDeviceErrorSupervisor::install(&device, Arc::clone(&fault_gate));
        let caps = production_caps(&adapter, &device, &profile);
        let device_id = profile.device_id();
        let generation = profile.generation();
        let memory_budget = profile.memory_budget();
        let submission_limits = profile.submission_limits();
        let diagnostic_readback_budget = profile.diagnostic_readback_budget();
        let timestamp_period_ns = queue.get_timestamp_period();

        Ok(Self {
            instance,
            adapter,
            device,
            queue: queue.clone(),
            ui_image_registry,
            profile,
            caps,
            fault_gate,
            _error_supervisor: error_supervisor,
            registry: Mutex::new(WgpuResourceRegistry::new(
                device_id,
                generation,
                memory_budget,
            )),
            surfaces: Mutex::new(WgpuSurfaceService::new(
                device_id,
                generation,
                submission_limits.max_terminal_statuses(),
            )),
            submissions: WgpuSubmissionService::new(
                queue,
                device_id,
                generation,
                memory_budget,
                submission_limits,
            ),
            diagnostics: Mutex::new(WgpuDiagnosticReadbackService::new(
                device_id,
                generation,
                diagnostic_readback_budget,
            )),
            timestamp_period_ns,
        })
    }

    /// Immutable identity and negotiated feature receipt for this owner.
    pub const fn profile(&self) -> &RenderDeviceProfile {
        &self.profile
    }

    /// Immutable timestamp conversion fact captured when this device owner is created.
    pub const fn timestamp_period_ns(&self) -> f32 {
        self.timestamp_period_ns
    }

    /// Creates a retained-UI context from the same native generation as neutral RHI work.
    ///
    /// The UI layer receives cloned typed WGPU handles plus a private reference to this exact
    /// owner. It can return opaque native packets without gaining queue, poll, or registry access.
    pub fn ui_surface_context(self: &Arc<Self>) -> WgpuUiSurfaceContext {
        WgpuUiSurfaceContext::new_with_render_device(
            self.instance.clone(),
            self.adapter.clone(),
            self.device.clone(),
            self.queue.clone(),
            Arc::clone(&self.ui_image_registry),
            Arc::clone(self),
        )
    }

    /// Exposes diagnostics only; admission remains internal to this owner.
    pub fn first_fault(&self) -> Option<zr_rhi::DeviceFaultRecord> {
        self.fault_gate.first_fault()
    }

    /// Checks the generation's single shared fault gate without exposing native submission state.
    pub fn ensure_device_admission(&self) -> Result<(), DeviceAdmissionError> {
        self.fault_gate.ensure_admission()
    }

    /// Shares fault observation with the outer frame scheduler; this does not transfer ownership.
    pub fn device_fault_gate(&self) -> Arc<DeviceFaultGate> {
        Arc::clone(&self.fault_gate)
    }

    #[cfg(test)]
    pub(crate) fn inject_test_fault(&self, kind: DeviceFaultKind) {
        self.fault_gate
            .record_first(kind, "production submission fault-injection test");
    }

    #[cfg(test)]
    pub(crate) fn command_context_pool_counts_for_tests(&self) -> (usize, usize) {
        self.submissions.command_context_pool_counts()
    }

    fn ensure_admission(&self) -> Result<(), RhiError> {
        match self.fault_gate.ensure_admission() {
            Ok(()) => Ok(()),
            Err(error) => {
                self.submissions
                    .terminalize_unresolved(submission_terminal_status(error));
                self.lock_diagnostics()
                    .terminalize_all(diagnostic_terminal_status(error));
                self.terminalize_surface_frames();
                self.prune_terminal_resources();
                Err(RhiError::DeviceAdmission(error))
            }
        }
    }

    fn lock_registry(&self) -> MutexGuard<'_, WgpuResourceRegistry> {
        self.registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_diagnostics(&self) -> MutexGuard<'_, WgpuDiagnosticReadbackService> {
        self.diagnostics
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn prune_terminal_resources(&self) {
        let mut registry = self.lock_registry();
        registry.prune_terminal_uses(|ticket| self.submissions.is_ticket_terminal(ticket));
        registry.reap_retired(|ticket| self.submissions.is_ticket_terminal(ticket));
    }

    fn cancel_accepted_packet(&self, ticket: SubmissionTicket) {
        if self.submissions.cancel(ticket).is_ok() {
            self.lock_diagnostics()
                .terminalize_submission(ticket, DiagnosticReadbackTerminal::Cancelled);
            self.prune_terminal_resources();
        }
    }
}

impl RenderDevice for WgpuRenderDevice {
    fn caps(&self) -> &RenderBackendCaps {
        &self.caps
    }

    fn device_id(&self) -> zr_rhi::DeviceId {
        self.profile.device_id()
    }

    fn generation(&self) -> zr_rhi::DeviceGeneration {
        self.profile.generation()
    }

    fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle, RhiError> {
        self.ensure_admission()?;
        self.lock_registry().create_buffer(&self.device, desc)
    }

    fn buffer_desc(&self, handle: BufferHandle) -> Result<BufferDesc, RhiError> {
        self.lock_registry().buffer_desc(handle)
    }

    fn destroy_buffer(&self, handle: BufferHandle) -> Result<(), RhiError> {
        self.lock_registry().destroy_buffer(handle)
    }

    fn create_texture(&self, desc: &TextureDesc) -> Result<TextureHandle, RhiError> {
        self.ensure_admission()?;
        self.lock_registry().create_texture(&self.device, desc)
    }

    fn texture_desc(&self, handle: TextureHandle) -> Result<TextureDesc, RhiError> {
        self.lock_registry().texture_desc(handle)
    }

    fn destroy_texture(&self, handle: TextureHandle) -> Result<(), RhiError> {
        self.lock_registry().destroy_texture(handle)
    }

    fn create_texture_view(&self, desc: &TextureViewDesc) -> Result<TextureViewHandle, RhiError> {
        self.ensure_admission()?;
        self.lock_registry().create_texture_view(desc)
    }

    fn texture_view_desc(&self, handle: TextureViewHandle) -> Result<TextureViewDesc, RhiError> {
        self.lock_registry().texture_view_desc(handle)
    }

    fn destroy_texture_view(&self, handle: TextureViewHandle) -> Result<(), RhiError> {
        self.lock_registry().destroy_texture_view(handle)
    }

    fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerHandle, RhiError> {
        self.ensure_admission()?;
        self.lock_registry().create_sampler(&self.device, desc)
    }

    fn sampler_desc(&self, handle: SamplerHandle) -> Result<SamplerDesc, RhiError> {
        self.lock_registry().sampler_desc(handle)
    }

    fn destroy_sampler(&self, handle: SamplerHandle) -> Result<(), RhiError> {
        self.lock_registry().destroy_sampler(handle)
    }

    fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDesc,
    ) -> Result<BindGroupLayoutHandle, RhiError> {
        self.ensure_admission()?;
        self.lock_registry()
            .create_bind_group_layout(&self.device, desc)
    }

    fn bind_group_layout_desc(
        &self,
        handle: BindGroupLayoutHandle,
    ) -> Result<BindGroupLayoutDesc, RhiError> {
        self.lock_registry().bind_group_layout_desc(handle)
    }

    fn destroy_bind_group_layout(&self, handle: BindGroupLayoutHandle) -> Result<(), RhiError> {
        self.lock_registry().destroy_bind_group_layout(handle)
    }

    fn create_bind_group(&self, desc: &BindGroupDesc) -> Result<BindGroupHandle, RhiError> {
        self.ensure_admission()?;
        self.lock_registry().create_bind_group(&self.device, desc)
    }

    fn bind_group_desc(&self, handle: BindGroupHandle) -> Result<BindGroupDesc, RhiError> {
        self.lock_registry().bind_group_desc(handle)
    }

    fn destroy_bind_group(&self, handle: BindGroupHandle) -> Result<(), RhiError> {
        self.lock_registry().destroy_bind_group(handle)
    }

    fn create_shader_module(
        &self,
        desc: &ShaderModuleDesc,
    ) -> Result<ShaderModuleHandle, RhiError> {
        self.ensure_admission()?;
        self.lock_registry()
            .create_shader_module(&self.device, desc)
    }

    fn shader_module_desc(&self, handle: ShaderModuleHandle) -> Result<ShaderModuleDesc, RhiError> {
        self.lock_registry().shader_module_desc(handle)
    }

    fn destroy_shader_module(&self, handle: ShaderModuleHandle) -> Result<(), RhiError> {
        self.lock_registry().destroy_shader_module(handle)
    }

    fn create_pipeline_layout(
        &self,
        desc: &PipelineLayoutDesc,
    ) -> Result<PipelineLayoutHandle, RhiError> {
        self.ensure_admission()?;
        self.lock_registry()
            .create_pipeline_layout(&self.device, desc)
    }

    fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<PipelineLayoutDesc, RhiError> {
        self.lock_registry().pipeline_layout_desc(handle)
    }

    fn destroy_pipeline_layout(&self, handle: PipelineLayoutHandle) -> Result<(), RhiError> {
        self.lock_registry().destroy_pipeline_layout(handle)
    }

    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError> {
        self.ensure_admission()?;
        self.lock_registry().create_pipeline(&self.device, desc)
    }

    fn pipeline_desc(&self, handle: PipelineHandle) -> Result<PipelineDesc, RhiError> {
        self.lock_registry().pipeline_desc(handle)
    }

    fn destroy_pipeline(&self, handle: PipelineHandle) -> Result<(), RhiError> {
        self.lock_registry().destroy_pipeline(handle)
    }

    fn create_surface_session(
        &self,
        descriptor: &RenderSurfaceDescriptor,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        self.create_surface_session_impl(descriptor)
    }

    fn reconfigure_surface_session(
        &self,
        session: SurfaceSession,
        swapchain: &SwapchainDesc,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        self.reconfigure_surface_session_impl(session, swapchain)
    }

    fn acquire_surface_frame(
        &self,
        session: SurfaceSession,
    ) -> Result<SurfaceAcquireOutcome, RhiError> {
        self.acquire_surface_frame_impl(session)
    }

    fn present_surface_frame(
        &self,
        frame: SurfaceFrameLease,
        submission: SubmissionTicket,
    ) -> Result<SurfacePresentReceipt, RhiError> {
        self.present_surface_frame_impl(frame, submission)
    }

    fn discard_surface_frame(&self, frame: SurfaceFrameLease) -> Result<(), RhiError> {
        self.discard_surface_frame_impl(frame)
    }

    fn destroy_surface_session(&self, session: SurfaceSession) -> Result<(), RhiError> {
        self.destroy_surface_session_impl(session)
    }

    fn create_command_list(
        &self,
        queue_class: RenderQueueClass,
        label: &str,
    ) -> Result<Box<dyn CommandList>, RhiError> {
        self.ensure_admission()?;
        if !self.caps.supports_queue(queue_class) {
            return Err(RhiError::UnsupportedQueue(queue_class));
        }
        if label.is_empty() {
            return Err(RhiError::InvalidDebugMarker {
                reason: "command list label must not be empty".to_string(),
            });
        }
        Ok(Box::new(WgpuCommandList::new(queue_class, label)))
    }

    fn enqueue_submission_packet(
        &self,
        packet: RhiSubmissionPacket,
    ) -> Result<SubmissionTicket, RhiError> {
        self.ensure_admission()?;
        if packet.device_id() != self.profile.device_id()
            || packet.generation() != self.profile.generation()
        {
            return Err(RhiError::SubmissionPacketDeviceMismatch {
                packet_device_id: packet.device_id(),
                packet_generation: packet.generation(),
                device_id: self.profile.device_id(),
                generation: self.profile.generation(),
            });
        }
        if !self.caps.supports_queue(packet.queue_class()) {
            return Err(RhiError::UnsupportedQueue(packet.queue_class()));
        }
        for command_list in packet.command_lists() {
            self.require_recorded_command_operations(command_list.recorded_commands())?;
        }
        let ticket = self.submissions.begin_packet(packet.queue_class())?;
        let mut diagnostic_frame = match packet.diagnostic_query_plan() {
            Some(plan) => match self.lock_diagnostics().prepare_query_frame(
                &self.device,
                self.timestamp_period_ns,
                ticket,
                plan,
            ) {
                Ok(frame) => frame,
                Err(error) => {
                    self.cancel_accepted_packet(ticket);
                    return Err(error);
                }
            },
            None => None,
        };

        let command_buffers = (|| {
            let mut registry = self.lock_registry();
            let mut command_buffers = Vec::with_capacity(
                packet.command_list_count() + usize::from(diagnostic_frame.is_some()),
            );
            for command_list in packet.command_lists() {
                registry.mark_command_list_use(ticket, command_list.recorded_commands())?;
                command_buffers.push(encode_command_list(
                    &self.device,
                    &registry,
                    command_list.as_ref(),
                    diagnostic_frame.as_ref(),
                    self.profile.device_limits(),
                )?);
            }
            if let Some(frame) = diagnostic_frame.as_ref() {
                command_buffers.push(frame.encode_resolve(&self.device));
            }
            Ok::<_, RhiError>(command_buffers)
        })();
        match command_buffers {
            Ok(command_buffers) => {
                if let Some(frame) = diagnostic_frame.take() {
                    self.lock_diagnostics().commit_query_frame(ticket, frame);
                }
                match self.submissions.commit_packet(ticket, command_buffers) {
                    Ok(()) => Ok(ticket),
                    Err(error) => {
                        self.cancel_accepted_packet(ticket);
                        Err(error)
                    }
                }
            }
            Err(error) => {
                if let Some(frame) = diagnostic_frame.take() {
                    self.lock_diagnostics()
                        .abandon_prepared_query_frame(frame, DiagnosticReadbackTerminal::MapFailed);
                }
                self.cancel_accepted_packet(ticket);
                Err(error)
            }
        }
    }

    fn flush_submissions(&self) -> Result<usize, RhiError> {
        self.ensure_admission()?;
        let count = self.submissions.flush()?;
        self.lock_diagnostics()
            .collect_completed_maps(|ticket| self.submissions.status(ticket))?;
        Ok(count)
    }

    fn submission_status(&self, ticket: SubmissionTicket) -> Result<SubmissionStatus, RhiError> {
        self.submissions.status(ticket)
    }

    fn append_submission_statuses(
        &self,
        tickets: &[SubmissionTicket],
        statuses: &mut Vec<Result<SubmissionStatus, RhiError>>,
    ) {
        self.submissions.append_statuses(tickets, statuses);
    }

    fn cancel_submission(&self, ticket: SubmissionTicket) -> Result<SubmissionStatus, RhiError> {
        let status = self.submissions.cancel(ticket)?;
        if status == SubmissionStatus::Cancelled {
            self.lock_diagnostics()
                .terminalize_submission(ticket, DiagnosticReadbackTerminal::Cancelled);
        }
        if status.is_terminal() {
            self.prune_terminal_resources();
        }
        Ok(status)
    }

    fn poll_submissions(&self) -> Result<SubmissionPollReceipt, RhiError> {
        if let Err(error) = self.fault_gate.ensure_admission() {
            let poll = self.submissions.issue_poll_receipt()?;
            self.submissions
                .terminalize_unresolved(submission_terminal_status(error));
            self.lock_diagnostics()
                .terminalize_all(diagnostic_terminal_status(error));
            self.terminalize_surface_frames();
            self.prune_terminal_resources();
            return Ok(poll);
        }
        let poll = self.submissions.poll(&self.device).inspect_err(|_| {
            self.submissions
                .terminalize_unresolved(SubmissionStatus::Failed);
            self.lock_diagnostics()
                .terminalize_all(DiagnosticReadbackTerminal::Shutdown);
            self.terminalize_surface_frames();
            self.prune_terminal_resources();
        })?;
        self.lock_diagnostics()
            .collect_completed_maps(|ticket| self.submissions.status(ticket))?;
        self.prune_terminal_resources();
        Ok(poll)
    }

    fn transient_allocator_stats(&self) -> TransientAllocatorStats {
        self.lock_registry().transient_allocator_stats()
    }

    fn memory_snapshot(&self) -> GpuMemorySnapshot {
        let mut snapshot = self.lock_registry().memory_snapshot();
        snapshot.pending_upload_bytes = self.submissions.pending_upload_bytes();
        snapshot
    }

    fn write_buffer_batch(&self, batch: BufferUploadBatch) -> Result<SubmissionTicket, RhiError> {
        if batch.is_empty() {
            return Err(RhiError::EmptyUploadBatch);
        }
        batch
            .payload_byte_len()
            .ok_or(RhiError::UploadByteCountOverflow)?;
        self.ensure_admission()?;
        let ticket = self.submissions.begin_packet(RenderQueueClass::Copy)?;
        let native_batch = (|| {
            let mut registry = self.lock_registry();
            let mut native_batch = WgpuBufferUploadBatch::new();
            for upload in batch.uploads() {
                let handle = upload.buffer();
                let desc = registry.buffer_desc(handle)?;
                ensure_wgpu_upload_range(
                    handle,
                    &desc,
                    upload.destination_offset(),
                    upload.payload_byte_len(),
                )?;
                let source_range = upload.source_range();
                let native = WgpuBufferUpload::new(
                    registry.buffer(handle)?.clone(),
                    upload.destination_offset(),
                    Arc::clone(upload.payload_owner()),
                    source_range.clone(),
                )
                .ok_or(RhiError::InvalidUploadSourceRange {
                    start: source_range.start,
                    end: source_range.end,
                    payload_bytes: upload.payload_owner().len(),
                })?;
                native_batch.push(native);
            }
            for upload in batch.uploads() {
                registry.mark_buffer_upload_use(upload.buffer(), ticket)?;
            }
            Ok::<_, RhiError>(native_batch)
        })();
        let native_batch = match native_batch {
            Ok(native_batch) => native_batch,
            Err(error) => {
                self.cancel_accepted_packet(ticket);
                return Err(error);
            }
        };
        match self
            .submissions
            .commit_buffer_upload_batch(ticket, native_batch)
        {
            Ok(()) => Ok(ticket),
            Err(error) => {
                self.cancel_accepted_packet(ticket);
                Err(error)
            }
        }
    }

    fn write_texture_batch(&self, batch: TextureUploadBatch) -> Result<SubmissionTicket, RhiError> {
        if batch.is_empty() {
            return Err(RhiError::EmptyUploadBatch);
        }
        batch
            .payload_byte_len()
            .ok_or(RhiError::UploadByteCountOverflow)?;
        self.ensure_admission()?;
        let ticket = self.submissions.begin_packet(RenderQueueClass::Copy)?;
        let native_batch = (|| {
            let mut registry = self.lock_registry();
            let mut native_batch = WgpuTextureUploadBatch::new();
            for upload in batch.uploads() {
                let handle = upload.texture();
                let desc = registry.texture_desc(handle)?;
                let region = upload.region();
                let (native_bytes_per_row, upload_bytes) = prepare_wgpu_texture_upload(
                    handle,
                    &desc,
                    region,
                    upload.bytes_per_row(),
                    upload.payload_byte_len(),
                )?;
                let source_range = upload.source_range();
                let source_end = source_range
                    .start
                    .checked_add(upload_bytes)
                    .ok_or(RhiError::UploadByteCountOverflow)?;
                let native_source_range = source_range.start..source_end;
                let native = WgpuTextureUpload::new(
                    registry.texture(handle)?.clone(),
                    region,
                    native_bytes_per_row,
                    region.height,
                    Arc::clone(upload.payload_owner()),
                    native_source_range.clone(),
                )
                .ok_or(RhiError::InvalidUploadSourceRange {
                    start: native_source_range.start,
                    end: native_source_range.end,
                    payload_bytes: upload.payload_owner().len(),
                })?;
                native_batch.push(native);
            }
            for upload in batch.uploads() {
                registry.mark_texture_upload_use(upload.texture(), ticket)?;
            }
            Ok::<_, RhiError>(native_batch)
        })();
        let native_batch = match native_batch {
            Ok(native_batch) => native_batch,
            Err(error) => {
                self.cancel_accepted_packet(ticket);
                return Err(error);
            }
        };
        match self
            .submissions
            .commit_texture_upload_batch(ticket, native_batch)
        {
            Ok(()) => Ok(ticket),
            Err(error) => {
                self.cancel_accepted_packet(ticket);
                Err(error)
            }
        }
    }

    fn read_buffer(
        &self,
        handle: BufferHandle,
        _offset: u64,
        _size: u64,
    ) -> Result<Vec<u8>, RhiError> {
        self.lock_registry().buffer_desc(handle)?;
        Err(RhiError::ReadbackUnavailable {
            reason:
                "synchronous buffer readback is not available; use the M5 async readback service"
                    .to_string(),
        })
    }

    fn read_texture(&self, handle: TextureHandle) -> Result<Vec<u8>, RhiError> {
        self.lock_registry().texture_desc(handle)?;
        Err(RhiError::ReadbackUnavailable {
            reason:
                "synchronous texture readback is not available; use the M5 async readback service"
                    .to_string(),
        })
    }
}

fn ensure_wgpu_upload_range(
    handle: BufferHandle,
    desc: &BufferDesc,
    offset: u64,
    size: u64,
) -> Result<(), RhiError> {
    ensure_wgpu_upload_usage(handle, desc)?;
    if offset % 4 != 0 || size % 4 != 0 {
        return Err(RhiError::InvalidCopy {
            reason: "WGPU queue buffer writes require four-byte aligned offset and data size"
                .to_string(),
        });
    }
    if offset.saturating_add(size) > desc.size_bytes {
        return Err(RhiError::WriteOutOfRange {
            buffer: handle.diagnostic_id(),
            offset,
            size,
        });
    }
    Ok(())
}

fn ensure_wgpu_upload_usage(handle: BufferHandle, desc: &BufferDesc) -> Result<(), RhiError> {
    if desc.usage.contains(BufferUsage::COPY_DST) {
        Ok(())
    } else {
        Err(RhiError::InvalidBufferUsage {
            buffer: handle.diagnostic_id(),
            required: BufferUsage::COPY_DST,
            actual: desc.usage,
        })
    }
}

fn prepare_wgpu_texture_upload(
    handle: TextureHandle,
    desc: &TextureDesc,
    region: TextureCopyRegion,
    bytes_per_row: u64,
    source_bytes: u64,
) -> Result<(u32, usize), RhiError> {
    if !desc.usage.contains(TextureUsage::COPY_DST) {
        return Err(RhiError::InvalidTextureUsage {
            texture: handle.diagnostic_id(),
            required: TextureUsage::COPY_DST,
            actual: desc.usage,
        });
    }
    validate_texture_copy_destination_aspect(handle, desc, region)?;
    let layout = texture_upload_layout(desc, region, bytes_per_row, source_bytes)
        .ok_or_else(|| texture_write_out_of_range(handle, source_bytes, bytes_per_row, region))?;
    let native_bytes_per_row = u32::try_from(bytes_per_row)
        .map_err(|_| texture_write_out_of_range(handle, source_bytes, bytes_per_row, region))?;
    let upload_bytes = texture_upload_byte_len(region, bytes_per_row, layout.copy_row_bytes)
        .ok_or_else(|| texture_write_out_of_range(handle, source_bytes, bytes_per_row, region))?;
    let upload_bytes = usize::try_from(upload_bytes)
        .map_err(|_| texture_write_out_of_range(handle, source_bytes, bytes_per_row, region))?;
    Ok((native_bytes_per_row, upload_bytes))
}
