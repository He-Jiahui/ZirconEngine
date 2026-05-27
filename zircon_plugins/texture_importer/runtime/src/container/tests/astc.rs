use super::super::*;
use super::common::*;

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
            assert_eq!(descriptor.format, "astc/6x6x4");
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
                    assert_eq!(format, "astc/6x6x4");
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
