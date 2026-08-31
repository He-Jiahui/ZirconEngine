use super::*;
use crate::asset::importer::{
    GltfTextureColorSpace, GltfTextureUsage, gltf_texture_color_space_usages, gltf_texture_label,
    gltf_texture_variant,
};
use crate::core::framework::render::TextureUsageHint;

#[test]
fn gltf_normal_data_conflict_emits_role_qualified_variants() {
    let usages = texture_usages(
        br#"{
            "asset": { "version": "2.0" },
            "textures": [{ "source": 0 }],
            "materials": [{
                "normalTexture": { "index": 0 },
                "pbrMetallicRoughness": { "metallicRoughnessTexture": { "index": 0 } }
            }]
        }"#,
    );
    let usage = usages[0];
    let normal_variant =
        gltf_texture_variant(GltfTextureColorSpace::Linear, TextureUsageHint::Normal);
    let data_variant = gltf_texture_variant(GltfTextureColorSpace::Linear, TextureUsageHint::Data);

    assert_eq!(usage.texture_variants(), vec![normal_variant, data_variant]);
    assert_eq!(
        gltf_texture_label(3, normal_variant, &[usage]),
        "Texture3/Normal"
    );
    assert_eq!(
        gltf_texture_label(3, data_variant, &[usage]),
        "Texture3/Data"
    );
}

#[test]
fn gltf_texture_labels_preserve_unambiguous_and_color_space_split_uris() {
    let normal_variant =
        gltf_texture_variant(GltfTextureColorSpace::Linear, TextureUsageHint::Normal);
    let srgb_variant = gltf_texture_variant(GltfTextureColorSpace::Srgb, TextureUsageHint::Albedo);

    assert_eq!(
        gltf_texture_label(
            0,
            normal_variant,
            &texture_usages(
                br#"{
                "asset": { "version": "2.0" },
                "textures": [{ "source": 0 }],
                "materials": [{ "normalTexture": { "index": 0 } }]
            }"#,
            )
        ),
        "Texture0"
    );
    let color_space_conflict = texture_usages(
        br#"{
            "asset": { "version": "2.0" },
            "textures": [{ "source": 0 }],
            "materials": [{
                "normalTexture": { "index": 0 },
                "pbrMetallicRoughness": { "baseColorTexture": { "index": 0 } }
            }]
        }"#,
    );
    assert_eq!(
        gltf_texture_label(0, srgb_variant, &color_space_conflict),
        "Texture0/Srgb"
    );
    assert_eq!(
        gltf_texture_label(0, normal_variant, &color_space_conflict),
        "Texture0/Linear"
    );
}

#[test]
fn gltf_normal_data_texture_variants_connect_imported_assets_to_material_slots() {
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
    let root_uri = AssetUri::parse("res://models/shared_normal_data.glb")
        .expect("fixture root URI must be valid");
    let root_asset = TextureAsset::new_rgba8(root_uri.clone(), 1, 1, vec![0, 0, 0, 255]);
    let outcome = AssetImportOutcome::new(root_uri.clone(), ImportedAsset::Texture(root_asset));
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
    assert!(
        outcome
            .entries
            .iter()
            .all(|entry| entry.locator != gltf_label_uri(&root_uri, "Texture0"))
    );
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

fn texture_usages(source: &[u8]) -> Vec<GltfTextureUsage> {
    let gltf = gltf::Gltf::from_slice(source).expect("texture usage fixture must parse");
    gltf_texture_color_space_usages(&gltf.document)
}
