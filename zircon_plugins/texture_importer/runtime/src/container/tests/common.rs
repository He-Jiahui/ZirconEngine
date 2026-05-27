use super::super::*;
use zircon_runtime::asset::{AssetImportContext, AssetUri};
pub(super) use zircon_runtime::asset::{ImportedAsset, TexturePayload};

pub(super) fn import_container_fixture(path: &str, bytes: Vec<u8>) -> ImportedAsset {
    import_container_fixture_with_settings(path, bytes, "")
}

pub(super) fn import_container_fixture_with_settings(
    path: &str,
    bytes: Vec<u8>,
    settings: &str,
) -> ImportedAsset {
    let report = crate::plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new(path))
        .unwrap();
    let uri = format!("res://textures/{path}");
    let settings = settings.parse().expect("valid texture import settings");
    let context =
        AssetImportContext::new(path.into(), AssetUri::parse(&uri).unwrap(), bytes, settings);
    importer
        .import(&context)
        .unwrap()
        .root_entry()
        .expect("root texture asset entry")
        .asset
        .clone()
}

pub(super) fn import_container_error(path: &str, bytes: Vec<u8>) -> String {
    import_container_error_with_settings(path, bytes, "")
}

pub(super) fn import_container_error_with_settings(
    path: &str,
    bytes: Vec<u8>,
    settings: &str,
) -> String {
    let report = crate::plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new(path))
        .unwrap();
    let uri = format!("res://textures/{path}");
    let settings = settings.parse().expect("valid texture import settings");
    let context =
        AssetImportContext::new(path.into(), AssetUri::parse(&uri).unwrap(), bytes, settings);
    importer.import(&context).unwrap_err().to_string()
}

pub(super) fn tiny_dds_bytes() -> Vec<u8> {
    let mut bytes = vec![0; 128];
    bytes[0..4].copy_from_slice(b"DDS ");
    write_u32(&mut bytes, 4, 124);
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_MIPMAPCOUNT);
    write_u32(&mut bytes, 12, 4);
    write_u32(&mut bytes, 16, 8);
    write_u32(&mut bytes, 28, 3);
    write_u32(&mut bytes, 76, 32);
    write_u32(&mut bytes, 80, DDPF_FOURCC);
    bytes[84..88].copy_from_slice(b"DXT1");
    write_u32(&mut bytes, 108, DDSCAPS_TEXTURE);
    bytes.resize(128 + dds_compressed_payload_len(8, 4, 8, 1, 3), 0);
    bytes
}

pub(super) fn tiny_dds_dx10_cubemap_array_bytes() -> Vec<u8> {
    dds_dx10_cubemap_array_bytes(2)
}

pub(super) fn dds_dx10_cubemap_array_bytes(array_size: u32) -> Vec<u8> {
    let mut bytes = tiny_dds_bytes();
    bytes.resize(148, 0);
    write_u32(&mut bytes, 12, 16);
    write_u32(&mut bytes, 16, 32);
    write_u32(&mut bytes, 28, 5);
    bytes[84..88].copy_from_slice(b"DX10");
    write_u32(&mut bytes, 112, DDSCAPS2_CUBEMAP_ALL_FACES);
    write_u32(&mut bytes, 128, 98);
    write_u32(&mut bytes, 132, DDS_DIMENSION_TEXTURE2D);
    write_u32(&mut bytes, 140, array_size);
    let layer_count = usize::try_from(array_size).expect("array size fits usize") * 6;
    bytes.resize(
        148 + dds_compressed_payload_len(32, 16, 16, layer_count, 5),
        0,
    );
    bytes
}

fn dds_compressed_payload_len(
    width: usize,
    height: usize,
    bytes_per_block: usize,
    layers: usize,
    mip_count: usize,
) -> usize {
    let mut total_len = 0;
    let mut mip_width = width;
    let mut mip_height = height;
    for _ in 0..mip_count {
        total_len += mip_width.div_ceil(4) * mip_height.div_ceil(4) * bytes_per_block * layers;
        mip_width = (mip_width / 2).max(1);
        mip_height = (mip_height / 2).max(1);
    }
    total_len
}

pub(super) fn tiny_ktx1_1d_bytes() -> Vec<u8> {
    ktx1_layer_face_bytes(0, 1)
}

pub(super) fn ktx1_layer_face_bytes(array_elements: u32, faces: u32) -> Vec<u8> {
    let mut bytes = vec![0; 68];
    bytes[0..12].copy_from_slice(KTX1_IDENTIFIER);
    write_u32(&mut bytes, 12, KTX_LITTLE_ENDIAN);
    write_u32(&mut bytes, 28, 0x8058);
    write_u32(&mut bytes, 36, 32);
    write_u32(&mut bytes, 40, 0);
    write_u32(&mut bytes, 44, 0);
    write_u32(&mut bytes, 48, array_elements);
    write_u32(&mut bytes, 52, faces);
    write_u32(&mut bytes, 56, 1);
    write_u32(&mut bytes, 64, 0);
    bytes
}

