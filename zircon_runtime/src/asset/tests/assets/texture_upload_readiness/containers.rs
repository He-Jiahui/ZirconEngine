//! 压缩容器的正向案例检查格式识别、设备特性和实际 payload 范围；上传计划随后供渲染资源解析与制品烹制使用。

use super::container_fixtures::*;
use crate::asset::{
    AssetUri, TextureAsset, TextureUploadCompressionFamily, TextureUploadReadiness,
    TextureUploadSupport,
};

// 有效容器与设备能力缺失必须分别报告；只有格式、头部和字节布局均成立才可形成上传计划。
#[test]
fn texture_upload_readiness_reports_compressed_container_support() {
    let uri = AssetUri::parse("res://textures/bc1.dds").unwrap();
    let bytes = dds_classic_fourcc_bytes("DXT1", 8);
    let texture = TextureAsset::new_container(uri, 4, 4, "dds/DXT1", bytes, 1, 1);

    let unsupported = texture.upload_readiness(TextureUploadSupport::uncompressed_only());
    assert_eq!(
        unsupported.unsupported_reason(),
        Some("gpu device does not support BC compressed textures")
    );
    let short = TextureAsset::new_container(
        AssetUri::parse("res://textures/short-bc1.dds").unwrap(),
        4,
        4,
        "dds/DXT1",
        dds_classic_fourcc_bytes("DXT1", 1),
        1,
        1,
    );
    // BUG: [CR-ASSET-TEST-TEXTURE-0001] 1 字节 DDS 生成非空子资源列表后先报缺少压缩子资源字节，旧连续长度文案断言必失败；证据：upload_support/dds.rs:92、compressed.rs:20-24,203-207。
    assert_eq!(
        short
            .upload_readiness(TextureUploadSupport {
                bc: true,
                ..TextureUploadSupport::uncompressed_only()
            })
            .unsupported_reason(),
        Some("container texture payload format dds/dxt1 has 1 image bytes but needs at least 8")
    );

    match texture.upload_readiness(TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    }) {
        TextureUploadReadiness::Ready { plan } => {
            assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
            assert_eq!(plan.data_offset, 128);
            assert_eq!(plan.data_length, None);
            assert_eq!(plan.bytes_per_block, 8);
        }
        other => panic!("expected BC upload-ready texture, got {other:?}"),
    }

    for (fourcc, expected_bytes_per_block) in [
        ("ATI1", 8),
        ("BC4U", 8),
        ("BC4S", 8),
        ("ATI2", 16),
        ("BC5U", 16),
        ("BC5S", 16),
    ] {
        let bytes = dds_classic_fourcc_bytes(fourcc, expected_bytes_per_block as usize);
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/{fourcc}.dds")).unwrap(),
            4,
            4,
            format!("dds/{fourcc}"),
            bytes,
            1,
            1,
        );
        match texture.upload_readiness(TextureUploadSupport {
            bc: true,
            ..TextureUploadSupport::uncompressed_only()
        }) {
            TextureUploadReadiness::Ready { plan } => {
                assert_eq!(plan.format, format!("dds/{}", fourcc.to_ascii_lowercase()));
                assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
                assert_eq!(plan.data_offset, 128);
                assert_eq!(plan.bytes_per_block, expected_bytes_per_block);
            }
            other => panic!("expected DDS {fourcc} upload-ready texture, got {other:?}"),
        }
    }

    for (dxgi, expected_bytes_per_block) in [
        (74, 16),
        (75, 16),
        (77, 16),
        (78, 16),
        (80, 8),
        (81, 8),
        (83, 16),
        (84, 16),
        (95, 16),
        (96, 16),
        (98, 16),
        (99, 16),
    ] {
        let bytes = dds_dx10_bytes(dxgi, expected_bytes_per_block as usize);
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/dxgi-{dxgi}.dds")).unwrap(),
            4,
            4,
            format!("dds/dxgi-{dxgi}"),
            bytes,
            1,
            1,
        );
        match texture.upload_readiness(TextureUploadSupport {
            bc: true,
            ..TextureUploadSupport::uncompressed_only()
        }) {
            TextureUploadReadiness::Ready { plan } => {
                assert_eq!(plan.format, format!("dds/dxgi-{dxgi}"));
                assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
                assert_eq!(plan.data_offset, 148);
                assert_eq!(plan.bytes_per_block, expected_bytes_per_block);
            }
            other => panic!("expected DXGI {dxgi} BC upload-ready texture, got {other:?}"),
        }
    }

    // typeless DXGI 编码不能直接交给 GPU；导入阶段须先解析为明确的可上传格式。
    for (dxgi, expected_bytes_per_block) in [
        (70, 8_usize),
        (73, 16),
        (76, 16),
        (79, 8),
        (82, 16),
        (94, 16),
        (97, 16),
    ] {
        let typeless_bytes = dds_dx10_bytes(dxgi, expected_bytes_per_block);
        let typeless = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/dxgi-{dxgi}.dds")).unwrap(),
            4,
            4,
            format!("dds/dxgi-{dxgi}"),
            typeless_bytes,
            1,
            1,
        );
        let readiness = typeless.upload_readiness(TextureUploadSupport {
            bc: true,
            ..TextureUploadSupport::uncompressed_only()
        });
        let expected_reason =
            format!("texture container format dds/dxgi-{dxgi} is not upload-ready");
        assert_eq!(
            readiness.unsupported_reason(),
            Some(expected_reason.as_str())
        );
    }
}

