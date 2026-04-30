use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::rhi::{
    BufferDesc, BufferHandle, BufferUsage, CommandList, CommandListCommand, FenceValue,
    PipelineDesc, PipelineHandle, RenderBackendCaps, RenderDevice, RenderQueueClass, RhiError,
    SamplerDesc, SamplerHandle, ShaderModuleDesc, ShaderModuleHandle, TextureDesc, TextureFormat,
    TextureHandle, TextureUsage,
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
    buffers: HashMap<BufferHandle, WgpuBufferResource>,
    textures: HashMap<TextureHandle, WgpuTextureResource>,
    samplers: HashMap<SamplerHandle, SamplerDesc>,
    shaders: HashMap<ShaderModuleHandle, ShaderModuleDesc>,
    pipelines: HashMap<PipelineHandle, PipelineDesc>,
}

#[derive(Clone, Debug)]
struct WgpuBufferResource {
    desc: BufferDesc,
    contents: Vec<u8>,
}

#[derive(Clone, Debug)]
struct WgpuTextureResource {
    desc: TextureDesc,
    contents: Vec<u8>,
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
        validate_texture_desc(desc)?;
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

    fn create_shader_module(
        &self,
        desc: &ShaderModuleDesc,
    ) -> Result<ShaderModuleHandle, RhiError> {
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

    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError> {
        let mut state = self.state.lock().unwrap();
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
        validate_recorded_commands(&state, command_list.recorded_commands())?;
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
        width: u32,
        height: u32,
    ) {
        self.commands.push(CommandListCommand::CopyBufferToTexture {
            source,
            destination,
            source_offset,
            bytes_per_row,
            width,
            height,
        });
    }

    fn copy_texture_to_buffer(
        &mut self,
        source: TextureHandle,
        destination: BufferHandle,
        destination_offset: u64,
        bytes_per_row: u64,
        width: u32,
        height: u32,
    ) {
        self.commands.push(CommandListCommand::CopyTextureToBuffer {
            source,
            destination,
            destination_offset,
            bytes_per_row,
            width,
            height,
        });
    }
}

fn validate_recorded_commands(
    state: &WgpuRenderDeviceState,
    commands: &[CommandListCommand],
) -> Result<(), RhiError> {
    for command in commands {
        match command {
            CommandListCommand::DebugMarker { .. } => {}
            CommandListCommand::CopyBufferToBuffer {
                source,
                destination,
                source_offset,
                destination_offset,
                size,
            } => {
                let source_buffer = state
                    .buffers
                    .get(source)
                    .ok_or(RhiError::UnknownBuffer(source.raw()))?;
                let destination_buffer = state
                    .buffers
                    .get(destination)
                    .ok_or(RhiError::UnknownBuffer(destination.raw()))?;
                ensure_buffer_usage(source.raw(), &source_buffer.desc, BufferUsage::COPY_SRC)?;
                ensure_buffer_usage(
                    destination.raw(),
                    &destination_buffer.desc,
                    BufferUsage::COPY_DST,
                )?;
                let source_end = source_offset.saturating_add(*size);
                let destination_end = destination_offset.saturating_add(*size);
                if source_end > source_buffer.desc.size_bytes
                    || destination_end > destination_buffer.desc.size_bytes
                {
                    return Err(RhiError::BufferCopyOutOfRange {
                        source_buffer: source.raw(),
                        destination_buffer: destination.raw(),
                        source_offset: *source_offset,
                        destination_offset: *destination_offset,
                        size: *size,
                    });
                }
            }
            CommandListCommand::CopyBufferToTexture {
                source,
                destination,
                source_offset,
                bytes_per_row,
                width,
                height,
            } => {
                let source_buffer = state
                    .buffers
                    .get(source)
                    .ok_or(RhiError::UnknownBuffer(source.raw()))?;
                let destination_texture = state
                    .textures
                    .get(destination)
                    .ok_or(RhiError::UnknownTexture(destination.raw()))?;
                ensure_buffer_usage(source.raw(), &source_buffer.desc, BufferUsage::COPY_SRC)?;
                ensure_texture_usage(
                    destination.raw(),
                    &destination_texture.desc,
                    TextureUsage::COPY_DST,
                )?;
                let row_size = u64::from(*width)
                    * u64::from(texture_bytes_per_pixel(destination_texture.desc.format));
                let copy_size = buffer_to_texture_copy_size(*height, *bytes_per_row, row_size);
                if *width > destination_texture.desc.width
                    || *height > destination_texture.desc.height
                    || *bytes_per_row < row_size
                    || source_offset.saturating_add(copy_size) > source_buffer.desc.size_bytes
                {
                    return Err(RhiError::BufferToTextureCopyOutOfRange {
                        source_buffer: source.raw(),
                        destination_texture: destination.raw(),
                        source_offset: *source_offset,
                        bytes_per_row: *bytes_per_row,
                        width: *width,
                        height: *height,
                    });
                }
            }
            CommandListCommand::CopyTextureToBuffer {
                source,
                destination,
                destination_offset,
                bytes_per_row,
                width,
                height,
            } => {
                let source_texture = state
                    .textures
                    .get(source)
                    .ok_or(RhiError::UnknownTexture(source.raw()))?;
                let destination_buffer = state
                    .buffers
                    .get(destination)
                    .ok_or(RhiError::UnknownBuffer(destination.raw()))?;
                ensure_texture_usage(source.raw(), &source_texture.desc, TextureUsage::COPY_SRC)?;
                ensure_buffer_usage(
                    destination.raw(),
                    &destination_buffer.desc,
                    BufferUsage::COPY_DST,
                )?;
                let row_size = u64::from(*width)
                    * u64::from(texture_bytes_per_pixel(source_texture.desc.format));
                let copy_size = buffer_to_texture_copy_size(*height, *bytes_per_row, row_size);
                if *width > source_texture.desc.width
                    || *height > source_texture.desc.height
                    || *bytes_per_row < row_size
                    || destination_offset.saturating_add(copy_size)
                        > destination_buffer.desc.size_bytes
                {
                    return Err(RhiError::TextureToBufferCopyOutOfRange {
                        source_texture: source.raw(),
                        destination_buffer: destination.raw(),
                        destination_offset: *destination_offset,
                        bytes_per_row: *bytes_per_row,
                        width: *width,
                        height: *height,
                    });
                }
            }
        }
    }
    Ok(())
}

fn execute_recorded_commands(
    state: &mut WgpuRenderDeviceState,
    commands: &[CommandListCommand],
) -> Result<(), RhiError> {
    for command in commands {
        match command {
            CommandListCommand::DebugMarker { .. } => {}
            CommandListCommand::CopyBufferToBuffer {
                source,
                destination,
                source_offset,
                destination_offset,
                size,
            } => {
                let source_start = *source_offset as usize;
                let source_end = source_start + *size as usize;
                let destination_start = *destination_offset as usize;
                let destination_end = destination_start + *size as usize;
                let bytes = state
                    .buffers
                    .get(source)
                    .ok_or(RhiError::UnknownBuffer(source.raw()))?
                    .contents[source_start..source_end]
                    .to_vec();
                state
                    .buffers
                    .get_mut(destination)
                    .ok_or(RhiError::UnknownBuffer(destination.raw()))?
                    .contents[destination_start..destination_end]
                    .copy_from_slice(&bytes);
            }
            CommandListCommand::CopyBufferToTexture {
                source,
                destination,
                source_offset,
                bytes_per_row,
                width,
                height,
            } => {
                let destination_desc = state
                    .textures
                    .get(destination)
                    .ok_or(RhiError::UnknownTexture(destination.raw()))?
                    .desc
                    .clone();
                let bytes_per_pixel = texture_bytes_per_pixel(destination_desc.format) as usize;
                let row_size = *width as usize * bytes_per_pixel;
                let source_offset = *source_offset as usize;
                let bytes_per_row = *bytes_per_row as usize;
                let source_contents = state
                    .buffers
                    .get(source)
                    .ok_or(RhiError::UnknownBuffer(source.raw()))?
                    .contents
                    .clone();
                let destination_stride = destination_desc.width as usize * bytes_per_pixel;
                let destination_texture = state
                    .textures
                    .get_mut(destination)
                    .ok_or(RhiError::UnknownTexture(destination.raw()))?;
                for row in 0..*height as usize {
                    let source_start = source_offset + row * bytes_per_row;
                    let source_end = source_start + row_size;
                    let destination_start = row * destination_stride;
                    let destination_end = destination_start + row_size;
                    destination_texture.contents[destination_start..destination_end]
                        .copy_from_slice(&source_contents[source_start..source_end]);
                }
            }
            CommandListCommand::CopyTextureToBuffer {
                source,
                destination,
                destination_offset,
                bytes_per_row,
                width,
                height,
            } => {
                let source_texture = state
                    .textures
                    .get(source)
                    .ok_or(RhiError::UnknownTexture(source.raw()))?
                    .clone();
                let bytes_per_pixel = texture_bytes_per_pixel(source_texture.desc.format) as usize;
                let row_size = *width as usize * bytes_per_pixel;
                let source_stride = source_texture.desc.width as usize * bytes_per_pixel;
                let destination_offset = *destination_offset as usize;
                let bytes_per_row = *bytes_per_row as usize;
                let destination_buffer = state
                    .buffers
                    .get_mut(destination)
                    .ok_or(RhiError::UnknownBuffer(destination.raw()))?;
                for row in 0..*height as usize {
                    let source_start = row * source_stride;
                    let source_end = source_start + row_size;
                    let destination_start = destination_offset + row * bytes_per_row;
                    let destination_end = destination_start + row_size;
                    destination_buffer.contents[destination_start..destination_end]
                        .copy_from_slice(&source_texture.contents[source_start..source_end]);
                }
            }
        }
    }
    Ok(())
}

fn ensure_buffer_usage(
    handle: u64,
    desc: &BufferDesc,
    required: BufferUsage,
) -> Result<(), RhiError> {
    if desc.usage.contains(required) {
        Ok(())
    } else {
        Err(RhiError::InvalidBufferUsage {
            buffer: handle,
            required,
            actual: desc.usage,
        })
    }
}

fn validate_buffer_desc(desc: &BufferDesc) -> Result<(), RhiError> {
    if desc.size_bytes == 0 {
        return Err(RhiError::InvalidBufferDescriptor {
            label: desc.label.clone(),
            reason: "size_bytes must be greater than zero".to_string(),
        });
    }
    if desc.usage == BufferUsage::NONE {
        return Err(RhiError::InvalidBufferDescriptor {
            label: desc.label.clone(),
            reason: "usage must not be empty".to_string(),
        });
    }
    Ok(())
}

fn validate_texture_desc(desc: &TextureDesc) -> Result<(), RhiError> {
    if desc.width == 0 || desc.height == 0 || desc.depth == 0 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "width, height, and depth must be greater than zero".to_string(),
        });
    }
    if desc.mip_levels == 0 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "mip_levels must be greater than zero".to_string(),
        });
    }
    if desc.sample_count == 0 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "sample_count must be greater than zero".to_string(),
        });
    }
    if desc.usage == TextureUsage::NONE {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "usage must not be empty".to_string(),
        });
    }
    let Some(storage_size) = checked_texture_storage_size(desc) else {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "storage size overflows u64".to_string(),
        });
    };
    if storage_size > usize::MAX as u64 {
        return Err(RhiError::InvalidTextureDescriptor {
            label: desc.label.clone(),
            reason: "storage size exceeds addressable memory".to_string(),
        });
    }
    Ok(())
}

fn ensure_texture_usage(
    handle: u64,
    desc: &TextureDesc,
    required: TextureUsage,
) -> Result<(), RhiError> {
    if desc.usage.contains(required) {
        Ok(())
    } else {
        Err(RhiError::InvalidTextureUsage {
            texture: handle,
            required,
            actual: desc.usage,
        })
    }
}

fn texture_bytes_per_pixel(format: TextureFormat) -> u32 {
    match format {
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Depth32Float => 4,
    }
}

fn texture_storage_size(desc: &TextureDesc) -> u64 {
    checked_texture_storage_size(desc).unwrap_or(u64::MAX)
}

fn checked_texture_storage_size(desc: &TextureDesc) -> Option<u64> {
    u64::from(desc.width)
        .checked_mul(u64::from(desc.height))?
        .checked_mul(u64::from(desc.depth))?
        .checked_mul(u64::from(texture_bytes_per_pixel(desc.format)))
}

fn buffer_to_texture_copy_size(height: u32, bytes_per_row: u64, row_size: u64) -> u64 {
    if height == 0 {
        0
    } else {
        u64::from(height - 1)
            .saturating_mul(bytes_per_row)
            .saturating_add(row_size)
    }
}
