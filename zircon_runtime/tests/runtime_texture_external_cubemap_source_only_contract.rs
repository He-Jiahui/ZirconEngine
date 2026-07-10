use zircon_runtime::asset::{
    external_source_cubemap_container_info, AssetUri, ExternalSourceCubemapContainerError,
    ExternalSourceCubemapContainerKind, TextureAsset, TextureUploadCompressionFamily,
    TextureUploadReadiness, TextureUploadSupport,
    EXTERNAL_SOURCE_CUBEMAP_UPLOAD_UNSUPPORTED_REASON,
};
use zircon_runtime::core::framework::render::source_cubemap_mip_count;

#[test]
fn dds_cubemap_with_complete_source_mips_is_ibl_source_only() {
    let face_size = 4;
    let mip_count = source_cubemap_mip_count(face_size);
    let texture = source_cubemap_texture(
        "res://textures/source-lakes.dds",
        "dds/D3DFMT-113",
        dds_classic_cubemap_mip_bytes(face_size, mip_count),
        face_size,
        mip_count,
    );

    let info = external_source_cubemap_container_info(&texture)
        .expect("valid external source cubemap header")
        .expect("dds cubemap should be classified as an external source cubemap");

    assert_eq!(info.kind, ExternalSourceCubemapContainerKind::Dds);
    assert_eq!(info.format, "dds/d3dfmt-113");
    assert_eq!(info.face_size, face_size);
    assert_eq!(info.mip_count, mip_count);
    assert_source_only_upload_rejection(&texture);
}

#[test]
fn ktx1_cubemap_with_complete_source_mips_is_ibl_source_only() {
    let face_size = 4;
    let mip_count = source_cubemap_mip_count(face_size);
    let texture = source_cubemap_texture(
        "res://textures/source-lakes.ktx",
        "ktx/gl-internal-0x0000881a",
        ktx1_bc1_cubemap_mip_bytes(face_size, mip_count),
        face_size,
        mip_count,
    );

    let info = external_source_cubemap_container_info(&texture)
        .expect("valid external source cubemap header")
        .expect("ktx1 cubemap should be classified as an external source cubemap");

    assert_eq!(info.kind, ExternalSourceCubemapContainerKind::Ktx1);
    assert_eq!(info.format, "ktx/gl-internal-0x0000881a");
    assert_eq!(info.face_size, face_size);
    assert_eq!(info.mip_count, mip_count);
    assert_source_only_upload_rejection(&texture);
}

#[test]
fn ktx2_cubemap_with_complete_source_mips_is_ibl_source_only() {
    let face_size = 4;
    let mip_count = source_cubemap_mip_count(face_size);
    let texture = source_cubemap_texture(
        "res://textures/source-lakes.ktx2",
        "ktx2/vk-97/supercompression-0",
        ktx2_bc1_cubemap_mip_bytes(face_size, mip_count),
        face_size,
        mip_count,
    );

    let info = external_source_cubemap_container_info(&texture)
        .expect("valid external source cubemap header")
        .expect("ktx2 cubemap should be classified as an external source cubemap");

    assert_eq!(info.kind, ExternalSourceCubemapContainerKind::Ktx2);
    assert_eq!(info.format, "ktx2/vk-97/supercompression-0");
    assert_eq!(info.face_size, face_size);
    assert_eq!(info.mip_count, mip_count);
    assert_source_only_upload_rejection(&texture);
}

#[test]
fn ordinary_2d_dds_stays_upload_ready_for_material_textures() {
    let texture = TextureAsset::new_container(
        test_uri("res://textures/base-color.dds"),
        4,
        4,
        "dds/DXT1",
        dds_classic_fourcc_bytes(4, 4, "DXT1", bc1_level_bytes(4, 4)),
        1,
        1,
    );

    assert_eq!(
        external_source_cubemap_container_info(&texture).expect("valid 2d dds container header"),
        None
    );
    let support = TextureUploadSupport {
        bc: true,
        ..TextureUploadSupport::uncompressed_only()
    };
    let readiness = texture.upload_readiness(support);
    let TextureUploadReadiness::Ready { plan } = readiness else {
        panic!("ordinary 2d DDS should remain upload-ready, got {readiness:?}");
    };
    assert_eq!(plan.format, "dds/dxt1");
    assert_eq!(plan.compression, TextureUploadCompressionFamily::Bc);
    assert_eq!(plan.data_offset, 128);
    assert_eq!(plan.block_width, 4);
    assert_eq!(plan.block_height, 4);
    assert_eq!(plan.bytes_per_block, 8);
}