// KTX 头部和层级索引不能作为纹理字节上传；此处固定范围与块布局，供后续切分子资源。
#[test]
fn texture_upload_readiness_extracts_ktx_level_payload_offsets() {
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };
    let ktx1 = TextureAsset::new_container(
        AssetUri::parse("res://textures/ktx-bc1.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000083f1",
        ktx1_bc1_level_bytes(),
        1,
        1,
    );

    match ktx1.upload_readiness(support) {
        TextureUploadReadiness::Ready { plan } => {
            assert_eq!(plan.format, "ktx/gl-internal-0x000083f1");
            assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
            assert_eq!(plan.data_offset, 68);
            assert_eq!(plan.data_length, Some(8));
            assert_eq!(plan.bytes_per_block, 8);
        }
        other => panic!("expected KTX1 BC upload-ready texture, got {other:?}"),
    }

    for (gl_internal_format, expected_bytes_per_block, expected_level_bytes) in [
        (0x8dbb, 8, 8),
        (0x8dbc, 8, 8),
        (0x8dbd, 16, 16),
        (0x8dbe, 16, 16),
        (0x8e8c, 16, 16),
        (0x8e8d, 16, 16),
        (0x8e8e, 16, 16),
        (0x8e8f, 16, 16),
    ] {
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/ktx-bc-{gl_internal_format:x}.ktx")).unwrap(),
            4,
            4,
            format!("ktx/gl-internal-0x{gl_internal_format:08x}"),
            ktx1_compressed_level_bytes(gl_internal_format, expected_level_bytes),
            1,
            1,
        );
        match texture.upload_readiness(support) {
            TextureUploadReadiness::Ready { plan } => {
                assert_eq!(
                    plan.format,
                    format!("ktx/gl-internal-0x{gl_internal_format:08x}")
                );
                assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
                assert_eq!(plan.data_offset, 68);
                assert_eq!(plan.data_length, Some(expected_level_bytes));
                assert_eq!(plan.bytes_per_block, expected_bytes_per_block);
            }
            other => panic!(
                "expected KTX1 BC gl-internal 0x{gl_internal_format:08x} upload-ready texture, got {other:?}"
            ),
        }
    }

    let ktx1_astc = TextureAsset::new_container(
        AssetUri::parse("res://textures/ktx-astc.ktx").unwrap(),
        4,
        4,
        "ktx/gl-internal-0x000093b0",
        ktx1_compressed_level_bytes(0x93b0, 16),
        1,
        1,
    );
    assert_eq!(
        ktx1_astc.upload_readiness(support).unsupported_reason(),
        Some("gpu device does not support ASTC compressed textures")
    );

    match ktx1_astc.upload_readiness(TextureUploadSupport {
        bc: true,
        astc_ldr: true,
        ..TextureUploadSupport::uncompressed_only()
    }) {
        TextureUploadReadiness::Ready { plan } => {
            assert_eq!(plan.format, "ktx/gl-internal-0x000093b0");
            assert_eq!(plan.compression, TextureUploadCompressionFamily::Astc);
            assert_eq!(plan.data_offset, 68);
            assert_eq!(plan.data_length, Some(16));
            assert_eq!(plan.block_width, 4);
            assert_eq!(plan.block_height, 4);
            assert_eq!(plan.bytes_per_block, 16);
        }
        other => panic!("expected KTX1 ASTC upload-ready texture, got {other:?}"),
    }

    let ktx2 = TextureAsset::new_container(
        AssetUri::parse("res://textures/ktx2-bc1.ktx2").unwrap(),
        4,
        4,
        "ktx2/vk-133/supercompression-0",
        ktx2_bc1_level_bytes(),
        1,
        1,
    );

    match ktx2.upload_readiness(support) {
        TextureUploadReadiness::Ready { plan } => {
            assert_eq!(plan.format, "ktx2/vk-133/supercompression-0");
            assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
            assert_eq!(plan.data_offset, KTX2_TEST_LEVEL_DATA_OFFSET);
            // BUG: [CR-ASSET-TEST-TEXTURE-0002] KTX2 正常计划将 data_length 设为 None，断言 Some(8) 必失败；证据：upload_support/ktx.rs:124-134。
            assert_eq!(plan.data_length, Some(8));
            assert_eq!(plan.bytes_per_block, 8);
        }
        other => panic!("expected KTX2 BC upload-ready texture, got {other:?}"),
    }

    for (vk_format, expected_bytes_per_block, expected_level_bytes) in [
        (139, 8, 8),
        (140, 8, 8),
        (141, 16, 16),
        (142, 16, 16),
        (143, 16, 16),
        (144, 16, 16),
    ] {
        let texture = TextureAsset::new_container(
            AssetUri::parse(&format!("res://textures/ktx2-bc-{vk_format}.ktx2")).unwrap(),
            4,
            4,
            format!("ktx2/vk-{vk_format}/supercompression-0"),
            ktx2_compressed_level_bytes(vk_format, expected_level_bytes),
            1,
            1,
        );
        match texture.upload_readiness(support) {
            TextureUploadReadiness::Ready { plan } => {
                assert_eq!(
                    plan.format,
                    format!("ktx2/vk-{vk_format}/supercompression-0")
                );
                assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
                assert_eq!(plan.data_offset, KTX2_TEST_LEVEL_DATA_OFFSET);
                assert_eq!(plan.data_length, Some(expected_level_bytes));
                assert_eq!(plan.bytes_per_block, expected_bytes_per_block);
            }
            other => panic!("expected KTX2 BC vk-{vk_format} upload-ready texture, got {other:?}"),
        }
    }
}
