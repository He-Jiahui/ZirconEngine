use std::collections::BTreeMap;

use crate::asset::{
    AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset,
    MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT,
};
use crate::core::framework::render::RenderMeshTopology;
use crate::graphics::scene::gpu_scene::GpuMorphPayload;

#[test]
fn morph_payload_projection_keeps_active_position_deltas_and_weights() {
    let payload = super::morph_payload_from_mesh_asset(&morph_test_mesh(), &[0.25, 0.0, 0.5], None)
        .expect("active position morph payload");

    assert_eq!(payload.vertex_count, 3);
    assert_eq!(payload.target_count, 2);
    assert_eq!(payload.deltas.len(), 24);
    assert_eq!(payload.weights.len(), 2);
    assert_eq!(payload.previous_weights.len(), 2);
    assert_eq!(payload.deltas[0].values, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(payload.weights[0].value, 0.25);
    assert_eq!(payload.previous_weights[0].value, 0.25);
    assert_eq!(payload.deltas[12].values, [2.0, 0.0, 0.0, 1.0]);
    assert_eq!(payload.weights[1].value, 0.5);
    assert_eq!(payload.previous_weights[1].value, 0.5);
}

#[test]
fn morph_payload_projection_keeps_normal_tangent_and_color_delta_rows() {
    let payload = super::morph_payload_from_mesh_asset(&morph_test_mesh(), &[0.25, 1.0, 0.0], None)
        .expect("active rich morph payload");

    assert_eq!(payload.target_count, 2);
    assert_eq!(payload.deltas[1].values, [0.0, 1.0, 0.0, 1.0]);
    assert_eq!(payload.deltas[2].values, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(payload.deltas[3].values, [0.5, 0.25, 0.0, -0.5]);
    assert_eq!(payload.weights[1].value, 1.0);
}

#[test]
fn morph_payload_projection_skips_zero_weight_targets() {
    let mesh = morph_test_mesh();

    assert!(super::morph_payload_from_mesh_asset(&mesh, &[0.0, 0.0, 0.0], None).is_none());
}

#[test]
fn morph_payload_projection_keeps_previous_only_targets_for_velocity() {
    let payload = super::morph_payload_from_mesh_asset(
        &morph_test_mesh(),
        &[0.0, 0.0, 0.0],
        Some(&[0.0, 0.0, 0.5]),
    )
    .expect("previous-only morph payload");

    assert_eq!(payload.target_count, 1);
    assert_eq!(payload.weights.len(), 1);
    assert_eq!(payload.previous_weights.len(), 1);
    assert_eq!(payload.weights[0].value, 0.0);
    assert_eq!(payload.previous_weights[0].value, 0.5);
    assert_eq!(payload.deltas[0].values, [2.0, 0.0, 0.0, 1.0]);
}

#[test]
fn morph_payload_collection_deduplicates_shared_draw_payloads() {
    let first = super::morph_payload_from_mesh_asset(&morph_test_mesh(), &[0.25], None)
        .expect("first payload");
    let second = super::morph_payload_from_mesh_asset(&morph_test_mesh(), &[0.5], None)
        .expect("second payload");

    let collected = super::collect_morph_payload_rows([first.clone(), first.clone(), second]);

    assert_eq!(collected.payloads.len(), 2);
    assert_eq!(collected.payloads[0], GpuMorphPayload::new(0, 0, 3, 1));
    assert_eq!(collected.payloads[1], GpuMorphPayload::new(12, 2, 3, 1));
    assert_eq!(collected.deltas.len(), 24);
    assert_eq!(collected.weights.len(), 4);
    assert_eq!(collected.weights[0].value, 0.25);
    assert_eq!(collected.weights[1].value, 0.25);
    assert_eq!(collected.weights[2].value, 0.5);
    assert_eq!(collected.weights[3].value, 0.5);
    assert_eq!(
        collected
            .slots_by_identity
            .get(&(std::sync::Arc::as_ptr(&first) as usize)),
        Some(&0)
    );
}

fn morph_test_mesh() -> MeshAsset {
    let mut mesh = MeshAsset::new(
        AssetUri::parse("res://meshes/direct-morph-payload.zmesh").unwrap(),
        RenderMeshTopology::TriangleList,
        BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        )]),
        Some(MeshIndices::U32(vec![0, 1, 2])),
    )
    .unwrap();
    mesh.morph_targets = vec![
        MeshMorphTargetAsset {
            name: Some("Lift".to_string()),
            attributes: BTreeMap::from([
                (
                    MESH_ATTRIBUTE_POSITION.to_string(),
                    MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
                ),
                (
                    MESH_ATTRIBUTE_NORMAL.to_string(),
                    MeshAttributeValues::Float32x3(vec![[0.0, 1.0, 0.0]; 3]),
                ),
                (
                    MESH_ATTRIBUTE_TANGENT.to_string(),
                    MeshAttributeValues::Float32x3(vec![[1.0, 0.0, 0.0]; 3]),
                ),
                (
                    MESH_ATTRIBUTE_COLOR.to_string(),
                    MeshAttributeValues::Float32x4(vec![[0.5, 0.25, 0.0, -0.5]; 3]),
                ),
            ]),
        },
        MeshMorphTargetAsset {
            name: Some("NormalOnly".to_string()),
            attributes: BTreeMap::from([(
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 1.0, 0.0]; 3]),
            )]),
        },
        MeshMorphTargetAsset {
            name: Some("Slide".to_string()),
            attributes: BTreeMap::from([(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[2.0, 0.0, 0.0]; 3]),
            )]),
        },
    ];
    mesh
}
