use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::core::framework::render::{
    RenderImageDescriptor, RenderSamplerAddressMode, RenderSamplerDescriptor, RenderSamplerFilter,
};

pub(crate) struct TextureSamplerCache {
    samplers: Mutex<HashMap<TextureSamplerKey, Arc<wgpu::Sampler>>>,
}

impl TextureSamplerCache {
    pub(in crate::graphics::scene::resources) fn new() -> Self {
        Self {
            samplers: Mutex::new(HashMap::new()),
        }
    }

    pub(in crate::graphics::scene::resources) fn sampler_for_image(
        &self,
        device: &wgpu::Device,
        descriptor: &RenderImageDescriptor,
    ) -> Arc<wgpu::Sampler> {
        self.sampler_for_image_with_anisotropy_cap(device, descriptor, u8::MAX)
    }

    pub(in crate::graphics::scene::resources) fn sampler_for_image_with_anisotropy_cap(
        &self,
        device: &wgpu::Device,
        descriptor: &RenderImageDescriptor,
        max_anisotropy: u8,
    ) -> Arc<wgpu::Sampler> {
        let key = TextureSamplerKey::from_image_descriptor(descriptor, max_anisotropy);
        let mut samplers = match self.samplers.lock() {
            Ok(samplers) => samplers,
            Err(poisoned) => poisoned.into_inner(),
        };
        if let Some(sampler) = samplers.get(&key) {
            return Arc::clone(sampler);
        }

        let sampler = Arc::new(device.create_sampler(
            &sampler_descriptor_for_image_with_anisotropy_cap(descriptor, max_anisotropy),
        ));
        samplers.insert(key, Arc::clone(&sampler));
        sampler
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TextureSamplerKey {
    address_mode_u: u8,
    address_mode_v: u8,
    address_mode_w: u8,
    mag_filter: u8,
    min_filter: u8,
    mipmap_filter: u8,
    anisotropy_clamp: u16,
}

impl TextureSamplerKey {
    fn from_image_descriptor(descriptor: &RenderImageDescriptor, max_anisotropy: u8) -> Self {
        let sampler = &descriptor.sampler;
        Self {
            address_mode_u: address_mode_key(sampler.address_mode_u),
            address_mode_v: address_mode_key(sampler.address_mode_v),
            address_mode_w: address_mode_key(sampler.address_mode_w),
            mag_filter: filter_key(sampler.mag_filter),
            min_filter: filter_key(sampler.min_filter),
            mipmap_filter: filter_key(sampler.mipmap_filter),
            anisotropy_clamp: sanitized_anisotropy_clamp_with_cap(descriptor, max_anisotropy),
        }
    }
}

pub(super) fn sampler_descriptor(
    descriptor: &RenderSamplerDescriptor,
) -> wgpu::SamplerDescriptor<'static> {
    wgpu::SamplerDescriptor {
        mag_filter: filter_mode(descriptor.mag_filter),
        min_filter: filter_mode(descriptor.min_filter),
        mipmap_filter: mipmap_filter_mode(descriptor.mipmap_filter),
        address_mode_u: address_mode(descriptor.address_mode_u),
        address_mode_v: address_mode(descriptor.address_mode_v),
        address_mode_w: address_mode(descriptor.address_mode_w),
        ..Default::default()
    }
}

pub(super) fn sampler_descriptor_for_image(
    descriptor: &RenderImageDescriptor,
) -> wgpu::SamplerDescriptor<'static> {
    sampler_descriptor_for_image_with_anisotropy_cap(descriptor, u8::MAX)
}

pub(super) fn sampler_descriptor_for_image_with_anisotropy_cap(
    descriptor: &RenderImageDescriptor,
    max_anisotropy: u8,
) -> wgpu::SamplerDescriptor<'static> {
    let mut sampler = sampler_descriptor(&descriptor.sampler);
    sampler.anisotropy_clamp = sanitized_anisotropy_clamp_with_cap(descriptor, max_anisotropy);
    sampler
}

pub(super) fn sanitized_anisotropy_clamp(descriptor: &RenderImageDescriptor) -> u16 {
    sanitized_anisotropy_clamp_with_cap(descriptor, u8::MAX)
}

pub(super) fn sanitized_anisotropy_clamp_with_cap(
    descriptor: &RenderImageDescriptor,
    max_anisotropy: u8,
) -> u16 {
    if descriptor.sampler.mag_filter != RenderSamplerFilter::Linear
        || descriptor.sampler.min_filter != RenderSamplerFilter::Linear
        || descriptor.sampler.mipmap_filter != RenderSamplerFilter::Linear
    {
        return 1;
    }
    let asset_max = match descriptor.metadata.max_anisotropy {
        2 | 4 | 8 | 16 => u16::from(descriptor.metadata.max_anisotropy),
        _ => 1,
    };
    asset_max.min(u16::from(normalize_anisotropy_cap(max_anisotropy)))
}

const fn normalize_anisotropy_cap(max_anisotropy: u8) -> u8 {
    match max_anisotropy {
        16.. => 16,
        8.. => 8,
        4.. => 4,
        2.. => 2,
        _ => 1,
    }
}

