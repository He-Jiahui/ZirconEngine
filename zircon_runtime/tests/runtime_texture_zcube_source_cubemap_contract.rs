use zircon_runtime::asset::{
    decode_zcube_source_cubemap_texture, texture_asset_from_source_cubemap_zcube, AssetUri,
    TexturePayload, TextureUploadReadiness, TextureUploadSupport, ZCUBE_SOURCE_CUBEMAP_FORMAT,
    ZCUBE_SOURCE_CUBEMAP_GPU_FORMAT, ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE,
};
use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, source_cubemap_sample_count, IblBakeArtifactBlob,
    IblBakeArtifactContents, IblBakeArtifactRequest, ProceduralSkyParams, RenderImageColorSpace,
    RenderImageDimension, RGBA16F_TEXEL_SIZE_BYTES, SOURCE_CUBEMAP_FACE_COUNT,
};
use zircon_runtime::core::math::Real;

#[test]
fn zcube_source_cubemap_texture_preserves_source_mips_only() {
    let source = high_frequency_source_cubemap();
    let texture = texture_asset_from_source_cubemap_zcube(test_uri(), &source);
    let descriptor = texture.texture_descriptor();

    assert_eq!(texture.width, source.source_face_size());
    assert_eq!(texture.height, source.source_face_size());
    assert_eq!(descriptor.format, ZCUBE_SOURCE_CUBEMAP_GPU_FORMAT);
    assert_eq!(descriptor.color_space, RenderImageColorSpace::Linear);
    assert_eq!(descriptor.dimension, RenderImageDimension::Cube);
    assert_eq!(descriptor.mip_count, source.source_mip_count());
    assert_eq!(
        descriptor.array_layer_count,
        SOURCE_CUBEMAP_FACE_COUNT as u32
    );

    let TexturePayload::Container {
        format,
        bytes,
        mip_count,
        array_layers,
    } = &texture.payload
    else {
        panic!(".zcube texture must use a container payload");
    };
    assert_eq!(format, ZCUBE_SOURCE_CUBEMAP_FORMAT);
    assert_eq!(*mip_count, source.source_mip_count());
    assert_eq!(*array_layers, SOURCE_CUBEMAP_FACE_COUNT as u32);
    assert_eq!(
        bytes.len(),
        ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE
            + source_cubemap_sample_count(source.source_face_size(), source.source_mip_count(),)
                * RGBA16F_TEXEL_SIZE_BYTES
    );

    let decoded = decode_zcube_source_cubemap_texture(&texture).expect("valid zcube source");
    assert_eq!(decoded.face_size(), source.source_face_size());
    assert_eq!(decoded.mip_count(), source.source_mip_count());
    assert_rgba16f_close(decoded.texels(), source.source_texels());

    assert_ne!(source.source_face_size(), source.pmrem_face_size());
    assert_ne!(source.source_texels().len(), source.pmrem_texels().len());
}

#[test]
fn zcube_source_cubemap_is_not_a_direct_upload_or_zribl_artifact() {
    let source = high_frequency_source_cubemap();
    let texture = texture_asset_from_source_cubemap_zcube(test_uri(), &source);

    let readiness = texture.upload_readiness(TextureUploadSupport::all_compressed());
    assert!(
        matches!(readiness, TextureUploadReadiness::Unsupported { .. }),
        ".zcube should stay source-only until an IBL bake/import path decodes it, got {readiness:?}"
    );
    assert!(
        readiness
            .unsupported_reason()
            .expect("unsupported reason")
            .contains("source cubemap mip container"),
        "unexpected upload readiness: {readiness:?}"
    );

    let TexturePayload::Container { bytes, .. } = &texture.payload else {
        panic!(".zcube texture must use a container payload");
    };
    let request = IblBakeArtifactRequest::new(
        ProceduralSkyParams::default_gradient().ibl_bake_key(),
        source.source_face_size(),
        source.source_mip_count(),
    )
    .with_required_contents(IblBakeArtifactContents::PMREM_SH9);

    assert!(
        IblBakeArtifactBlob::decode_current_for_request(&request, bytes).is_err(),
        ".zcube bytes must not be accepted as a reusable PMREM/SH9 .zribl artifact"
    );
}

fn high_frequency_source_cubemap() -> zircon_runtime::core::framework::render::SourceCubemapMipChain
{
    build_source_cubemap_from_equirect(8, |u, v| {
        let cell_x = (u * 31.0).floor() as i32;
        let cell_y = (v * 17.0).floor() as i32;
        let stripe = if (cell_x + cell_y) & 1 == 0 {
            0.15
        } else {
            1.75
        };
        [stripe, 0.35 + u * 0.8, 0.25 + (1.0 - v) * 1.1, 1.0]
    })
}

fn test_uri() -> AssetUri {
    AssetUri::parse("res://textures/test_environment.zcube").expect("valid texture uri")
}

fn assert_rgba16f_close(actual: &[[Real; 4]], expected: &[[Real; 4]]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        for channel in 0..4 {
            let delta = (actual[channel] - expected[channel]).abs();
            assert!(
                delta <= 0.0015,
                "texel {index} channel {channel} differs after RGBA16F zcube roundtrip: actual={}, expected={}, delta={delta}",
                actual[channel],
                expected[channel]
            );
        }
    }
}
