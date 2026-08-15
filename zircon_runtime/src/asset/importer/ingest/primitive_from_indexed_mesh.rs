use crate::core::math::{Vec2, Vec3};

use super::generate_normals::generate_normals;
use crate::asset::assets::{ModelAsset, ModelPrimitiveAsset};
use crate::asset::{
    AssetImportError, MeshSdfCookBudget, MeshSdfCookRequest, MeshVertex,
    VirtualGeometryCookRequest, cook_mesh_sdf_or_fallback, cook_virtual_geometry_from_mesh,
};

pub(super) fn primitive_from_indexed_mesh(
    positions: &[f32],
    normals: &[f32],
    texcoords: &[f32],
    texcoords1: &[f32],
    tangents: &[[f32; 4]],
    colors: &[[f32; 4]],
    indices: &[u32],
    joint_indices: &[[u16; 4]],
    joint_weights: &[[f32; 4]],
    mesh_name: Option<&str>,
    source_hint: &str,
    virtual_geometry_request: &VirtualGeometryCookRequest,
    mesh_sdf_request: &MeshSdfCookRequest,
    mesh_sdf_budget: &mut MeshSdfCookBudget,
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
            let uv1 = if texcoords1.len() >= (index + 1) * 2 {
                Vec2::new(texcoords1[index * 2], texcoords1[index * 2 + 1])
            } else {
                Vec2::ZERO
            };
            let tangent = tangents.get(index).copied().unwrap_or([1.0, 0.0, 0.0, 1.0]);
            let color = colors.get(index).copied().unwrap_or([1.0, 1.0, 1.0, 1.0]);
            MeshVertex::new(
                position,
                if normal.length_squared() <= f32::EPSILON {
                    Vec3::Y
                } else {
                    normal.normalize_or_zero()
                },
                uv,
            )
            .with_uv1(uv1)
            .with_tangent(tangent)
            .with_color(color)
            .with_skinning(
                joint_indices.get(index).copied().unwrap_or([0, 0, 0, 0]),
                joint_weights
                    .get(index)
                    .copied()
                    .unwrap_or([0.0, 0.0, 0.0, 0.0]),
            )
        })
        .collect();

    // Automatic VG payloads currently encode vertex ordinals in joint slots;
    // skinned imports preserve those slots for authored joint data instead.
    let virtual_geometry = if uses_skinning_channels(joint_weights) {
        None
    } else {
        virtual_geometry_request
            .cook_config_for(mesh_name, source_hint)
            .and_then(|config| cook_virtual_geometry_from_mesh(&vertices, indices, config))
    };
    let mesh_sdf = match mesh_sdf_request.settings() {
        Some(settings) => cook_mesh_sdf_or_fallback(&vertices, indices, settings, mesh_sdf_budget)
            .map_err(|error| AssetImportError::Parse(format!("cook mesh SDF: {error}")))?,
        None => None,
    };

    let mut primitive = ModelPrimitiveAsset {
        vertices,
        indices: indices.to_vec(),
        mesh: None,
        mesh_sdf,
        virtual_geometry,
    };
    primitive.assign_virtual_geometry_vertex_ordinals();
    Ok(primitive)
}

pub(super) fn backfill_mesh_sdf_for_model(
    model: &mut ModelAsset,
    mesh_sdf_request: &MeshSdfCookRequest,
) -> Result<(), AssetImportError> {
    let Some(settings) = mesh_sdf_request.settings() else {
        return Ok(());
    };
    let mut mesh_sdf_budget = MeshSdfCookBudget::default();
    for primitive in &mut model.primitives {
        if primitive.mesh_sdf.is_none() && !primitive.vertices.is_empty() {
            primitive.mesh_sdf = cook_mesh_sdf_or_fallback(
                &primitive.vertices,
                &primitive.indices,
                settings,
                &mut mesh_sdf_budget,
            )
            .map_err(|error| AssetImportError::Parse(format!("cook mesh SDF: {error}")))?;
        }
    }
    Ok(())
}

pub(super) fn backfill_virtual_geometry_for_model(
    model: &mut ModelAsset,
    virtual_geometry_request: &VirtualGeometryCookRequest,
) {
    let source_hint = model.uri.to_string();
    for (primitive_index, primitive) in model.primitives.iter_mut().enumerate() {
        // The VG ordinal assignment below would overwrite active skinning
        // channels, so weighted primitives stay on the skinned mesh path.
        if primitive.uses_skinning_channels() {
            continue;
        }
        if primitive.virtual_geometry.is_none() {
            let mesh_name = format!("primitive_{primitive_index}");
            primitive.virtual_geometry = virtual_geometry_request
                .cook_config_for(Some(&mesh_name), &source_hint)
                .and_then(|config| {
                    cook_virtual_geometry_from_mesh(&primitive.vertices, &primitive.indices, config)
                });
        }
        primitive.assign_virtual_geometry_vertex_ordinals();
    }
}

fn uses_skinning_channels(joint_weights: &[[f32; 4]]) -> bool {
    joint_weights
        .iter()
        .flatten()
        .any(|weight| weight.abs() > f32::EPSILON)
}