#[test]
fn incomplete_external_cubemap_source_mip_chain_is_rejected_before_bake() {
    let face_size = 4;
    let texture = source_cubemap_texture(
        "res://textures/incomplete-source-lakes.dds",
        "dds/DXT1",
        dds_classic_cubemap_mip_bytes(face_size, 2),
        face_size,
        2,
    );

    let error = external_source_cubemap_container_info(&texture)
        .expect_err("incomplete source cubemap chain must be a typed classification error");

    assert!(matches!(
        error,
        ExternalSourceCubemapContainerError::IncompleteMipChain {
            kind: ExternalSourceCubemapContainerKind::Dds,
            face_size: 4,
            expected: 3,
            actual: 2,
        }
    ));
}

fn source_cubemap_texture(
    uri: &str,
    format: &str,
    bytes: Vec<u8>,
    face_size: u32,
    mip_count: u32,
) -> TextureAsset {
    TextureAsset::new_container(
        test_uri(uri),
        face_size,
        face_size,
        format,
        bytes,
        mip_count,
        SOURCE_CUBEMAP_FACE_COUNT,
    )
}

fn assert_source_only_upload_rejection(texture: &TextureAsset) {
    let readiness = texture.upload_readiness(TextureUploadSupport::all_compressed());
    assert_eq!(
        readiness.unsupported_reason(),
        Some(EXTERNAL_SOURCE_CUBEMAP_UPLOAD_UNSUPPORTED_REASON),
        "external source cubemap should not be treated as a PMREM or material upload payload: {readiness:?}"
    );
}

fn dds_classic_fourcc_bytes(
    width: u32,
    height: u32,
    fourcc: &str,
    payload_bytes: usize,
) -> Vec<u8> {
    let mut bytes = vec![0_u8; 128];
    bytes[0..4].copy_from_slice(b"DDS ");
    write_u32_le(&mut bytes, 4, 124);
    write_u32_le(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE);
    write_u32_le(&mut bytes, 12, height);
    write_u32_le(&mut bytes, 16, width);
    write_u32_le(&mut bytes, 20, payload_bytes as u32);
    write_u32_le(&mut bytes, 76, 32);
    write_u32_le(&mut bytes, 80, DDPF_FOURCC);
    bytes[84..88].copy_from_slice(fourcc.as_bytes());
    write_u32_le(&mut bytes, 108, DDSCAPS_TEXTURE);
    bytes.extend(vec![1_u8; payload_bytes]);
    bytes
}

fn dds_classic_cubemap_mip_bytes(face_size: u32, mip_count: u32) -> Vec<u8> {
    let payload_bytes = rgba16f_cubemap_mip_payload_bytes(face_size, mip_count);
    let mut bytes = dds_classic_fourcc_bytes(face_size, face_size, "q\0\0\0", payload_bytes);
    write_u32_le(
        &mut bytes,
        8,
        DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE | DDSD_MIPMAPCOUNT,
    );
    write_u32_le(&mut bytes, 28, mip_count);
    write_u32_le(
        &mut bytes,
        108,
        DDSCAPS_TEXTURE | DDSCAPS_COMPLEX | DDSCAPS_MIPMAP,
    );
    write_u32_le(&mut bytes, 112, DDSCAPS2_CUBEMAP_ALL_FACES);
    bytes
}

fn ktx1_bc1_cubemap_mip_bytes(face_size: u32, mip_count: u32) -> Vec<u8> {
    let mut bytes = vec![0_u8; 64];
    bytes[0..12].copy_from_slice(b"\xABKTX 11\xBB\r\n\x1A\n");
    write_u32_le(&mut bytes, 12, 0x0403_0201);
    write_u32_le(&mut bytes, 16, 0x140b);
    write_u32_le(&mut bytes, 20, 2);
    write_u32_le(&mut bytes, 24, 0x1908);
    write_u32_le(&mut bytes, 28, 0x881a);
    write_u32_le(&mut bytes, 32, 0x1908);
    write_u32_le(&mut bytes, 36, face_size);
    write_u32_le(&mut bytes, 40, face_size);
    write_u32_le(&mut bytes, 52, SOURCE_CUBEMAP_FACE_COUNT);
    write_u32_le(&mut bytes, 56, mip_count);
    write_u32_le(&mut bytes, 60, 0);
    for level in 0..mip_count {
        let extent = mip_extent(face_size, level);
        let face_bytes = extent as usize * extent as usize * 8;
        write_u32_to_vec(&mut bytes, face_bytes as u32);
        bytes.extend(vec![1_u8; face_bytes * SOURCE_CUBEMAP_FACE_COUNT as usize]);
    }
    bytes
}

