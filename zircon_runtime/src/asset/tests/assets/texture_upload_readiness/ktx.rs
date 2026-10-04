//! KTX1/KTX2 回归测试检查层级索引、数据格式与描述符的交叉验证；不可信范围必须在进入压缩上传计划前被拒绝。

use super::container_fixtures::*;
use crate::asset::{AssetUri, TextureAsset, TextureUploadSupport};

// KTX1 的层级长度与 KTX2 的索引范围都要覆盖实际压缩块，不能只验证头部存在。
#[test]
fn texture_upload_readiness_rejects_short_ktx_level_declarations() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };
    let mut short_ktx1 = ktx1_bc1_level_bytes();
    write_u32_le(&mut short_ktx1, 64, 1);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/short-level.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        short_ktx1,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some(
            "container texture payload format ktx/gl-internal-0x000083f1 declares 1 image bytes but needs at least 8"
        )
    );

    let mut truncated_ktx2 = ktx2_bc1_level_bytes();
    write_u64_le(&mut truncated_ktx2, 88, 16);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/truncated-level.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        truncated_ktx2,
        1,
        1,
    );
    // BUG: [CR-ASSET-TEST-TEXTURE-0004] KTX2 层声明 16 字节而夹具仅含 8 字节，计划构造先失败并返回通用索引诊断，旧详细文案断言必失败；证据：upload_support/ktx.rs:174-188、upload_support.rs:240-244。
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some(
            "container texture payload format ktx2/vk-133/supercompression-0 declares 16 image bytes but only 8 are available"
        )
    );
}

// 文件标识和字节序先于层级解析；错误头部不能被碰巧合理的偏移掩盖。
#[test]
fn texture_upload_readiness_rejects_malformed_ktx_headers_before_level_parsing() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };
    let mut bad_ktx1_magic = ktx1_bc1_level_bytes();
    bad_ktx1_magic[0] = 0;
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-magic.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        bad_ktx1_magic,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut bad_ktx1_endian = ktx1_bc1_level_bytes();
    write_u32_le(&mut bad_ktx1_endian, 12, 0);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-endian.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        bad_ktx1_endian,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut bad_ktx2_magic = ktx2_bc1_level_bytes();
    bad_ktx2_magic[0] = 0;
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-magic.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        bad_ktx2_magic,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}

// KTX2 索引须完整且未压缩长度与无超压缩声明一致，后续才可按 mip 切分。
#[test]
fn texture_upload_readiness_rejects_malformed_ktx2_level_index_entries() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut incomplete_index = ktx2_bc1_level_bytes();
    incomplete_index.truncate(96);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/incomplete-index.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        incomplete_index,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let mut mismatched_uncompressed_len = ktx2_bc1_level_bytes();
    write_u64_le(&mut mismatched_uncompressed_len, 96, 16);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-index.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        mismatched_uncompressed_len,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}

// 导入描述符与容器的 mip 数是双重来源；失配时不得凭单方信息推断上传范围。
#[test]
fn texture_upload_readiness_rejects_ktx_header_mip_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut mismatched_ktx1_mips = ktx1_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx1_mips, 56, 2);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-mips.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        mismatched_ktx1_mips,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut mismatched_ktx2_levels = ktx2_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx2_levels, 40, 2);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-level-count.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        mismatched_ktx2_levels,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}

// KTX 层与面数共同决定 GPU 图像形状；合法数组仍受当前 KTX1 子资源布局能力限制。
#[test]
fn texture_upload_readiness_rejects_ktx_header_layer_face_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut mismatched_ktx1_layers = ktx1_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx1_layers, 48, 2);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-layer-count.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        mismatched_ktx1_layers,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut mismatched_ktx2_faces = ktx2_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx2_faces, 36, 6);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/stale-face-count.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        mismatched_ktx2_faces,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let mut array_ktx1_bytes = ktx1_bc1_level_bytes();
    write_u32_le(&mut array_ktx1_bytes, 48, 2);
    let array_ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/array-bc1.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        array_ktx1_bytes,
        1,
        2,
    );
    assert_eq!(
        array_ktx1.upload_readiness(support).unsupported_reason(),
        Some("compressed texture array/cubemap upload is not implemented")
    );
}

// 格式 token 与容器头的 GL/Vulkan 编码、超压缩声明必须一致；不做隐式转码或重解释。
#[test]
fn texture_upload_readiness_rejects_ktx_descriptor_header_format_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut mismatched_ktx1_format = ktx1_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx1_format, 28, 0x9274);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-format.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        mismatched_ktx1_format,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut mismatched_ktx2_format = ktx2_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx2_format, 12, 147);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-format.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        mismatched_ktx2_format,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let mut mismatched_ktx2_supercompression = ktx2_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx2_supercompression, 44, 1);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-supercompression.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        mismatched_ktx2_supercompression,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let missing_supercompression_token = TextureAsset::new_container(
        AssetUri::parse("res://textures/missing-supercompression.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133",
        ktx2_bc1_level_bytes(),
        1,
        1,
    );
    assert_eq!(
        missing_supercompression_token
            .upload_readiness(support)
            .unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}

// 结构字段和 DFD 范围同样限定可信 payload；错误元数据即使不碰图像字节也应拒绝。
#[test]
fn texture_upload_readiness_rejects_ktx_structural_header_mismatches() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };

    let mut bad_ktx1_type_size = ktx1_bc1_level_bytes();
    write_u32_le(&mut bad_ktx1_type_size, 20, 2);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-type-size.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        bad_ktx1_type_size,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut bad_ktx1_format_pair = ktx1_bc1_level_bytes();
    write_u32_le(&mut bad_ktx1_format_pair, 24, 0x1908);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-format-pair.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        bad_ktx1_format_pair,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut bad_ktx1_metadata_len = ktx1_bc1_level_bytes();
    write_u32_le(&mut bad_ktx1_metadata_len, 60, 2);
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-metadata-len.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        bad_ktx1_metadata_len,
        1,
        1,
    );
    assert_eq!(
        ktx1.upload_readiness(support).unsupported_reason(),
        Some("ktx texture format or level payload is not upload-ready")
    );

    let mut bad_ktx2_type_size = ktx2_bc1_level_bytes();
    write_u32_le(&mut bad_ktx2_type_size, 16, 0);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/bad-type-size.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        bad_ktx2_type_size,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let mut missing_ktx2_dfd = ktx2_bc1_level_bytes();
    write_u32_le(&mut missing_ktx2_dfd, 48, 0);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/missing-dfd.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        missing_ktx2_dfd,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let mut mismatched_ktx2_dfd_size = ktx2_bc1_level_bytes();
    write_u32_le(&mut mismatched_ktx2_dfd_size, KTX2_TEST_DFD_OFFSET, 16);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/mismatched-dfd-size.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        mismatched_ktx2_dfd_size,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );

    let mut overlapping_ktx2_dfd = ktx2_bc1_level_bytes();
    write_u32_le(&mut overlapping_ktx2_dfd, 48, 128);
    write_u32_le(&mut overlapping_ktx2_dfd, 52, 16);
    write_u32_le(&mut overlapping_ktx2_dfd, 128, 16);
    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/overlapping-dfd.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        overlapping_ktx2_dfd,
        1,
        1,
    );
    assert_eq!(
        ktx2.upload_readiness(support).unsupported_reason(),
        Some("ktx2 texture format or level index is not upload-ready")
    );
}
