use crate::asset::assets::ModelPrimitiveAsset;
use crate::asset::importer::{IndexedMeshSource, project_indexed_mesh_primitive};
use crate::asset::{
    AssetImportError, MeshSdfCookBudget, MeshSdfCookRequest, VirtualGeometryCookRequest,
};

pub(super) use crate::asset::importer::{
    IndexedMeshMissingNormalPolicy as MissingNormalPolicy, backfill_mesh_sdf_for_model,
    backfill_virtual_geometry_for_model,
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
mod tests {
    use super::*;
    use std::panic::AssertUnwindSafe;

    #[test]
    fn primitive_rejects_out_of_range_indices_with_authored_normals() {
        let mut mesh_sdf_budget = MeshSdfCookBudget::default();
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            primitive_from_indexed_mesh(
                &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                MissingNormalPolicy::Smooth,
                &[],
                &[],
                &[],
                &[],
                &[0, 1, 3],
                &[],
                &[],
                Some("malformed"),
                "runtime-index-admission-test",
                &VirtualGeometryCookRequest::default(),
                &MeshSdfCookRequest::default(),
                &mut mesh_sdf_budget,
            )
        }));

        assert!(result.is_ok(), "malformed indices must not unwind");
        let error = result
            .unwrap()
            .expect_err("out-of-range mesh index must be rejected");
        assert!(matches!(
            error,
            AssetImportError::Parse(message)
                if message.contains("mesh index 3") && message.contains("vertex count 3")
        ));
    }

    #[test]
    fn primitive_expands_shared_vertices_for_missing_flat_normals_before_tangent_generation() {
        let mut mesh_sdf_budget = MeshSdfCookBudget::default();
        let primitive = primitive_from_indexed_mesh(
            &[
                0.0, 0.0, 0.0, // shared origin
                1.0, 0.0, 0.0, // shared edge
                0.0, 1.0, 0.0, // +Z face
                0.0, 0.0, 1.0, // +Y face
            ],
            &[],
            MissingNormalPolicy::Flat,
            &[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            &[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
            &[],
            &[
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 1.0],
                [1.0, 1.0, 0.0, 1.0],
            ],
            &[0, 1, 2, 0, 3, 1],
            &[[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]],
            &[
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            Some("flat-hard-edge"),
            "runtime-flat-normal-test",
            &VirtualGeometryCookRequest::default(),
            &MeshSdfCookRequest::default(),
            &mut mesh_sdf_budget,
        )
        .unwrap();

        assert_eq!(primitive.vertices.len(), 6);
        assert_eq!(primitive.indices, [0, 1, 2, 3, 4, 5]);
        for vertex in &primitive.vertices[..3] {
            assert_eq!(vertex.normal, [0.0, 0.0, 1.0]);
        }
        for vertex in &primitive.vertices[3..] {
            assert_eq!(vertex.normal, [0.0, 1.0, 0.0]);
        }

        let source_indices = [0, 1, 2, 0, 3, 1];
        let source_uv0 = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
        let source_uv1 = [[0.1, 0.2], [0.3, 0.4], [0.5, 0.6], [0.7, 0.8]];
        let source_colors = [
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            [1.0, 1.0, 0.0, 1.0],
        ];
        let source_joints = [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]];
        let source_weights = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        for (vertex, source_index) in primitive.vertices.iter().zip(source_indices) {
            assert_eq!(vertex.uv, source_uv0[source_index]);
            assert_eq!(vertex.uv1, source_uv1[source_index]);
            assert_eq!(vertex.color, source_colors[source_index]);
            assert_eq!(vertex.joint_indices, source_joints[source_index]);
            assert_eq!(vertex.joint_weights, source_weights[source_index]);
        }
    }
}
