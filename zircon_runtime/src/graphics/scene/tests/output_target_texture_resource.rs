use super::*;
use crate::core::framework::render::RenderImageDimension;
use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageFallbackKind, RenderSamplerDescriptor,
};

#[test]
fn output_target_texture_usages_keep_render_targets_graph_readable() {
    let descriptor = texture_descriptor(vec![RenderImageUsage::RenderTarget]);

    let usages = output_target_texture_usages(&descriptor, wgpu::TextureFormat::Rgba8UnormSrgb);

    assert!(usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT));
    assert!(usages.contains(wgpu::TextureUsages::COPY_SRC));
    assert!(usages.contains(wgpu::TextureUsages::COPY_DST));
    assert!(usages.contains(wgpu::TextureUsages::TEXTURE_BINDING));
}

#[test]
fn output_target_texture_usages_preserve_copy_and_sampled_authoring_flags() {
    let descriptor = texture_descriptor(vec![
        RenderImageUsage::RenderTarget,
        RenderImageUsage::Sampled,
        RenderImageUsage::CopySrc,
    ]);

    let usages = output_target_texture_usages(&descriptor, wgpu::TextureFormat::Rgba8Unorm);

    assert!(usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT));
    assert!(usages.contains(wgpu::TextureUsages::TEXTURE_BINDING));
    assert!(usages.contains(wgpu::TextureUsages::COPY_SRC));
    assert!(usages.contains(wgpu::TextureUsages::COPY_DST));
}

#[test]
fn output_target_wgpu_format_uses_descriptor_label() {
    assert_eq!(
        output_target_wgpu_format(&texture_descriptor_with_format(RGBA8_UNORM_FORMAT)),
        Some(wgpu::TextureFormat::Rgba8Unorm)
    );
    assert_eq!(
        output_target_wgpu_format(&texture_descriptor_with_format(RGBA8_UNORM_SRGB_FORMAT)),
        Some(wgpu::TextureFormat::Rgba8UnormSrgb)
    );
    assert_eq!(
        output_target_wgpu_format(&texture_descriptor_with_format("dds/dxt1")),
        None
    );
}

#[test]
fn output_target_rhi_descriptor_matches_wgpu_allocation_contract() {
    let descriptor = texture_descriptor(vec![
        RenderImageUsage::RenderTarget,
        RenderImageUsage::Sampled,
        RenderImageUsage::Storage,
    ]);

    let format = output_target_rhi_format(&descriptor).expect("supported target format");
    let usages = output_target_rhi_usages(&descriptor, format);

    assert_eq!(format, TextureFormat::Rgba8UnormSrgb);
    assert!(usages.contains(TextureUsage::RENDER_ATTACHMENT));
    assert!(usages.contains(TextureUsage::SAMPLED));
    assert!(usages.contains(TextureUsage::COPY_SRC));
    assert!(usages.contains(TextureUsage::COPY_DST));
    assert!(!usages.contains(TextureUsage::STORAGE));
}

#[test]
fn output_target_rhi_descriptor_keeps_render_targets_graph_readable() {
    let descriptor = texture_descriptor(vec![RenderImageUsage::RenderTarget]);
    let format = output_target_rhi_format(&descriptor).expect("supported target format");

    let usages = output_target_rhi_usages(&descriptor, format);

    assert!(usages.contains(TextureUsage::RENDER_ATTACHMENT));
    assert!(usages.contains(TextureUsage::SAMPLED));
    assert!(usages.contains(TextureUsage::COPY_SRC));
    assert!(usages.contains(TextureUsage::COPY_DST));
}

#[test]
fn validate_output_target_descriptor_rejects_sampled_only_target() {
    let descriptor = texture_descriptor(vec![RenderImageUsage::Sampled]);

    let error = validate_output_target_descriptor(
        ResourceId::from_stable_label("tests/target"),
        &descriptor,
    )
    .unwrap_err();

    assert!(
        matches!(error, GraphicsError::Asset(message) if message.contains("render_target usage"))
    );
}

fn texture_descriptor(usage: Vec<RenderImageUsage>) -> RenderImageDescriptor {
    RenderImageDescriptor {
        width: 64,
        height: 64,
        depth_or_array_layers: 1,
        dimension: RenderImageDimension::D2,
        format: RGBA8_UNORM_SRGB_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Srgb,
        metadata: TextureMetadata::default(),
        sampler: RenderSamplerDescriptor::default(),
        usage,
        asset_usage: Vec::new(),
        mip_count: 1,
        fallback: RenderImageFallbackKind::MissingImage,
    }
}

fn texture_descriptor_with_format(format: &str) -> RenderImageDescriptor {
    RenderImageDescriptor {
        format: format.to_string(),
        ..texture_descriptor(vec![RenderImageUsage::RenderTarget])
    }
}
