use crate::asset::{AssetUri, TextureAsset, TextureAssetDescriptor};
use crate::core::framework::render::RenderImageDimension;

pub(super) fn ktx1_bc1_level_bytes() -> Vec<u8> {
    ktx1_compressed_level_bytes(0x83f1, 8)
}

pub(super) fn dds_legacy_bytes(fourcc: &str, payload_bytes: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; 128];
    bytes[0..4].copy_from_slice(b"DDS ");
    write_u32_le(&mut bytes, 4, 124);
    write_u32_le(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE);
    write_u32_le(&mut bytes, 12, 4);
    write_u32_le(&mut bytes, 16, 4);
    write_u32_le(&mut bytes, 20, payload_bytes as u32);
    write_u32_le(&mut bytes, 76, 32);
    write_u32_le(&mut bytes, 80, DDPF_FOURCC);
    bytes[84..88].copy_from_slice(fourcc.as_bytes());
    write_u32_le(&mut bytes, 108, DDSCAPS_TEXTURE);
    bytes.extend(vec![1_u8; payload_bytes]);
    bytes
}

pub(super) fn dds_legacy_mip_bytes(fourcc: &str, mip_count: u32, payload_bytes: usize) -> Vec<u8> {
    let mut bytes = dds_legacy_bytes(fourcc, payload_bytes);
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
    bytes
}

pub(super) fn dds_legacy_cubemap_bytes(fourcc: &str, payload_bytes: usize) -> Vec<u8> {
    let mut bytes = dds_legacy_bytes(fourcc, payload_bytes);
    write_u32_le(&mut bytes, 108, DDSCAPS_TEXTURE | DDSCAPS_COMPLEX);
    write_u32_le(&mut bytes, 112, DDSCAPS2_CUBEMAP_ALL_FACES);
    bytes
}

pub(super) fn dds_dx10_bytes(dxgi_format: u32, payload_bytes: usize) -> Vec<u8> {
    let mut bytes = dds_legacy_bytes("DX10", 0);
    bytes.resize(148, 0);
    write_u32_le(&mut bytes, 20, payload_bytes as u32);
    write_u32_le(&mut bytes, 128, dxgi_format);
    write_u32_le(&mut bytes, 132, 3);
    write_u32_le(&mut bytes, 140, 1);
    bytes.extend(vec![1_u8; payload_bytes]);
    bytes
}

pub(super) fn dds_dx10_array_bytes(
    dxgi_format: u32,
    array_size: u32,
    texturecube: bool,
    payload_bytes: usize,
) -> Vec<u8> {
    let mut bytes = dds_dx10_bytes(dxgi_format, payload_bytes);
    write_u32_le(&mut bytes, 136, if texturecube { 0x4 } else { 0 });
    write_u32_le(&mut bytes, 140, array_size);
    bytes
}

pub(super) fn astc_3d_container(
    uri: &str,
    width: u32,
    height: u32,
    depth: u32,
    format: &str,
    bytes: Vec<u8>,
) -> TextureAsset {
    let mut descriptor = TextureAssetDescriptor::container(format, 1, 1);
    descriptor.dimension = RenderImageDimension::D3;
    descriptor.depth_or_array_layers = depth.max(1);
    descriptor.array_layer_count = 1;
    TextureAsset::new_container(
        AssetUri::parse(uri).unwrap(),
        width,
        height,
        format,
        bytes,
        1,
        1,
    )
    .with_descriptor(descriptor)
}

pub(super) fn astc_container_bytes(
    block_width: u8,
    block_height: u8,
    block_depth: u8,
    width: u32,
    height: u32,
    depth: u32,
    payload_bytes: usize,
) -> Vec<u8> {
    let mut bytes = vec![0_u8; 16];
    bytes[0..4].copy_from_slice(b"\x13\xAB\xA1\x5C");
    bytes[4] = block_width;
    bytes[5] = block_height;
    bytes[6] = block_depth;
    write_u24_le(&mut bytes, 7, width);
    write_u24_le(&mut bytes, 10, height);
    write_u24_le(&mut bytes, 13, depth);
    bytes.extend(vec![1_u8; payload_bytes]);
    bytes
}

pub(super) fn ktx1_compressed_level_bytes(gl_internal_format: u32, level_bytes: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; 64];
    bytes[0..12].copy_from_slice(b"\xABKTX 11\xBB\r\n\x1A\n");
    write_u32_le(&mut bytes, 12, 0x0403_0201);
    write_u32_le(&mut bytes, 20, 1);
    write_u32_le(&mut bytes, 28, gl_internal_format);
    write_u32_le(&mut bytes, 32, gl_base_internal_format(gl_internal_format));
    write_u32_le(&mut bytes, 36, 4);
    write_u32_le(&mut bytes, 40, 4);
    write_u32_le(&mut bytes, 52, 1);
    write_u32_le(&mut bytes, 56, 1);
    write_u32_le(&mut bytes, 60, 0);
    bytes.extend_from_slice(&(level_bytes as u32).to_le_bytes());
    bytes.extend(vec![1_u8; level_bytes]);
    bytes
}

