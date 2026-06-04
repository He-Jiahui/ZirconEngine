use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::rhi::{
    BindGroupDesc, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutHandle, BufferDesc,
    BufferHandle, BufferUsage, CommandList, CommandListCommand, FenceValue, IndexFormat,
    PipelineDesc, PipelineHandle, PipelineLayoutDesc, PipelineLayoutHandle, RenderBackendCaps,
    RenderDevice, RenderPassColorAttachmentDesc, RenderPassDepthStencilAttachmentDesc,
    RenderQueueClass, RenderScissorRect, RenderViewportDesc, RhiError, SamplerDesc, SamplerHandle,
    ShaderModuleDesc, ShaderModuleHandle, TextureCopyRegion, TextureDesc, TextureHandle,
    TextureUsage, TransientAllocatorStats,
};

use super::bind_group_validation::{validate_bind_group_desc, BindGroupResourceLookup};
use super::capabilities::wgpu_backend_caps;
use super::command_validation::{execute_recorded_commands, validate_recorded_commands};
use super::pipeline_validation::{
    validate_pipeline_desc, validate_pipeline_layout_desc, validate_shader_module_desc,
    PipelineResourceLookup,
};
use super::resource_validation::{
    ensure_buffer_usage, ensure_texture_usage, texture_storage_size,
    validate_bind_group_layout_desc, validate_buffer_desc, validate_sampler_desc,
    validate_texture_desc,
};

