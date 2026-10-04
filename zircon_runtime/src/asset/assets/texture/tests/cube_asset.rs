use super::*;

fn uri(path: &str) -> AssetUri {
    AssetUri::parse(path).expect("texture test URI must be valid")
}

#[test]
fn runtime92_owned_descriptors_recovery_batch_cubemap_preserves_face_order_and_extent() {
    let sources = (0..CUBEMAP_FACE_COUNT)
        .map(|face| {
            AssetReference::from_locator(uri(&format!(
                "res://textures/runtime92-cubemap-{face}.png"
            )))
        })
        .collect::<Vec<_>>();
    let asset = CubemapAsset {
        uri: uri("res://textures/runtime92-cubemap.ztexture"),
        descriptor: TextureAssetDescriptor::default(),
        source_layout: CubemapSourceLayout::SixFiles,
        sources,
    };
    let faces = (0..CUBEMAP_FACE_COUNT)
        .map(|face| {
            let value = u8::try_from(face).unwrap();
            TextureAsset::new_rgba8(
                uri(&format!("res://textures/runtime92-cubemap-{face}.png")),
                1,
                1,
                vec![value, value, value, 255],
            )
        })
        .collect::<Vec<_>>();

    let output = texture_asset_from_cubemap_faces(asset, faces).unwrap();

    let expected = (0..CUBEMAP_FACE_COUNT)
        .flat_map(|face| {
            let value = u8::try_from(face).unwrap();
            [value, value, value, 255]
        })
        .collect::<Vec<_>>();
    assert_eq!(output.rgba, expected);
    let descriptor = output.descriptor.unwrap();
    assert_eq!(descriptor.dimension, RenderImageDimension::Cube);
    assert_eq!(descriptor.depth_or_array_layers, CUBEMAP_FACE_COUNT as u32);
    assert_eq!(descriptor.array_layer_count(), CUBEMAP_FACE_COUNT as u32);
}