pub(super) fn ktx1_with_key_value_metadata_bytes(metadata: &[u8]) -> Vec<u8> {
    let mut bytes = ktx1_layer_face_bytes(0, 1);
    bytes.truncate(64);
    write_u32(
        &mut bytes,
        60,
        u32::try_from(metadata.len()).expect("metadata length fits u32"),
    );
    bytes.extend_from_slice(metadata);
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes
}

pub(super) fn ktx1_key_value_metadata_record(key_and_value: &[u8]) -> Vec<u8> {
    let key_and_value_len =
        u32::try_from(key_and_value.len()).expect("metadata record length fits u32");
    let mut metadata = Vec::new();
    metadata.extend_from_slice(&key_and_value_len.to_le_bytes());
    metadata.extend_from_slice(key_and_value);
    metadata.resize(metadata.len() + ktx1_padding(key_and_value.len()), 0);
    metadata
}

pub(super) fn ktx1_two_mip_prefix_bytes(first_level_image_size: u32) -> Vec<u8> {
    let mut bytes = ktx1_layer_face_bytes(0, 1);
    write_u32(&mut bytes, 56, 2);
    write_u32(&mut bytes, 64, first_level_image_size);
    let first_level_len =
        usize::try_from(first_level_image_size).expect("u32 image size fits usize");
    bytes.resize(
        bytes.len() + first_level_len + ktx1_padding(first_level_len),
        0,
    );
    bytes
}

fn ktx1_padding(byte_len: usize) -> usize {
    (4 - (byte_len % 4)) % 4
}

pub(super) fn tiny_ktx2_bytes() -> Vec<u8> {
    ktx2_layer_face_bytes(2, 6)
}

pub(super) fn ktx2_layer_face_bytes(layer_count: u32, face_count: u32) -> Vec<u8> {
    let level_count = 4_u32;
    let mut bytes = vec![
        0;
        KTX2_HEADER_SIZE
            + KTX2_LEVEL_INDEX_ENTRY_SIZE * usize::try_from(level_count).unwrap()
    ];
    bytes[0..12].copy_from_slice(KTX2_IDENTIFIER);
    write_u32(&mut bytes, 12, 37);
    write_u32(&mut bytes, 16, 1);
    write_u32(&mut bytes, 20, 16);
    write_u32(&mut bytes, 24, 8);
    write_u32(&mut bytes, 32, layer_count);
    write_u32(&mut bytes, 36, face_count);
    write_u32(&mut bytes, 40, level_count);
    write_u32(&mut bytes, 44, 1);
    write_u64(&mut bytes, KTX2_HEADER_SIZE, 176);
    write_u64(&mut bytes, KTX2_HEADER_SIZE + 8, 0);
    bytes
}

pub(super) fn tiny_ktx2_3d_bytes() -> Vec<u8> {
    let mut bytes = tiny_ktx2_bytes();
    write_u32(&mut bytes, 28, 5);
    write_u32(&mut bytes, 32, 0);
    write_u32(&mut bytes, 36, 1);
    write_u32(&mut bytes, 40, 1);
    bytes
}

pub(super) fn tiny_astc_bytes() -> Vec<u8> {
    let mut bytes = vec![0; 16];
    bytes[0..4].copy_from_slice(ASTC_MAGIC);
    bytes[4] = 6;
    bytes[5] = 6;
    bytes[6] = 1;
    write_u24(&mut bytes, 7, 32);
    write_u24(&mut bytes, 10, 16);
    write_u24(&mut bytes, 13, 1);
    bytes.resize(astc_total_len(6, 6, 1, 32, 16, 1), 0);
    bytes
}

pub(super) fn tiny_astc_3d_bytes() -> Vec<u8> {
    let mut bytes = tiny_astc_bytes();
    bytes[6] = 4;
    write_u24(&mut bytes, 13, 8);
    bytes.resize(astc_total_len(6, 6, 4, 32, 16, 8), 0);
    bytes
}

fn astc_total_len(
    block_x: usize,
    block_y: usize,
    block_z: usize,
    width: usize,
    height: usize,
    depth: usize,
) -> usize {
    16 + width.div_ceil(block_x) * height.div_ceil(block_y) * depth.div_ceil(block_z) * 16
}

pub(super) fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_u24(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset] = (value & 0xff) as u8;
    bytes[offset + 1] = ((value >> 8) & 0xff) as u8;
    bytes[offset + 2] = ((value >> 16) & 0xff) as u8;
}
