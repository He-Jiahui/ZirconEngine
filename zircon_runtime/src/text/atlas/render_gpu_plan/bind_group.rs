const GLYPH_ATLAS_GPU_BIND_GROUP_INDEX: u32 = 0;
const GLYPH_ATLAS_GPU_ATLAS_TEXTURE_BINDING: u32 = 0;
const GLYPH_ATLAS_GPU_ATLAS_SAMPLER_BINDING: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasGpuTextureSampleType {
    FloatFilterable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasGpuTextureViewDimension {
    D2Array,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphAtlasGpuSamplerBindingType {
    Filtering,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuTextureBinding {
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) sample_type: GlyphAtlasGpuTextureSampleType,
    pub(crate) view_dimension: GlyphAtlasGpuTextureViewDimension,
    pub(crate) multisampled: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuSamplerBinding {
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) binding_type: GlyphAtlasGpuSamplerBindingType,
}

/// Fixed texture-array and sampler binding contract consumed by the future wgpu atlas renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuBindGroupLayout {
    pub(crate) atlas_texture: GlyphAtlasGpuTextureBinding,
    pub(crate) atlas_sampler: GlyphAtlasGpuSamplerBinding,
}

pub(crate) fn glyph_atlas_gpu_bind_group_layout() -> GlyphAtlasGpuBindGroupLayout {
    GlyphAtlasGpuBindGroupLayout {
        atlas_texture: GlyphAtlasGpuTextureBinding {
            group: GLYPH_ATLAS_GPU_BIND_GROUP_INDEX,
            binding: GLYPH_ATLAS_GPU_ATLAS_TEXTURE_BINDING,
            sample_type: GlyphAtlasGpuTextureSampleType::FloatFilterable,
            view_dimension: GlyphAtlasGpuTextureViewDimension::D2Array,
            multisampled: false,
        },
        atlas_sampler: GlyphAtlasGpuSamplerBinding {
            group: GLYPH_ATLAS_GPU_BIND_GROUP_INDEX,
            binding: GLYPH_ATLAS_GPU_ATLAS_SAMPLER_BINDING,
            binding_type: GlyphAtlasGpuSamplerBindingType::Filtering,
        },
    }
}
