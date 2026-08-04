use std::sync::Arc;

use crate::graphics::scene::resources::{GpuTextureResource, OutputTargetTextureResource};

#[derive(Clone)]
pub(in crate::graphics::scene) enum MaterialTextureBinding {
    Texture {
        resource: Arc<GpuTextureResource>,
        max_anisotropy: u8,
        sampler_variant: Option<Arc<wgpu::Sampler>>,
    },
    OutputTarget(Arc<OutputTargetTextureResource>),
}

impl MaterialTextureBinding {
    pub(in crate::graphics::scene) fn texture(resource: Arc<GpuTextureResource>) -> Self {
        Self::Texture {
            resource,
            max_anisotropy: 16,
            sampler_variant: None,
        }
    }

    pub(in crate::graphics::scene) fn output_target(
        resource: Arc<OutputTargetTextureResource>,
    ) -> Self {
        Self::OutputTarget(resource)
    }

    pub(in crate::graphics::scene) fn view(&self) -> &wgpu::TextureView {
        match self {
            Self::Texture { resource, .. } => resource.view(),
            Self::OutputTarget(resource) => resource.view(),
        }
    }

    pub(in crate::graphics::scene) fn sampler(&self) -> &wgpu::Sampler {
        match self {
            Self::Texture {
                resource,
                sampler_variant,
                ..
            } => match sampler_variant.as_deref() {
                Some(sampler) => sampler,
                None => resource.sampler(),
            },
            Self::OutputTarget(resource) => resource.sampler(),
        }
    }

    pub(in crate::graphics::scene) fn set_max_anisotropy(&mut self, max_anisotropy: u8) {
        if let Self::Texture {
            max_anisotropy: current,
            sampler_variant,
            ..
        } = self
        {
            *current = max_anisotropy;
            *sampler_variant = None;
        }
    }

    pub(in crate::graphics::scene) fn prepare_sampler_variant(&mut self, device: &wgpu::Device) {
        if let Self::Texture {
            resource,
            max_anisotropy,
            sampler_variant,
        } = self
        {
            *sampler_variant = resource.sampler_variant_for_max_anisotropy(device, *max_anisotropy);
        }
    }

    pub(in crate::graphics::scene) fn identity(&self) -> usize {
        match self {
            Self::Texture {
                resource,
                max_anisotropy,
                ..
            } => (Arc::as_ptr(resource) as usize).rotate_left(5) ^ usize::from(*max_anisotropy),
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

    pub(in crate::graphics::scene) fn set_max_anisotropy(&mut self, max_anisotropy: u8) {
        self.base_color.set_max_anisotropy(max_anisotropy);
        self.normal.set_max_anisotropy(max_anisotropy);
        self.metallic_roughness.set_max_anisotropy(max_anisotropy);
        self.occlusion.set_max_anisotropy(max_anisotropy);
        self.emissive.set_max_anisotropy(max_anisotropy);
        self.clearcoat_normal.set_max_anisotropy(max_anisotropy);
    }

    pub(in crate::graphics::scene) fn prepare_sampler_variants(&mut self, device: &wgpu::Device) {
        self.base_color.prepare_sampler_variant(device);
        self.normal.prepare_sampler_variant(device);
        self.metallic_roughness.prepare_sampler_variant(device);
        self.occlusion.prepare_sampler_variant(device);
        self.emissive.prepare_sampler_variant(device);
        self.clearcoat_normal.prepare_sampler_variant(device);
    }
}
