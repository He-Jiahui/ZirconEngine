use zircon_runtime::asset::{
    decode_zcube_source_cubemap_texture, texture_asset_from_source_cubemap_zcube, AssetUri,
    TexturePayload, TextureUploadReadiness, TextureUploadSupport, ZCUBE_SOURCE_CUBEMAP_FORMAT,
    ZCUBE_SOURCE_CUBEMAP_GPU_FORMAT, ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE,
};
use zircon_runtime::core::framework::render::{
    build_source_cubemap_from_equirect, source_cubemap_face_mip_offset,
    source_cubemap_sample_count, CubemapFace, IblBakeArtifactBlob, IblBakeArtifactContents,
    IblBakeArtifactRequest, ProceduralSkyParams, RenderImageColorSpace, RenderImageDimension,
    RGBA16F_TEXEL_SIZE_BYTES, SOURCE_CUBEMAP_FACE_COUNT,
};
use zircon_runtime::core::math::Real;

#[test]
fn zcube_source_cubemap_texture_preserves_source_mips_only() {
    let source = high_frequency_source_cubemap();
    let texture = texture_asset_from_source_cubemap_zcube(test_uri(), &source);
    let descriptor = texture.texture_descriptor();

    assert_eq!(texture.width, source.face_size());
    assert_eq!(texture.height, source.face_size());
    assert_eq!(descriptor.format, ZCUBE_SOURCE_CUBEMAP_GPU_FORMAT);
    assert_eq!(descriptor.color_space, RenderImageColorSpace::Linear);
    assert_eq!(descriptor.dimension, RenderImageDimension::Cube);
    assert_eq!(descriptor.mip_count, source.mip_count());
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
    assert_eq!(*mip_count, source.mip_count());
    assert_eq!(*array_layers, SOURCE_CUBEMAP_FACE_COUNT as u32);
    assert_eq!(
        bytes.len(),
        ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE
            + source_cubemap_sample_count(source.face_size(), source.mip_count())
                * RGBA16F_TEXEL_SIZE_BYTES
    );

    let decoded = decode_zcube_source_cubemap_texture(&texture).expect("valid zcube source");
    assert_eq!(decoded.face_size(), source.face_size());
    assert_eq!(decoded.mip_count(), source.mip_count());
    assert_rgba16f_close(decoded.texels(), source.source_texels());

    let rough_source_pmrem_delta = mip_average_delta(
        source.source_texels(),
        source.texels(),
        source.face_size(),
        source.mip_count(),
        source.mip_count().saturating_sub(2),
    );
    assert!(
        rough_source_pmrem_delta > 0.005,
        ".zcube should preserve the source mip pyramid, not the PMREM chain; delta={rough_source_pmrem_delta}"
    );
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
        source.face_size(),
        source.mip_count(),
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

fn mip_average_delta(
    lhs: &[[Real; 4]],
    rhs: &[[Real; 4]],
    face_size: u32,
    mip_count: u32,
    mip_level: u32,
) -> Real {
    let mip_level = mip_level.min(mip_count.saturating_sub(1));
    let mip_size = (face_size >> mip_level).max(1);
    let mut sum = 0.0;
    let mut count = 0_u32;
    for face in CubemapFace::ALL {
        let offset = source_cubemap_face_mip_offset(face_size, mip_count, face, mip_level);
        let texel_count = mip_size as usize * mip_size as usize;
        for index in offset..offset + texel_count {
            sum += (lhs[index][0] - rhs[index][0]).abs();
            sum += (lhs[index][1] - rhs[index][1]).abs();
            sum += (lhs[index][2] - rhs[index][2]).abs();
            count += 3;
        }
    }
    sum / count.max(1) as Real
}
