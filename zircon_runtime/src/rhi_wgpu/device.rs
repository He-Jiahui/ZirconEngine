use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::rhi::{
    BindGroupDesc, BindGroupHandle, BindGroupLayoutDesc, BindGroupLayoutHandle, BufferDesc,
    BufferHandle, BufferUsage, CommandList, FenceValue, PipelineDesc, PipelineHandle,
    PipelineLayoutDesc, PipelineLayoutHandle, RenderBackendCaps, RenderDevice, RenderQueueClass,
    RhiError, SamplerDesc, SamplerHandle, ShaderModuleDesc, ShaderModuleHandle, TextureDesc,
    TextureHandle, TextureUsage, TransientAllocatorStats,
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

mod command_list;

pub use self::command_list::WgpuCommandList;

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
            caps: wgpu_backend_caps(
                "wgpu",
                wgpu::Features::empty(),
                wgpu::Limits::default(),
                false,
            ),
            state: Arc::new(Mutex::new(WgpuRenderDeviceState {
                next_handle: 1,
                next_fence: 1,
                ..WgpuRenderDeviceState::default()
            })),
        }
    }

    pub fn new_with_surface_support() -> Self {
        Self {
            caps: wgpu_backend_caps(
                "wgpu",
                wgpu::Features::empty(),
                wgpu::Limits::default(),
                true,
            ),
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

    fn lock_state(&self) -> MutexGuard<'_, WgpuRenderDeviceState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl RenderDevice for WgpuRenderDevice {
    fn caps(&self) -> &RenderBackendCaps {
        &self.caps
    }

    fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle, RhiError> {
        validate_buffer_desc(desc)?;
        let mut state = self.lock_state();
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
        let state = self.lock_state();
        state
            .buffers
            .get(&handle)
            .map(|buffer| buffer.desc.clone())
            .ok_or(RhiError::UnknownBuffer(handle.raw()))
    }

    fn destroy_buffer(&self, handle: BufferHandle) -> Result<(), RhiError> {
        let mut state = self.lock_state();
        state
            .buffers
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownBuffer(handle.raw()))
    }

    fn create_texture(&self, desc: &TextureDesc) -> Result<TextureHandle, RhiError> {
        validate_texture_desc(desc, self.caps.supports_sparse_texture)?;
        let mut state = self.lock_state();
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
        let state = self.lock_state();
        state
            .textures
            .get(&handle)
            .map(|texture| texture.desc.clone())
            .ok_or(RhiError::UnknownTexture(handle.raw()))
    }

    fn destroy_texture(&self, handle: TextureHandle) -> Result<(), RhiError> {
        let mut state = self.lock_state();
        state
            .textures
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownTexture(handle.raw()))
    }

    fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerHandle, RhiError> {
        validate_sampler_desc(desc)?;
        let mut state = self.lock_state();
        let handle = SamplerHandle::new(Self::allocate_handle(&mut state));
        state.samplers.insert(handle, desc.clone());
        Ok(handle)
    }

    fn sampler_desc(&self, handle: SamplerHandle) -> Result<SamplerDesc, RhiError> {
        let state = self.lock_state();
        state
            .samplers
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownSampler(handle.raw()))
    }

    fn destroy_sampler(&self, handle: SamplerHandle) -> Result<(), RhiError> {
        let mut state = self.lock_state();
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
        let mut state = self.lock_state();
        let handle = BindGroupLayoutHandle::new(Self::allocate_handle(&mut state));
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
            .ok_or(RhiError::UnknownBindGroupLayout(handle.raw()))
    }

    fn destroy_bind_group_layout(&self, handle: BindGroupLayoutHandle) -> Result<(), RhiError> {
        let mut state = self.lock_state();
        state
            .bind_group_layouts
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownBindGroupLayout(handle.raw()))
    }

    fn create_bind_group(&self, desc: &BindGroupDesc) -> Result<BindGroupHandle, RhiError> {
        let mut state = self.lock_state();
        validate_bind_group_desc(&*state, desc)?;
        let handle = BindGroupHandle::new(Self::allocate_handle(&mut state));
        state
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
            .ok_or(RhiError::UnknownBindGroup(handle.raw()))
    }

    fn destroy_bind_group(&self, handle: BindGroupHandle) -> Result<(), RhiError> {
        let mut state = self.lock_state();
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
        let mut state = self.lock_state();
        let handle = ShaderModuleHandle::new(Self::allocate_handle(&mut state));
        state.shaders.insert(handle, desc.clone());
        Ok(handle)
    }

    fn shader_module_desc(&self, handle: ShaderModuleHandle) -> Result<ShaderModuleDesc, RhiError> {
        let state = self.lock_state();
        state
            .shaders
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownShaderModule(handle.raw()))
    }

    fn destroy_shader_module(&self, handle: ShaderModuleHandle) -> Result<(), RhiError> {
        let mut state = self.lock_state();
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
        let mut state = self.lock_state();
        validate_pipeline_layout_desc(&*state, desc)?;
        let handle = PipelineLayoutHandle::new(Self::allocate_handle(&mut state));
        state.pipeline_layouts.insert(handle, desc.clone());
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
            .ok_or(RhiError::UnknownPipelineLayout(handle.raw()))
    }

    fn destroy_pipeline_layout(&self, handle: PipelineLayoutHandle) -> Result<(), RhiError> {
        let mut state = self.lock_state();
        state
            .pipeline_layouts
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownPipelineLayout(handle.raw()))
    }

    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError> {
        let mut state = self.lock_state();
        validate_pipeline_desc(&*state, desc)?;
        let handle = PipelineHandle::new(Self::allocate_handle(&mut state));
        state.pipelines.insert(handle, desc.clone());
        Ok(handle)
    }

    fn pipeline_desc(&self, handle: PipelineHandle) -> Result<PipelineDesc, RhiError> {
        let state = self.lock_state();
        state
            .pipelines
            .get(&handle)
            .cloned()
            .ok_or(RhiError::UnknownPipeline(handle.raw()))
    }

    fn destroy_pipeline(&self, handle: PipelineHandle) -> Result<(), RhiError> {
        let mut state = self.lock_state();
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
        let mut state = self.lock_state();
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
        let state = self.lock_state();
        if fence.0 == 0 || fence.0 >= state.next_fence {
            return Err(RhiError::UnknownFence(fence.0));
        }
        Ok(fence.0 <= state.completed_fence)
    }

    fn transient_allocator_stats(&self) -> TransientAllocatorStats {
        self.lock_state().transient_allocator_stats()
    }

    fn write_buffer(&self, handle: BufferHandle, offset: u64, data: &[u8]) -> Result<(), RhiError> {
        let mut state = self.lock_state();
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
        let state = self.lock_state();
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
        let state = self.lock_state();
        let texture = state
            .textures
            .get(&handle)
            .ok_or(RhiError::UnknownTexture(handle.raw()))?;
        ensure_texture_usage(handle.raw(), &texture.desc, TextureUsage::COPY_SRC)?;
        Ok(texture.contents.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wgpu_render_device_state_accessors_recover_poisoned_lock() {
        let device = WgpuRenderDevice::new_headless();
        let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _state = device.state.lock().unwrap();
            panic!("poison wgpu render device state lock");
        }));
        assert!(poison.is_err());

        assert_eq!(
            device.transient_allocator_stats(),
            TransientAllocatorStats::default()
        );
        let buffer = device
            .create_buffer(&BufferDesc::new(
                "poisoned-staging",
                4,
                BufferUsage::STAGING_READ | BufferUsage::STAGING_WRITE,
            ))
            .expect("poisoned render device state lock should recover for creates");
        device
            .write_buffer(buffer, 0, &[1, 2, 3, 4])
            .expect("poisoned render device state lock should recover for writes");
        assert_eq!(
            device
                .read_buffer(buffer, 0, 4)
                .expect("poisoned render device state lock should recover for reads"),
            vec![1, 2, 3, 4]
        );
    }
}
