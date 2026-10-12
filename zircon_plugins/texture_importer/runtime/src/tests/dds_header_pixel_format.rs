use super::common::*;
use crate::container::support::{
    DDPF_ALPHA, DDPF_ALPHAPIXELS, DDPF_BUMPDUDV, DDPF_LUMINANCE, DDPF_RGB, DDPF_YUV,
    DDSCAPS2_VOLUME, DDSD_CAPS, DDSD_DEPTH, DDSD_HEIGHT, DDSD_LINEARSIZE, DDSD_MIPMAPCOUNT,
    DDSD_PITCH, DDSD_PIXELFORMAT, DDSD_REQUIRED_FLAGS, DDSD_WIDTH,
};
use zircon_runtime::asset::ImportedAsset;

#[test]
fn dds_container_importer_rejects_short_header() {
    let mut bytes = tiny_dds_bytes();
    bytes.truncate(127);

    let error = import_container_error("short-header.dds", bytes);

    assert!(
        error.contains("dds header requires at least 128 bytes, got 127"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_missing_magic() {
    let mut bytes = tiny_dds_bytes();
    bytes[0..4].copy_from_slice(b"BAD!");

    let error = import_container_error("missing-dds-magic.dds", bytes);

    assert!(
        error.contains("dds header missing DDS magic"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_invalid_header_size() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 4, 0);

    let error = import_container_error("invalid-header-size.dds", bytes);

    assert!(
        error.contains("dds header size must be 124 bytes"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_missing_required_header_flags() {
    let cases = [
        (
            DDSD_REQUIRED_FLAGS & !DDSD_CAPS,
            "dds header flags must include DDSD_CAPS",
        ),
        (
            DDSD_REQUIRED_FLAGS & !DDSD_HEIGHT,
            "dds header flags must include DDSD_HEIGHT",
        ),
        (
            DDSD_REQUIRED_FLAGS & !DDSD_WIDTH,
            "dds header flags must include DDSD_WIDTH",
        ),
        (
            DDSD_REQUIRED_FLAGS & !DDSD_PIXELFORMAT,
            "dds header flags must include DDSD_PIXELFORMAT",
        ),
    ];

    for (flags, expected) in cases {
        let mut bytes = tiny_dds_bytes();
        write_u32(&mut bytes, 8, flags);

        let error = import_container_error("missing-header-flag.dds", bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn dds_container_importer_rejects_fourcc_without_pixel_format_flag() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 80, 0);

    let error = import_container_error("fourcc-without-flag.dds", bytes);

    assert!(
        error.contains("dds FourCC field is nonzero but DDPF_FOURCC flag is missing"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_fourcc_flag_without_fourcc_field() {
    let mut bytes = tiny_dds_bytes();
    bytes[84..88].copy_from_slice(&[0, 0, 0, 0]);

    let error = import_container_error("flag-without-fourcc.dds", bytes);

    assert!(
        error.contains("dds pixel format flags include DDPF_FOURCC but FourCC field is empty"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_missing_pixel_format_layout_flags() {
    let cases = [
        (
            0,
            "dds pixel format flags must declare DDPF_FOURCC or an uncompressed layout flag",
        ),
        (
            DDPF_ALPHAPIXELS,
            "dds pixel format flags include DDPF_ALPHAPIXELS without a color, luminance, alpha, or bump layout flag",
        ),
    ];

    for (pixel_format_flags, expected) in cases {
        let mut bytes = tiny_dds_bytes();
        write_u32(&mut bytes, 80, pixel_format_flags);
        bytes[84..88].copy_from_slice(&[0, 0, 0, 0]);

        let error = import_container_error("missing-pixel-format-layout.dds", bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}

#[test]
fn dds_container_importer_rejects_invalid_pixel_format_size() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 76, 0);

    let error = import_container_error("invalid-pixel-format-size.dds", bytes);

    assert!(
        error.contains("dds pixel format size must be 32 bytes"),
        "unexpected error: {error}"
    );
}

fn write_uncompressed_pixel_format(bytes: &mut Vec<u8>, pixel_format_flags: u32) {
    write_u32(bytes, 80, pixel_format_flags);
    bytes[84..88].copy_from_slice(&[0, 0, 0, 0]);
    write_u32(bytes, 88, 32);
    write_u32(bytes, 92, 0x00ff_0000);
    write_u32(bytes, 96, 0x0000_ff00);
    write_u32(bytes, 100, 0x0000_00ff);
    write_u32(bytes, 104, 0xff00_0000);
}

#[test]
fn dds_container_importer_rejects_missing_compressed_linear_size_flag() {
    let mut bytes = tiny_dds_bytes();
    write_u32(
        &mut bytes,
        8,
        DDSD_REQUIRED_FLAGS | DDSD_MIPMAPCOUNT | DDSD_PITCH,
    );

    let error = import_container_error("compressed-without-linear-size.dds", bytes);

    assert!(
        error.contains("dds compressed pixel data must declare DDSD_LINEARSIZE"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_compressed_pitch_and_linear_size_flags() {
    let mut bytes = tiny_dds_bytes();
    write_u32(
        &mut bytes,
        8,
        DDSD_REQUIRED_FLAGS | DDSD_MIPMAPCOUNT | DDSD_PITCH | DDSD_LINEARSIZE,
    );

    let error = import_container_error("compressed-pitch-and-linear-size.dds", bytes);

    assert!(
        error.contains(
            "dds compressed pixel data must not declare both DDSD_PITCH and DDSD_LINEARSIZE"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_zero_compressed_linear_size() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 20, 0);

    let error = import_container_error("zero-linear-size.dds", bytes);

    assert!(
        error.contains("dds compressed pixel data linear size must be nonzero"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_missing_uncompressed_pitch_flag() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_RGB);

    let error = import_container_error("uncompressed-without-pitch.dds", bytes);

    assert!(
        error.contains("dds uncompressed pixel data must declare DDSD_PITCH"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_uncompressed_pitch_and_linear_size_flags() {
    let mut bytes = tiny_dds_bytes();
    write_u32(
        &mut bytes,
        8,
        DDSD_REQUIRED_FLAGS | DDSD_PITCH | DDSD_LINEARSIZE,
    );
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_RGB);

    let error = import_container_error("uncompressed-pitch-and-linear-size.dds", bytes);

    assert!(
        error.contains(
            "dds uncompressed pixel data must not declare both DDSD_PITCH and DDSD_LINEARSIZE"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_zero_uncompressed_pitch() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 0);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_RGB);

    let error = import_container_error("zero-pitch.dds", bytes);

    assert!(
        error.contains("dds uncompressed pixel data pitch must be nonzero"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_missing_uncompressed_bit_count() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_u32(&mut bytes, 80, DDPF_RGB);
    bytes[84..88].copy_from_slice(&[0, 0, 0, 0]);

    let error = import_container_error("uncompressed-without-bit-count.dds", bytes);

    assert!(
        error.contains("dds uncompressed pixel data bit count must be nonzero"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_missing_uncompressed_channel_masks() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_u32(&mut bytes, 80, DDPF_RGB);
    write_u32(&mut bytes, 88, 32);
    bytes[84..88].copy_from_slice(&[0, 0, 0, 0]);

    let error = import_container_error("uncompressed-without-channel-masks.dds", bytes);

    assert!(
        error.contains("dds uncompressed pixel data must declare at least one channel bit mask"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_uncompressed_channel_masks_outside_bit_count() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_RGB);
    write_u32(&mut bytes, 88, 8);

    let error = import_container_error("uncompressed-mask-outside-bit-count.dds", bytes);

    assert!(
        error.contains("dds uncompressed pixel data channel masks must fit within bit count"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_uncompressed_bit_count_larger_than_mask_width() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_RGB);
    write_u32(&mut bytes, 88, 33);

    let error = import_container_error("uncompressed-bit-count-too-large.dds", bytes);

    assert!(
        error.contains("dds uncompressed pixel data bit count must be 1..=32"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_overlapping_uncompressed_channel_masks() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_RGB);
    write_u32(&mut bytes, 96, 0x00ff_0000);

    let error = import_container_error("uncompressed-overlapping-channel-masks.dds", bytes);

    assert!(
        error.contains("dds uncompressed pixel data channel masks must not overlap"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_rgb_layout_without_color_masks() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_RGB | DDPF_ALPHAPIXELS);
    write_u32(&mut bytes, 92, 0);
    write_u32(&mut bytes, 96, 0);
    write_u32(&mut bytes, 100, 0);

    let error = import_container_error("rgb-layout-without-color-masks.dds", bytes);

    assert!(
        error.contains("dds DDPF_RGB layout must declare at least one RGB channel bit mask"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_luminance_layout_without_luminance_mask() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_LUMINANCE);
    write_u32(&mut bytes, 92, 0);
    write_u32(&mut bytes, 96, 0x0000_ff00);
    write_u32(&mut bytes, 100, 0);
    write_u32(&mut bytes, 104, 0);

    let error = import_container_error("luminance-layout-without-luminance-mask.dds", bytes);

    assert!(
        error.contains("dds DDPF_LUMINANCE layout must declare a nonzero luminance bit mask"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_yuv_layout_without_yuv_masks() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_YUV | DDPF_ALPHAPIXELS);
    write_u32(&mut bytes, 92, 0);
    write_u32(&mut bytes, 96, 0);
    write_u32(&mut bytes, 100, 0);

    let error = import_container_error("yuv-layout-without-yuv-masks.dds", bytes);

    assert!(
        error.contains("dds DDPF_YUV layout must declare at least one YUV channel bit mask"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_bump_layout_without_bump_masks() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_BUMPDUDV | DDPF_ALPHAPIXELS);
    write_u32(&mut bytes, 92, 0);
    write_u32(&mut bytes, 96, 0);
    write_u32(&mut bytes, 100, 0);

    let error = import_container_error("bump-layout-without-bump-masks.dds", bytes);

    assert!(
        error.contains("dds DDPF_BUMPDUDV layout must declare at least one bump channel bit mask"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_multiple_uncompressed_layout_flags() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_RGB | DDPF_LUMINANCE);

    let error = import_container_error("multiple-uncompressed-layout-flags.dds", bytes);

    assert!(
        error.contains(
            "dds uncompressed pixel format flags must declare exactly one primary layout flag"
        ),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_alpha_layout_without_alpha_mask() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_ALPHA);
    write_u32(&mut bytes, 104, 0);

    let error = import_container_error("alpha-layout-without-alpha-mask.dds", bytes);

    assert!(
        error.contains("dds DDPF_ALPHA layout must declare a nonzero alpha bit mask"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_rejects_alpha_pixels_without_alpha_mask() {
    let mut bytes = tiny_dds_bytes();
    write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
    write_u32(&mut bytes, 20, 32);
    write_u32(&mut bytes, 28, 1);
    write_uncompressed_pixel_format(&mut bytes, DDPF_RGB | DDPF_ALPHAPIXELS);
    write_u32(&mut bytes, 104, 0);

    let error = import_container_error("alpha-pixels-without-alpha-mask.dds", bytes);

    assert!(
        error.contains("dds alpha pixel flags must declare a nonzero alpha bit mask"),
        "unexpected error: {error}"
    );
}

#[test]
fn dds_container_importer_accepts_uncompressed_pixel_format_layout_flags() {
    let cases = [
        DDPF_RGB,
        DDPF_YUV,
        DDPF_LUMINANCE,
        DDPF_ALPHA,
        DDPF_BUMPDUDV,
    ];

    for pixel_format_flags in cases {
        let mut bytes = tiny_dds_bytes();
        write_u32(&mut bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_PITCH);
        write_u32(&mut bytes, 20, 32);
        write_u32(&mut bytes, 28, 1);
        write_uncompressed_pixel_format(&mut bytes, pixel_format_flags);

        let imported = import_container_fixture("uncompressed-layout.dds", bytes);

        match imported {
            ImportedAsset::Texture(texture) => {
                assert_eq!(texture.render_image_descriptor().format, "dds/uncompressed");
            }
            other => panic!("unexpected imported asset: {other:?}"),
        }
    }
}

#[test]
fn dds_container_importer_rejects_volume_headers() {
    let cases: [(&str, fn(&mut Vec<u8>)); 3] = [
        ("volume-depth.dds", |bytes: &mut Vec<u8>| {
            write_u32(bytes, 24, 4);
        }),
        ("volume-depth-flag.dds", |bytes: &mut Vec<u8>| {
            write_u32(bytes, 8, DDSD_REQUIRED_FLAGS | DDSD_LINEARSIZE | DDSD_DEPTH);
        }),
        ("volume-caps.dds", |bytes: &mut Vec<u8>| {
            write_u32(bytes, 112, DDSCAPS2_VOLUME);
        }),
    ];

    for (path, mutate) in cases {
        let mut bytes = tiny_dds_bytes();
        mutate(&mut bytes);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains("dds volume textures are not supported by container importer yet"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn dds_container_importer_rejects_malformed_fourcc_tokens() {
    let cases: [(&str, [u8; 4], &str); 2] = [
        (
            "non-ascii-fourcc.dds",
            [b'D', b'X', 0xff, b'1'],
            "dds fourcc must contain printable ASCII bytes",
        ),
        (
            "embedded-nul-fourcc.dds",
            [b'D', b'X', 0, b'1'],
            "dds fourcc must not contain embedded NUL bytes",
        ),
    ];

    for (path, fourcc, expected) in cases {
        let mut bytes = tiny_dds_bytes();
        bytes[84..88].copy_from_slice(&fourcc);

        let error = import_container_error(path, bytes);

        assert!(
            error.contains(expected),
            "expected `{expected}` in `{error}`"
        );
    }
}