fn address_mode_key(mode: RenderSamplerAddressMode) -> u8 {
    match mode {
        RenderSamplerAddressMode::ClampToEdge => 0,
        RenderSamplerAddressMode::Repeat => 1,
        RenderSamplerAddressMode::MirrorRepeat => 2,
    }
}

fn filter_key(filter: RenderSamplerFilter) -> u8 {
    match filter {
        RenderSamplerFilter::Nearest => 0,
        RenderSamplerFilter::Linear => 1,
    }
}

fn filter_mode(filter: RenderSamplerFilter) -> wgpu::FilterMode {
    match filter {
        RenderSamplerFilter::Nearest => wgpu::FilterMode::Nearest,
        RenderSamplerFilter::Linear => wgpu::FilterMode::Linear,
    }
}

fn mipmap_filter_mode(filter: RenderSamplerFilter) -> wgpu::MipmapFilterMode {
    match filter {
        RenderSamplerFilter::Nearest => wgpu::MipmapFilterMode::Nearest,
        RenderSamplerFilter::Linear => wgpu::MipmapFilterMode::Linear,
    }
}

fn address_mode(mode: RenderSamplerAddressMode) -> wgpu::AddressMode {
    match mode {
        RenderSamplerAddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
        RenderSamplerAddressMode::Repeat => wgpu::AddressMode::Repeat,
        RenderSamplerAddressMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{
        RenderImageColorSpace, RenderImageDimension, RenderImageFallbackKind, RenderImageUsage,
        TextureMetadata,
    };

    fn test_descriptor() -> RenderImageDescriptor {
        RenderImageDescriptor {
            width: 4,
            height: 4,
            depth_or_array_layers: 1,
            dimension: RenderImageDimension::D2,
            format: "rgba8unorm_srgb".to_string(),
            color_space: RenderImageColorSpace::Srgb,
            metadata: TextureMetadata::default(),
            sampler: RenderSamplerDescriptor::default(),
            usage: vec![RenderImageUsage::Sampled],
            asset_usage: Vec::new(),
            mip_count: 1,
            array_layer_count: 1,
            fallback: RenderImageFallbackKind::MissingImage,
        }
    }

    #[test]
    fn texture_sampler_key_reuses_equal_effective_sampler_state() {
        let mut first = test_descriptor();
        first.metadata.max_anisotropy = 16;
        let second = first.clone();

        assert_eq!(
            TextureSamplerKey::from_image_descriptor(&first, 16),
            TextureSamplerKey::from_image_descriptor(&second, 16)
        );
        assert_ne!(
            TextureSamplerKey::from_image_descriptor(&first, 16),
            TextureSamplerKey::from_image_descriptor(&first, 4)
        );
    }

    #[test]
    fn texture_sampler_key_distinguishes_each_address_and_filter_field() {
        let baseline = test_descriptor();
        let baseline_key = TextureSamplerKey::from_image_descriptor(&baseline, 16);

        let mut address_u = baseline.clone();
        address_u.sampler.address_mode_u = RenderSamplerAddressMode::Repeat;
        assert_ne!(
            baseline_key,
            TextureSamplerKey::from_image_descriptor(&address_u, 16)
        );

        let mut address_v = baseline.clone();
        address_v.sampler.address_mode_v = RenderSamplerAddressMode::Repeat;
        assert_ne!(
            baseline_key,
            TextureSamplerKey::from_image_descriptor(&address_v, 16)
        );

        let mut address_w = baseline.clone();
        address_w.sampler.address_mode_w = RenderSamplerAddressMode::Repeat;
        assert_ne!(
            baseline_key,
            TextureSamplerKey::from_image_descriptor(&address_w, 16)
        );

        let mut mag_filter = baseline.clone();
        mag_filter.sampler.mag_filter = RenderSamplerFilter::Nearest;
        assert_ne!(
            baseline_key,
            TextureSamplerKey::from_image_descriptor(&mag_filter, 16)
        );

        let mut min_filter = baseline.clone();
        min_filter.sampler.min_filter = RenderSamplerFilter::Nearest;
        assert_ne!(
            baseline_key,
            TextureSamplerKey::from_image_descriptor(&min_filter, 16)
        );

        let mut mipmap_filter = baseline;
        mipmap_filter.sampler.mipmap_filter = RenderSamplerFilter::Nearest;
        assert_ne!(
            baseline_key,
            TextureSamplerKey::from_image_descriptor(&mipmap_filter, 16)
        );
    }

    #[test]
    fn texture_sampler_key_disables_anisotropy_for_non_linear_filtering() {
        let mut descriptor = test_descriptor();
        descriptor.metadata.max_anisotropy = 16;
        descriptor.sampler.mag_filter = RenderSamplerFilter::Nearest;

        assert_eq!(
            TextureSamplerKey::from_image_descriptor(&descriptor, 16),
            TextureSamplerKey::from_image_descriptor(&descriptor, 1)
        );
    }
}
