use crate::rhi::{BufferUsage, TextureDimension, TextureFormat, TextureUsage};

/// Declared physical contract for a graph resource.
///
/// Names remain diagnostic labels. Allocation policy is driven by this schema
/// whenever a feature needs a resource shape that cannot be inferred from a
/// built-in product contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderResourceSchema {
    Texture(RenderTextureSchema),
    Buffer(RenderBufferSchema),
}

impl RenderResourceSchema {
    pub const fn texture(schema: RenderTextureSchema) -> Self {
        Self::Texture(schema)
    }

    pub const fn texture_schema(self) -> Option<RenderTextureSchema> {
        match self {
            Self::Texture(schema) => Some(schema),
            Self::Buffer(_) => None,
        }
    }

    pub const fn buffer(schema: RenderBufferSchema) -> Self {
        Self::Buffer(schema)
    }

    pub const fn buffer_schema(self) -> Option<RenderBufferSchema> {
        match self {
            Self::Texture(_) => None,
            Self::Buffer(schema) => Some(schema),
        }
    }
}

/// Extent source for a texture declared by a feature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderTextureExtentPolicy {
    Render,
    View,
    Relative {
        reference: RenderTextureExtentReference,
        numerator: u32,
        denominator: u32,
        rounding: RenderTextureExtentRounding,
    },
    Fixed {
        width: u32,
        height: u32,
        depth_or_array_layers: u32,
    },
}

/// Full-size extent used as the reference for a relative texture allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderTextureExtentReference {
    Render,
    View,
}

/// Integer rounding rule used after applying a relative texture scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderTextureExtentRounding {
    Floor,
    Ceil,
}

/// Explicit behavior when a requested resource contract cannot be admitted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderResourceFallback {
    #[default]
    Reject,
}

/// A texture schema carries format, size policy, topology and declared usage
/// independently of the debug resource name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderTextureSchema {
    pub format: TextureFormat,
    pub extent: RenderTextureExtentPolicy,
    pub dimension: TextureDimension,
    pub mip_levels: u32,
    pub sample_count: u32,
    pub usage: TextureUsage,
    pub fallback: RenderResourceFallback,
}

impl RenderTextureSchema {
    pub const fn new(format: TextureFormat, usage: TextureUsage) -> Self {
        Self {
            format,
            extent: RenderTextureExtentPolicy::Render,
            dimension: TextureDimension::D2,
            mip_levels: 1,
            sample_count: 1,
            usage,
            fallback: RenderResourceFallback::Reject,
        }
    }

    pub const fn with_extent(mut self, extent: RenderTextureExtentPolicy) -> Self {
        self.extent = extent;
        self
    }

    pub const fn with_dimension(mut self, dimension: TextureDimension) -> Self {
        self.dimension = dimension;
        self
    }

    pub const fn with_mip_levels(mut self, mip_levels: u32) -> Self {
        self.mip_levels = mip_levels;
        self
    }

    pub const fn with_sample_count(mut self, sample_count: u32) -> Self {
        self.sample_count = sample_count;
        self
    }

    pub const fn with_fallback(mut self, fallback: RenderResourceFallback) -> Self {
        self.fallback = fallback;
        self
    }
}

/// A buffer schema carries an exact byte capacity and declared usage
/// independently of the debug resource name.
///
/// Dynamic built-in buffer policies are resolved by the future resource
/// catalog before they reach this physical contract. Plugin-owned buffers use
/// this exact form so graph authoring never derives capacity from viewport
/// pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderBufferSchema {
    pub size_bytes: u64,
    pub usage: BufferUsage,
    pub fallback: RenderResourceFallback,
}

impl RenderBufferSchema {
    pub const fn new(size_bytes: u64, usage: BufferUsage) -> Self {
        Self {
            size_bytes,
            usage,
            fallback: RenderResourceFallback::Reject,
        }
    }

    pub const fn with_fallback(mut self, fallback: RenderResourceFallback) -> Self {
        self.fallback = fallback;
        self
    }
}
