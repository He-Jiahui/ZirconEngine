use crate::rhi::{TextureDesc, TextureDimension, TextureFormat, TextureUsage};

use super::{resolved_range_count, validate_texture_descriptor_for_tracking};

#[test]
fn resolved_texture_ranges_are_checked_before_expansion() {
    assert_eq!(resolved_range_count(0, None, u32::MAX), Some(u32::MAX));
    assert_eq!(
        resolved_range_count(u32::MAX - 1, Some(1), u32::MAX),
        Some(1)
    );
    assert_eq!(resolved_range_count(u32::MAX, Some(1), u32::MAX), None);
    assert_eq!(resolved_range_count(0, Some(0), 1), None);
    assert_eq!(resolved_range_count(u32::MAX - 1, Some(2), u32::MAX), None);
}

#[test]
fn tracker_rejects_zero_sample_texture_before_scope_expansion() {
    let descriptor = TextureDesc::new(
        "zero-samples",
        4,
        4,
        TextureFormat::Rgba8Unorm,
        TextureUsage::SAMPLED,
    )
    .with_sample_count(0);
    let error = validate_texture_descriptor_for_tracking(
        crate::render_graph::RenderGraphResource::TransientTexture(
            crate::render_graph::RgTextureHandle::from_index(0, 0),
        ),
        &descriptor,
    )
    .expect_err("zero sample count must fail before tracker expansion");
    assert!(matches!(
        error,
        crate::render_graph::RenderGraphError::TextureDescriptorInvalid { reason, .. }
            if reason == "sample_count must be greater than zero"
    ));
}

#[test]
fn tracker_rejects_multisample_shape_mismatch_before_scope_expansion() {
    let array_descriptor = TextureDesc::new(
        "multisample-array",
        4,
        4,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    )
    .with_sample_count(4)
    .with_dimension(TextureDimension::D2Array)
    .with_array_layers(2);
    let array_error = validate_texture_descriptor_for_tracking(
        crate::render_graph::RenderGraphResource::TransientTexture(
            crate::render_graph::RgTextureHandle::from_index(0, 0),
        ),
        &array_descriptor,
    )
    .expect_err("multisampling must reject array textures");
    assert!(matches!(
        array_error,
        crate::render_graph::RenderGraphError::TextureDescriptorInvalid { reason, .. }
            if reason == "multisampling is only valid for 2D textures"
    ));

    let mip_descriptor = TextureDesc::new(
        "multisample-mips",
        8,
        8,
        TextureFormat::Rgba8Unorm,
        TextureUsage::RENDER_ATTACHMENT,
    )
    .with_sample_count(4)
    .with_mip_levels(2);
    let mip_error = validate_texture_descriptor_for_tracking(
        crate::render_graph::RenderGraphResource::TransientTexture(
            crate::render_graph::RgTextureHandle::from_index(1, 0),
        ),
        &mip_descriptor,
    )
    .expect_err("multisampling must reject mip chains");
    assert!(matches!(
        mip_error,
        crate::render_graph::RenderGraphError::TextureDescriptorInvalid { reason, .. }
            if reason == "multisampled textures cannot declare mip levels"
    ));
}
