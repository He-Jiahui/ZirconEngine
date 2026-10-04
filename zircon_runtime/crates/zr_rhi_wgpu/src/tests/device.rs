//! 仅用于中立 RHI 契约测试的确定性主机设备；上传和拷贝修改 CPU 镜像，绘制/计算只验证状态。
//! 入队、flush 与完成观察保持独立，便于测试背压、取消和票据生命期；没有真实 GPU 完成事件。

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

use zr_rhi::{
    BindGroupDesc, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutHandle, BufferDesc,
    BufferHandle, BufferUploadBatch, BufferUsage, CommandList, CommandListCommand,
    DeviceGeneration, DeviceId, GpuMemoryBudget, GpuMemoryClass, GpuMemorySnapshot, PipelineDesc,
    PipelineHandle, PipelineLayoutDesc, PipelineLayoutHandle, RenderBackendCaps, RenderDevice,
    RenderQueueClass, RenderResourceHandleAllocator, RenderSurfaceDescriptor,
    RenderSurfaceHandleAllocator, RhiError, RhiGraphExecutionReceipt, RhiSubmissionPacket,
    SamplerDesc, SamplerHandle, ShaderModuleDesc, ShaderModuleHandle, SubmissionHistory,
    SubmissionPollReceipt, SubmissionStatus, SubmissionTicket, SurfaceAcquireOutcome,
    SurfaceFrameId, SurfaceFrameLease, SurfaceFrameTerminal, SurfaceFrameTerminalHistory,
    SurfacePresentReceipt, SurfaceSession, SurfaceSessionCreateOutcome, SurfaceSessionReceipt,
    SwapchainDesc, TextureCopyRegion, TextureDesc, TextureHandle, TextureUploadBatch, TextureUsage,
    TextureViewDesc, TextureViewHandle, TransientAllocatorStats,
};

use super::bind_group_validation::validate_bind_group_desc;
use super::command_validation::{execute_recorded_commands, validate_recorded_commands};
use super::pipeline_validation::{
    validate_pipeline_desc, validate_pipeline_layout_desc, validate_shader_module_desc,
};
use super::resource_validation::{
    ensure_buffer_usage, ensure_texture_usage, texture_storage_size,
    validate_bind_group_layout_desc, validate_buffer_desc, validate_sampler_desc,
    validate_texture_desc,
};
#[path = "../device/tests/command_list.rs"]
mod command_list;
#[path = "../device/tests/construction.rs"]
mod construction;
#[path = "../device/tests/contract_caps.rs"]
mod contract_caps;
#[path = "../device/tests/resources.rs"]
mod resources;
#[path = "../device/tests/state.rs"]
mod state;
#[path = "../device/tests/surfaces.rs"]
mod surfaces;
#[path = "../device/tests/uploads.rs"]
mod uploads;
#[path = "../device/tests/views.rs"]
mod views;

#[cfg(test)]
#[path = "../device/tests/graph_receipt_tests.rs"]
mod graph_receipt_tests;

pub(crate) use self::command_list::DeterministicRhiContractCommandList;

/// 克隆共享同一注册表与票据历史；独立设备测试应显式选择不同身份以检查跨设备拒绝。
#[derive(Clone, Debug)]
pub(crate) struct DeterministicRhiContractDevice {
    caps: RenderBackendCaps,
    device_id: DeviceId,
    generation: DeviceGeneration,
    memory_budget: GpuMemoryBudget,
    handle_allocator: RenderResourceHandleAllocator,
    surface_handle_allocator: RenderSurfaceHandleAllocator,
    state: Arc<Mutex<DeterministicRhiContractDeviceState>>,
}

