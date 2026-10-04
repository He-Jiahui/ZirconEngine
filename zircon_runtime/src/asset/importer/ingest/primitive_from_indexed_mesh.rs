use crate::asset::assets::ModelPrimitiveAsset;
use crate::asset::importer::{project_indexed_mesh_primitive, IndexedMeshSource};
use crate::asset::{
    AssetImportError, MeshSdfCookBudget, MeshSdfCookRequest, VirtualGeometryCookRequest,
};

pub(super) use crate::asset::importer::{
    backfill_mesh_sdf_for_model, backfill_virtual_geometry_for_model,
    IndexedMeshMissingNormalPolicy as MissingNormalPolicy,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn primitive_from_indexed_mesh(
    positions: &[f32],
    normals: &[f32],
    missing_normal_policy: MissingNormalPolicy,
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
    project_indexed_mesh_primitive(
        IndexedMeshSource {
            positions,
            normals,
            texcoords0: texcoords,
            texcoords1,
            tangents,
            colors,
            indices,
            joint_indices,
            joint_weights,
            missing_normal_policy,
        },
        mesh_name,
        source_hint,
        virtual_geometry_request,
        mesh_sdf_request,
        mesh_sdf_budget,
    )
}

#[cfg(test)]
#[path = "tests/primitive_from_indexed_mesh.rs"]
mod tests;
