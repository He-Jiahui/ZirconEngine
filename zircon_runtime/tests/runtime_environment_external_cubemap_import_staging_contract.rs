use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::{
    decode_external_source_cubemap, stage_external_source_cubemap_texture, AssetUri,
    EnvironmentIblSourceStagingStatus, TextureAsset,
};
use zircon_runtime::core::framework::render::{
    encode_rgba16f_texels, source_cubemap_face_mip_offset, source_cubemap_mip_count,
    source_cubemap_mip_size, CubemapFace, SOURCE_CUBEMAP_FACE_COUNT,
};

#[test]
fn cmft_rgba16f_dds_face_major_source_mips_stage_and_rebuild_pmrem() {
    let face_size = 4;
    let mip_count = source_cubemap_mip_count(face_size);
    let texture = source_texture(
        "res://textures/cmft-source.dds",
        "dds/D3DFMT-113",
        cmft_dds_rgba16f(face_size, mip_count),
        face_size,
        mip_count,
    );

    let decoded = decode_external_source_cubemap(&texture)
        .expect("decode cmft DDS source cubemap")
        .expect("DDS should be classified as source cubemap");

    assert_source_face_mip_identity(&decoded, 0, 0);
    assert_source_face_mip_identity(&decoded, 5, 2);
    assert_ne!(
        decoded.texels(),
        decoded.source_texels(),
        "external source mips must not be accepted as Zircon PMREM"
    );

    let root = unique_temp_root("external_cmft_dds_staging");
    let staged = stage_external_source_cubemap_texture(&texture, &root)
        .expect("stage cmft DDS source cubemap");
    assert_eq!(staged.status(), EnvironmentIblSourceStagingStatus::Written);
    assert!(staged.source_zcube_path().expect("zcube path").is_file());
    assert!(staged.asset_derived_path().expect("zribl path").is_file());

    let reused = stage_external_source_cubemap_texture(&texture, &root)
        .expect("reuse staged cmft DDS source cubemap");
    assert_eq!(reused.status(), EnvironmentIblSourceStagingStatus::Reused);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn cmft_rgba16f_ktx_mip_major_payload_reorders_to_face_major_source_chain() {
    let face_size = 4;
    let mip_count = source_cubemap_mip_count(face_size);
    let texture = source_texture(
        "res://textures/cmft-source.ktx",
        "ktx/gl-internal-0x0000881a",
        cmft_ktx1_rgba16f(face_size, mip_count),
        face_size,
        mip_count,
    );

    let decoded = decode_external_source_cubemap(&texture)
        .expect("decode cmft KTX source cubemap")
        .expect("KTX should be classified as source cubemap");

    for face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip in 0..mip_count {
            assert_source_face_mip_identity(&decoded, face, mip);
        }
    }
}

#[test]
fn rgba16f_ktx2_mip_major_payload_reorders_to_face_major_source_chain() {
    let face_size = 4;
    let mip_count = source_cubemap_mip_count(face_size);
    let texture = source_texture(
        "res://textures/source.ktx2",
        "ktx2/vk-97/supercompression-0",
        ktx2_rgba16f(face_size, mip_count),
        face_size,
        mip_count,
    );

    let decoded = decode_external_source_cubemap(&texture)
        .expect("decode KTX2 source cubemap")
        .expect("KTX2 should be classified as source cubemap");

    for face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip in 0..mip_count {
            assert_source_face_mip_identity(&decoded, face, mip);
        }
    }
}

fn source_texture(
    uri: &str,
    format: &str,
    bytes: Vec<u8>,
    face_size: u32,
    mip_count: u32,
) -> TextureAsset {
    TextureAsset::new_container(
        AssetUri::parse(uri).expect("valid source URI"),
        face_size,
        face_size,
        format,
        bytes,
        mip_count,
        SOURCE_CUBEMAP_FACE_COUNT as u32,
    )
}

fn cmft_dds_rgba16f(face_size: u32, mip_count: u32) -> Vec<u8> {
    let mut bytes = vec![0_u8; 128];
    bytes[0..4].copy_from_slice(b"DDS ");
    write_u32(&mut bytes, 4, 124);
    write_u32(&mut bytes, 8, DDSD_REQUIRED | DDSD_PITCH | DDSD_MIPMAPCOUNT);
    write_u32(&mut bytes, 12, face_size);
    write_u32(&mut bytes, 16, face_size);
    write_u32(&mut bytes, 20, face_size * 8);
    write_u32(&mut bytes, 28, mip_count);
    write_u32(&mut bytes, 76, 32);
    write_u32(&mut bytes, 80, DDPF_FOURCC);
    write_u32(&mut bytes, 84, D3DFMT_A16B16G16R16F);
    write_u32(
        &mut bytes,
        108,
        DDSCAPS_TEXTURE | DDSCAPS_COMPLEX | DDSCAPS_MIPMAP,
    );
    write_u32(&mut bytes, 112, DDSCAPS2_CUBEMAP_ALL_FACES);

    for face in 0..SOURCE_CUBEMAP_FACE_COUNT {
        for mip in 0..mip_count {
            bytes.extend_from_slice(&encoded_face_mip(
                face,
                mip,
                source_cubemap_mip_size(face_size, mip),
            ));
        }
    }
    bytes
}

