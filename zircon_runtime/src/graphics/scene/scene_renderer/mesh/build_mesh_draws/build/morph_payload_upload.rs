use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::{
    MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT,
    MeshAsset,
};
use crate::graphics::scene::gpu_scene::{
    GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GpuScene, GpuSceneMorphUploadReport,
};

use super::pending_mesh_draw::{PendingMeshDraw, PendingMorphPayload};

const MORPH_DELTA_ROWS_PER_VERTEX_TARGET: usize = 4;

pub(super) fn morph_payload_from_mesh_asset(
    mesh_asset: &MeshAsset,
    morph_weights: &[f32],
    previous_morph_weights: Option<&[f32]>,
) -> Option<Arc<PendingMorphPayload>> {
    let vertex_count = mesh_asset.vertex_count().ok()?;
    let vertex_count_u32 = u32::try_from(vertex_count).ok()?;
    if vertex_count == 0 {
        return None;
    }

    let mut deltas = Vec::new();
    let mut weights = Vec::new();
    let mut previous_weights = Vec::new();
    let mut target_count = 0u32;

    for target_index in 0..mesh_asset.morph_targets.len() {
        let weight = morph_weights.get(target_index).copied().unwrap_or_default();
        let previous_weight = previous_morph_weights
            .and_then(|weights| weights.get(target_index))
            .copied()
            .unwrap_or(weight);
        if weight.abs() <= f32::EPSILON && previous_weight.abs() <= f32::EPSILON {
            continue;
        }

        let position_deltas = morph_float32x3_named_attribute(
            mesh_asset,
            target_index,
            MESH_ATTRIBUTE_POSITION,
            vertex_count,
        );
        let normal_deltas = morph_float32x3_named_attribute(
            mesh_asset,
            target_index,
            MESH_ATTRIBUTE_NORMAL,
            vertex_count,
        );
        let tangent_deltas = morph_float32x3_named_attribute(
            mesh_asset,
            target_index,
            MESH_ATTRIBUTE_TANGENT,
            vertex_count,
        );
        let color_deltas = morph_float32x4_named_attribute(
            mesh_asset,
            target_index,
            MESH_ATTRIBUTE_COLOR,
            vertex_count,
        );
        if position_deltas.is_none()
            && normal_deltas.is_none()
            && tangent_deltas.is_none()
            && color_deltas.is_none()
        {
            continue;
        }

        target_count = target_count.saturating_add(1);
        weights.push(GpuMorphWeight::new(weight));
        previous_weights.push(GpuMorphWeight::new(previous_weight));
        for vertex_index in 0..vertex_count {
            deltas.extend(morph_vertex_delta_rows(
                position_deltas.map(|values| values[vertex_index]),
                normal_deltas.map(|values| values[vertex_index]),
                tangent_deltas.map(|values| values[vertex_index]),
                color_deltas.map(|values| values[vertex_index]),
            ));
        }
    }

    (!deltas.is_empty()).then_some(Arc::new(PendingMorphPayload {
        vertex_count: vertex_count_u32,
        target_count,
        deltas,
        weights,
        previous_weights,
    }))
}

pub(super) fn upload_morph_payloads(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_scene: &mut GpuScene,
    pending_draws: &mut [PendingMeshDraw],
) -> GpuSceneMorphUploadReport {
    let collected = collect_morph_payload_rows(
        pending_draws
            .iter()
            .filter_map(|pending_draw| pending_draw.morph_payload.clone()),
    );
    for pending_draw in pending_draws {
        pending_draw.morph_payload_slot = pending_draw.morph_payload.as_ref().and_then(|payload| {
            collected
                .slots_by_identity
                .get(&(Arc::as_ptr(payload) as usize))
                .copied()
        });
    }
    gpu_scene.upload_morph_buffers(
        device,
        queue,
        &collected.payloads,
        &collected.deltas,
        &collected.weights,
    )
}

