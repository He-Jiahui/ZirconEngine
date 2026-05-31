use super::common::*;
use crate::asset::{AssetUri, TextureAsset, TextureAssetDescriptor, TextureUploadSupport};
use crate::core::framework::render::RenderImageDimension;

#[test]
fn texture_upload_readiness_rejects_compressed_mips_and_arrays_until_full_upload_exists() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mip_chain = TextureAsset::new_container(
        AssetUri::parse("res://textures/bc1-mips.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        dds_legacy_mip_bytes("DXT1", 2, 16),
        2,
        1,
    );
    assert_eq!(
        mip_chain.upload_readiness(support).unsupported_reason(),
        Some("compressed texture mip-chain upload is not implemented")
    );

    let array_layers = TextureAsset::new_container(
        AssetUri::parse("res://textures/bc1-array.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        dds_legacy_cubemap_bytes("DXT1", 48),
        1,
        6,
    );
    assert_eq!(
        array_layers.upload_readiness(support).unsupported_reason(),
        Some("compressed texture array/cubemap upload is not implemented")
    );
}

#[test]
fn texture_upload_readiness_rejects_compressed_1d_and_etc2_3d_boundaries() {
    let support = TextureUploadSupport {
        bc: true,
        etc2: true,
        ..TextureUploadSupport::uncompressed_only()
    };
    let mut d1_descriptor = TextureAssetDescriptor::container("ktx/gl-internal-0x000083f1", 1, 1);
    d1_descriptor.dimension = RenderImageDimension::D1;
    let mut d1_bytes = ktx1_bc1_level_bytes();
    write_u32_le(&mut d1_bytes, 40, 0);
    let d1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/line-bc1.ktx").unwrap(),
        4,
        1,
        "ktx/gl-internal-0x000083f1",
        d1_bytes,
        1,
        1,
    )
    .with_descriptor(d1_descriptor);
    assert_eq!(
        d1.upload_readiness(support).unsupported_reason(),
        Some("compressed texture 1d upload is not implemented")
    );

    let mut etc2_3d_descriptor =
        TextureAssetDescriptor::container("ktx2/vk-147/supercompression-0", 1, 1);
    etc2_3d_descriptor.dimension = RenderImageDimension::D3;
    etc2_3d_descriptor.depth_or_array_layers = 4;
    etc2_3d_descriptor.array_layer_count = 1;
    let mut etc2_3d_bytes = ktx2_etc2_level_bytes();
    write_u32_le(&mut etc2_3d_bytes, 28, 4);
    let etc2_3d = TextureAsset::new_container(
        AssetUri::parse("res://textures/volume-etc2.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-147/supercompression-0",
        etc2_3d_bytes,
        1,
        1,
    )
    .with_descriptor(etc2_3d_descriptor);
    assert_eq!(
        etc2_3d.upload_readiness(support).unsupported_reason(),
        Some("compressed texture ETC2 3d upload is not implemented")
    );
}

#[test]
fn texture_upload_readiness_rejects_astc_descriptor_header_format_mismatches() {
    let support = TextureUploadSupport {
        astc_ldr: true,
        astc_sliced_3d: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut bad_magic = astc_container_bytes(4, 4, 1, 4, 4, 1, 16);
    bad_magic[0] = 0;
    let texture = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-magic.astc").unwrap(),
        4,
        4,
        "astc/4x4x1",
        bad_magic,
        1,
        1,
    );
    assert_eq!(
        texture.upload_readiness(support).unsupported_reason(),
        Some("texture container format astc/4x4x1 is not upload-ready")
    );

    let mismatched_block = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-block.astc").unwrap(),
        4,
        4,
        "astc/4x4x1",
        astc_container_bytes(6, 6, 1, 4, 4, 1, 16),
        1,
        1,
    );
    assert_eq!(
        mismatched_block
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format astc/4x4x1 is not upload-ready")
    );

    let mismatched_extent = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-extent.astc").unwrap(),
        4,
        4,
        "astc/4x4x1",
        astc_container_bytes(4, 4, 1, 8, 4, 1, 16),
        1,
        1,
    );
    assert_eq!(
        mismatched_extent
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format astc/4x4x1 is not upload-ready")
    );

    let volume_depth_mismatch = astc_3d_container(
        "res://textures/mismatched-depth.astc",
        4,
        4,
        4,
        "astc/4x4x4",
        astc_container_bytes(4, 4, 4, 4, 4, 2, 16),
    );
    assert_eq!(
        volume_depth_mismatch
            .upload_readiness(support)
            .unsupported_reason(),
        Some("texture container format astc/4x4x4 is not upload-ready")
    );
}

