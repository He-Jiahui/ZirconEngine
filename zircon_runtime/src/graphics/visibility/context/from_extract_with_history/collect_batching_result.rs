use std::collections::BTreeMap;

use crate::core::framework::render::{
    PrimitiveRelevance, RenderFrameExtract, RenderMaterialAlphaMode, RenderMeshBounds,
};
use crate::core::framework::scene::Mobility;

use super::super::super::culling::{
    mesh_bounds::mesh_bounds_from_local, visibility_entries::visibility_mesh_indices,
};
use super::super::super::declarations::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBvhInstance, VisibilityHistoryEntry,
    VisibilityRelevanceEntry,
};
use super::batching_result::{BatchingResult, EntitySet};

pub(super) fn collect_batching_result(value: &RenderFrameExtract) -> BatchingResult {
    collect_batching_result_with_prepared_bounds(value, None)
}

pub(super) fn collect_batching_result_with_prepared_bounds(
    value: &RenderFrameExtract,
    prepared_local_bounds: Option<&[Option<RenderMeshBounds>]>,
) -> BatchingResult {
    let mut material_alpha_modes =
        vec![RenderMaterialAlphaMode::Opaque; value.geometry.meshes.len()];
    for phase_input in &value.geometry.phase_inputs {
        if let Some(alpha_mode) = material_alpha_modes.get_mut(phase_input.mesh_index) {
            *alpha_mode = phase_input.material_alpha_mode;
        }
    }
    let mesh_indices = visibility_mesh_indices(value);
    let entity_capacity = mesh_indices.len();
    let mut renderable_entities = EntitySet::with_capacity(entity_capacity);
    let mut static_entities = EntitySet::with_capacity(entity_capacity);
    let mut dynamic_entities = EntitySet::with_capacity(entity_capacity);
    let mut primitive_relevance = Vec::with_capacity(entity_capacity);
    let mut batches = BTreeMap::<VisibilityBatchKey, Vec<(u64, _)>>::new();
    let mut bvh_instances = Vec::with_capacity(entity_capacity);
    let mut history_entries = Vec::with_capacity(entity_capacity);

    for mesh_index in mesh_indices {
        let mesh = &value.geometry.meshes[mesh_index];
        let entity = mesh.node_id;
        let mobility = mesh.mobility;
        renderable_entities.insert(entity);
        match mobility {
            Mobility::Static => {
                static_entities.insert(entity);
            }
            Mobility::Dynamic => {
                dynamic_entities.insert(entity);
            }
        }
        let material_alpha_mode = material_alpha_modes[mesh_index];
        let relevance = PrimitiveRelevance::for_mesh_view(
            value.view.selected_camera_layers(),
            value.view.core_pipeline,
            &mesh.common.layer_mask,
            mobility,
            material_alpha_mode,
        );
        primitive_relevance.push(VisibilityRelevanceEntry {
            entity,
            stable_instance_key: mesh.stable_instance_key,
            relevance,
        });

        let key = VisibilityBatchKey {
            render_layer_mask: mesh.common.layer_mask.clone(),
            material_id: mesh.material.id(),
            model_id: mesh.model.id(),
            mobility,
        };
        let local_bounds = prepared_local_bounds
            .and_then(|bounds| bounds.get(mesh_index))
            .copied()
            .flatten();
        let bounds = mesh_bounds_from_local(mesh, local_bounds);
        bvh_instances.push(VisibilityBvhInstance {
            entity,
            stable_instance_key: mesh.stable_instance_key,
            key: key.clone(),
            bounds,
        });
        history_entries.push(VisibilityHistoryEntry {
            entity,
            stable_instance_key: mesh.stable_instance_key,
            key: key.clone(),
            bounds,
        });
        batches
            .entry(key)
            .or_default()
            .push((mesh.stable_instance_key, entity));
    }

    BatchingResult {
        renderable_entities,
        static_entities,
        dynamic_entities,
        primitive_relevance,
        batches: batches
            .into_iter()
            .map(|(key, members)| {
                let (stable_instance_keys, entities) = members.into_iter().unzip();
                VisibilityBatch {
                    key,
                    stable_instance_keys,
                    entities,
                }
            })
            .collect(),
        bvh_instances,
        history_entries,
    }
}

#[cfg(test)]
#[path = "tests/collect_batching_result.rs"]
mod tests;

#[cfg(test)]
#[path = "collect_batching_result/tests/optimization_batch_io_runtime625_tests.rs"]
mod optimization_batch_io_runtime625_tests;
