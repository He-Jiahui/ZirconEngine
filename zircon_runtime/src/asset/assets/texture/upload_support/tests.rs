use super::ktx::{KTX2_IDENTIFIER, KTX2_LEVEL_INDEX_OFFSET};
use super::TextureUploadSupport;
use crate::asset::{
    AssetUri, TextureAsset, TextureAssetDescriptor, RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT,
};
use crate::core::framework::render::{RenderImageColorSpace, RenderImageDimension};

#[test]
fn ktx2_upload_plan_rejects_level_payload_inside_level_index() {
    let mut bytes = ktx2_bc1_level_bytes();
    write_u64_le(&mut bytes, KTX2_LEVEL_INDEX_OFFSET, 88);
    let texture = TextureAsset::new_container(
        AssetUri::parse("res://textures/overlapping-index.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        bytes,
        1,
        1,
    );

    assert_eq!(
        texture
            .upload_readiness(TextureUploadSupport {
                bc: true,
                ..TextureUploadSupport::uncompressed_only()
            })
            .unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}

#[test]
fn rgba8_upload_readiness_rejects_layered_shapes_before_byte_length_check() {
    let mut array_descriptor = TextureAssetDescriptor::rgba8_srgb();
    array_descriptor.depth_or_array_layers = 2;
    array_descriptor.array_layer_count = 2;
    let array_texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/stacked-array.png").unwrap(),
        2,
        2,
        vec![0_u8; 2 * 2 * 2 * 4],
    )
    .with_descriptor(array_descriptor);
    assert_eq!(
        array_texture
            .upload_readiness(TextureUploadSupport::uncompressed_only())
            .unsupported_reason(),
        Some("rgba8 texture array/cubemap upload is not implemented")
    );

    let mut volume_descriptor = TextureAssetDescriptor::rgba8_srgb();
    volume_descriptor.dimension = RenderImageDimension::D3;
    volume_descriptor.depth_or_array_layers = 4;
    volume_descriptor.array_layer_count = 1;
    let volume_texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/volume.png").unwrap(),
        2,
        2,
        vec![0_u8; 2 * 2 * 4 * 4],
    )
    .with_descriptor(volume_descriptor);
    assert_eq!(
        volume_texture
            .upload_readiness(TextureUploadSupport::uncompressed_only())
            .unsupported_reason(),
        Some("rgba8 texture 3d upload is not implemented")
    );

    let mut mip_descriptor = TextureAssetDescriptor::rgba8_srgb();
    mip_descriptor.mip_count = 2;
    let mip_texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/mips.png").unwrap(),
        2,
        2,
        vec![0_u8; 2 * 2 * 4],
    )
    .with_descriptor(mip_descriptor);
    assert_eq!(
        mip_texture
            .upload_readiness(TextureUploadSupport::uncompressed_only())
            .unsupported_reason(),
        Some("rgba8 texture mip-chain upload is not implemented")
    );
}

#[test]
fn rgba8_upload_readiness_reports_linear_upload_format_for_linear_color_space() {
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.color_space = RenderImageColorSpace::Linear;
    let linear_texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/linear-mask.png").unwrap(),
        2,
        2,
        vec![0_u8; 2 * 2 * 4],
    )
    .with_descriptor(descriptor);
    let super::TextureUploadReadiness::Ready { plan } =
        linear_texture.upload_readiness(TextureUploadSupport::uncompressed_only())
    else {
        panic!("linear rgba8 texture should be upload-ready");
    };
    assert_eq!(plan.format, RGBA8_UNORM_FORMAT);

    let srgb_texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/albedo.png").unwrap(),
        2,
        2,
        vec![0_u8; 2 * 2 * 4],
    );
    let super::TextureUploadReadiness::Ready { plan } =
        srgb_texture.upload_readiness(TextureUploadSupport::uncompressed_only())
    else {
        panic!("srgb rgba8 texture should be upload-ready");
    };
    assert_eq!(plan.format, RGBA8_UNORM_SRGB_FORMAT);
}

#[test]
fn rgba8_upload_readiness_rejects_descriptor_formats_that_need_conversion() {
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.format = "rgba16float".to_string();
    descriptor.color_space = RenderImageColorSpace::Linear;
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/height.png").unwrap(),
        2,
        2,
        vec![0_u8; 2 * 2 * 4],
    )
    .with_descriptor(descriptor);

    assert_eq!(
        texture
            .upload_readiness(TextureUploadSupport::uncompressed_only())
            .unsupported_reason(),
        Some("rgba8 texture descriptor format rgba16float requires conversion before upload")
    );
}

#[test]
fn compressed_upload_readiness_reports_shape_before_feature_support() {
    let texture = TextureAsset::new_container(
        AssetUri::parse("res://textures/mip-chain.astc").unwrap(),
        4,
        4,
        "astc/4x4x1",
        astc_4x4_level_bytes(),
        2,
        1,
    );

    assert_eq!(
        texture
            .upload_readiness(TextureUploadSupport::uncompressed_only())
            .unsupported_reason(),
        Some("compressed texture mip-chain upload is not implemented")
    );
}

fn ktx2_bc1_level_bytes() -> Vec<u8> {
    let mut bytes = vec![0_u8; KTX2_TEST_LEVEL_DATA_OFFSET];
    bytes[0..12].copy_from_slice(KTX2_IDENTIFIER);
    write_u32_le(&mut bytes, 12, 133);
    write_u32_le(&mut bytes, 16, 1);
    write_u32_le(&mut bytes, 20, 4);
    write_u32_le(&mut bytes, 24, 4);
    write_u32_le(&mut bytes, 40, 1);
    write_u32_le(&mut bytes, 44, 0);
    write_u32_le(&mut bytes, 48, KTX2_TEST_DFD_OFFSET as u32);
    write_u32_le(&mut bytes, 52, KTX2_TEST_DFD_LENGTH as u32);
    write_u64_le(&mut bytes, 80, KTX2_TEST_LEVEL_DATA_OFFSET as u64);
    write_u64_le(&mut bytes, 88, 8);
    write_u64_le(&mut bytes, 96, 8);
    write_u32_le(
        &mut bytes,
        KTX2_TEST_DFD_OFFSET,
        KTX2_TEST_DFD_LENGTH as u32,
    );
    bytes.extend_from_slice(&[1_u8; 8]);
    bytes
}

fn astc_4x4_level_bytes() -> Vec<u8> {
    let mut bytes = vec![0_u8; 32];
    bytes[0..4].copy_from_slice(b"\x13\xAB\xA1\x5C");
    bytes[4] = 4;
    bytes[5] = 4;
    bytes[6] = 1;
    write_u24_le(&mut bytes, 7, 4);
    write_u24_le(&mut bytes, 10, 4);
    write_u24_le(&mut bytes, 13, 1);
    bytes[16..32].fill(1);
    bytes
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64_le(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn write_u24_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 3].copy_from_slice(&value.to_le_bytes()[..3]);
}

const KTX2_TEST_DFD_OFFSET: usize = 104;
const KTX2_TEST_DFD_LENGTH: usize = 32;
const KTX2_TEST_LEVEL_DATA_OFFSET: usize = 136;
