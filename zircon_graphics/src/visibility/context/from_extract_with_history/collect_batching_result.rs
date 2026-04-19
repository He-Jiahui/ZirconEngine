use std::collections::{BTreeMap, BTreeSet, HashMap};

use zircon_framework::render::RenderFrameExtract;
use zircon_framework::scene::Mobility;

use super::super::super::culling::{
    is_mesh_visible::is_mesh_visible, mesh_bounds::mesh_bounds,
    visibility_entries::visibility_entries,
};
use super::super::super::declarations::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBvhInstance, VisibilityHistoryEntry,
};
use super::batching_result::BatchingResult;

pub(super) fn collect_batching_result(value: &RenderFrameExtract) -> BatchingResult {
    let mesh_lookup = value
        .geometry
        .meshes
        .iter()
        .map(|mesh| (mesh.node_id, mesh))
        .collect::<HashMap<_, _>>();
    let mut entries_by_entity = BTreeMap::new();
    for entry in visibility_entries(value) {
        entries_by_entity.insert(entry.entity, entry);
    }

    let mut renderable_entities = BTreeSet::new();
    let mut static_entities = BTreeSet::new();
    let mut dynamic_entities = BTreeSet::new();
    let mut visible_entities = BTreeSet::new();
    let mut culled_entities = BTreeSet::new();
    let mut batches = BTreeMap::<VisibilityBatchKey, Vec<_>>::new();
    let mut visible_members = BTreeMap::<VisibilityBatchKey, Vec<_>>::new();
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
        let key = VisibilityBatchKey {
            render_layer_mask: entry.render_layer_mask,
            material_id: mesh.material.id(),
            model_id: mesh.model.id(),
            mobility: entry.mobility,
        };
        let bounds = mesh_bounds(mesh);
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
        if is_mesh_visible(mesh, &value.view.camera) {
            visible_entities.insert(entity);
            visible_members.entry(key).or_default().push(entity);
        } else {
            culled_entities.insert(entity);
        }
    }

    BatchingResult {
        renderable_entities,
        static_entities,
        dynamic_entities,
        visible_entities,
        culled_entities,
        batches: batches
            .into_iter()
            .map(|(key, entities)| VisibilityBatch { key, entities })
            .collect(),
        visible_batches: visible_members
            .into_iter()
            .map(|(key, entities)| VisibilityBatch { key, entities })
            .collect(),
        bvh_instances,
        history_entries,
    }
}

