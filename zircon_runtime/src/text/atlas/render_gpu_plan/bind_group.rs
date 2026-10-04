//! CPU 图集计划与渲染资源创建共享的纹理数组、过滤采样器及视口绑定约定。
//! 绑定编号必须与拼接后的 WGSL 一致，逻辑图集页索引对应纹理数组层。

const GLYPH_ATLAS_GPU_BIND_GROUP_INDEX: u32 = 0;
const GLYPH_ATLAS_GPU_ATLAS_TEXTURE_BINDING: u32 = 0;
const GLYPH_ATLAS_GPU_ATLAS_SAMPLER_BINDING: u32 = 1;
const GLYPH_ATLAS_GPU_VIEWPORT_UNIFORM_BINDING: u32 = 2;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuViewportUniformBinding {
    pub(crate) group: u32,
    pub(crate) binding: u32,
}

/// Fixed texture-array and sampler binding contract consumed by the future wgpu atlas renderer.
/// 由图集渲染器资源创建路径消费，绑定编号与着色器布局必须同步。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasGpuBindGroupLayout {
    pub(crate) atlas_texture: GlyphAtlasGpuTextureBinding,
    pub(crate) atlas_sampler: GlyphAtlasGpuSamplerBinding,
    pub(crate) viewport_uniform: GlyphAtlasGpuViewportUniformBinding,
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
        viewport_uniform: GlyphAtlasGpuViewportUniformBinding {
            group: GLYPH_ATLAS_GPU_BIND_GROUP_INDEX,
            binding: GLYPH_ATLAS_GPU_VIEWPORT_UNIFORM_BINDING,
        },
    }
}