#[derive(Debug)]
pub(super) struct DeterministicRhiContractDeviceState {
    next_submission_sequence: u64,
    next_poll_sequence: u64,
    submission_history: SubmissionHistory,
    pending_submissions: Vec<QueuedDeterministicSubmission>,
    submitted_submissions: Vec<SubmissionTicket>,
    submitted_graph_receipts: HashMap<SubmissionTicket, RhiGraphExecutionReceipt>,
    pub(super) buffers: HashMap<BufferHandle, WgpuBufferResource>,
    pub(super) textures: HashMap<TextureHandle, WgpuTextureResource>,
    pub(super) texture_views: HashMap<TextureViewHandle, TextureViewDesc>,
    texture_view_counts: HashMap<TextureHandle, u32>,
    surface_sessions: HashMap<SurfaceSession, surfaces::DeterministicSurfaceSession>,
    surface_frames: HashMap<SurfaceFrameId, surfaces::DeterministicSurfaceFrame>,
    pub(super) terminal_surface_frames: SurfaceFrameTerminalHistory,
    pub(super) surface_owned_textures: HashSet<TextureHandle>,
    pub(super) surface_owned_texture_views: HashSet<TextureViewHandle>,
    samplers: HashMap<SamplerHandle, SamplerDesc>,
    bind_group_layouts: HashMap<BindGroupLayoutHandle, BindGroupLayoutDesc>,
    bind_groups: HashMap<BindGroupHandle, WgpuBindGroupResource>,
    shaders: HashMap<ShaderModuleHandle, ShaderModuleDesc>,
    pipeline_layouts: HashMap<PipelineLayoutHandle, PipelineLayoutDesc>,
    pub(super) pipelines: HashMap<PipelineHandle, PipelineDesc>,
}

#[derive(Debug)]
enum QueuedDeterministicSubmission {
    Command {
        ticket: SubmissionTicket,
        commands: Vec<CommandListCommand>,
        graph_execution_receipt: Option<RhiGraphExecutionReceipt>,
    },
    Upload {
        ticket: SubmissionTicket,
        batch: BufferUploadBatch,
    },
    TextureUpload {
        ticket: SubmissionTicket,
        batch: TextureUploadBatch,
    },
}

impl QueuedDeterministicSubmission {
    const fn ticket(&self) -> SubmissionTicket {
        match self {
            Self::Command { ticket, .. }
            | Self::Upload { ticket, .. }
            | Self::TextureUpload { ticket, .. } => *ticket,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct WgpuBufferResource {
    pub(super) desc: BufferDesc,
    pub(super) contents: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(super) struct WgpuTextureResource {
    pub(super) desc: TextureDesc,
    pub(super) contents: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(super) struct WgpuBindGroupResource {
    pub(super) desc: BindGroupDesc,
}

impl DeterministicRhiContractDevice {
    fn lock_state(&self) -> MutexGuard<'_, DeterministicRhiContractDeviceState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl RenderDevice for DeterministicRhiContractDevice {
    fn caps(&self) -> &RenderBackendCaps {
        &self.caps
    }

    fn device_id(&self) -> DeviceId {
        self.device_id
    }

    fn generation(&self) -> DeviceGeneration {
        self.generation
    }

    fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle, RhiError> {
        validate_buffer_desc(desc)?;
        let mut state = self.lock_state();
        let snapshot = state.memory_snapshot();
        resources::ensure_memory_capacity(
            GpuMemoryClass::Buffer,
            snapshot.active_buffer_bytes,
            desc.size_bytes,
            self.memory_budget.transient_buffer_bytes(),
        )?;
        let contents =
            resources::allocate_zeroed_contents(GpuMemoryClass::Buffer, desc.size_bytes)?;
        let handle = self.handle_allocator.allocate_buffer()?;
        state.buffers.insert(
            handle,
            WgpuBufferResource {
                desc: desc.clone(),
                contents,
            },
        );
        Ok(handle)
    }

    fn buffer_desc(&self, handle: BufferHandle) -> Result<BufferDesc, RhiError> {
        let state = self.lock_state();
        state
            .buffers
            .get(&handle)
            .map(|buffer| buffer.desc.clone())
            .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))
    }

    fn destroy_buffer(&self, handle: BufferHandle) -> Result<(), RhiError> {
        let removed = self.lock_state().buffers.remove(&handle).is_some();
        if !removed {
            return Err(RhiError::UnknownBuffer(handle.diagnostic_id()));
        }
        self.handle_allocator.release_buffer(handle)?;
        Ok(())
    }

    fn create_texture(&self, desc: &TextureDesc) -> Result<TextureHandle, RhiError> {
        validate_texture_desc(desc, self.caps.supports_sparse_texture)?;
        if desc.usage.contains(TextureUsage::PRESENT) {
            return Err(RhiError::InvalidTextureDescriptor {
                label: desc.label.clone(),
                reason: "PRESENT textures must be created by the surface owner".to_string(),
            });
        }
        let mut state = self.lock_state();
        let requested_bytes = texture_storage_size(desc);
        let snapshot = state.memory_snapshot();
        resources::ensure_memory_capacity(
            GpuMemoryClass::Texture,
            snapshot.active_texture_bytes,
            requested_bytes,
            self.memory_budget.transient_texture_bytes(),
        )?;
        let contents =
            resources::allocate_zeroed_contents(GpuMemoryClass::Texture, requested_bytes)?;
        let handle = self.handle_allocator.allocate_texture()?;
        state.textures.insert(
            handle,
            WgpuTextureResource {
                desc: desc.clone(),
                contents,
            },
        );
        Ok(handle)
    }

    fn texture_desc(&self, handle: TextureHandle) -> Result<TextureDesc, RhiError> {
        let state = self.lock_state();
        state
            .textures
            .get(&handle)
            .map(|texture| texture.desc.clone())
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))
    }

    fn destroy_texture(&self, handle: TextureHandle) -> Result<(), RhiError> {
        self.lock_state()
            .destroy_texture(&self.handle_allocator, handle)
    }

    fn create_texture_view(&self, desc: &TextureViewDesc) -> Result<TextureViewHandle, RhiError> {
        self.lock_state()
            .create_texture_view(&self.handle_allocator, desc)
    }

    fn texture_view_desc(&self, handle: TextureViewHandle) -> Result<TextureViewDesc, RhiError> {
        self.lock_state().texture_view_desc(handle)
    }

    fn destroy_texture_view(&self, handle: TextureViewHandle) -> Result<(), RhiError> {
        self.lock_state()
            .destroy_texture_view(&self.handle_allocator, handle)
    }

    fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerHandle, RhiError> {
        validate_sampler_desc(desc)?;
        let handle = self.handle_allocator.allocate_sampler()?;
        let mut state = self.lock_state();
        state.samplers.insert(handle, desc.clone());
        Ok(handle)
    }

    fn sampler_desc(&self, handle: SamplerHandle) -> Result<SamplerDesc, RhiError> {
        let state = self.lock_state();
        state
            .samplers
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownSampler(handle.diagnostic_id()))
    }

