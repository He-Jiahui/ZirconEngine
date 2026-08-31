use super::RenderGraphShaderStages;

/// Backend-neutral use classification for a graph resource access.
///
/// Compiler validation and future barrier lowering consume this instead of
/// inferring WGPU state from a broad read/write flag. `Legacy` preserves the
/// existing whole-resource APIs until product authoring supplies exact intent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RenderGraphResourceAccessIntent {
    Legacy,
    SampledTexture { stages: RenderGraphShaderStages },
    StorageTextureRead { stages: RenderGraphShaderStages },
    StorageTextureWrite { stages: RenderGraphShaderStages },
    ColorAttachment,
    DepthStencilAttachment,
    UniformBuffer { stages: RenderGraphShaderStages },
    StorageBufferRead { stages: RenderGraphShaderStages },
    StorageBufferReadWrite { stages: RenderGraphShaderStages },
    CopySource,
    CopyDestination,
    Indirect,
    Present,
    Readback,
}

impl RenderGraphResourceAccessIntent {
    pub const fn sampled_texture(stages: RenderGraphShaderStages) -> Self {
        Self::SampledTexture { stages }
    }

    pub const fn storage_texture_write(stages: RenderGraphShaderStages) -> Self {
        Self::StorageTextureWrite { stages }
    }

    pub const fn storage_buffer_read(stages: RenderGraphShaderStages) -> Self {
        Self::StorageBufferRead { stages }
    }

    pub const fn storage_buffer_read_write(stages: RenderGraphShaderStages) -> Self {
        Self::StorageBufferReadWrite { stages }
    }
}
