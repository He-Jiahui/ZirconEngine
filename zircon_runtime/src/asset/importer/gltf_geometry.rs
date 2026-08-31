use crate::asset::AssetImportError;
use crate::asset::assets::{
    MESH_ATTRIBUTE_UV0, MESH_ATTRIBUTE_UV1, MeshAttributeValues, MeshMorphTargetAsset,
};

use super::{gltf_clearcoat_normal_texture_projection, project_gltf_texture_transform};

pub fn resolve_gltf_normal_texture_tangent_uv_attribute(
    primitive: &gltf::Primitive<'_>,
    tangents_missing: bool,
    texcoords0: &[f32],
    texcoords1: &[f32],
) -> Result<Option<&'static str>, AssetImportError> {
    let base_normal = gltf_normal_texture_tangent_uv_attribute(primitive)?;
    let clearcoat_normal = gltf_clearcoat_normal_texture_projection(&primitive.material())
        .map(|projection| gltf_tangent_uv_attribute(projection.uv_channel))
        .transpose()?;
    ensure_gltf_tangent_uv_attribute_present_for_texture(
        base_normal,
        texcoords0,
        texcoords1,
        "base normal texture",
    )?;
    ensure_gltf_tangent_uv_attribute_present_for_texture(
        clearcoat_normal,
        texcoords0,
        texcoords1,
        "clearcoat normal texture",
    )?;

    if tangents_missing && base_normal.is_none() && clearcoat_normal.is_some() {
        return Err(AssetImportError::Parse(
            "glTF clearcoat normal texture requires authored NORMAL and TANGENT attributes when the base material has no normal texture"
                .to_string(),
        ));
    }
    Ok(base_normal)
}

pub fn gltf_normal_texture_tangent_uv_attribute(
    primitive: &gltf::Primitive<'_>,
) -> Result<Option<&'static str>, AssetImportError> {
    let Some(normal_texture) = primitive.material().normal_texture() else {
        return Ok(None);
    };
    let projection = project_gltf_texture_transform(
        normal_texture.tex_coord(),
        normal_texture.extension_value("KHR_texture_transform"),
    );
    gltf_tangent_uv_attribute(projection.uv_channel).map(Some)
}

pub fn gltf_tangent_uv_attribute(uv_channel: u32) -> Result<&'static str, AssetImportError> {
    match uv_channel {
        0 => Ok(MESH_ATTRIBUTE_UV0),
        1 => Ok(MESH_ATTRIBUTE_UV1),
        unsupported => Err(AssetImportError::Parse(format!(
            "glTF normal texture references unsupported TEXCOORD_{unsupported}; Zircon's mesh shader ABI supports TEXCOORD_0 and TEXCOORD_1"
        ))),
    }
}

pub fn ensure_gltf_tangent_uv_attribute_present(
    uv_attribute: Option<&'static str>,
    texcoords0: &[f32],
    texcoords1: &[f32],
) -> Result<(), AssetImportError> {
    ensure_gltf_tangent_uv_attribute_present_for_texture(
        uv_attribute,
        texcoords0,
        texcoords1,
        "normal texture",
    )
}

fn ensure_gltf_tangent_uv_attribute_present_for_texture(
    uv_attribute: Option<&'static str>,
    texcoords0: &[f32],
    texcoords1: &[f32],
    texture_kind: &str,
) -> Result<(), AssetImportError> {
    let missing = match uv_attribute {
        Some(MESH_ATTRIBUTE_UV0) => texcoords0.is_empty(),
        Some(MESH_ATTRIBUTE_UV1) => texcoords1.is_empty(),
        None => false,
        Some(_) => unreachable!("tangent UV attributes are constrained by the glTF projector"),
    };
    if missing {
        return Err(AssetImportError::Parse(format!(
            "glTF {texture_kind} requires missing mesh attribute `{}`",
            uv_attribute.unwrap()
        )));
    }
    Ok(())
}

pub fn remap_gltf_morph_targets_for_flat_normals(
    targets: &mut [MeshMorphTargetAsset],
    source_indices: &[u32],
) -> Result<(), AssetImportError> {
    for target in targets {
        for (attribute, values) in &mut target.attributes {
            *values = remap_morph_attribute_values(values, source_indices, attribute)?;
        }
    }
    Ok(())
}

fn remap_morph_attribute_values(
    values: &MeshAttributeValues,
    source_indices: &[u32],
    attribute: &str,
) -> Result<MeshAttributeValues, AssetImportError> {
    Ok(match values {
        MeshAttributeValues::Float32x2(values) => {
            MeshAttributeValues::Float32x2(remap_morph_values(values, source_indices, attribute)?)
        }
        MeshAttributeValues::Float32x3(values) => {
            MeshAttributeValues::Float32x3(remap_morph_values(values, source_indices, attribute)?)
        }
        MeshAttributeValues::Float32x4(values) => {
            MeshAttributeValues::Float32x4(remap_morph_values(values, source_indices, attribute)?)
        }
        MeshAttributeValues::Uint16x4(values) => {
            MeshAttributeValues::Uint16x4(remap_morph_values(values, source_indices, attribute)?)
        }
        MeshAttributeValues::Uint32x4(values) => {
            MeshAttributeValues::Uint32x4(remap_morph_values(values, source_indices, attribute)?)
        }
    })
}

fn remap_morph_values<T: Copy>(
    values: &[T],
    source_indices: &[u32],
    attribute: &str,
) -> Result<Vec<T>, AssetImportError> {
    source_indices
        .iter()
        .map(|&source_index| {
            values.get(source_index as usize).copied().ok_or_else(|| {
                AssetImportError::Parse(format!(
                    "glTF morph target attribute `{attribute}` has {} values but flat-normal expansion references vertex {source_index}",
                    values.len()
                ))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
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
}