    fn destroy_sampler(&self, handle: SamplerHandle) -> Result<(), RhiError> {
        let removed = self.lock_state().samplers.remove(&handle).is_some();
        if !removed {
            return Err(RhiError::UnknownSampler(handle.diagnostic_id()));
        }
        self.handle_allocator.release_sampler(handle)?;
        Ok(())
    }

    fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDesc,
    ) -> Result<BindGroupLayoutHandle, RhiError> {
        validate_bind_group_layout_desc(desc)?;
        let handle = self.handle_allocator.allocate_bind_group_layout()?;
        let mut state = self.lock_state();
        state.bind_group_layouts.insert(handle, desc.clone());
        Ok(handle)
    }

    fn bind_group_layout_desc(
        &self,
        handle: BindGroupLayoutHandle,
    ) -> Result<BindGroupLayoutDesc, RhiError> {
        let state = self.lock_state();
        state
            .bind_group_layouts
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownBindGroupLayout(handle.diagnostic_id()))
    }

    fn destroy_bind_group_layout(&self, handle: BindGroupLayoutHandle) -> Result<(), RhiError> {
        let removed = self
            .lock_state()
            .bind_group_layouts
            .remove(&handle)
            .is_some();
        if !removed {
            return Err(RhiError::UnknownBindGroupLayout(handle.diagnostic_id()));
        }
        self.handle_allocator.release_bind_group_layout(handle)?;
        Ok(())
    }

    fn create_bind_group(&self, desc: &BindGroupDesc) -> Result<BindGroupHandle, RhiError> {
        {
            let state = self.lock_state();
            validate_bind_group_desc(&*state, desc)?;
        }
        let handle = self.handle_allocator.allocate_bind_group()?;
        self.lock_state()
            .bind_groups
            .insert(handle, WgpuBindGroupResource { desc: desc.clone() });
        Ok(handle)
    }

    fn bind_group_desc(&self, handle: BindGroupHandle) -> Result<BindGroupDesc, RhiError> {
        let state = self.lock_state();
        state
            .bind_groups
            .get(&handle)
            .map(|bind_group| bind_group.desc.clone())
            .ok_or(RhiError::UnknownBindGroup(handle.diagnostic_id()))
    }

    fn destroy_bind_group(&self, handle: BindGroupHandle) -> Result<(), RhiError> {
        let removed = self.lock_state().bind_groups.remove(&handle).is_some();
        if !removed {
            return Err(RhiError::UnknownBindGroup(handle.diagnostic_id()));
        }
        self.handle_allocator.release_bind_group(handle)?;
        Ok(())
    }

    fn create_shader_module(
        &self,
        desc: &ShaderModuleDesc,
    ) -> Result<ShaderModuleHandle, RhiError> {
        validate_shader_module_desc(desc)?;
        let handle = self.handle_allocator.allocate_shader_module()?;
        let mut state = self.lock_state();
        state.shaders.insert(handle, desc.clone());
        Ok(handle)
    }

    fn shader_module_desc(&self, handle: ShaderModuleHandle) -> Result<ShaderModuleDesc, RhiError> {
        let state = self.lock_state();
        state
            .shaders
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownShaderModule(handle.diagnostic_id()))
    }

    fn destroy_shader_module(&self, handle: ShaderModuleHandle) -> Result<(), RhiError> {
        let removed = self.lock_state().shaders.remove(&handle).is_some();
        if !removed {
            return Err(RhiError::UnknownShaderModule(handle.diagnostic_id()));
        }
        self.handle_allocator.release_shader_module(handle)?;
        Ok(())
    }

    fn create_pipeline_layout(
        &self,
        desc: &PipelineLayoutDesc,
    ) -> Result<PipelineLayoutHandle, RhiError> {
        {
            let state = self.lock_state();
            validate_pipeline_layout_desc(&*state, desc)?;
        }
        let handle = self.handle_allocator.allocate_pipeline_layout()?;
        self.lock_state()
            .pipeline_layouts
            .insert(handle, desc.clone());
        Ok(handle)
    }

    fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<PipelineLayoutDesc, RhiError> {
        let state = self.lock_state();
        state
            .pipeline_layouts
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownPipelineLayout(handle.diagnostic_id()))
    }

    fn destroy_pipeline_layout(&self, handle: PipelineLayoutHandle) -> Result<(), RhiError> {
        let removed = self.lock_state().pipeline_layouts.remove(&handle).is_some();
        if !removed {
            return Err(RhiError::UnknownPipelineLayout(handle.diagnostic_id()));
        }
        self.handle_allocator.release_pipeline_layout(handle)?;
        Ok(())
    }

    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError> {
        {
            let state = self.lock_state();
            validate_pipeline_desc(&*state, desc)?;
        }
        let handle = self.handle_allocator.allocate_pipeline()?;
        self.lock_state().pipelines.insert(handle, desc.clone());
        Ok(handle)
    }

    fn pipeline_desc(&self, handle: PipelineHandle) -> Result<PipelineDesc, RhiError> {
        let state = self.lock_state();
        state
            .pipelines
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownPipeline(handle.diagnostic_id()))
    }

    fn destroy_pipeline(&self, handle: PipelineHandle) -> Result<(), RhiError> {
        let removed = self.lock_state().pipelines.remove(&handle).is_some();
        if !removed {
            return Err(RhiError::UnknownPipeline(handle.diagnostic_id()));
        }
        self.handle_allocator.release_pipeline(handle)?;
        Ok(())
    }

    fn create_surface_session(
        &self,
        descriptor: &RenderSurfaceDescriptor,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        if !self.caps.supports_surface {
            return Err(RhiError::SurfaceUnavailable(
                "surface sessions are disabled by this device capability receipt".to_string(),
            ));
        }
        self.lock_state()
            .create_surface_session(&self.surface_handle_allocator, descriptor)
    }

    fn reconfigure_surface_session(
        &self,
        session: SurfaceSession,
        swapchain: &SwapchainDesc,
    ) -> Result<SurfaceSessionCreateOutcome, RhiError> {
        let mut retired_submissions = Vec::new();
        let result = self.lock_state().reconfigure_surface_session(
            &self.handle_allocator,
            &self.surface_handle_allocator,
            session,
            swapchain,
            &mut retired_submissions,
        );
        drop(retired_submissions);
        result
    }

    fn acquire_surface_frame(
        &self,
        session: SurfaceSession,
    ) -> Result<SurfaceAcquireOutcome, RhiError> {
        self.lock_state().acquire_surface_frame(
            &self.handle_allocator,
            &self.surface_handle_allocator,
            self.memory_budget,
            session,
        )
    }

    fn present_surface_frame(
        &self,
        frame: SurfaceFrameLease,
        submission: SubmissionTicket,
    ) -> Result<SurfacePresentReceipt, RhiError> {
        if submission.device_id() != self.device_id || submission.generation() != self.generation {
            return Err(RhiError::SurfaceFrameSubmissionMismatch {
                frame: frame.frame(),
                submission,
            });
        }
        let status = self.submission_status(submission)?;
        if !matches!(
            status,
            SubmissionStatus::Submitted | SubmissionStatus::Completed
        ) {
            return Err(RhiError::SurfaceFrameSubmissionNotReady {
                frame: frame.frame(),
                status,
            });
        }
        let mut retired_submissions = Vec::new();
        let mut state = self.lock_state();
        if !state.surface_frame_has_submission(
            &self.surface_handle_allocator,
            &frame,
            submission,
        )? {
            return Err(RhiError::SurfaceFrameSubmissionMissingTarget {
                frame: frame.frame(),
                submission,
            });
        }
        state.terminalize_surface_frame(
            &self.handle_allocator,
            &self.surface_handle_allocator,
            frame.frame(),
            SurfaceFrameTerminal::Presented,
            &mut retired_submissions,
        )?;
        drop(state);
        drop(retired_submissions);
        Ok(SurfacePresentReceipt {
            frame: frame.frame(),
            submission,
            terminal: SurfaceFrameTerminal::Presented,
        })
    }

    fn discard_surface_frame(&self, frame: SurfaceFrameLease) -> Result<(), RhiError> {
        let mut retired_submissions = Vec::new();
        let mut state = self.lock_state();
        state.validate_surface_frame_lease(&self.surface_handle_allocator, &frame)?;
        let result = state.terminalize_surface_frame(
            &self.handle_allocator,
            &self.surface_handle_allocator,
            frame.frame(),
            SurfaceFrameTerminal::Discarded,
            &mut retired_submissions,
        );
        drop(state);
        drop(retired_submissions);
        result
    }

    fn destroy_surface_session(&self, session: SurfaceSession) -> Result<(), RhiError> {
        let mut retired_submissions = Vec::new();
        let result = self.lock_state().destroy_surface_session(
            &self.handle_allocator,
            &self.surface_handle_allocator,
            session,
            &mut retired_submissions,
        );
        drop(retired_submissions);
        result
    }

    fn create_command_list(
        &self,
        queue_class: RenderQueueClass,
        label: &str,
    ) -> Result<Box<dyn CommandList>, RhiError> {
        if !self.caps.supports_queue(queue_class) {
            return Err(RhiError::UnsupportedQueue(queue_class));
        }

        Ok(Box::new(DeterministicRhiContractCommandList::new(
            queue_class,
            label.to_string(),
        )))
    }

    // 接受当前设备/代际的不可变命令快照；接受成功仍不代表镜像内容已经更新。
    fn enqueue_submission_packet(
        &self,
        packet: RhiSubmissionPacket,
    ) -> Result<SubmissionTicket, RhiError> {
        if packet.device_id() != self.device_id || packet.generation() != self.generation {
            return Err(RhiError::SubmissionPacketDeviceMismatch {
                packet_device_id: packet.device_id(),
                packet_generation: packet.generation(),
                device_id: self.device_id,
                generation: self.generation,
            });
        }
        if !self.caps.supports_queue(packet.queue_class()) {
            return Err(RhiError::UnsupportedQueue(packet.queue_class()));
        }
        for command_list in packet.command_lists() {
            self.require_recorded_command_operations(command_list.recorded_commands())?;
        }
        let limits =
            self.caps
                .device_limits
                .as_ref()
                .ok_or_else(|| RhiError::InvalidBindGroupUsage {
                    reason: "deterministic device is missing negotiated binding limits".to_string(),
                })?;
        let mut state = self.lock_state();
        let mut commands = Vec::new();
        for command_list in packet.command_lists() {
            validate_recorded_commands(
                &state,
                command_list.recorded_commands(),
                packet.queue_class(),
                limits,
            )?;
            commands.extend_from_slice(command_list.recorded_commands());
        }
        let ticket = state.allocate_submission_ticket(
            self.device_id,
            self.generation,
            packet.queue_class(),
        )?;
        let (command_lists, diagnostic_query_plan, graph_execution_receipt) = packet.into_parts();
        state.record_surface_frame_submission(ticket, &commands);
        state
            .pending_submissions
            .push(QueuedDeterministicSubmission::Command {
                ticket,
                commands,
                graph_execution_receipt,
            });
        drop(state);
        drop(command_lists);
        drop(diagnostic_query_plan);
        Ok(ticket)
    }

    // 批量执行后只进入已提交状态；某项失败时仍处理剩余项，返回首个执行错误。
    fn flush_submissions(&self) -> Result<usize, RhiError> {
        let mut retired_graph_receipts = Vec::new();
        let mut state = self.lock_state();
        let submissions = std::mem::take(&mut state.pending_submissions);
        let count = submissions.len();
        let mut first_error = None;

        for submission in submissions {
            let ticket = match &submission {
                QueuedDeterministicSubmission::Command { ticket, .. }
                | QueuedDeterministicSubmission::Upload { ticket, .. }
                | QueuedDeterministicSubmission::TextureUpload { ticket, .. } => *ticket,
            };
            state
                .submission_history
                .transition(ticket, SubmissionStatus::Submitted);
            let (result, graph_execution_receipt) = match submission {
                QueuedDeterministicSubmission::Command {
                    commands,
                    graph_execution_receipt,
                    ..
                } => (
                    execute_recorded_commands(&mut state, &commands),
                    graph_execution_receipt,
                ),
                QueuedDeterministicSubmission::Upload { batch, .. } => (
                    uploads::execute_buffer_upload_batch(&mut state, batch),
                    None,
                ),
                QueuedDeterministicSubmission::TextureUpload { batch, .. } => (
                    uploads::execute_texture_upload_batch(&mut state, batch),
                    None,
                ),
            };
            match result {
                Ok(()) => {
                    if let Some(receipt) = graph_execution_receipt {
                        let previous = state.submitted_graph_receipts.insert(ticket, receipt);
                        debug_assert!(previous.is_none());
                    }
                    state.submitted_submissions.push(ticket);
                }
                Err(error) => {
                    state
                        .submission_history
                        .transition(ticket, SubmissionStatus::Failed);
                    if let Some(receipt) = graph_execution_receipt {
                        retired_graph_receipts.push(receipt);
                    }
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }

        drop(state);
        drop(retired_graph_receipts);
        match first_error {
            Some(error) => Err(error),
            None => Ok(count),
        }
    }

    fn submission_status(&self, ticket: SubmissionTicket) -> Result<SubmissionStatus, RhiError> {
        if ticket.device_id() != self.device_id || ticket.generation() != self.generation {
            return Err(RhiError::UnknownSubmissionTicket(ticket));
        }
        let state = self.lock_state();
        state
            .submission_history
            .status(ticket)
            .ok_or(RhiError::UnknownSubmissionTicket(ticket))
    }

    fn append_submission_statuses(
        &self,
        tickets: &[SubmissionTicket],
        statuses: &mut Vec<Result<SubmissionStatus, RhiError>>,
    ) {
        self.lock_state().append_submission_statuses(
            self.device_id,
            self.generation,
            tickets,
            statuses,
        );
    }

    fn cancel_submission(&self, ticket: SubmissionTicket) -> Result<SubmissionStatus, RhiError> {
        if ticket.device_id() != self.device_id || ticket.generation() != self.generation {
            return Err(RhiError::UnknownSubmissionTicket(ticket));
        }
        let mut state = self.lock_state();
        let status = state
            .submission_history
            .status(ticket)
            .ok_or(RhiError::UnknownSubmissionTicket(ticket))?;
        match status {
            SubmissionStatus::Accepted => {
                let pending_index = state
                    .pending_submissions
                    .iter()
                    .position(|submission| match submission {
                        QueuedDeterministicSubmission::Command {
                            ticket: pending_ticket,
                            ..
                        }
                        | QueuedDeterministicSubmission::Upload {
                            ticket: pending_ticket,
                            ..
                        }
                        | QueuedDeterministicSubmission::TextureUpload {
                            ticket: pending_ticket,
                            ..
                        } => *pending_ticket == ticket,
                    })
                    .ok_or(RhiError::UnknownSubmissionTicket(ticket))?;
                let retired_submission = state.pending_submissions.remove(pending_index);
                state
                    .submission_history
                    .transition(ticket, SubmissionStatus::Cancelled);
                drop(state);
                drop(retired_submission);
                Ok(SubmissionStatus::Cancelled)
            }
            SubmissionStatus::Submitted => Err(RhiError::SubmissionCannotCancel { ticket, status }),
            terminal => Ok(terminal),
        }
    }

    // 测试模型把已执行项推进为完成并生成单调回执；不能用它推导原生完成延迟。
    fn poll_submissions(&self) -> Result<SubmissionPollReceipt, RhiError> {
        let mut retired_graph_receipts = Vec::new();
        let mut state = self.lock_state();
        for ticket in std::mem::take(&mut state.submitted_submissions) {
            if state.submission_history.status(ticket) == Some(SubmissionStatus::Submitted) {
                state
                    .submission_history
                    .transition(ticket, SubmissionStatus::Completed);
                if let Some(receipt) = state.submitted_graph_receipts.remove(&ticket) {
                    retired_graph_receipts.push(receipt);
                }
            }
        }
        let result = state.issue_poll_receipt(self.device_id, self.generation);
        drop(state);
        drop(retired_graph_receipts);
        result
    }

    fn transient_allocator_stats(&self) -> TransientAllocatorStats {
        self.lock_state().transient_allocator_stats()
    }

    fn memory_snapshot(&self) -> GpuMemorySnapshot {
        self.lock_state().memory_snapshot()
    }

    fn write_buffer_batch(&self, batch: BufferUploadBatch) -> Result<SubmissionTicket, RhiError> {
        uploads::enqueue_buffer_upload_batch(self, batch)
    }

    fn write_texture_batch(&self, batch: TextureUploadBatch) -> Result<SubmissionTicket, RhiError> {
        uploads::enqueue_texture_upload_batch(self, batch)
    }

    fn read_buffer(
        &self,
        handle: BufferHandle,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, RhiError> {
        let state = self.lock_state();
        let buffer = state
            .buffers
            .get(&handle)
            .ok_or(RhiError::UnknownBuffer(handle.diagnostic_id()))?;
        ensure_buffer_usage(
            handle.diagnostic_id(),
            &buffer.desc,
            BufferUsage::STAGING_READ,
        )?;
        if offset.saturating_add(size) > buffer.desc.size_bytes {
            return Err(RhiError::ReadbackOutOfRange {
                buffer: handle.diagnostic_id(),
                offset,
                size,
            });
        }
        let start = offset as usize;
        let end = start + size as usize;
        Ok(buffer.contents[start..end].to_vec())
    }

    fn read_texture(&self, handle: TextureHandle) -> Result<Vec<u8>, RhiError> {
        let state = self.lock_state();
        let texture = state
            .textures
            .get(&handle)
            .ok_or(RhiError::UnknownTexture(handle.diagnostic_id()))?;
        ensure_texture_usage(
            handle.diagnostic_id(),
            &texture.desc,
            TextureUsage::COPY_SRC,
        )?;
        Ok(texture.contents.clone())
    }
}

#[cfg(test)]
#[path = "../device/tests/cases.rs"]
mod tests;
