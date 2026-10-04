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

#[test]
fn default_image_descriptor_matches_the_generation_linear_clamp_key() {
    assert_eq!(
        TextureSamplerKey::from_image_descriptor(&test_descriptor(), 16),
        TextureSamplerKey::LINEAR_CLAMP
    );
}
