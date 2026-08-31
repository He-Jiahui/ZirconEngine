use gltf::image::{Data as GltfImageData, Format as GltfImageFormat};
use zircon_runtime::asset::assets::TextureAssetDescriptor;
use zircon_runtime::asset::{
    AssetImportOutcome, AssetUri, ImportedAsset, ModelAsset, TextureAsset,
};
use zircon_runtime::core::framework::render::TextureUsageHint;

use super::{add_gltf_material_subassets, add_gltf_texture_subassets, gltf_label_uri};

#[test]
fn normal_and_data_slots_emit_distinct_role_variants() {
    let gltf = gltf::Gltf::from_slice(
        br#"{
            "asset": { "version": "2.0" },
            "images": [{ "uri": "shared-linear.png" }],
            "textures": [{ "source": 0 }],
            "materials": [{
                "normalTexture": { "index": 0 },
                "pbrMetallicRoughness": {
                    "metallicRoughnessTexture": { "index": 0 }
                }
            }]
        }"#,
    )
    .expect("normal/data texture fixture must parse");
    let root_uri = AssetUri::parse("res://models/plugin_shared_normal_data.glb")
        .expect("fixture root URI must be valid");
    let outcome = AssetImportOutcome::new(
        root_uri.clone(),
        ImportedAsset::Model(ModelAsset {
            uri: root_uri.clone(),
            primitives: Vec::new(),
        }),
    );
    let outcome = add_gltf_texture_subassets(
        outcome,
        &root_uri,
        &gltf.document,
        vec![GltfImageData {
            pixels: vec![128, 128, 255, 255],
            format: GltfImageFormat::R8G8B8A8,
            width: 1,
            height: 1,
        }],
    )
    .expect("normal/data texture variants must import");
    let outcome = add_gltf_material_subassets(outcome, &root_uri, &gltf.document);
    let normal_uri = gltf_label_uri(&root_uri, "Texture0/Normal");
    let data_uri = gltf_label_uri(&root_uri, "Texture0/Data");

    let normal_entry = outcome
        .entries
        .iter()
        .find(|entry| entry.locator == normal_uri)
        .expect("normal derived texture entry");
    let data_entry = outcome
        .entries
        .iter()
        .find(|entry| entry.locator == data_uri)
        .expect("data derived texture entry");
    assert!(outcome
        .entries
        .iter()
        .all(|entry| entry.locator != gltf_label_uri(&root_uri, "Texture0")));

    let ImportedAsset::Texture(normal_texture) = &normal_entry.asset else {
        panic!("normal derived entry must contain a texture");
    };
    let ImportedAsset::Texture(data_texture) = &data_entry.asset else {
        panic!("data derived entry must contain a texture");
    };
    assert_eq!(
        normal_texture.descriptor.as_ref(),
        Some(&TextureAssetDescriptor::decoded_rgba8_for_import_usage(
            TextureUsageHint::Normal
        ))
    );
    assert_eq!(
        data_texture.descriptor.as_ref(),
        Some(&TextureAssetDescriptor::decoded_rgba8_for_import_usage(
            TextureUsageHint::Data
        ))
    );

    let material_entry = outcome
        .entries
        .iter()
        .find(|entry| entry.locator == gltf_label_uri(&root_uri, "Material0"))
        .expect("glTF material entry");
    let ImportedAsset::Material(material) = &material_entry.asset else {
        panic!("material entry must contain a material");
    };
    assert_eq!(
        material
            .normal_texture
            .as_ref()
            .map(|reference| &reference.locator),
        Some(&normal_uri)
    );
    assert_eq!(
        material
            .metallic_roughness_texture
            .as_ref()
            .map(|reference| &reference.locator),
        Some(&data_uri)
    );
    assert!(material_entry.dependencies.contains(&normal_uri));
    assert!(material_entry.dependencies.contains(&data_uri));
}