fn ktx2_bc1_cubemap_mip_bytes(face_size: u32, mip_count: u32) -> Vec<u8> {
    let level_count = mip_count as usize;
    let level_index_offset = 80;
    let level_index_entry_size = 24;
    let level_index_end = level_index_offset + level_count * level_index_entry_size;
    let dfd_offset = align4(level_index_end);
    let dfd_length = 32;
    let mut data_offset = align4(dfd_offset + dfd_length);
    let mut level_payloads = Vec::new();
    for level in 0..mip_count {
        let extent = mip_extent(face_size, level);
        level_payloads
            .push(extent as usize * extent as usize * 8 * SOURCE_CUBEMAP_FACE_COUNT as usize);
    }

    let mut bytes = vec![0_u8; data_offset];
    bytes[0..12].copy_from_slice(b"\xABKTX 20\xBB\r\n\x1A\n");
    write_u32_le(&mut bytes, 12, 97);
    write_u32_le(&mut bytes, 16, 8);
    write_u32_le(&mut bytes, 20, face_size);
    write_u32_le(&mut bytes, 24, face_size);
    write_u32_le(&mut bytes, 36, SOURCE_CUBEMAP_FACE_COUNT);
    write_u32_le(&mut bytes, 40, mip_count);
    write_u32_le(&mut bytes, 44, 0);
    write_u32_le(&mut bytes, 48, dfd_offset as u32);
    write_u32_le(&mut bytes, 52, dfd_length as u32);
    write_u32_le(&mut bytes, dfd_offset, dfd_length as u32);

    for (level, payload_bytes) in level_payloads.iter().enumerate() {
        let index_offset = level_index_offset + level * level_index_entry_size;
        write_u64_le(&mut bytes, index_offset, data_offset as u64);
        write_u64_le(&mut bytes, index_offset + 8, *payload_bytes as u64);
        write_u64_le(&mut bytes, index_offset + 16, *payload_bytes as u64);
        bytes.extend(vec![1_u8; *payload_bytes]);
        data_offset = align4(data_offset + *payload_bytes);
        bytes.resize(data_offset, 0);
    }
    bytes
}

fn rgba16f_cubemap_mip_payload_bytes(face_size: u32, mip_count: u32) -> usize {
    (0..mip_count)
        .map(|level| {
            let extent = mip_extent(face_size, level) as usize;
            extent * extent * 8 * SOURCE_CUBEMAP_FACE_COUNT as usize
        })
        .sum()
}

fn bc1_level_bytes(width: u32, height: u32) -> usize {
    let block_columns = width.max(1).div_ceil(4);
    let block_rows = height.max(1).div_ceil(4);
    (block_columns * block_rows * 8) as usize
}

fn mip_extent(value: u32, level: u32) -> u32 {
    (value >> level).max(1)
}

fn align4(value: usize) -> usize {
    (value + 3) & !3
}

fn test_uri(uri: &str) -> AssetUri {
    AssetUri::parse(uri).expect("valid test texture uri")
}

fn write_u32_to_vec(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64_le(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

const SOURCE_CUBEMAP_FACE_COUNT: u32 = 6;
const DDPF_FOURCC: u32 = 0x0000_0004;
const DDSCAPS_COMPLEX: u32 = 0x0000_0008;
const DDSCAPS_MIPMAP: u32 = 0x0040_0000;
const DDSCAPS_TEXTURE: u32 = 0x0000_1000;
const DDSCAPS2_CUBEMAP: u32 = 0x0000_0200;
const DDSCAPS2_CUBEMAP_POSITIVEX: u32 = 0x0000_0400;
const DDSCAPS2_CUBEMAP_NEGATIVEX: u32 = 0x0000_0800;
const DDSCAPS2_CUBEMAP_POSITIVEY: u32 = 0x0000_1000;
const DDSCAPS2_CUBEMAP_NEGATIVEY: u32 = 0x0000_2000;
const DDSCAPS2_CUBEMAP_POSITIVEZ: u32 = 0x0000_4000;
const DDSCAPS2_CUBEMAP_NEGATIVEZ: u32 = 0x0000_8000;
const DDSCAPS2_CUBEMAP_ALL_FACES: u32 = DDSCAPS2_CUBEMAP
    | DDSCAPS2_CUBEMAP_POSITIVEX
    | DDSCAPS2_CUBEMAP_NEGATIVEX
    | DDSCAPS2_CUBEMAP_POSITIVEY
    | DDSCAPS2_CUBEMAP_NEGATIVEY
    | DDSCAPS2_CUBEMAP_POSITIVEZ
    | DDSCAPS2_CUBEMAP_NEGATIVEZ;
const DDSD_CAPS: u32 = 0x0000_0001;
const DDSD_HEIGHT: u32 = 0x0000_0002;
const DDSD_WIDTH: u32 = 0x0000_0004;
const DDSD_PIXELFORMAT: u32 = 0x0000_1000;
const DDSD_LINEARSIZE: u32 = 0x0008_0000;
const DDSD_MIPMAPCOUNT: u32 = 0x0002_0000;
const DDSD_REQUIRED_FLAGS: u32 = DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT;
