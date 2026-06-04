use serde::{Deserialize, Serialize};
use std::ops::{BitOr, BitOrAssign};

mod pipeline;

pub use pipeline::{
    BlendComponentDesc, BlendFactor, BlendOperation, BlendStateDesc, ColorTargetDesc,
    ColorWriteMask, CullMode, DepthStencilStateDesc, FrontFace, PipelineDesc, PipelineKind,
    PipelineLayoutDesc, PrimitiveStateDesc, PrimitiveTopology, RasterPipelineStateDesc,
    ShaderModuleDesc, ShaderStage, VertexAttributeDesc, VertexBufferLayoutDesc, VertexFormat,
    VertexInputLayoutDesc, VertexStepMode,
};

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
    pub const ALL: Self = Self(
        Self::VERTEX.0
            | Self::INDEX.0
            | Self::UNIFORM.0
            | Self::STORAGE.0
            | Self::STAGING_READ.0
            | Self::STAGING_WRITE.0
            | Self::INDIRECT.0
            | Self::COPY_SRC.0
            | Self::COPY_DST.0,
    );

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn has_unknown_bits(self) -> bool {
        (self.0 & !Self::ALL.0) != 0
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
    R8Unorm,
    R16Float,
    R32Float,
    Rg16Float,
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Bgra8UnormSrgb,
    Rgba16Float,
    Rgba32Float,
    Depth24Plus,
    Depth24PlusStencil8,
    Depth32Float,
}

impl TextureFormat {
    pub const fn bytes_per_pixel(self) -> u32 {
        match self {
            Self::R8Unorm => 1,
            Self::R16Float => 2,
            Self::R32Float | Self::Rg16Float => 4,
            Self::Rgba8Unorm
            | Self::Rgba8UnormSrgb
            | Self::Bgra8Unorm
            | Self::Bgra8UnormSrgb
            | Self::Depth24Plus
            | Self::Depth24PlusStencil8
            | Self::Depth32Float => 4,
            Self::Rgba16Float => 8,
            Self::Rgba32Float => 16,
        }
    }

    pub const fn is_depth(self) -> bool {
        matches!(
            self,
            Self::Depth24Plus | Self::Depth24PlusStencil8 | Self::Depth32Float
        )
    }

    pub const fn has_stencil(self) -> bool {
        matches!(self, Self::Depth24PlusStencil8)
    }

    pub const fn is_hdr_color(self) -> bool {
        matches!(
            self,
            Self::R16Float
                | Self::R32Float
                | Self::Rg16Float
                | Self::Rgba16Float
                | Self::Rgba32Float
        )
    }
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
    D2Array,
    D3,
    Cube,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureResidency {
    #[default]
    Dense,
    SparseReserved,
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
    #[serde(default)]
    pub residency: TextureResidency,
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
            residency: TextureResidency::Dense,
        }
    }

    pub fn with_dimension(mut self, dimension: TextureDimension) -> Self {
        self.dimension = dimension;
        self
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_array_layers(mut self, layers: u32) -> Self {
        self.depth = layers;
        self
    }

    pub fn with_mip_levels(mut self, mip_levels: u32) -> Self {
        self.mip_levels = mip_levels;
        self
    }

    pub fn with_sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    pub fn with_sparse_residency(mut self) -> Self {
        self.residency = TextureResidency::SparseReserved;
        self
    }

    pub const fn is_sparse_reserved(&self) -> bool {
        matches!(self.residency, TextureResidency::SparseReserved)
    }

    pub const fn max_full_mip_levels(&self) -> u32 {
        let max_extent = match self.dimension {
            TextureDimension::D1 => self.width,
            TextureDimension::D2 | TextureDimension::D2Array | TextureDimension::Cube => {
                max_u32(self.width, self.height)
            }
            TextureDimension::D3 => max_u32(max_u32(self.width, self.height), self.depth),
        };
        if max_extent == 0 {
            0
        } else {
            u32::BITS - max_extent.leading_zeros()
        }
    }

    pub const fn mip_levels_fit_shape(&self) -> bool {
        self.mip_levels > 0 && self.mip_levels <= self.max_full_mip_levels()
    }

    pub fn checked_storage_size_bytes(&self) -> Option<u64> {
        let mut total = 0_u64;
        for level in 0..self.mip_levels {
            let width = mip_extent(self.width, level);
            let height = mip_extent(self.height, level);
            let depth = match self.dimension {
                TextureDimension::D3 => mip_extent(self.depth, level),
                TextureDimension::D1
                | TextureDimension::D2
                | TextureDimension::D2Array
                | TextureDimension::Cube => self.depth,
            };
            let level_size = u64::from(width)
                .checked_mul(u64::from(height))?
                .checked_mul(u64::from(depth))?
                .checked_mul(u64::from(self.sample_count))?
                .checked_mul(u64::from(self.format.bytes_per_pixel()))?;
            total = total.checked_add(level_size)?;
        }
        Some(total)
    }
}

