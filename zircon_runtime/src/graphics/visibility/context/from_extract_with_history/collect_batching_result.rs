use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::core::framework::render::{
    PrimitiveRelevance, RenderFrameExtract, RenderMaterialAlphaMode,
};
use crate::core::framework::scene::Mobility;

use super::super::super::culling::{
    mesh_bounds::mesh_bounds,
    parallel_frustum::{mesh_frustum_visibility, MeshFrustumCandidate},
    visibility_entries::visibility_entries,
};
use super::super::super::declarations::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBvhInstance, VisibilityHistoryEntry,
    VisibilityRelevanceEntry,
};
use super::super::super::view_context::FrameVisibility;
use super::batching_result::BatchingResult;

pub(super) fn collect_batching_result(value: &RenderFrameExtract) -> BatchingResult {
    let mesh_lookup = value
        .geometry
        .meshes
        .iter()
        .map(|mesh| (mesh.node_id, mesh))
        .collect::<HashMap<_, _>>();
    let phase_inputs_by_entity = value
        .geometry
        .phase_inputs
        .iter()
        .map(|input| (input.entity, input))
        .collect::<HashMap<_, _>>();
    let mut entries_by_entity = BTreeMap::new();
    for entry in visibility_entries(value) {
        entries_by_entity.insert(entry.entity, entry);
    }
    let frustum_candidates = entries_by_entity
        .keys()
        .filter_map(|entity| {
            let mesh = mesh_lookup.get(entity)?;
            Some(MeshFrustumCandidate {
                entity: *entity,
                bounds: mesh_bounds(mesh),
            })
        })
        .collect::<Vec<_>>();
    let bounds_by_entity = frustum_candidates
        .iter()
        .map(|candidate| (candidate.entity, candidate.bounds))
        .collect::<HashMap<_, _>>();
    let frustum_visibility_by_entity =
        mesh_frustum_visibility(&frustum_candidates, &value.view.camera)
            .into_iter()
            .map(|entry| (entry.entity, entry.visible))
            .collect::<HashMap<_, _>>();

    let mut renderable_entities = BTreeSet::new();
    let mut static_entities = BTreeSet::new();
    let mut dynamic_entities = BTreeSet::new();
    let mut visible_entities = BTreeSet::new();
    let mut culled_entities = BTreeSet::new();
    let mut primitive_relevance = Vec::new();
    let mut batches = BTreeMap::<VisibilityBatchKey, Vec<_>>::new();
    let mut bvh_instances = Vec::new();
    let mut history_entries = Vec::new();

    for (entity, entry) in entries_by_entity {
        let Some(mesh) = mesh_lookup.get(&entity) else {
            continue;
        };
        renderable_entities.insert(entity);
        match entry.mobility {
            Mobility::Static => {
                static_entities.insert(entity);
            }
            Mobility::Dynamic => {
                dynamic_entities.insert(entity);
            }
        }
        let material_alpha_mode = phase_inputs_by_entity
            .get(&entity)
            .map(|input| input.material_alpha_mode)
            .unwrap_or(RenderMaterialAlphaMode::Opaque);
        let relevance = PrimitiveRelevance::for_mesh_view(
            &value.view.camera.render_layers,
            value.view.core_pipeline,
            entry.render_layer_mask,
            entry.mobility,
            material_alpha_mode,
        );
        primitive_relevance.push(VisibilityRelevanceEntry { entity, relevance });

        let key = VisibilityBatchKey {
            render_layer_mask: entry.render_layer_mask,
            material_id: mesh.material.id(),
            model_id: mesh.model.id(),
            mobility: entry.mobility,
        };
        let bounds = bounds_by_entity
            .get(&entity)
            .copied()
            .unwrap_or_else(|| mesh_bounds(mesh));
        bvh_instances.push(VisibilityBvhInstance {
            entity,
            key,
            bounds,
        });
        history_entries.push(VisibilityHistoryEntry {
            entity,
            key,
            bounds,
        });
        batches.entry(key).or_default().push(entity);
        if relevance.main_view()
            && frustum_visibility_by_entity
                .get(&entity)
                .copied()
                .unwrap_or(false)
        {
            visible_entities.insert(entity);
        } else {
            culled_entities.insert(entity);
        }
    }

    let frame_visibility = FrameVisibility::from_frame_views(
        &value.view.camera,
        &value.lighting.directional_lights,
        &bvh_instances,
        &primitive_relevance,
        &visible_entities,
    );

    BatchingResult {
        frame_visibility,
        renderable_entities,
        static_entities,
        dynamic_entities,
        visible_entities,
        culled_entities,
        primitive_relevance,
        batches: batches
            .into_iter()
            .map(|(key, entities)| VisibilityBatch { key, entities })
            .collect(),
        bvh_instances,
        history_entries,
    }
}
