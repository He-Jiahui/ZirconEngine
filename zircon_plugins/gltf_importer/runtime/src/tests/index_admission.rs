use super::*;
use zircon_runtime::asset::{MeshSdfCookRequest, VirtualGeometryCookRequest};

#[test]
fn gltf_index_admission_rejects_out_of_range_vertices_without_panicking() {
    let result = std::panic::catch_unwind(|| {
        let mut budget = MeshSdfCookBudget::default();
        project_indexed_mesh_primitive(
            IndexedMeshSource {
                positions: &[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                normals: &[],
                texcoords0: &[],
                texcoords1: &[],
                tangents: &[],
                colors: &[],
                indices: &[0, 1, 3],
                joint_indices: &[],
                joint_weights: &[],
                missing_normal_policy: IndexedMeshMissingNormalPolicy::Flat,
            },
            Some("malformed"),
            "gltf-index-admission-test",
            &VirtualGeometryCookRequest::default(),
            &MeshSdfCookRequest::default(),
            &mut budget,
        )
    });

    assert!(result.is_ok(), "malformed glTF indices must not unwind");
    let error = result
        .unwrap()
        .expect_err("out-of-range glTF index must be rejected");
    assert!(matches!(
        error,
        AssetImportError::Parse(message)
            if message.contains("mesh index 3") && message.contains("vertex count 3")
    ));
}
