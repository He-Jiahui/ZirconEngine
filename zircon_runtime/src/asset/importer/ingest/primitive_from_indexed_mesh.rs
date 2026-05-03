#[cfg(test)]
use crate::core::math::{Vec2, Vec3};

#[cfg(test)]
use super::generate_normals::generate_normals;
use crate::asset::assets::ModelAsset;
#[cfg(test)]
use crate::asset::assets::ModelPrimitiveAsset;
use crate::asset::{cook_virtual_geometry_from_mesh, VirtualGeometryCookConfig};
#[cfg(test)]
use crate::asset::{AssetImportError, MeshVertex};

#[cfg(test)]
pub(super) fn primitive_from_indexed_mesh(
    positions: &[f32],
    normals: &[f32],
    texcoords: &[f32],
    indices: &[u32],
    joint_indices: &[[u16; 4]],
    joint_weights: &[[f32; 4]],
    mesh_name: Option<&str>,
    source_hint: &str,
) -> Result<ModelPrimitiveAsset, AssetImportError> {
    if positions.len() % 3 != 0 {
        return Err(AssetImportError::Parse(
            "vertex positions were not a multiple of 3".to_string(),
        ));
    }
    let vertex_count = positions.len() / 3;
    let mut computed_normals = if normals.is_empty() {
        generate_normals(positions, indices)
    } else {
        normals.to_vec()
    };
    if computed_normals.len() < vertex_count * 3 {
        computed_normals.resize(vertex_count * 3, 0.0);
    }

    let vertices: Vec<MeshVertex> = (0..vertex_count)
        .map(|index| {
            let position = Vec3::new(
                positions[index * 3],
                positions[index * 3 + 1],
                positions[index * 3 + 2],
            );
            let normal = Vec3::new(
                computed_normals[index * 3],
                computed_normals[index * 3 + 1],
                computed_normals[index * 3 + 2],
            );
            let uv = if texcoords.len() >= (index + 1) * 2 {
                Vec2::new(texcoords[index * 2], texcoords[index * 2 + 1])
            } else {
                Vec2::ZERO
            };
            MeshVertex::new(
                position,
                if normal.length_squared() <= f32::EPSILON {
                    Vec3::Y
                } else {
                    normal.normalize_or_zero()
                },
                uv,
            )
            .with_skinning(
                joint_indices.get(index).copied().unwrap_or([0, 0, 0, 0]),
                joint_weights
                    .get(index)
                    .copied()
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]),
            )
        })
        .collect();

    let virtual_geometry = cook_virtual_geometry_from_mesh(
        &vertices,
        indices,
        VirtualGeometryCookConfig {
            mesh_name: mesh_name.map(str::to_owned),
            source_hint: Some(source_hint.to_string()),
            ..VirtualGeometryCookConfig::default()
        },
    );

    Ok(ModelPrimitiveAsset {
        vertices,
        indices: indices.to_vec(),
        virtual_geometry,
    })
}

pub(super) fn backfill_virtual_geometry_for_model(model: &mut ModelAsset) {
    let source_hint = model.uri.to_string();
    for (primitive_index, primitive) in model.primitives.iter_mut().enumerate() {
        if primitive.virtual_geometry.is_some() {
            continue;
        }
        primitive.virtual_geometry = cook_virtual_geometry_from_mesh(
            &primitive.vertices,
            &primitive.indices,
            VirtualGeometryCookConfig {
                mesh_name: Some(format!("primitive_{primitive_index}")),
                source_hint: Some(source_hint.clone()),
                ..VirtualGeometryCookConfig::default()
            },
        );
    }
}
