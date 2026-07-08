use zircon_runtime::asset::assets::TextureDescriptorError;
use zircon_runtime::asset::{
    AssetUri, TextureAsset, TextureAssetDescriptor, TextureUploadReadiness, TextureUploadSupport,
};
use zircon_runtime::core::framework::render::RenderImageDimension;

#[test]
fn runtime_texture_cube_descriptor_contract_defaults_to_six_faces() {
    let settings = r#"dimension = "cube""#.parse::<toml::Table>().expect("valid toml");

    let descriptor = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect("cube descriptor should normalize");

    assert_eq!(descriptor.dimension, RenderImageDimension::Cube);
    assert_eq!(descriptor.array_layer_count, 6);
    assert_eq!(descriptor.depth_or_array_layers, 6);
}

#[test]
fn runtime_texture_cube_descriptor_contract_rejects_non_face_multiple_layers() {
    let settings = r#"
dimension = "cubemap"
array_layers = 5
"#
    .parse::<toml::Table>()
    .expect("valid toml");

    let error = TextureAssetDescriptor::default()
        .apply_import_settings(&settings)
        .expect_err("five layers cannot form complete cubemap faces");

    assert_eq!(error, TextureDescriptorError::CubeLayerCount { layers: 5 });
}

#[test]
fn runtime_texture_cube_upload_contract_accepts_complete_rgba8_face_payloads() {
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.dimension = RenderImageDimension::Cube;
    descriptor.depth_or_array_layers = 6;
    descriptor.array_layer_count = 6;

    let face_size = 2;
    let layer_count = 6;
    let bytes = vec![255; face_size * face_size * layer_count * 4];
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/test_cube.ztex").expect("valid texture uri"),
        face_size as u32,
        face_size as u32,
        bytes,
    )
    .with_descriptor(descriptor);

    let readiness = texture.upload_readiness(TextureUploadSupport::default());

    assert!(
        matches!(readiness, TextureUploadReadiness::Ready { .. }),
        "cube texture should be upload-ready, got {readiness:?}"
    );
}
