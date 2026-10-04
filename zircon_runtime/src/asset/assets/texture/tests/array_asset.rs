use super::*;

fn uri(path: &str) -> AssetUri {
    AssetUri::parse(path).expect("texture test URI must be valid")
}

#[test]
fn runtime92_owned_descriptors_recovery_batch_array_preserves_payload_and_extent() {
    let asset = Texture2DArrayAsset {
        uri: uri("res://textures/runtime92-array.ztexture"),
        descriptor: TextureAssetDescriptor::default(),
        layers: vec![
            TextureArrayLayerSource::Reference(AssetReference::from_locator(uri(
                "res://textures/runtime92-array-0.png",
            ))),
            TextureArrayLayerSource::Reference(AssetReference::from_locator(uri(
                "res://textures/runtime92-array-1.png",
            ))),
        ],
    };
    let layers = vec![
        TextureAsset::new_rgba8(
            uri("res://textures/runtime92-array-0.png"),
            2,
            1,
            vec![1; 8],
        ),
        TextureAsset::new_rgba8(
            uri("res://textures/runtime92-array-1.png"),
            2,
            1,
            vec![2; 8],
        ),
    ];

    let output = texture_asset_from_array_layers(asset, layers).unwrap();

    assert_eq!(output.rgba, [vec![1; 8], vec![2; 8]].concat());
    let descriptor = output.descriptor.unwrap();
    assert_eq!(descriptor.dimension, RenderImageDimension::D2);
    assert_eq!(descriptor.depth_or_array_layers, 2);
    assert_eq!(descriptor.array_layer_count(), 2);
}
