use zr_rhi::{
    BindGroupDesc, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutHandle, BufferDesc,
    BufferHandle, DeviceGeneration, DeviceId, GpuMemorySnapshot, PipelineLayoutDesc,
    PipelineLayoutHandle, RhiError, SamplerDesc, SamplerHandle, ShaderModuleDesc,
    ShaderModuleHandle, SubmissionHistory, SubmissionLimits, SubmissionPollReceipt,
    SubmissionTicket, TextureDesc, TextureHandle, TextureViewDesc, TextureViewHandle,
    TransientAllocatorStats,
};

use crate::bind_group_validation::BindGroupResourceLookup;
use crate::pipeline_validation::PipelineResourceLookup;
use crate::render_pass_validation::RenderPassResourceLookup;

use super::{
    resources::saturating_u32, DeterministicRhiContractDeviceState, QueuedDeterministicSubmission,
};

impl Default for DeterministicRhiContractDeviceState {
    fn default() -> Self {
        Self {
            next_submission_sequence: 1,
            next_poll_sequence: 1,
            submission_history: SubmissionHistory::new(SubmissionLimits::default()),
            pending_submissions: Vec::new(),
            submitted_submissions: Vec::new(),
            buffers: Default::default(),
            textures: Default::default(),
            texture_views: Default::default(),
            texture_view_counts: Default::default(),
            surface_sessions: Default::default(),
            surface_frames: Default::default(),
            terminal_surface_frames: Default::default(),
            surface_owned_textures: Default::default(),
            surface_owned_texture_views: Default::default(),
            samplers: Default::default(),
            bind_group_layouts: Default::default(),
            bind_groups: Default::default(),
            shaders: Default::default(),
            pipeline_layouts: Default::default(),
            pipelines: Default::default(),
        }
    }
}

impl DeterministicRhiContractDeviceState {
    pub(super) fn issue_poll_receipt(
        &mut self,
        device_id: DeviceId,
        generation: DeviceGeneration,
    ) -> Result<SubmissionPollReceipt, RhiError> {
        let sequence = self.next_poll_sequence;
        self.next_poll_sequence =
            sequence
                .checked_add(1)
                .ok_or(RhiError::SubmissionPollSequenceExhausted {
                    device_id,
                    generation,
                })?;
        Ok(SubmissionPollReceipt::new(device_id, generation, sequence))
    }

    pub(super) fn allocate_submission_ticket(
        &mut self,
        device_id: DeviceId,
        generation: DeviceGeneration,
        queue_class: zr_rhi::RenderQueueClass,
    ) -> Result<SubmissionTicket, RhiError> {
        if !self.submission_history.can_accept() {
            return Err(RhiError::SubmissionBackpressure {
                unresolved_submissions: self.submission_history.unresolved_count(),
                limit: self
                    .submission_history
                    .limits()
                    .max_unresolved_submissions(),
            });
        }
        let sequence = self.next_submission_sequence;
        self.next_submission_sequence =
            sequence
                .checked_add(1)
                .ok_or(RhiError::SubmissionSequenceExhausted {
                    device_id,
                    generation,
                })?;
        let ticket = SubmissionTicket::new(device_id, generation, queue_class, sequence);
        debug_assert!(self.submission_history.record_accepted(ticket));
        Ok(ticket)
    }

    pub(super) fn append_submission_statuses(
        &self,
        device_id: DeviceId,
        generation: DeviceGeneration,
        tickets: &[SubmissionTicket],
        statuses: &mut Vec<Result<zr_rhi::SubmissionStatus, RhiError>>,
    ) {
        statuses.reserve(tickets.len());
        statuses.extend(tickets.iter().copied().map(|ticket| {
            if ticket.device_id() != device_id || ticket.generation() != generation {
                return Err(RhiError::UnknownSubmissionTicket(ticket));
            }
            self.submission_history
                .status(ticket)
                .ok_or(RhiError::UnknownSubmissionTicket(ticket))
        }));
    }

    pub(crate) fn bind_group_desc_ref(
        &self,
        handle: BindGroupHandle,
    ) -> Result<&BindGroupDesc, RhiError> {
        self.bind_groups
            .get(&handle)
            .map(|bind_group| &bind_group.desc)
            .ok_or(RhiError::UnknownBindGroup(handle.diagnostic_id()))
    }

