use serde::{Deserialize, Serialize};
use std::ops::{BitOr, BitOrAssign};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferUsage(u32);

impl BufferUsage {
    pub const NONE: Self = Self(0);
    pub const VERTEX: Self = Self(1 << 0);
    pub const INDEX: Self = Self(1 << 1);
    pub const UNIFORM: Self = Self(1 << 2);
    pub const STORAGE: Self = Self(1 << 3);
    pub const STAGING_READ: Self = Self(1 << 4);
    pub const STAGING_WRITE: Self = Self(1 << 5);
    pub const INDIRECT: Self = Self(1 << 6);
    pub const COPY_SRC: Self = Self(1 << 7);
    pub const COPY_DST: Self = Self(1 << 8);

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl BitOr for BufferUsage {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for BufferUsage {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextureUsage(u32);

impl TextureUsage {
    pub const NONE: Self = Self(0);
    pub const RENDER_ATTACHMENT: Self = Self(1 << 0);
    pub const SAMPLED: Self = Self(1 << 1);
    pub const STORAGE: Self = Self(1 << 2);
    pub const COPY_SRC: Self = Self(1 << 3);
    pub const COPY_DST: Self = Self(1 << 4);
    pub const PRESENT: Self = Self(1 << 5);

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl BitOr for TextureUsage {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for TextureUsage {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
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