fn cmft_ktx1_rgba16f(face_size: u32, mip_count: u32) -> Vec<u8> {
    let mut bytes = vec![0_u8; 64];
    bytes[0..12].copy_from_slice(b"\xABKTX 11\xBB\r\n\x1A\n");
    write_u32(&mut bytes, 12, 0x0403_0201);
    write_u32(&mut bytes, 16, 0x140b);
    write_u32(&mut bytes, 20, 2);
    write_u32(&mut bytes, 24, 0x1908);
    write_u32(&mut bytes, 28, 0x881a);
    write_u32(&mut bytes, 32, 0x1908);
    write_u32(&mut bytes, 36, face_size);
    write_u32(&mut bytes, 40, face_size);
    write_u32(&mut bytes, 52, SOURCE_CUBEMAP_FACE_COUNT as u32);
    write_u32(&mut bytes, 56, mip_count);

    for mip in 0..mip_count {
        let mip_size = source_cubemap_mip_size(face_size, mip);
        let face_bytes = mip_size as usize * mip_size as usize * 8;
        bytes.extend_from_slice(&(face_bytes as u32).to_le_bytes());
        for face in 0..SOURCE_CUBEMAP_FACE_COUNT {
            bytes.extend_from_slice(&encoded_face_mip(face, mip, mip_size));
        }
    }
    bytes
}

fn ktx2_rgba16f(face_size: u32, mip_count: u32) -> Vec<u8> {
    let level_index_end = 80 + mip_count as usize * 24;
    let dfd_offset = align4(level_index_end);
    let dfd_length = 32;
    let mut data_offset = align4(dfd_offset + dfd_length);
    let mut bytes = vec![0_u8; data_offset];
    bytes[0..12].copy_from_slice(b"\xABKTX 20\xBB\r\n\x1A\n");
    write_u32(&mut bytes, 12, 97);
    write_u32(&mut bytes, 16, 8);
    write_u32(&mut bytes, 20, face_size);
    write_u32(&mut bytes, 24, face_size);
    write_u32(&mut bytes, 36, SOURCE_CUBEMAP_FACE_COUNT as u32);
    write_u32(&mut bytes, 40, mip_count);
    write_u32(&mut bytes, 48, dfd_offset as u32);
    write_u32(&mut bytes, 52, dfd_length as u32);
    write_u32(&mut bytes, dfd_offset, dfd_length as u32);

    for mip in 0..mip_count {
        let mip_size = source_cubemap_mip_size(face_size, mip);
        let face_bytes = mip_size as usize * mip_size as usize * 8;
        let level_bytes = face_bytes * SOURCE_CUBEMAP_FACE_COUNT;
        let index_offset = 80 + mip as usize * 24;
        write_u64(&mut bytes, index_offset, data_offset as u64);
        write_u64(&mut bytes, index_offset + 8, level_bytes as u64);
        write_u64(&mut bytes, index_offset + 16, level_bytes as u64);
        for face in 0..SOURCE_CUBEMAP_FACE_COUNT {
            bytes.extend_from_slice(&encoded_face_mip(face, mip, mip_size));
        }
        data_offset = align4(data_offset + level_bytes);
        bytes.resize(data_offset, 0);
    }
    bytes
}

fn encoded_face_mip(face: usize, mip: u32, mip_size: u32) -> Vec<u8> {
    let texel = source_identity(face, mip);
    encode_rgba16f_texels(&vec![texel; mip_size as usize * mip_size as usize])
}

fn source_identity(face: usize, mip: u32) -> [f32; 4] {
    [face as f32 + 1.0, mip as f32 * 0.125 + 0.25, 0.5, 1.0]
}

fn assert_source_face_mip_identity(
    decoded: &zircon_runtime::core::framework::render::SourceCubemapMipChain,
    face: usize,
    mip: u32,
) {
    let face = CubemapFace::ALL[face];
    let offset =
        source_cubemap_face_mip_offset(decoded.face_size(), decoded.mip_count(), face, mip);
    let actual = decoded.source_texels()[offset];
    let expected = source_identity(face.index(), mip);
    for channel in 0..4 {
        assert!(
            (actual[channel] - expected[channel]).abs() <= 0.001,
            "face={:?} mip={mip} channel={channel}: actual={} expected={}",
            face,
            actual[channel],
            expected[channel]
        );
    }
}

fn unique_temp_root(name: &str) -> std::path::PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "zircon_{name}_{}_{}",
        std::process::id(),
        timestamp
    ))
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn align4(value: usize) -> usize {
    (value + 3) & !3
}

const D3DFMT_A16B16G16R16F: u32 = 113;
const DDPF_FOURCC: u32 = 0x0000_0004;
const DDSCAPS_COMPLEX: u32 = 0x0000_0008;
const DDSCAPS_TEXTURE: u32 = 0x0000_1000;
const DDSCAPS_MIPMAP: u32 = 0x0040_0000;
const DDSD_CAPS: u32 = 0x0000_0001;
const DDSD_HEIGHT: u32 = 0x0000_0002;
const DDSD_WIDTH: u32 = 0x0000_0004;
const DDSD_PITCH: u32 = 0x0000_0008;
const DDSD_PIXELFORMAT: u32 = 0x0000_1000;
const DDSD_MIPMAPCOUNT: u32 = 0x0002_0000;
const DDSD_REQUIRED: u32 = DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT;
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
