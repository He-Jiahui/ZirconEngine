use super::*;
use crate::core::framework::render::{
    RenderImageAssetUsage, RenderImageFallbackKind, RenderSamplerAddressMode,
    RenderSamplerDescriptor, RenderSamplerFilter,
};

#[test]
fn rgba8_wgpu_format_uses_upload_plan_format() {
    assert_eq!(
        rgba8_wgpu_format(RGBA8_UNORM_FORMAT),
        wgpu::TextureFormat::Rgba8Unorm
    );
    assert_eq!(
        rgba8_wgpu_format(RGBA8_UNORM_SRGB_FORMAT),
        wgpu::TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(
        rgba8_wgpu_format("rgba16float"),
        wgpu::TextureFormat::Rgba8UnormSrgb
    );
}

#[test]
fn rgba8_mip_uploads_pack_levels_and_layers_in_payload_order() {
    assert_eq!(
        rgba8_mip_uploads(4, 2, 4, 1),
        vec![
            Rgba8MipUpload {
                level: 0,
                layer: 0,
                width: 4,
                height: 2,
                offset: 0
            },
            Rgba8MipUpload {
                level: 1,
                layer: 0,
                width: 2,
                height: 1,
                offset: 32
            },
            Rgba8MipUpload {
                level: 2,
                layer: 0,
                width: 1,
                height: 1,
                offset: 40
            },
            Rgba8MipUpload {
                level: 3,
                layer: 0,
                width: 1,
                height: 1,
                offset: 44
            },
        ]
    );
}

#[test]
fn rgba8_mip_uploads_pack_layers_inside_each_mip_level() {
    assert_eq!(
        rgba8_mip_uploads(4, 2, 2, 2),
        vec![
            Rgba8MipUpload {
                level: 0,
                layer: 0,
                width: 4,
                height: 2,
                offset: 0
            },
            Rgba8MipUpload {
                level: 0,
                layer: 1,
                width: 4,
                height: 2,
                offset: 32
            },
            Rgba8MipUpload {
                level: 1,
                layer: 0,
                width: 2,
                height: 1,
                offset: 64
            },
            Rgba8MipUpload {
                level: 1,
                layer: 1,
                width: 2,
                height: 1,
                offset: 72
            },
        ]
    );
}

#[test]
fn rgba8_material_texture_view_keeps_current_d2_binding_contract() {
    let view = texture_view_descriptor(&test_descriptor(vec![RenderImageUsage::Sampled]));

    assert_eq!(view.dimension, Some(wgpu::TextureViewDimension::D2));
    assert_eq!(view.base_array_layer, 0);
    assert_eq!(view.array_layer_count, Some(1));
}

#[test]
fn texture_array_view_uses_d2_array_dimension_and_all_layers() {
    let mut descriptor = test_descriptor(vec![RenderImageUsage::Sampled]);
    descriptor.depth_or_array_layers = 4;
    descriptor.array_layer_count = 4;

    let view = texture_view_descriptor(&descriptor);

    assert_eq!(view.dimension, Some(wgpu::TextureViewDimension::D2Array));
    assert_eq!(view.base_array_layer, 0);
    assert_eq!(view.array_layer_count, Some(4));
}

#[test]
fn cube_texture_view_uses_cube_dimension_and_all_faces() {
    let mut descriptor = test_descriptor(vec![RenderImageUsage::Sampled]);
    descriptor.dimension = RenderImageDimension::Cube;
    descriptor.depth_or_array_layers = 6;
    descriptor.array_layer_count = 6;

    let view = texture_view_descriptor(&descriptor);

    assert_eq!(
        wgpu_dimension(descriptor.dimension),
        wgpu::TextureDimension::D2
    );
    assert_eq!(view.dimension, Some(wgpu::TextureViewDimension::Cube));
    assert_eq!(view.base_array_layer, 0);
    assert_eq!(view.array_layer_count, Some(6));
}

#[test]
fn wgpu_texture_usages_maps_render_image_usage_for_asset_residency() {
    let descriptor = test_descriptor(vec![
        RenderImageUsage::RenderTarget,
        RenderImageUsage::Sampled,
        RenderImageUsage::Storage,
        RenderImageUsage::CopySrc,
    ]);

    let usages = wgpu_texture_usages(&descriptor, wgpu::TextureFormat::Rgba8Unorm, true);

    assert!(usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT));
    assert!(usages.contains(wgpu::TextureUsages::TEXTURE_BINDING));
    assert!(usages.contains(wgpu::TextureUsages::STORAGE_BINDING));
    assert!(usages.contains(wgpu::TextureUsages::COPY_SRC));
    assert!(
        usages.contains(wgpu::TextureUsages::COPY_DST),
        "asset residency uploads CPU/container payload bytes even when the asset metadata does not expose copy-dst"
    );
}