const fn max_u32(left: u32, right: u32) -> u32 {
    if left >= right {
        left
    } else {
        right
    }
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    let shifted = if level >= u32::BITS {
        0
    } else {
        value >> level
    };
    if shifted == 0 {
        1
    } else {
        shifted
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MipmapFilterMode {
    Nearest,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompareFunction {
    Never,
    Less,
    LessEqual,
    Equal,
    GreaterEqual,
    Greater,
    NotEqual,
    Always,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SamplerDesc {
    pub label: Option<String>,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub mipmap_filter: MipmapFilterMode,
    pub address_mode_u: AddressMode,
    pub address_mode_v: AddressMode,
    pub address_mode_w: AddressMode,
    pub lod_min_clamp: f32,
    pub lod_max_clamp: f32,
    pub compare: Option<CompareFunction>,
    pub anisotropy_clamp: u16,
}

impl SamplerDesc {
    pub fn linear(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilterMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
        }
    }

    pub fn nearest(label: impl Into<String>) -> Self {
        Self {
            label: Some(label.into()),
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: MipmapFilterMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            lod_min_clamp: 0.0,
            lod_max_clamp: 32.0,
            compare: None,
            anisotropy_clamp: 1,
        }
    }

    pub fn linear_mipmap_linear(label: impl Into<String>) -> Self {
        Self::linear(label).with_mipmap_filter(MipmapFilterMode::Linear)
    }

    pub const fn with_mipmap_filter(mut self, mipmap_filter: MipmapFilterMode) -> Self {
        self.mipmap_filter = mipmap_filter;
        self
    }

    pub const fn with_lod_clamp(mut self, min: f32, max: f32) -> Self {
        self.lod_min_clamp = min;
        self.lod_max_clamp = max;
        self
    }

    pub const fn with_compare(mut self, compare: CompareFunction) -> Self {
        self.compare = Some(compare);
        self
    }

    pub const fn with_anisotropy_clamp(mut self, anisotropy_clamp: u16) -> Self {
        self.anisotropy_clamp = anisotropy_clamp;
        self
    }

    pub const fn uses_anisotropy(&self) -> bool {
        self.anisotropy_clamp > 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingResourceType {
    UniformBuffer,
    StorageBuffer,
    Texture,
    StorageTexture,
    Sampler,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindGroupLayoutEntryDesc {
    pub binding: u32,
    pub visibility: Vec<ShaderStage>,
    pub resource_type: BindingResourceType,
}

impl BindGroupLayoutEntryDesc {
    pub fn new(
        binding: u32,
        resource_type: BindingResourceType,
        visibility: impl Into<Vec<ShaderStage>>,
    ) -> Self {
        Self {
            binding,
            visibility: visibility.into(),
            resource_type,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindGroupLayoutDesc {
    pub label: Option<String>,
    pub entries: Vec<BindGroupLayoutEntryDesc>,
}

impl BindGroupLayoutDesc {
    pub fn new(label: impl Into<String>, entries: Vec<BindGroupLayoutEntryDesc>) -> Self {
        Self {
            label: Some(label.into()),
            entries,
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
