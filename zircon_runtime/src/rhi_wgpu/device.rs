use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::rhi::{
    BufferDesc, BufferHandle, CommandList, FenceValue, PipelineDesc, PipelineHandle,
    RenderBackendCaps, RenderDevice, RenderQueueClass, RhiError, SamplerDesc, SamplerHandle,
    ShaderModuleDesc, ShaderModuleHandle, TextureDesc, TextureHandle,
};

use super::capabilities::wgpu_backend_caps;

#[derive(Clone, Debug)]
pub struct WgpuRenderDevice {
    caps: RenderBackendCaps,
    state: Arc<Mutex<WgpuRenderDeviceState>>,
}

#[derive(Clone, Debug, Default)]
struct WgpuRenderDeviceState {
    next_handle: u64,
    next_fence: u64,
    completed_fence: u64,
    buffers: HashMap<BufferHandle, BufferDesc>,
    textures: HashMap<TextureHandle, TextureDesc>,
    samplers: HashMap<SamplerHandle, SamplerDesc>,
    shaders: HashMap<ShaderModuleHandle, ShaderModuleDesc>,
    pipelines: HashMap<PipelineHandle, PipelineDesc>,
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
        let mut state = self.state.lock().unwrap();
        let handle = BufferHandle::new(Self::allocate_handle(&mut state));
        state.buffers.insert(handle, desc.clone());
        Ok(handle)
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
        let mut state = self.state.lock().unwrap();
        let handle = TextureHandle::new(Self::allocate_handle(&mut state));
        state.textures.insert(handle, desc.clone());
        Ok(handle)
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
        let mut state = self.state.lock().unwrap();
        let handle = SamplerHandle::new(Self::allocate_handle(&mut state));
        state.samplers.insert(handle, desc.clone());
        Ok(handle)
    }

    fn destroy_sampler(&self, handle: SamplerHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .samplers
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownSampler(handle.raw()))
    }

    fn create_shader_module(
        &self,
        desc: &ShaderModuleDesc,
    ) -> Result<ShaderModuleHandle, RhiError> {
        let mut state = self.state.lock().unwrap();
        let handle = ShaderModuleHandle::new(Self::allocate_handle(&mut state));
        state.shaders.insert(handle, desc.clone());
        Ok(handle)
    }

    fn destroy_shader_module(&self, handle: ShaderModuleHandle) -> Result<(), RhiError> {
        let mut state = self.state.lock().unwrap();
        state
            .shaders
            .remove(&handle)
            .map(|_| ())
            .ok_or(RhiError::UnknownShaderModule(handle.raw()))
    }

    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError> {
        let mut state = self.state.lock().unwrap();
        let handle = PipelineHandle::new(Self::allocate_handle(&mut state));
        state.pipelines.insert(handle, desc.clone());
        Ok(handle)
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
        let fence = FenceValue(state.next_fence);
        state.next_fence += 1;
        state.completed_fence = fence.0;
        Ok(fence)
    }

    fn is_fence_complete(&self, fence: FenceValue) -> Result<bool, RhiError> {
        let state = self.state.lock().unwrap();
        Ok(fence.0 <= state.completed_fence)
    }

    fn read_buffer(
        &self,
        handle: BufferHandle,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, RhiError> {
        let state = self.state.lock().unwrap();
        let desc = state
            .buffers
            .get(&handle)
            .ok_or(RhiError::UnknownBuffer(handle.raw()))?;
        if offset.saturating_add(size) > desc.size_bytes {
            return Err(RhiError::ReadbackOutOfRange {
                buffer: handle.raw(),
                offset,
                size,
            });
        }
        Ok(vec![0; size as usize])
    }
}

#[derive(Clone, Debug)]
pub struct WgpuCommandList {
    queue_class: RenderQueueClass,
    label: Option<String>,
}

impl WgpuCommandList {
    pub fn new(queue_class: RenderQueueClass, label: impl Into<String>) -> Self {
        Self {
            queue_class,
            label: Some(label.into()),
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
}