    pub(crate) fn pipeline_layout_desc_ref(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<&PipelineLayoutDesc, RhiError> {
        self.pipeline_layouts
            .get(&handle)
            .ok_or(RhiError::UnknownPipelineLayout(handle.diagnostic_id()))
    }

    pub(super) fn texture_desc_ref(&self, handle: TextureHandle) -> Result<&TextureDesc, RhiError> {
        self.textures
            .get(&handle)
            .map(|texture| &texture.desc)
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))
    }

    pub(super) fn transient_allocator_stats(&self) -> TransientAllocatorStats {
        let snapshot = self.memory_snapshot();
        TransientAllocatorStats {
            bytes_reserved: snapshot.reserved_resource_bytes(),
            allocations: snapshot.reserved_resource_allocations(),
        }
    }

    pub(super) fn memory_snapshot(&self) -> GpuMemorySnapshot {
        let active_buffer_bytes = self
            .buffers
            .values()
            .map(|buffer| buffer.desc.size_bytes)
            .fold(0_u64, u64::saturating_add);
        let active_texture_bytes = self
            .textures
            .values()
            .map(|texture| texture.contents.len() as u64)
            .fold(0_u64, u64::saturating_add);
        let (_, pending_upload_bytes) = self.pending_upload_stats();
        GpuMemorySnapshot {
            active_buffer_bytes,
            active_texture_bytes,
            pending_upload_bytes,
            active_allocations: saturating_u32(
                self.buffers.len().saturating_add(self.textures.len()),
            ),
            ..GpuMemorySnapshot::default()
        }
    }

    pub(super) fn pending_upload_stats(&self) -> (usize, u64) {
        self.pending_submissions
            .iter()
            .fold(
                (0_usize, 0_u64),
                |(count, bytes), submission| match submission {
                    QueuedDeterministicSubmission::Upload { batch, .. } => (
                        count.saturating_add(1),
                        bytes.saturating_add(batch.payload_byte_len().unwrap_or(u64::MAX)),
                    ),
                    QueuedDeterministicSubmission::TextureUpload { batch, .. } => (
                        count.saturating_add(1),
                        bytes.saturating_add(batch.payload_byte_len().unwrap_or(u64::MAX)),
                    ),
                    QueuedDeterministicSubmission::Command { .. } => (count, bytes),
                },
            )
    }
}

impl BindGroupResourceLookup for DeterministicRhiContractDeviceState {
    fn layout_desc(&self, handle: BindGroupLayoutHandle) -> Result<&BindGroupLayoutDesc, RhiError> {
        self.bind_group_layouts
            .get(&handle)
            .ok_or(RhiError::UnknownBindGroupLayout(handle.diagnostic_id()))
    }

    fn buffer_desc(&self, handle: BufferHandle) -> Result<&BufferDesc, RhiError> {
        self.buffers
            .get(&handle)
            .map(|buffer| &buffer.desc)
            .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))
    }

    fn texture_desc(&self, handle: TextureHandle) -> Result<&TextureDesc, RhiError> {
        self.textures
            .get(&handle)
            .map(|texture| &texture.desc)
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))
    }

    fn texture_view_desc(&self, handle: TextureViewHandle) -> Result<&TextureViewDesc, RhiError> {
        self.texture_views
            .get(&handle)
            .ok_or(RhiError::UnknownTextureView(handle.diagnostic_id()))
    }

    fn sampler_desc(&self, handle: SamplerHandle) -> Result<&SamplerDesc, RhiError> {
        self.samplers
            .get(&handle)
            .ok_or(RhiError::UnknownSampler(handle.diagnostic_id()))
    }
}

impl PipelineResourceLookup for DeterministicRhiContractDeviceState {
    fn bind_group_layout_exists(&self, handle: BindGroupLayoutHandle) -> bool {
        self.bind_group_layouts.contains_key(&handle)
    }

    fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<&PipelineLayoutDesc, RhiError> {
        self.pipeline_layouts
            .get(&handle)
            .ok_or(RhiError::UnknownPipelineLayout(handle.diagnostic_id()))
    }

    fn shader_module_desc(
        &self,
        handle: ShaderModuleHandle,
    ) -> Result<&ShaderModuleDesc, RhiError> {
        self.shaders
            .get(&handle)
            .ok_or(RhiError::UnknownShaderModule(handle.diagnostic_id()))
    }
}

impl RenderPassResourceLookup for DeterministicRhiContractDeviceState {
    fn texture_desc(&self, handle: TextureHandle) -> Result<&TextureDesc, RhiError> {
        self.texture_desc_ref(handle)
    }

    fn texture_view_desc(&self, handle: TextureViewHandle) -> Result<&TextureViewDesc, RhiError> {
        self.texture_views
            .get(&handle)
            .ok_or(RhiError::UnknownTextureView(handle.diagnostic_id()))
    }
}
