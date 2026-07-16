use std::sync::Arc;

use crate::graphics::scene::resources::{GpuTextureResource, OutputTargetTextureResource};

#[derive(Clone)]
pub(in crate::graphics::scene) enum MaterialTextureBinding {
    Texture(Arc<GpuTextureResource>),
    OutputTarget(Arc<OutputTargetTextureResource>),
}

impl MaterialTextureBinding {
    pub(in crate::graphics::scene) fn texture(resource: Arc<GpuTextureResource>) -> Self {
        Self::Texture(resource)
    }

    pub(in crate::graphics::scene) fn output_target(
        resource: Arc<OutputTargetTextureResource>,
    ) -> Self {
        Self::OutputTarget(resource)
    }

    pub(in crate::graphics::scene) fn view(&self) -> &wgpu::TextureView {
        match self {
            Self::Texture(resource) => resource.view(),
            Self::OutputTarget(resource) => resource.view(),
        }
    }

    pub(in crate::graphics::scene) fn sampler(&self) -> &wgpu::Sampler {
        match self {
            Self::Texture(resource) => resource.sampler(),
            Self::OutputTarget(resource) => resource.sampler(),
        }
    }

    pub(in crate::graphics::scene) fn identity(&self) -> usize {
        match self {
            Self::Texture(resource) => Arc::as_ptr(resource) as usize,
            Self::OutputTarget(resource) => {
                (Arc::as_ptr(resource) as usize) ^ (1_usize << (usize::BITS - 1))
            }
        }
    }
}

#[derive(Clone)]
pub(in crate::graphics::scene) struct MaterialTextureSet {
    pub(in crate::graphics::scene) base_color: MaterialTextureBinding,
    pub(in crate::graphics::scene) normal: MaterialTextureBinding,
    pub(in crate::graphics::scene) metallic_roughness: MaterialTextureBinding,
    pub(in crate::graphics::scene) occlusion: MaterialTextureBinding,
    pub(in crate::graphics::scene) emissive: MaterialTextureBinding,
    pub(in crate::graphics::scene) clearcoat_normal: MaterialTextureBinding,
}

impl MaterialTextureSet {
    pub(in crate::graphics::scene) fn new(
        base_color: MaterialTextureBinding,
        normal: MaterialTextureBinding,
        metallic_roughness: MaterialTextureBinding,
        occlusion: MaterialTextureBinding,
        emissive: MaterialTextureBinding,
        clearcoat_normal: MaterialTextureBinding,
    ) -> Self {
        Self {
            base_color,
            normal,
            metallic_roughness,
            occlusion,
            emissive,
            clearcoat_normal,
        }
    }
}