#[test]
fn wgpu_texture_usages_does_not_add_upload_dst_when_not_required() {
    let descriptor = test_descriptor(vec![RenderImageUsage::RenderTarget]);

    let usages = wgpu_texture_usages(&descriptor, wgpu::TextureFormat::Rgba8Unorm, false);

    assert_eq!(usages, wgpu::TextureUsages::RENDER_ATTACHMENT);
}

#[test]
fn wgpu_texture_usages_skips_storage_for_non_storage_formats() {
    let descriptor = test_descriptor(vec![
        RenderImageUsage::Sampled,
        RenderImageUsage::Storage,
        RenderImageUsage::CopyDst,
    ]);

    let usages = wgpu_texture_usages(&descriptor, wgpu::TextureFormat::Rgba8UnormSrgb, false);

    assert!(usages.contains(wgpu::TextureUsages::TEXTURE_BINDING));
    assert!(usages.contains(wgpu::TextureUsages::COPY_DST));
    assert!(!usages.contains(wgpu::TextureUsages::STORAGE_BINDING));
}

#[test]
fn wgpu_texture_usages_skips_render_attachment_for_non_renderable_formats() {
    let descriptor = test_descriptor(vec![
        RenderImageUsage::RenderTarget,
        RenderImageUsage::Sampled,
        RenderImageUsage::CopyDst,
    ]);

    let usages = wgpu_texture_usages(&descriptor, wgpu::TextureFormat::Bc1RgbaUnorm, false);

    assert!(usages.contains(wgpu::TextureUsages::TEXTURE_BINDING));
    assert!(usages.contains(wgpu::TextureUsages::COPY_DST));
    assert!(!usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT));
}

#[test]
fn sampler_descriptor_maps_texture_asset_sampler_settings() {
    let descriptor = RenderSamplerDescriptor {
        address_mode_u: RenderSamplerAddressMode::Repeat,
        address_mode_v: RenderSamplerAddressMode::MirrorRepeat,
        address_mode_w: RenderSamplerAddressMode::ClampToEdge,
        mag_filter: RenderSamplerFilter::Nearest,
        min_filter: RenderSamplerFilter::Linear,
        mipmap_filter: RenderSamplerFilter::Nearest,
    };

    let sampler = sampler_descriptor(&descriptor);

    assert_eq!(sampler.address_mode_u, wgpu::AddressMode::Repeat);
    assert_eq!(sampler.address_mode_v, wgpu::AddressMode::MirrorRepeat);
    assert_eq!(sampler.address_mode_w, wgpu::AddressMode::ClampToEdge);
    assert_eq!(sampler.mag_filter, wgpu::FilterMode::Nearest);
    assert_eq!(sampler.min_filter, wgpu::FilterMode::Linear);
    assert_eq!(sampler.mipmap_filter, wgpu::MipmapFilterMode::Nearest);
}

fn test_descriptor(usage: Vec<RenderImageUsage>) -> RenderImageDescriptor {
    RenderImageDescriptor {
        width: 4,
        height: 4,
        depth_or_array_layers: 1,
        dimension: RenderImageDimension::D2,
        format: RGBA8_UNORM_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Linear,
        sampler: RenderSamplerDescriptor::default(),
        usage,
        asset_usage: vec![
            RenderImageAssetUsage::MainWorld,
            RenderImageAssetUsage::RenderWorld,
        ],
        mip_count: 1,
        array_layer_count: 1,
        fallback: RenderImageFallbackKind::MissingImage,
    }
}