#[derive(Clone, Debug)]
pub struct WgpuRenderDevice {
    caps: RenderBackendCaps,
    state: Arc<Mutex<WgpuRenderDeviceState>>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct WgpuRenderDeviceState {
    next_handle: u64,
    next_fence: u64,
    completed_fence: u64,
    pub(super) buffers: HashMap<BufferHandle, WgpuBufferResource>,
    pub(super) textures: HashMap<TextureHandle, WgpuTextureResource>,
    samplers: HashMap<SamplerHandle, SamplerDesc>,
    bind_group_layouts: HashMap<BindGroupLayoutHandle, BindGroupLayoutDesc>,
    bind_groups: HashMap<BindGroupHandle, WgpuBindGroupResource>,
    shaders: HashMap<ShaderModuleHandle, ShaderModuleDesc>,
    pipeline_layouts: HashMap<PipelineLayoutHandle, PipelineLayoutDesc>,
    pub(super) pipelines: HashMap<PipelineHandle, PipelineDesc>,
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

impl WgpuRenderDeviceState {
    pub(super) fn bind_group_desc_ref(
        &self,
        handle: BindGroupHandle,
    ) -> Result<&BindGroupDesc, RhiError> {
        self.bind_groups
            .get(&handle)
            .map(|bind_group| &bind_group.desc)
            .ok_or(RhiError::UnknownBindGroup(handle.raw()))
    }

    pub(super) fn pipeline_layout_desc_ref(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<&PipelineLayoutDesc, RhiError> {
        self.pipeline_layouts
            .get(&handle)
            .ok_or(RhiError::UnknownPipelineLayout(handle.raw()))
    }

    pub(super) fn texture_desc_ref(&self, handle: TextureHandle) -> Result<&TextureDesc, RhiError> {
        self.textures
            .get(&handle)
            .map(|texture| &texture.desc)
            .ok_or(RhiError::UnknownTexture(handle.raw()))
    }

    fn transient_allocator_stats(&self) -> TransientAllocatorStats {
        let buffer_bytes = self
            .buffers
            .values()
            .map(|buffer| buffer.desc.size_bytes)
            .sum::<u64>();
        let texture_bytes = self
            .textures
            .values()
            .map(|texture| texture.contents.len() as u64)
            .sum::<u64>();
        TransientAllocatorStats {
            bytes_reserved: buffer_bytes.saturating_add(texture_bytes),
            allocations: self.buffers.len().saturating_add(self.textures.len()) as u32,
        }
    }
}

impl BindGroupResourceLookup for WgpuRenderDeviceState {
    fn layout_desc(&self, handle: BindGroupLayoutHandle) -> Result<&BindGroupLayoutDesc, RhiError> {
        self.bind_group_layouts
            .get(&handle)
            .ok_or(RhiError::UnknownBindGroupLayout(handle.raw()))
    }

    fn buffer_desc(&self, handle: BufferHandle) -> Result<&BufferDesc, RhiError> {
        self.buffers
            .get(&handle)
            .map(|buffer| &buffer.desc)
            .ok_or(RhiError::UnknownBuffer(handle.raw()))
    }

    fn texture_desc(&self, handle: TextureHandle) -> Result<&TextureDesc, RhiError> {
        self.textures
            .get(&handle)
            .map(|texture| &texture.desc)
            .ok_or(RhiError::UnknownTexture(handle.raw()))
    }

    fn sampler_desc(&self, handle: SamplerHandle) -> Result<&SamplerDesc, RhiError> {
        self.samplers
            .get(&handle)
            .ok_or(RhiError::UnknownSampler(handle.raw()))
    }
}

impl PipelineResourceLookup for WgpuRenderDeviceState {
    fn bind_group_layout_exists(&self, handle: BindGroupLayoutHandle) -> bool {
        self.bind_group_layouts.contains_key(&handle)
    }

    fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<&PipelineLayoutDesc, RhiError> {
        self.pipeline_layouts
            .get(&handle)
            .ok_or(RhiError::UnknownPipelineLayout(handle.raw()))
    }

    fn shader_module_desc(
        &self,
        handle: ShaderModuleHandle,
    ) -> Result<&ShaderModuleDesc, RhiError> {
        self.shaders
            .get(&handle)
            .ok_or(RhiError::UnknownShaderModule(handle.raw()))
    }
}

impl WgpuRenderDevice {
    pub fn new_headless() -> Self {
        Self {
            caps: wgpu_backend_caps("wgpu", wgpu::Features::empty(), false),
            state: Arc::new(Mutex::new(WgpuRenderDeviceState {
                next_handle: 1,
                next_fence: 1,
                ..WgpuRenderDeviceState::default()
            })),
        }
    }

    pub fn new_with_surface_support() -> Self {
        Self {
            caps: wgpu_backend_caps("wgpu", wgpu::Features::empty(), true),
            state: Arc::new(Mutex::new(WgpuRenderDeviceState {
                next_handle: 1,
                next_fence: 1,
                ..WgpuRenderDeviceState::default()
            })),
        }
    }

    fn allocate_handle(state: &mut WgpuRenderDeviceState) -> u64 {
        let handle = state.next_handle;
        state.next_handle += 1;
        handle
    }
}

impl RenderDevice for WgpuRenderDevice {
    fn caps(&self) -> &RenderBackendCaps {
        &self.caps
    }

    fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle, RhiError> {
        validate_buffer_desc(desc)?;
        let mut state = self.state.lock().unwrap();
        let handle = BufferHandle::new(Self::allocate_handle(&mut state));
        state.buffers.insert(
            handle,
            WgpuBufferResource {
                desc: desc.clone(),
                contents: vec![0; desc.size_bytes as usize],
            },
        );
        Ok(handle)
    }

    fn buffer_desc(&self, handle: BufferHandle) -> Result<BufferDesc, RhiError> {
        let state = self.state.lock().unwrap();
        state
            .buffers
            .get(&handle)
            .map(|buffer| buffer.desc.clone())
            .ok_or(RhiError::UnknownBuffer(handle.raw()))
    }

    fn destroy_buffer(&self, handle: BufferHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .buffers
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownBuffer(handle.raw()))
    }

    fn create_texture(&self, desc: &TextureDesc) -> Result<TextureHandle, RhiError> {
        validate_texture_desc(desc, self.caps.supports_sparse_texture)?;
        let mut state = self.state.lock().unwrap();
        let handle = TextureHandle::new(Self::allocate_handle(&mut state));
        state.textures.insert(
            handle,
            WgpuTextureResource {
                desc: desc.clone(),
                contents: vec![0; texture_storage_size(desc) as usize],
            },
        );
        Ok(handle)
    }

    fn texture_desc(&self, handle: TextureHandle) -> Result<TextureDesc, RhiError> {
        let state = self.state.lock().unwrap();
        state
            .textures
            .get(&handle)
            .map(|texture| texture.desc.clone())
            .ok_or(RhiError::UnknownTexture(handle.raw()))
    }

    fn destroy_texture(&self, handle: TextureHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .textures
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownTexture(handle.raw()))
    }

    fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerHandle, RhiError> {
        validate_sampler_desc(desc)?;
        let mut state = self.state.lock().unwrap();
        let handle = SamplerHandle::new(Self::allocate_handle(&mut state));
        state.samplers.insert(handle, desc.clone());
        Ok(handle)
    }

    fn sampler_desc(&self, handle: SamplerHandle) -> Result<SamplerDesc, RhiError> {
        let state = self.state.lock().unwrap();
        state
            .samplers
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownSampler(handle.raw()))
    }