#[derive(Default)]
struct CollectedMorphPayloadRows {
    payloads: Vec<GpuMorphPayload>,
    deltas: Vec<GpuMorphDelta>,
    weights: Vec<GpuMorphWeight>,
    slots_by_identity: HashMap<usize, u32>,
}

fn collect_morph_payload_rows(
    payloads: impl IntoIterator<Item = Arc<PendingMorphPayload>>,
) -> CollectedMorphPayloadRows {
    let mut slots_by_identity = HashMap::new();
    let mut payload_rows = Vec::new();
    let mut deltas = Vec::new();
    let mut weights = Vec::new();

    for payload in payloads {
        let identity = Arc::as_ptr(&payload) as usize;
        if slots_by_identity.contains_key(&identity) {
            continue;
        }
        let payload_slot = u32::try_from(payload_rows.len()).unwrap_or(u32::MAX);
        slots_by_identity.insert(identity, payload_slot);
        payload_rows.push(GpuMorphPayload::new(
            u32::try_from(deltas.len()).unwrap_or(u32::MAX),
            u32::try_from(weights.len()).unwrap_or(u32::MAX),
            payload.vertex_count,
            payload.target_count,
        ));
        deltas.extend_from_slice(&payload.deltas);
        weights.extend_from_slice(&payload.weights);
        weights.extend_from_slice(&payload.previous_weights);
    }

    CollectedMorphPayloadRows {
        payloads: payload_rows,
        deltas,
        weights,
        slots_by_identity,
    }
}

fn morph_float32x3_named_attribute<'a>(
    mesh_asset: &'a MeshAsset,
    target_index: usize,
    name: &str,
    vertex_count: usize,
) -> Option<&'a [[f32; 3]]> {
    mesh_asset
        .morph_targets
        .get(target_index)?
        .attributes
        .get(name)
        .and_then(|values| values.as_float32x3())
        .filter(|values| values.len() == vertex_count)
}

fn morph_float32x4_named_attribute<'a>(
    mesh_asset: &'a MeshAsset,
    target_index: usize,
    name: &str,
    vertex_count: usize,
) -> Option<&'a [[f32; 4]]> {
    mesh_asset
        .morph_targets
        .get(target_index)?
        .attributes
        .get(name)
        .and_then(|values| values.as_float32x4())
        .filter(|values| values.len() == vertex_count)
}

fn morph_vertex_delta_rows(
    position: Option<[f32; 3]>,
    normal: Option<[f32; 3]>,
    tangent: Option<[f32; 3]>,
    color: Option<[f32; 4]>,
) -> [GpuMorphDelta; MORPH_DELTA_ROWS_PER_VERTEX_TARGET] {
    [
        position
            .map(|delta| GpuMorphDelta::position_xyz(delta[0], delta[1], delta[2]))
            .unwrap_or_default(),
        normal
            .map(|delta| GpuMorphDelta::normal_xyz(delta[0], delta[1], delta[2]))
            .unwrap_or_default(),
        tangent
            .map(|delta| GpuMorphDelta::tangent_xyz(delta[0], delta[1], delta[2]))
            .unwrap_or_default(),
        color
            .map(|delta| GpuMorphDelta::color_rgba(delta[0], delta[1], delta[2], delta[3]))
            .unwrap_or_default(),
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::asset::{
        AssetUri, MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION,
        MESH_ATTRIBUTE_TANGENT, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset,
    };
    use crate::core::framework::render::RenderMeshTopology;
    use crate::graphics::scene::gpu_scene::GpuMorphPayload;

    #[test]
    fn morph_payload_projection_keeps_active_position_deltas_and_weights() {
        let payload =
            super::morph_payload_from_mesh_asset(&morph_test_mesh(), &[0.25, 0.0, 0.5], None)
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
        let payload =
            super::morph_payload_from_mesh_asset(&morph_test_mesh(), &[0.25, 1.0, 0.0], None)
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
                MeshAttributeValues::Float32x3(vec![
                    [1.0, 0.0, 0.0],
                    [0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ]),
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
}
