use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BufferUsage {
    Vertex,
    Index,
    Uniform,
    Storage,
    StagingRead,
    StagingWrite,
    Indirect,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BufferDesc {
    pub label: Option<String>,
    pub size_bytes: u64,
    pub usage: BufferUsage,
}

impl BufferDesc {
    pub fn new(label: impl Into<String>, size_bytes: u64, usage: BufferUsage) -> Self {
        Self {
            label: Some(label.into()),
            size_bytes,
            usage,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFormat {
    Rgba8UnormSrgb,
    Depth32Float,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureUsage {
    RenderAttachment,
    Sampled,
    Storage,
    CopySrc,
    CopyDst,
    Present,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureDimension {
    D1,
    D2,
    D3,
    Cube,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextureDesc {
    pub label: Option<String>,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub mip_levels: u32,
    pub sample_count: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
    pub dimension: TextureDimension,
}

impl TextureDesc {
    pub fn new(
        label: impl Into<String>,
        width: u32,
        height: u32,
        format: TextureFormat,
        usage: TextureUsage,
    ) -> Self {
        Self {
            label: Some(label.into()),
            width,
            height,
            depth: 1,
            mip_levels: 1,
            sample_count: 1,
            format,
            usage,
            dimension: TextureDimension::D2,
        }
    }

    pub fn with_dimension(mut self, dimension: TextureDimension) -> Self {
        self.dimension = dimension;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamplerDesc {
    pub label: Option<String>,
    pub linear_filtering: bool,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
}

impl SamplerDesc {
    pub fn linear(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            linear_filtering: true,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
        }
    }

    pub fn nearest(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            linear_filtering: false,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderModuleDesc {
    pub label: Option<String>,
    pub source: String,
    pub stage: ShaderStage,
    pub entry_point: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineKind {
    Raster,
    Compute,
    RayTracing,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineDesc {
    pub label: Option<String>,
    pub kind: PipelineKind,
}

impl PipelineDesc {
    pub fn new(label: impl Into<String>, kind: PipelineKind) -> Self {
        Self {
            label: Some(label.into()),
            kind,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresentMode {
    Immediate,
    Fifo,
    Mailbox,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwapchainDesc {
    pub width: u32,
    pub height: u32,
    pub present_mode: PresentMode,
    pub format: TextureFormat,
}
