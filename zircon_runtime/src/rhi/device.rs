use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::capabilities::{RenderBackendCaps, RenderQueueClass};
use super::descriptors::{BufferDesc, PipelineDesc, SamplerDesc, ShaderModuleDesc, TextureDesc};

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
    #[error("readback range is outside buffer `{buffer}`: offset {offset}, size {size}")]
    ReadbackOutOfRange { buffer: u64, offset: u64, size: u64 },
}

pub trait CommandList: Send {
    fn queue_class(&self) -> RenderQueueClass;
    fn label(&self) -> Option<&str>;
}

pub trait RenderDevice: Send + Sync {
    fn caps(&self) -> &RenderBackendCaps;

    fn backend_name(&self) -> &str {
        &self.caps().backend_name
    }

    fn create_buffer(&self, desc: &BufferDesc) -> Result<BufferHandle, RhiError>;
    fn destroy_buffer(&self, handle: BufferHandle) -> Result<(), RhiError>;
    fn create_texture(&self, desc: &TextureDesc) -> Result<TextureHandle, RhiError>;
    fn destroy_texture(&self, handle: TextureHandle) -> Result<(), RhiError>;
    fn create_sampler(&self, desc: &SamplerDesc) -> Result<SamplerHandle, RhiError>;
    fn destroy_sampler(&self, handle: SamplerHandle) -> Result<(), RhiError>;
    fn create_shader_module(&self, desc: &ShaderModuleDesc)
        -> Result<ShaderModuleHandle, RhiError>;
    fn destroy_shader_module(&self, handle: ShaderModuleHandle) -> Result<(), RhiError>;
    fn create_pipeline(&self, desc: &PipelineDesc) -> Result<PipelineHandle, RhiError>;
    fn destroy_pipeline(&self, handle: PipelineHandle) -> Result<(), RhiError>;
    fn create_command_list(
        &self,
        queue_class: RenderQueueClass,
        label: impl Into<String>,
    ) -> Result<Box<dyn CommandList>, RhiError>;
    fn submit(&self, command_list: Box<dyn CommandList>) -> Result<FenceValue, RhiError>;
    fn is_fence_complete(&self, fence: FenceValue) -> Result<bool, RhiError>;
    fn read_buffer(
        &self,
        handle: BufferHandle,
        offset: u64,
        size: u64,
    ) -> Result<Vec<u8>, RhiError>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FenceValue(pub u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransientAllocatorStats {
    pub bytes_reserved: u64,
    pub allocations: u32,
}