    fn destroy_sampler(&self, handle: SamplerHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .samplers
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownSampler(handle.raw()))
    }

    fn create_bind_group_layout(
        &self,
        desc: &BindGroupLayoutDesc,
    ) -> Result<BindGroupLayoutHandle, RhiError> {
        validate_bind_group_layout_desc(desc)?;
        let mut state = self.state.lock().unwrap();
        let handle = BindGroupLayoutHandle::new(Self::allocate_handle(&mut state));
        state.bind_group_layouts.insert(handle, desc.clone());
        Ok(handle)
    }

    fn bind_group_layout_desc(
        &self,
        handle: BindGroupLayoutHandle,
    ) -> Result<BindGroupLayoutDesc, RhiError> {
        let state = self.state.lock().unwrap();
        state
            .bind_group_layouts
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownBindGroupLayout(handle.raw()))
    }

    fn destroy_bind_group_layout(&self, handle: BindGroupLayoutHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .bind_group_layouts
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownBindGroupLayout(handle.raw()))
    }

    fn create_bind_group(&self, desc: &BindGroupDesc) -> Result<BindGroupHandle, RhiError> {
        let mut state = self.state.lock().unwrap();
        validate_bind_group_desc(&*state, desc)?;
        let handle = BindGroupHandle::new(Self::allocate_handle(&mut state));
        state
            .bind_groups
            .insert(handle, WgpuBindGroupResource { desc: desc.clone() });
        Ok(handle)
    }

    fn bind_group_desc(&self, handle: BindGroupHandle) -> Result<BindGroupDesc, RhiError> {
        let state = self.state.lock().unwrap();
        state
            .bind_groups
            .get(&handle)
            .map(|bind_group| bind_group.desc.clone())
            .ok_or(RhiError::UnknownBindGroup(handle.raw()))
    }

    fn destroy_bind_group(&self, handle: BindGroupHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .bind_groups
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownBindGroup(handle.raw()))
    }

    fn create_shader_module(
        &self,
        desc: &ShaderModuleDesc,
    ) -> Result<ShaderModuleHandle, RhiError> {
        validate_shader_module_desc(desc)?;
        let mut state = self.state.lock().unwrap();
        let handle = ShaderModuleHandle::new(Self::allocate_handle(&mut state));
        state.shaders.insert(handle, desc.clone());
        Ok(handle)
    }

    fn shader_module_desc(&self, handle: ShaderModuleHandle) -> Result<ShaderModuleDesc, RhiError> {
        let state = self.state.lock().unwrap();
        state
            .shaders
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownShaderModule(handle.raw()))
    }

    fn destroy_shader_module(&self, handle: ShaderModuleHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .shaders
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownShaderModule(handle.raw()))
    }

    fn create_pipeline_layout(
        &self,
        desc: &PipelineLayoutDesc,
    ) -> Result<PipelineLayoutHandle, RhiError> {
        let mut state = self.state.lock().unwrap();
        validate_pipeline_layout_desc(&*state, desc)?;
        let handle = PipelineLayoutHandle::new(Self::allocate_handle(&mut state));
        state.pipeline_layouts.insert(handle, desc.clone());
        Ok(handle)
    }

    fn pipeline_layout_desc(
        &self,
        handle: PipelineLayoutHandle,
    ) -> Result<PipelineLayoutDesc, RhiError> {
        let state = self.state.lock().unwrap();
        state
            .pipeline_layouts
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownPipelineLayout(handle.raw()))
    }

    fn destroy_pipeline_layout(&self, handle: PipelineLayoutHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .pipeline_layouts
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownPipelineLayout(handle.raw()))
    }

    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError> {
        let mut state = self.state.lock().unwrap();
        validate_pipeline_desc(&*state, desc)?;
        let handle = PipelineHandle::new(Self::allocate_handle(&mut state));
        state.pipelines.insert(handle, desc.clone());
        Ok(handle)
    }

    fn pipeline_desc(&self, handle: PipelineHandle) -> Result<PipelineDesc, RhiError> {
        let state = self.state.lock().unwrap();
        state
            .pipelines
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownPipeline(handle.raw()))
    }

    fn destroy_pipeline(&self, handle: PipelineHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .pipelines
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownPipeline(handle.raw()))
    }

    fn create_command_list(
        &self,
        queue_class: RenderQueueClass,
        label: impl Into<String>,
    ) -> Result<Box<dyn CommandList>, RhiError> {
        if !self.caps.supports_queue(queue_class) {
            return Err(RhiError::UnsupportedQueue(queue_class));
        }

        Ok(Box::new(WgpuCommandList::new(queue_class, label)))
    }

    fn submit(&self, command_list: Box<dyn CommandList>) -> Result<FenceValue, RhiError> {
        if !self.caps.supports_queue(command_list.queue_class()) {
            return Err(RhiError::UnsupportedQueue(command_list.queue_class()));
        }
        let mut state = self.state.lock().unwrap();
        validate_recorded_commands(
            &state,
            command_list.recorded_commands(),
            command_list.queue_class(),
        )?;
        execute_recorded_commands(&mut state, command_list.recorded_commands())?;
        let fence = FenceValue(state.next_fence);
        state.next_fence += 1;
        state.completed_fence = fence.0;
        Ok(fence)
    }

    fn is_fence_complete(&self, fence: FenceValue) -> Result<bool, RhiError> {
        let state = self.state.lock().unwrap();
        if fence.0 == 0 || fence.0 >= state.next_fence {
            return Err(RhiError::UnknownFence(fence.0));
        }
        Ok(fence.0 <= state.completed_fence)
    }

    fn transient_allocator_stats(&self) -> TransientAllocatorStats {
        self.state.lock().unwrap().transient_allocator_stats()
    }

    fn write_buffer(&self, handle: BufferHandle, offset: u64, data: &[u8]) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        let buffer = state
            .buffers
            .get_mut(&handle)
            .ok_or(RhiError::UnknownBuffer(handle.raw()))?;
        ensure_buffer_usage(handle.raw(), &buffer.desc, BufferUsage::STAGING_WRITE)?;
        let size = data.len() as u64;
        if offset.saturating_add(size) > buffer.desc.size_bytes {
            return Err(RhiError::WriteOutOfRange {
                buffer: handle.raw(),
                offset,
                size,
            });
        }
        let start = offset as usize;
        let end = start + data.len();
        buffer.contents[start..end].copy_from_slice(data);
        Ok(())
    }

    fn read_buffer(
        &self,
        handle: BufferHandle,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, RhiError> {
        let state = self.state.lock().unwrap();
        let buffer = state
            .buffers
            .get(&handle)
            .ok_or(RhiError::UnknownBuffer(handle.raw()))?;
        ensure_buffer_usage(handle.raw(), &buffer.desc, BufferUsage::STAGING_READ)?;
        if offset.saturating_add(size) > buffer.desc.size_bytes {
            return Err(RhiError::ReadbackOutOfRange {
                buffer: handle.raw(),
                offset,
                size,
            });
        }
        let start = offset as usize;
        let end = start + size as usize;
        Ok(buffer.contents[start..end].to_vec())
    }

    fn read_texture(&self, handle: TextureHandle) -> Result<Vec<u8>, RhiError> {
        let state = self.state.lock().unwrap();
        let texture = state
            .textures
            .get(&handle)
            .ok_or(RhiError::UnknownTexture(handle.raw()))?;
        ensure_texture_usage(handle.raw(), &texture.desc, TextureUsage::COPY_SRC)?;
        Ok(texture.contents.clone())
    }
}

