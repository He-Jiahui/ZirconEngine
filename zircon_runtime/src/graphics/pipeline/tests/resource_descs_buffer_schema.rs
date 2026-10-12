use super::{
    buffer_desc_from_schema, builtin_buffer_desc_for, resolve_relative_extent_axis,
    texture_desc_from_schema,
};
use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderFrameExtract, RenderWorldSnapshotHandle,
};
use crate::graphics::{
    RenderBufferSchema, RenderResourceSchema, RenderTextureExtentPolicy,
    RenderTextureExtentReference, RenderTextureExtentRounding, RenderTextureSchema,
};
use crate::rhi::{BufferUsage, TextureFormat, TextureUsage};
use crate::scene::world::World;

fn test_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(1),
        World::new().to_render_snapshot(),
    )
}

#[test]
fn explicit_buffer_schema_rejects_zero_size_and_empty_usage() {
    let zero_size = buffer_desc_from_schema(
        "zero-size",
        RenderResourceSchema::buffer(RenderBufferSchema::new(0, BufferUsage::STORAGE)),
        None,
    )
    .expect_err("buffer schemas require a non-zero byte size");
    assert!(zero_size.contains("non-zero byte size"), "{zero_size}");

    let empty_usage = buffer_desc_from_schema(
        "empty-usage",
        RenderResourceSchema::buffer(RenderBufferSchema::new(16, BufferUsage::NONE)),
        None,
    )
    .expect_err("buffer schemas require usage");
    assert!(
        empty_usage.contains("usage must not be empty"),
        "{empty_usage}"
    );
}

#[test]
fn explicit_texture_schema_rejects_empty_usage() {
    let error = texture_desc_from_schema(
        "empty-texture-usage",
        RenderResourceSchema::texture(RenderTextureSchema::new(
            TextureFormat::Rgba8Unorm,
            TextureUsage::NONE,
        )),
        &test_extract(),
    )
    .expect_err("texture schemas require usage");

    assert!(error.contains("usage must not be empty"), "{error}");
}

#[test]
fn relative_texture_extent_ceil_divides_the_selected_reference_extent() {
    let extract = test_extract();
    let schema = RenderResourceSchema::texture(
        RenderTextureSchema::new(TextureFormat::R8Unorm, TextureUsage::STORAGE).with_extent(
            RenderTextureExtentPolicy::Relative {
                reference: RenderTextureExtentReference::Render,
                numerator: 1,
                denominator: 2,
                rounding: RenderTextureExtentRounding::Ceil,
            },
        ),
    );

    let desc = texture_desc_from_schema("half-render", schema, &extract)
        .expect("a valid relative render extent should resolve");
    let render_extent = extract
        .view
        .view_family_pipeline()
        .resolution()
        .primary_allocation_extent();

    assert_eq!(desc.width, render_extent.x.div_ceil(2).max(1));
    assert_eq!(desc.height, render_extent.y.div_ceil(2).max(1));
    assert_eq!(
        resolve_relative_extent_axis(5, 1, 2, RenderTextureExtentRounding::Ceil)
            .expect("ceil-scaled odd extent"),
        3
    );
    assert_eq!(
        resolve_relative_extent_axis(5, 1, 2, RenderTextureExtentRounding::Floor)
            .expect("floor-scaled odd extent"),
        2
    );
}

#[test]
fn relative_texture_extent_rejects_zero_scale_terms_and_u32_overflow() {
    let extract = test_extract();
    for (numerator, denominator, expected) in [
        (0, 2, "non-zero numerator and denominator"),
        (1, 0, "non-zero numerator and denominator"),
    ] {
        let schema = RenderResourceSchema::texture(
            RenderTextureSchema::new(TextureFormat::R8Unorm, TextureUsage::STORAGE).with_extent(
                RenderTextureExtentPolicy::Relative {
                    reference: RenderTextureExtentReference::View,
                    numerator,
                    denominator,
                    rounding: RenderTextureExtentRounding::Floor,
                },
            ),
        );

        let error = texture_desc_from_schema("invalid-relative", schema, &extract)
            .expect_err("invalid relative extents must be rejected");
        assert!(error.contains(expected), "unexpected error: {error}");
    }

    let overflow = resolve_relative_extent_axis(2, u32::MAX, 1, RenderTextureExtentRounding::Floor)
        .expect_err("relative extents larger than u32 must be rejected");
    assert!(overflow.contains("exceeds the supported u32 texture extent"));
}

#[test]
fn builtin_packet_buffer_requires_the_producer_owned_capacity() {
    let extract = test_extract();
    let error = builtin_buffer_desc_for(
        PostProcessGraphResourceNames::HYBRID_GI_SCENE,
        &extract,
        None,
    )
    .expect_err("packet buffers cannot infer their capacity from a name");
    assert!(
        error.contains("requires a producer-declared minimum_size_bytes"),
        "{error}"
    );

    let desc = builtin_buffer_desc_for(
        PostProcessGraphResourceNames::HYBRID_GI_SCENE,
        &extract,
        Some(128),
    )
    .expect("catalog resolves a producer-owned packet capacity")
    .expect("hybrid GI scene is a built-in packet buffer");
    assert_eq!(desc.size_bytes, 128);
}

#[test]
fn builtin_light_list_uses_the_cluster_buffer_capacity_policy() {
    let extract = test_extract();
    let desc = builtin_buffer_desc_for(PostProcessGraphResourceNames::LIGHT_LIST, &extract, None)
        .expect("catalog resolves the light-list capacity")
        .expect("light-list is a built-in graph buffer");
    let expected = u64::try_from(crate::graphics::scene::cluster_buffer_bytes_for_size(
        extract.view.effective_render_size(),
    ))
    .expect("usize must fit the RHI buffer capacity");

    assert_eq!(desc.size_bytes, expected);
}

#[test]
fn builtin_sss_uniforms_use_exact_shader_layout_capacities() {
    use crate::graphics::scene::{
        SSS_PARAMS_BUFFER_SIZE_BYTES, SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES,
    };

    let extract = test_extract();
    for (name, expected_size) in [
        (
            PostProcessGraphResourceNames::SSS_PARAMS,
            SSS_PARAMS_BUFFER_SIZE_BYTES,
        ),
        (
            PostProcessGraphResourceNames::SSS_PROFILES,
            SSS_PROFILE_TABLE_BUFFER_SIZE_BYTES,
        ),
    ] {
        let desc = builtin_buffer_desc_for(name, &extract, None)
            .expect("SSS uniform catalog lookup succeeds")
            .expect("SSS uniform is a built-in graph buffer");
        assert_eq!(desc.size_bytes, expected_size);
        assert_eq!(desc.usage, BufferUsage::UNIFORM | BufferUsage::COPY_DST);
    }
}
