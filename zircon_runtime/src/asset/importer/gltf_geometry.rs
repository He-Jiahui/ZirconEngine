use crate::asset::assets::{
    MeshAttributeValues, MeshMorphTargetAsset, MESH_ATTRIBUTE_UV0, MESH_ATTRIBUTE_UV1,
};
use crate::asset::AssetImportError;

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
#[path = "tests/gltf_geometry.rs"]
mod tests;