#[test]
fn texture_upload_readiness_rejects_container_header_extent_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        etc2: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut mismatched_dds_height = dds_legacy_bytes("DXT1", 8);
    write_u32_le(&mut mismatched_dds_height, 12, 8);
    let dds = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-height.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        mismatched_dds_height,
        1,
        1,
    );
    assert_eq!(
        dds.upload_readiness(support).unsupported_reason(),
        Some("texture container format dds/DXT1 is not upload-ready")
    );

    let mut mismatched_ktx1_height = ktx1_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx1_height, 40, 2);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-height.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        mismatched_ktx1_height,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut ktx2_volume_descriptor =
        TextureAssetDescriptor::container("ktx2/vk-147/supercompression-0", 1, 1);
    ktx2_volume_descriptor.dimension = RenderImageDimension::D3;
    ktx2_volume_descriptor.depth_or_array_layers = 4;
    ktx2_volume_descriptor.array_layer_count = 1;
    let mut mismatched_ktx2_depth = ktx2_etc2_level_bytes();
    write_u32_le(&mut mismatched_ktx2_depth, 28, 2);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-volume-depth.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-147/supercompression-0",
        mismatched_ktx2_depth,
        1,
        1,
    )
    .with_descriptor(ktx2_volume_descriptor);
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}

#[test]
fn texture_upload_readiness_reports_supercompression_and_astc_3d_boundaries() {
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/super.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-37/supercompression-1",
        vec![0; 96],
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(TextureUploadSupport::all_compressed())
            .unsupported_reason(),
        Some("ktx2 supercompression 1 requires a transcoding backend")
    );

    let astc_3d = astc_3d_container(
        "res://textures/volume.astc",
        4,
        4,
        4,
        "astc/4x4x4",
        astc_container_bytes(4, 4, 4, 4, 4, 4, 16),
    );
    assert_eq!(
        astc_3d
            .upload_readiness(TextureUploadSupport {
                astc_ldr: true,
                ..TextureUploadSupport::uncompressed_only()
            })
            .unsupported_reason(),
        Some("gpu device does not support ASTC sliced 3d textures")
    );

    let astc_3d = astc_3d_container(
        "res://textures/volume-3x3x3.astc",
        3,
        3,
        3,
        "astc/3x3x3",
        astc_container_bytes(3, 3, 3, 3, 3, 3, 16),
    );
    assert_eq!(
        astc_3d
            .upload_readiness(TextureUploadSupport {
                astc_ldr: true,
                astc_sliced_3d: true,
                ..TextureUploadSupport::uncompressed_only()
            })
            .unsupported_reason(),
        Some("astc 3d block payload upload is not implemented")
    );

    let astc_unknown = TextureAsset::new_container(
        AssetUri::parse("res://textures/unknown-block.astc").unwrap(),
        7,
        7,
        "astc/7x7x7",
        vec![0_u8; 32],
        1,
        1,
    );
    assert_eq!(
        astc_unknown
            .upload_readiness(TextureUploadSupport::all_compressed())
            .unsupported_reason(),
        Some("texture container format astc/7x7x7 is not upload-ready")
    );
}
