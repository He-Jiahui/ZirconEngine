use std::collections::BTreeMap;

use crate::asset::assets::MESH_ATTRIBUTE_POSITION;

use super::*;

#[test]
fn tangent_uv_attribute_supports_only_the_mesh_shader_abi_channels() {
    assert_eq!(gltf_tangent_uv_attribute(0).unwrap(), MESH_ATTRIBUTE_UV0);
    assert_eq!(gltf_tangent_uv_attribute(1).unwrap(), MESH_ATTRIBUTE_UV1);
    assert!(matches!(
        gltf_tangent_uv_attribute(2),
        Err(AssetImportError::Parse(message))
            if message.contains("TEXCOORD_2") && message.contains("TEXCOORD_0 and TEXCOORD_1")
    ));
}

#[test]
fn clearcoat_only_normal_texture_requires_authored_tangent_space() {
    let gltf = gltf_with_normal_textures(None, Some(1));
    let primitive = gltf
        .document
        .meshes()
        .next()
        .unwrap()
        .primitives()
        .next()
        .unwrap();

    let error = resolve_gltf_normal_texture_tangent_uv_attribute(
        &primitive,
        true,
        &[0.0, 0.0],
        &[0.0, 0.0],
    )
    .unwrap_err();

    assert!(matches!(
        error,
        AssetImportError::Parse(message)
            if message.contains("clearcoat normal texture")
                && message.contains("authored NORMAL and TANGENT")
    ));
}

#[test]
fn base_normal_uv_owns_generated_tangents_when_clearcoat_uses_another_uv() {
    let gltf = gltf_with_normal_textures(Some(0), Some(1));
    let primitive = gltf
        .document
        .meshes()
        .next()
        .unwrap()
        .primitives()
        .next()
        .unwrap();

    let tangent_uv = resolve_gltf_normal_texture_tangent_uv_attribute(
        &primitive,
        true,
        &[0.0, 0.0],
        &[0.0, 0.0],
    )
    .unwrap();

    assert_eq!(tangent_uv, Some(MESH_ATTRIBUTE_UV0));
}

#[test]
fn authored_tangents_do_not_hide_a_missing_clearcoat_uv_attribute() {
    let gltf = gltf_with_normal_textures(Some(0), Some(1));
    let primitive = gltf
        .document
        .meshes()
        .next()
        .unwrap()
        .primitives()
        .next()
        .unwrap();

    let error =
        resolve_gltf_normal_texture_tangent_uv_attribute(&primitive, false, &[0.0, 0.0], &[])
            .unwrap_err();

    assert!(matches!(
        error,
        AssetImportError::Parse(message)
            if message.contains("clearcoat normal texture")
                && message.contains(MESH_ATTRIBUTE_UV1)
    ));
}

#[test]
fn flat_normal_expansion_remaps_every_morph_vertex() {
    let mut targets = vec![MeshMorphTargetAsset {
        name: Some("hard-edge".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ]),
        )]),
    }];

    remap_gltf_morph_targets_for_flat_normals(&mut targets, &[0, 1, 2, 0, 3, 1]).unwrap();

    assert_eq!(
        targets[0].attributes[MESH_ATTRIBUTE_POSITION]
            .as_float32x3()
            .unwrap(),
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
        ]
    );
}

fn gltf_with_normal_textures(
    base_normal_uv: Option<u32>,
    clearcoat_normal_uv: Option<u32>,
) -> gltf::Gltf {
    let normal_texture = base_normal_uv
        .map(|uv| format!(r#", "normalTexture": {{ "index": 0, "texCoord": {uv} }}"#))
        .unwrap_or_default();
    let clearcoat = clearcoat_normal_uv
        .map(|uv| {
            format!(
                r#", "extensions": {{ "KHR_materials_clearcoat": {{ "clearcoatNormalTexture": {{ "index": 0, "texCoord": {uv} }} }} }}"#
            )
        })
        .unwrap_or_default();
    let source = format!(
        r#"{{
                "asset": {{ "version": "2.0" }},
                "images": [{{ "uri": "clearcoat-normal.png" }}],
                "textures": [{{ "source": 0 }}],
                "materials": [{{ "pbrMetallicRoughness": {{}}{normal_texture}{clearcoat} }}],
                "meshes": [{{ "primitives": [{{ "attributes": {{}}, "material": 0 }}] }}]
            }}"#
    );
    gltf::Gltf::from_slice_without_validation(source.as_bytes())
        .expect("normal-texture glTF fixture parses")
}
