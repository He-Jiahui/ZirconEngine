use crate::asset::assets::{
    MESH_ATTRIBUTE_JOINT_INDEX, MeshAsset, MeshAttributeValues, ModelAsset, ModelPrimitiveAsset,
};
use crate::asset::{
    AssetImportError, MeshSdfCookBudget, MeshSdfCookRequest, MeshVertex,
    VirtualGeometryCookRequest, cook_mesh_sdf_or_fallback, cook_virtual_geometry_from_mesh,
};
use crate::core::math::{Vec2, Vec3};

use super::generate_normals::{generate_normals, validate_triangle_indices};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexedMeshMissingNormalPolicy {
    Smooth,
    Flat,
}

#[derive(Clone, Copy, Debug)]
pub struct IndexedMeshSource<'a> {
    pub positions: &'a [f32],
    pub normals: &'a [f32],
    pub texcoords0: &'a [f32],
    pub texcoords1: &'a [f32],
    pub tangents: &'a [[f32; 4]],
    pub colors: &'a [[f32; 4]],
    pub indices: &'a [u32],
    pub joint_indices: &'a [[u16; 4]],
    pub joint_weights: &'a [[f32; 4]],
    pub missing_normal_policy: IndexedMeshMissingNormalPolicy,
}

pub fn project_indexed_mesh_primitive(
    source: IndexedMeshSource<'_>,
    mesh_name: Option<&str>,
    source_hint: &str,
    virtual_geometry_request: &VirtualGeometryCookRequest,
    mesh_sdf_request: &MeshSdfCookRequest,
    mesh_sdf_budget: &mut MeshSdfCookBudget,
) -> Result<ModelPrimitiveAsset, AssetImportError> {
    if source.positions.len() % 3 != 0 {
        return Err(AssetImportError::Parse(
            "vertex positions were not a multiple of 3".to_string(),
        ));
    }
    let vertex_count = source.positions.len() / 3;
    let mut computed_normals = match (source.normals.is_empty(), source.missing_normal_policy) {
        (true, IndexedMeshMissingNormalPolicy::Smooth) => {
            generate_normals(source.positions, source.indices)?
        }
        (true, IndexedMeshMissingNormalPolicy::Flat) => {
            validate_triangle_indices(source.indices, vertex_count)?;
            Vec::new()
        }
        (false, _) => {
            validate_triangle_indices(source.indices, vertex_count)?;
            source.normals.to_vec()
        }
    };
    if computed_normals.len() < vertex_count * 3 {
        computed_normals.resize(vertex_count * 3, 0.0);
    }

    let (vertices, mesh_indices) = if source.normals.is_empty()
        && source.missing_normal_policy == IndexedMeshMissingNormalPolicy::Flat
    {
        flat_vertices_from_indexed_mesh(source)?
    } else {
        let vertices = (0..vertex_count)
            .map(|index| {
                mesh_vertex_from_source(
                    source,
                    index,
                    Vec3::new(
                        computed_normals[index * 3],
                        computed_normals[index * 3 + 1],
                        computed_normals[index * 3 + 2],
                    ),
                    !source.normals.is_empty(),
                )
            })
            .collect();
        (vertices, source.indices.to_vec())
    };

    // Automatic VG payloads currently encode vertex ordinals in joint slots;
    // skinned imports preserve those slots for authored joint data instead.
    let virtual_geometry = if uses_skinning_channels(source.joint_weights) {
        None
    } else {
        virtual_geometry_request
            .cook_config_for(mesh_name, source_hint)
            .and_then(|config| cook_virtual_geometry_from_mesh(&vertices, &mesh_indices, config))
    };
    let mesh_sdf = match mesh_sdf_request.settings() {
        Some(settings) => {
            cook_mesh_sdf_or_fallback(&vertices, &mesh_indices, settings, mesh_sdf_budget)
                .map_err(|error| AssetImportError::Parse(format!("cook mesh SDF: {error}")))?
        }
        None => None,
    };

    let mut primitive = ModelPrimitiveAsset {
        vertices,
        indices: mesh_indices,
        mesh: None,
        mesh_sdf,
        virtual_geometry,
    };
    primitive.assign_virtual_geometry_vertex_ordinals();
    Ok(primitive)
}

