use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::capabilities::{RenderBackendCaps, RenderQueueClass};
use super::descriptors::{
    BufferDesc, BufferUsage, PipelineDesc, SamplerDesc, ShaderModuleDesc, TextureDesc, TextureUsage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferHandle(u64);

impl BufferHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextureHandle(u64);

impl TextureHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SamplerHandle(u64);

impl SamplerHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderModuleHandle(u64);

impl ShaderModuleHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineHandle(u64);

impl PipelineHandle {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RhiError {
    #[error("render queue `{0:?}` is not supported by this backend")]
    UnsupportedQueue(RenderQueueClass),
    #[error("buffer `{0}` does not exist")]
    UnknownBuffer(u64),
    #[error("texture `{0}` does not exist")]
    UnknownTexture(u64),
    #[error("sampler `{0}` does not exist")]
    UnknownSampler(u64),
    #[error("shader module `{0}` does not exist")]
    UnknownShaderModule(u64),
    #[error("pipeline `{0}` does not exist")]
    UnknownPipeline(u64),
    #[error("fence `{0}` was not issued by this device")]
    UnknownFence(u64),
    #[error("invalid buffer descriptor `{label:?}`: {reason}")]
    InvalidBufferDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("invalid texture descriptor `{label:?}`: {reason}")]
    InvalidTextureDescriptor {
        label: Option<String>,
        reason: String,
    },
    #[error("readback range is outside buffer `{buffer}`: offset {offset}, size {size}")]
    ReadbackOutOfRange { buffer: u64, offset: u64, size: u64 },
    #[error("write range is outside buffer `{buffer}`: offset {offset}, size {size}")]
    WriteOutOfRange { buffer: u64, offset: u64, size: u64 },
    #[error("buffer `{buffer}` usage {actual:?} does not include required usage {required:?}")]
    InvalidBufferUsage {
        buffer: u64,
        required: BufferUsage,
        actual: BufferUsage,
    },
    #[error("texture `{texture}` usage {actual:?} does not include required usage {required:?}")]
    InvalidTextureUsage {
        texture: u64,
        required: TextureUsage,
        actual: TextureUsage,
    },
    #[error(
        "buffer copy range is outside source `{source_buffer}` or destination `{destination_buffer}`: source offset {source_offset}, destination offset {destination_offset}, size {size}"
    )]
    BufferCopyOutOfRange {
        source_buffer: u64,
        destination_buffer: u64,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
    },
    #[error(
        "buffer-to-texture copy is outside source `{source_buffer}` or destination `{destination_texture}`: source offset {source_offset}, bytes per row {bytes_per_row}, width {width}, height {height}"
    )]
    BufferToTextureCopyOutOfRange {
        source_buffer: u64,
        destination_texture: u64,
        source_offset: u64,
        bytes_per_row: u64,
        width: u32,
        height: u32,
    },
    #[error(
        "texture-to-buffer copy is outside source `{source_texture}` or destination `{destination_buffer}`: destination offset {destination_offset}, bytes per row {bytes_per_row}, width {width}, height {height}"
    )]
    TextureToBufferCopyOutOfRange {
        source_texture: u64,
        destination_buffer: u64,
        destination_offset: u64,
        bytes_per_row: u64,
        width: u32,
        height: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandListCommand {
    DebugMarker {
        label: String,
    },
    CopyBufferToBuffer {
        source: BufferHandle,
        destination: BufferHandle,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
    },
    CopyBufferToTexture {
        source: BufferHandle,
        destination: TextureHandle,
        source_offset: u64,
        bytes_per_row: u64,
        width: u32,
        height: u32,
    },
    CopyTextureToBuffer {
        source: TextureHandle,
        destination: BufferHandle,
        destination_offset: u64,
        bytes_per_row: u64,
        width: u32,
        height: u32,
    },
}

pub trait CommandList: Send {
    fn queue_class(&self) -> RenderQueueClass;
    fn label(&self) -> Option<&str>;
    fn recorded_commands(&self) -> &[CommandListCommand];

    fn recorded_command_count(&self) -> usize {
        self.recorded_commands().len()
    }

    fn push_debug_marker(&mut self, label: &str);

    fn copy_buffer_to_buffer(
        &mut self,
        source: BufferHandle,
        destination: BufferHandle,
        source_offset: u64,
        destination_offset: u64,
        size: u64,
    );

    fn copy_buffer_to_texture(
        &mut self,
        source: BufferHandle,
        destination: TextureHandle,
        source_offset: u64,
        bytes_per_row: u64,
        width: u32,
        height: u32,
    );

    fn copy_texture_to_buffer(
        &mut self,
        source: TextureHandle,
        destination: BufferHandle,
        destination_offset: u64,
        bytes_per_row: u64,
        width: u32,
        height: u32,
    );
}

pub trait RenderDevice: Send + Sync {
    fn caps(&self) -> &RenderBackendCaps;

    fn backend_name(&self) -> &str {
        &self.caps().backend_name
    }

    fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle, RhiError>;
    fn buffer_desc(&self, handle: BufferHandle) -> Result<BufferDesc, RhiError>;
    fn destroy_buffer(&self, handle: BufferHandle) -> Result<(), RhiError>;
    fn create_texture(&self, desc: &TextureDesc) -> Result<TextureHandle, RhiError>;
    fn texture_desc(&self, handle: TextureHandle) -> Result<TextureDesc, RhiError>;
    fn destroy_texture(&self, handle: TextureHandle) -> Result<(), RhiError>;
    fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerHandle, RhiError>;
    fn sampler_desc(&self, handle: SamplerHandle) -> Result<SamplerDesc, RhiError>;
    fn destroy_sampler(&self, handle: SamplerHandle) -> Result<(), RhiError>;
    fn create_shader_module(&self, desc: &ShaderModuleDesc)
        -> Result<ShaderModuleHandle, RhiError>;
    fn shader_module_desc(&self, handle: ShaderModuleHandle) -> Result<ShaderModuleDesc, RhiError>;
    fn destroy_shader_module(&self, handle: ShaderModuleHandle) -> Result<(), RhiError>;
    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError>;
    fn pipeline_desc(&self, handle: PipelineHandle) -> Result<PipelineDesc, RhiError>;
    fn destroy_pipeline(&self, handle: PipelineHandle) -> Result<(), RhiError>;
    fn create_command_list(
        &self,
        queue_class: RenderQueueClass,
        label: impl Into<String>,
    ) -> Result<Box<dyn CommandList>, RhiError>;
    fn submit(&self, command_list: Box<dyn CommandList>) -> Result<FenceValue, RhiError>;
    fn is_fence_complete(&self, fence: FenceValue) -> Result<bool, RhiError>;
    fn write_buffer(&self, handle: BufferHandle, offset: u64, data: &[u8]) -> Result<(), RhiError>;
    fn read_buffer(
        &self,
        handle: BufferHandle,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, RhiError>;
    fn read_texture(&self, handle: TextureHandle) -> Result<Vec<u8>, RhiError>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FenceValue(pub u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransientAllocatorStats {
    pub bytes_reserved: u64,
    pub allocations: u32,
}
