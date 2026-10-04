use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::{
    MeshAsset, MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION,
    MESH_ATTRIBUTE_TANGENT,
};
use crate::graphics::scene::gpu_scene::{
    GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GpuScene, GpuScenePreparedMorphUpload,
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

    let active_target_capacity = active_morph_target_capacity(
        mesh_asset.morph_targets.len(),
        morph_weights,
        previous_morph_weights,
    );
    let delta_capacity = active_target_capacity
        .saturating_mul(vertex_count)
        .saturating_mul(MORPH_DELTA_ROWS_PER_VERTEX_TARGET);
    let mut deltas = Vec::with_capacity(delta_capacity);
    let mut weights = Vec::with_capacity(active_target_capacity);
    let mut previous_weights = Vec::with_capacity(active_target_capacity);
    let mut target_count = 0u32;

    for target_index in 0..mesh_asset.morph_targets.len() {
        let weight = morph_weights.get(target_index).copied().unwrap_or_default();
        let previous_weight = previous_morph_weights
            .and_then(|weights| weights.get(target_index))
            .copied()
            .unwrap_or(weight);
        // 上一帧仍有权重时不能跳过该 target，否则 velocity pass 会丢失前一帧顶点位置。
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
        // 每个 target/vertex 固定按 position、normal、tangent、color 四行写入，须与 WGSL 行偏移一致。
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

fn active_morph_target_capacity(
    target_count: usize,
    morph_weights: &[f32],
    previous_morph_weights: Option<&[f32]>,
) -> usize {
    (0..target_count)
        .filter(|target_index| {
            let weight = morph_weights
                .get(*target_index)
                .copied()
                .unwrap_or_default();
            let previous_weight = previous_morph_weights
                .and_then(|weights| weights.get(*target_index))
                .copied()
                .unwrap_or(weight);
            weight.abs() > f32::EPSILON || previous_weight.abs() > f32::EPSILON
        })
        .count()
}

pub(super) fn upload_morph_payloads(
    device: &wgpu::Device,
    gpu_scene: &mut GpuScene,
    pending_draws: &mut [PendingMeshDraw],
) -> GpuScenePreparedMorphUpload {
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
    gpu_scene.prepare_morph_buffers(
        device,
        collected.payloads,
        collected.deltas,
        collected.weights,
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
        // 多个 draw 共享同一 Arc payload 时复用首个槽位，避免重复上传且保持槽索引一致。
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
        // WGSL 先读 target_count 个当前权重，再以同样长度偏移读取上一帧权重。
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
#[path = "tests/morph_payload_upload.rs"]
mod tests;

#[cfg(test)]
#[path = "morph_payload_upload/tests/capacity_tests.rs"]
mod capacity_tests;
