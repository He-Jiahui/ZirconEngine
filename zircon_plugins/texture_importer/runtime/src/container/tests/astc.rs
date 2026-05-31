use super::common::*;
use zircon_runtime::asset::{ImportedAsset, TexturePayload};
use zircon_runtime::core::framework::render::RenderImageDimension;

#[test]
fn astc_container_importer_reads_block_and_size() {
    let imported = import_container_fixture("tile.astc", tiny_astc_bytes());

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 32);
            assert_eq!(texture.height, 16);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.format, "astc/6x6x1");
            assert_eq!(descriptor.dimension, RenderImageDimension::D2);
            assert_eq!(descriptor.depth_or_array_layers, 1);
            match texture.payload {
                TexturePayload::Container {
                    format,
                    mip_count,
                    array_layers,
                    ..
                } => {
                    assert_eq!(format, "astc/6x6x1");
                    assert_eq!(mip_count, 1);
                    assert_eq!(array_layers, 1);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn astc_container_importer_reads_3d_block_and_depth() {
    let imported = import_container_fixture("volume.astc", tiny_astc_3d_bytes());

    match imported {
        ImportedAsset::Texture(texture) => {
            assert_eq!(texture.width, 32);
            assert_eq!(texture.height, 16);
            let descriptor = texture.render_image_descriptor();
            assert_eq!(descriptor.format, "astc/6x6x6");
            assert_eq!(descriptor.dimension, RenderImageDimension::D3);
            assert_eq!(descriptor.depth_or_array_layers, 8);
            assert_eq!(descriptor.array_layer_count, 1);
            match texture.payload {
                TexturePayload::Container {
                    format,
                    mip_count,
                    array_layers,
                    ..
                } => {
                    assert_eq!(format, "astc/6x6x6");
                    assert_eq!(mip_count, 1);
                    assert_eq!(array_layers, 1);
                }
                other => panic!("unexpected texture payload: {other:?}"),
            }
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn astc_container_importer_rejects_truncated_block_payload() {
    let mut bytes = tiny_astc_bytes();
    bytes.truncate(16);

    let error = import_container_error("truncated.astc", bytes);

    assert!(
        error.contains("astc block payload requires at least 304 bytes, got 16"),
        "unexpected error: {error}"
    );
}

#[test]
fn astc_container_importer_rejects_truncated_3d_block_payload() {
    let mut bytes = tiny_astc_3d_bytes();
    bytes.truncate(16);

    let error = import_container_error("truncated-volume.astc", bytes);

    assert!(
        error.contains("astc block payload requires at least 592 bytes, got 16"),
        "unexpected error: {error}"
    );
}

#[test]
fn astc_container_importer_rejects_unsupported_block_footprints() {
    let cases = [(7, 7, 1), (4, 4, 2), (7, 7, 7)];

    for (block_x, block_y, block_z) in cases {
        let mut bytes = tiny_astc_bytes();
        bytes[4] = block_x;
        bytes[5] = block_y;
        bytes[6] = block_z;

        let error = import_container_error("unsupported-footprint.astc", bytes);

        assert!(
            error.contains(&format!(
                "astc block footprint {block_x}x{block_y}x{block_z} is not supported"
            )),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn astc_container_importer_rejects_volume_depth_with_2d_block_footprint() {
    let mut bytes = tiny_astc_bytes();
    write_u24(&mut bytes, 13, 2);

    let error = import_container_error("2d-footprint-volume.astc", bytes);

    assert!(
        error.contains("astc 2d block footprint requires depth 1, got 2"),
        "unexpected error: {error}"
    );
}

#[test]
fn astc_container_importer_rejects_payload_range_overflow() {
    let mut bytes = tiny_astc_bytes();
    bytes.truncate(16);
    bytes[4] = 4;
    bytes[5] = 4;
    bytes[6] = 4;
    write_u24(&mut bytes, 7, 0x00ff_ffff);
    write_u24(&mut bytes, 10, 0x00ff_ffff);
    write_u24(&mut bytes, 13, 0x00ff_ffff);

    let error = import_container_error("overflow.astc", bytes);

    assert!(
        error.contains("astc block payload range overflows usize"),
        "unexpected error: {error}"
    );
}
