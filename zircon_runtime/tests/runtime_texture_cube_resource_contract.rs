use zircon_runtime::asset::assets::TextureDescriptorError;
use zircon_runtime::asset::{
    texture_asset_from_array_layers, texture_asset_from_cubemap_faces, AssetReference, AssetUri,
    CubemapAsset, CubemapAssetError, CubemapSourceLayout, Texture2DArrayAsset,
    Texture2DArrayAssetError, TextureArrayLayerSource, TextureAsset, TextureAssetDescriptor,
    TextureUploadReadiness, TextureUploadSupport,
};
use zircon_runtime::core::framework::render::RenderImageDimension;
use zircon_runtime::core::resource::ResourceLocator;

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

#[test]
fn runtime_cubemap_asset_contract_preserves_wgpu_face_order() {
    let cubemap = cubemap_asset(CubemapSourceLayout::SixFiles, 6);
    let faces = (0_u8..6)
        .map(|face| face_texture(face, 2, "rgba8unorm_srgb"))
        .collect();

    let texture = texture_asset_from_cubemap_faces(cubemap, faces)
        .expect("six matching faces should build a cubemap texture");

    assert_eq!(texture.width, 2);
    assert_eq!(texture.height, 2);
    assert_eq!(
        texture.render_image_descriptor().dimension,
        RenderImageDimension::Cube
    );
    assert_eq!(texture.render_image_descriptor().array_layer_count, 6);
    for face in 0_usize..6 {
        let face_offset = face * 2 * 2 * 4;
        assert_eq!(texture.rgba[face_offset], face as u8);
    }
}

#[test]
fn runtime_cubemap_asset_contract_rejects_face_dimension_mismatch() {
    let cubemap = cubemap_asset(CubemapSourceLayout::SixFiles, 6);
    let mut faces = (0_u8..6)
        .map(|face| face_texture(face, 2, "rgba8unorm_srgb"))
        .collect::<Vec<_>>();
    faces[4] = face_texture(4, 4, "rgba8unorm_srgb");

    let error = texture_asset_from_cubemap_faces(cubemap, faces)
        .expect_err("different face dimensions must be rejected");

    assert_eq!(
        error,
        CubemapAssetError::FaceDimensionMismatch {
            face: 4,
            expected: 2,
            width: 4,
            height: 4,
        }
    );
}

#[test]
fn runtime_cubemap_asset_contract_rejects_face_format_mismatch() {
    let cubemap = cubemap_asset(CubemapSourceLayout::SixFiles, 6);
    let mut faces = (0_u8..6)
        .map(|face| face_texture(face, 2, "rgba8unorm_srgb"))
        .collect::<Vec<_>>();
    faces[2] = face_texture(2, 2, "rgba8unorm");

    let error = texture_asset_from_cubemap_faces(cubemap, faces)
        .expect_err("different face formats must be rejected");

    assert_eq!(
        error,
        CubemapAssetError::FaceFormatMismatch {
            face: 2,
            expected: "rgba8unorm_srgb".to_string(),
            actual: "rgba8unorm".to_string(),
        }
    );
}

#[test]
fn runtime_texture_array_asset_contract_preserves_layer_order() {
    let references = (0..3)
        .map(face_reference)
        .map(TextureArrayLayerSource::Reference)
        .collect::<Vec<_>>();
    let asset = Texture2DArrayAsset {
        uri: AssetUri::parse("res://textures/test_array.zarray").expect("valid array uri"),
        descriptor: TextureAssetDescriptor::rgba8_srgb(),
        layers: references,
    };
    let layers = (0_u8..3)
        .map(|layer| face_texture(layer, 2, "rgba8unorm_srgb"))
        .collect();

    let texture = texture_asset_from_array_layers(asset, layers)
        .expect("matching layers should build a texture array");

    let descriptor = texture.render_image_descriptor();
    assert_eq!(descriptor.dimension, RenderImageDimension::D2);
    assert_eq!(descriptor.array_layer_count, 3);
    for layer in 0_usize..3 {
        let layer_offset = layer * 2 * 2 * 4;
        assert_eq!(texture.rgba[layer_offset], layer as u8);
    }
}

#[test]
fn runtime_texture_array_asset_contract_rejects_layer_dimension_mismatch() {
    let asset = Texture2DArrayAsset {
        uri: AssetUri::parse("res://textures/test_array.zarray").expect("valid array uri"),
        descriptor: TextureAssetDescriptor::rgba8_srgb(),
        layers: vec![
            TextureArrayLayerSource::Reference(face_reference(0)),
            TextureArrayLayerSource::Reference(face_reference(1)),
        ],
    };
    let layers = vec![
        face_texture(0, 2, "rgba8unorm_srgb"),
        face_texture(1, 4, "rgba8unorm_srgb"),
    ];

    let error = texture_asset_from_array_layers(asset, layers)
        .expect_err("different layer dimensions must be rejected");

    assert_eq!(
        error,
        Texture2DArrayAssetError::LayerDimensionMismatch {
            layer: 1,
            expected_width: 2,
            expected_height: 2,
            width: 4,
            height: 4,
        }
    );
}

fn cubemap_asset(layout: CubemapSourceLayout, source_count: usize) -> CubemapAsset {
    CubemapAsset {
        uri: AssetUri::parse("res://textures/test_cube.zcube").expect("valid cubemap uri"),
        descriptor: TextureAssetDescriptor::rgba8_srgb(),
        source_layout: layout,
        sources: (0..source_count).map(face_reference).collect(),
    }
}

fn face_reference(index: usize) -> AssetReference {
    AssetReference::from_locator(
        ResourceLocator::parse(&format!("res://textures/face_{index}.png"))
            .expect("valid face locator"),
    )
}

fn face_texture(face: u8, size: u32, format: &str) -> TextureAsset {
    let mut descriptor = TextureAssetDescriptor::rgba8_srgb();
    descriptor.format = format.to_string();
    TextureAsset::new_rgba8(
        AssetUri::parse(&format!("res://textures/face_{face}.png")).expect("valid texture uri"),
        size,
        size,
        vec![face; size as usize * size as usize * 4],
    )
    .with_descriptor(descriptor)
}