pub(super) fn ktx2_bc1_level_bytes() -> Vec<u8> {
    ktx2_compressed_level_bytes(133, 8)
}

pub(super) fn ktx2_etc2_level_bytes() -> Vec<u8> {
    ktx2_compressed_level_bytes(147, 32)
}

pub(super) fn ktx2_compressed_level_bytes(vk_format: u32, level_bytes: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; KTX2_TEST_LEVEL_DATA_OFFSET];
    bytes[0..12].copy_from_slice(b"\xABKTX 20\xBB\r\n\x1A\n");
    write_u32_le(&mut bytes, 12, vk_format);
    write_u32_le(&mut bytes, 16, 1);
    write_u32_le(&mut bytes, 20, 4);
    write_u32_le(&mut bytes, 24, 4);
    write_u32_le(&mut bytes, 36, 1);
    write_u32_le(&mut bytes, 40, 1);
    write_u32_le(&mut bytes, 44, 0);
    write_u32_le(&mut bytes, 48, KTX2_TEST_DFD_OFFSET as u32);
    write_u32_le(&mut bytes, 52, KTX2_TEST_DFD_LENGTH as u32);
    write_u64_le(&mut bytes, 80, KTX2_TEST_LEVEL_DATA_OFFSET as u64);
    write_u64_le(&mut bytes, 88, level_bytes as u64);
    write_u64_le(&mut bytes, 96, level_bytes as u64);
    write_u32_le(
        &mut bytes,
        KTX2_TEST_DFD_OFFSET,
        KTX2_TEST_DFD_LENGTH as u32,
    );
    bytes.extend(vec![1_u8; level_bytes]);
    bytes
}

pub(super) fn gl_base_internal_format(gl_internal_format: u32) -> u32 {
    match gl_internal_format {
        0x83f0 | 0x83f1 | 0x83f2 | 0x83f3 | 0x8dbb | 0x8dbd | 0x8e8c | 0x8e8e | 0x8c4c | 0x8c4e
        | 0x9270 | 0x9272 | 0x9274 | 0x9276 => 0x1907,
        _ => 0x1908,
    }
}

pub(super) fn write_u32_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_u64_le(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub(super) const DDPF_FOURCC: u32 = 0x0000_0004;
pub(super) const DDSCAPS_COMPLEX: u32 = 0x0000_0008;
pub(super) const DDSCAPS_MIPMAP: u32 = 0x0040_0000;
pub(super) const DDSCAPS_TEXTURE: u32 = 0x0000_1000;
pub(super) const DDSCAPS2_CUBEMAP: u32 = 0x0000_0200;
pub(super) const DDSCAPS2_CUBEMAP_POSITIVEX: u32 = 0x0000_0400;
pub(super) const DDSCAPS2_CUBEMAP_NEGATIVEX: u32 = 0x0000_0800;
pub(super) const DDSCAPS2_CUBEMAP_POSITIVEY: u32 = 0x0000_1000;
pub(super) const DDSCAPS2_CUBEMAP_NEGATIVEY: u32 = 0x0000_2000;
pub(super) const DDSCAPS2_CUBEMAP_POSITIVEZ: u32 = 0x0000_4000;
pub(super) const DDSCAPS2_CUBEMAP_NEGATIVEZ: u32 = 0x0000_8000;
pub(super) const DDSCAPS2_CUBEMAP_ALL_FACES: u32 = DDSCAPS2_CUBEMAP
    | DDSCAPS2_CUBEMAP_POSITIVEX
    | DDSCAPS2_CUBEMAP_NEGATIVEX
    | DDSCAPS2_CUBEMAP_POSITIVEY
    | DDSCAPS2_CUBEMAP_NEGATIVEY
    | DDSCAPS2_CUBEMAP_POSITIVEZ
    | DDSCAPS2_CUBEMAP_NEGATIVEZ;
pub(super) const DDSCAPS2_VOLUME: u32 = 0x0020_0000;
pub(super) const DDSD_CAPS: u32 = 0x0000_0001;
pub(super) const DDSD_HEIGHT: u32 = 0x0000_0002;
pub(super) const DDSD_WIDTH: u32 = 0x0000_0004;
pub(super) const DDSD_PITCH: u32 = 0x0000_0008;
pub(super) const DDSD_PIXELFORMAT: u32 = 0x0000_1000;
pub(super) const DDSD_LINEARSIZE: u32 = 0x0008_0000;
pub(super) const DDSD_MIPMAPCOUNT: u32 = 0x0002_0000;
pub(super) const DDSD_REQUIRED_FLAGS: u32 = DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT;
pub(super) const KTX2_TEST_DFD_OFFSET: usize = 104;
pub(super) const KTX2_TEST_DFD_LENGTH: usize = 32;
pub(super) const KTX2_TEST_LEVEL_DATA_OFFSET: usize = 136;

pub(super) fn write_u24_le(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
    bytes[offset + 2] = (value >> 16) as u8;
}