pub fn cook_mesh_asset_derived_data(
    mesh: &mut MeshAsset,
    mesh_name: Option<&str>,
    source_hint: &str,
    virtual_geometry_request: &VirtualGeometryCookRequest,
    mesh_sdf_request: &MeshSdfCookRequest,
    mesh_sdf_budget: &mut MeshSdfCookBudget,
) -> Result<(), AssetImportError> {
    let virtual_geometry_config = virtual_geometry_request.cook_config_for(mesh_name, source_hint);
    let mesh_sdf_settings = mesh_sdf_request.settings();
    if virtual_geometry_config.is_none() && mesh_sdf_settings.is_none() {
        return Ok(());
    }
    if mesh.virtual_geometry.is_some() || mesh.mesh_sdf.is_some() {
        return Err(AssetImportError::Parse(
            "final mesh derived data must be cooked exactly once after geometry processing"
                .to_string(),
        ));
    }

    let mut primitive = mesh.to_model_primitive().map_err(|error| {
        AssetImportError::Parse(format!("project final mesh for derived-data cook: {error}"))
    })?;
    let virtual_geometry = if mesh.skin.is_some() || primitive.uses_skinning_channels() {
        None
    } else {
        virtual_geometry_config.and_then(|config| {
            cook_virtual_geometry_from_mesh(&primitive.vertices, &primitive.indices, config)
        })
    };
    let mesh_sdf = match mesh_sdf_settings {
        Some(settings) => cook_mesh_sdf_or_fallback(
            &primitive.vertices,
            &primitive.indices,
            settings,
            mesh_sdf_budget,
        )
        .map_err(|error| AssetImportError::Parse(format!("cook mesh SDF: {error}")))?,
        None => None,
    };

    let generated_joint_indices = if virtual_geometry.is_some() {
        primitive.virtual_geometry = virtual_geometry.clone();
        primitive.assign_virtual_geometry_vertex_ordinals();
        Some(MeshAttributeValues::Uint16x4(
            primitive
                .vertices
                .iter()
                .map(|vertex| vertex.joint_indices)
                .collect(),
        ))
    } else {
        None
    };

    let previous_joint_indices = generated_joint_indices.map(|joint_indices| {
        mesh.attributes
            .insert(MESH_ATTRIBUTE_JOINT_INDEX.to_string(), joint_indices)
    });
    mesh.virtual_geometry = virtual_geometry;
    mesh.mesh_sdf = mesh_sdf;
    if let Err(error) = mesh.validate() {
        mesh.virtual_geometry = None;
        mesh.mesh_sdf = None;
        if let Some(previous_joint_indices) = previous_joint_indices {
            if let Some(previous_joint_indices) = previous_joint_indices {
                mesh.attributes.insert(
                    MESH_ATTRIBUTE_JOINT_INDEX.to_string(),
                    previous_joint_indices,
                );
            } else {
                mesh.attributes.remove(MESH_ATTRIBUTE_JOINT_INDEX);
            }
        }
        return Err(AssetImportError::Parse(format!(
            "validate final mesh derived data: {error}"
        )));
    }
    Ok(())
}

fn mesh_vertex_from_source(
    source: IndexedMeshSource<'_>,
    index: usize,
    normal: Vec3,
    use_authored_tangent: bool,
) -> MeshVertex {
    let position = position_at(source.positions, index);
    let uv = vec2_at_or_zero(source.texcoords0, index);
    let uv1 = vec2_at_or_zero(source.texcoords1, index);
    let tangent = use_authored_tangent
        .then(|| source.tangents.get(index).copied())
        .flatten()
        .unwrap_or([1.0, 0.0, 0.0, 1.0]);
    let normal = if normal.length_squared() <= f32::EPSILON {
        Vec3::Y
    } else {
        normal.normalize_or_zero()
    };
    MeshVertex::new(position, normal, uv)
        .with_uv1(uv1)
        .with_tangent(tangent)
        .with_color(source.colors.get(index).copied().unwrap_or([1.0; 4]))
        .with_skinning(
            source.joint_indices.get(index).copied().unwrap_or([0; 4]),
            source.joint_weights.get(index).copied().unwrap_or([0.0; 4]),
        )
}

fn flat_vertices_from_indexed_mesh(
    source: IndexedMeshSource<'_>,
) -> Result<(Vec<MeshVertex>, Vec<u32>), AssetImportError> {
    let mut vertices = Vec::with_capacity(source.indices.len());
    let mut expanded_indices = Vec::with_capacity(source.indices.len());
    for triangle in source.indices.chunks_exact(3) {
        let [a, b, c] = [
            triangle[0] as usize,
            triangle[1] as usize,
            triangle[2] as usize,
        ];
        let face_normal = (position_at(source.positions, b) - position_at(source.positions, a))
            .cross(position_at(source.positions, c) - position_at(source.positions, a));
        for source_index in [a, b, c] {
            let expanded_index = u32::try_from(vertices.len()).map_err(|_| {
                AssetImportError::Parse(
                    "flat-normal vertex expansion exceeded the u32 index range".to_string(),
                )
            })?;
            vertices.push(mesh_vertex_from_source(
                source,
                source_index,
                face_normal,
                false,
            ));
            expanded_indices.push(expanded_index);
        }
    }
    Ok((vertices, expanded_indices))
}

fn position_at(positions: &[f32], index: usize) -> Vec3 {
    Vec3::new(
        positions[index * 3],
        positions[index * 3 + 1],
        positions[index * 3 + 2],
    )
}

fn vec2_at_or_zero(values: &[f32], index: usize) -> Vec2 {
    if values.len() >= (index + 1) * 2 {
        Vec2::new(values[index * 2], values[index * 2 + 1])
    } else {
        Vec2::ZERO
    }
}

pub fn backfill_mesh_sdf_for_model(
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

pub fn backfill_virtual_geometry_for_model(
    model: &mut ModelAsset,
    virtual_geometry_request: &VirtualGeometryCookRequest,
) {
    let source_hint = model.uri.to_string();
    for (primitive_index, primitive) in model.primitives.iter_mut().enumerate() {
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