#[derive(Clone, Debug)]
pub struct WgpuCommandList {
    queue_class: RenderQueueClass,
    label: Option<String>,
    commands: Vec<CommandListCommand>,
}

impl WgpuCommandList {
    pub fn new(queue_class: RenderQueueClass, label: impl Into<String>) -> Self {
        Self {
            queue_class,
            label: Some(label.into()),
            commands: Vec::new(),
        }
    }
}

impl CommandList for WgpuCommandList {
    fn queue_class(&self) -> RenderQueueClass {
        self.queue_class
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn recorded_commands(&self) -> &[CommandListCommand] {
        &self.commands
    }

    fn push_debug_marker(&mut self, label: &str) {
        self.commands.push(CommandListCommand::DebugMarker {
            label: label.to_string(),
        });
    }

    fn push_debug_group(&mut self, label: &str) {
        self.commands.push(CommandListCommand::PushDebugGroup {
            label: label.to_string(),
        });
    }

    fn pop_debug_group(&mut self) {
        self.commands.push(CommandListCommand::PopDebugGroup);
    }

    fn copy_buffer_to_buffer(
        &mut self,
        source: BufferHandle,
        destination: BufferHandle,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
    ) {
        self.commands.push(CommandListCommand::CopyBufferToBuffer {
            source,
            destination,
            source_offset,
            destination_offset,
            size,
        });
    }

    fn copy_buffer_to_texture(
        &mut self,
        source: BufferHandle,
        destination: TextureHandle,
        source_offset: u64,
        bytes_per_row: u64,
        region: TextureCopyRegion,
    ) {
        self.commands.push(CommandListCommand::CopyBufferToTexture {
            source,
            destination,
            source_offset,
            bytes_per_row,
            region,
        });
    }

    fn copy_texture_to_buffer(
        &mut self,
        source: TextureHandle,
        destination: BufferHandle,
        destination_offset: u64,
        bytes_per_row: u64,
        region: TextureCopyRegion,
    ) {
        self.commands.push(CommandListCommand::CopyTextureToBuffer {
            source,
            destination,
            destination_offset,
            bytes_per_row,
            region,
        });
    }

    fn begin_render_pass(
        &mut self,
        label: &str,
        color_attachments: Vec<RenderPassColorAttachmentDesc>,
        depth_stencil_attachment: Option<RenderPassDepthStencilAttachmentDesc>,
    ) {
        self.commands.push(CommandListCommand::BeginRenderPass {
            label: label.to_string(),
            color_attachments,
            depth_stencil_attachment,
        });
    }

    fn end_render_pass(&mut self) {
        self.commands.push(CommandListCommand::EndRenderPass);
    }

    fn set_pipeline(&mut self, pipeline: PipelineHandle) {
        self.commands
            .push(CommandListCommand::SetPipeline { pipeline });
    }

    fn set_bind_group(&mut self, slot: u32, bind_group: BindGroupHandle) {
        self.commands
            .push(CommandListCommand::SetBindGroup { slot, bind_group });
    }

    fn set_viewport(&mut self, viewport: RenderViewportDesc) {
        self.commands
            .push(CommandListCommand::SetViewport { viewport });
    }

    fn set_scissor_rect(&mut self, rect: RenderScissorRect) {
        self.commands
            .push(CommandListCommand::SetScissorRect { rect });
    }

    fn set_vertex_buffer(&mut self, slot: u32, buffer: BufferHandle, offset: u64, size: u64) {
        self.commands.push(CommandListCommand::SetVertexBuffer {
            slot,
            buffer,
            offset,
            size,
        });
    }

    fn set_index_buffer(
        &mut self,
        buffer: BufferHandle,
        offset: u64,
        size: u64,
        format: IndexFormat,
    ) {
        self.commands.push(CommandListCommand::SetIndexBuffer {
            buffer,
            offset,
            size,
            format,
        });
    }

    fn draw(
        &mut self,
        vertex_start: u32,
        vertex_count: u32,
        instance_start: u32,
        instance_count: u32,
    ) {
        self.commands.push(CommandListCommand::Draw {
            vertex_start,
            vertex_count,
            instance_start,
            instance_count,
        });
    }

    fn draw_indexed(
        &mut self,
        index_start: u32,
        index_count: u32,
        base_vertex: i32,
        instance_start: u32,
        instance_count: u32,
    ) {
        self.commands.push(CommandListCommand::DrawIndexed {
            index_start,
            index_count,
            base_vertex,
            instance_start,
            instance_count,
        });
    }

    fn dispatch_compute(&mut self, x: u32, y: u32, z: u32) {
        self.commands
            .push(CommandListCommand::DispatchCompute { x, y, z });
    }
}
