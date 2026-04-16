use std::collections::{BTreeMap, BTreeSet, HashMap};

use zircon_scene::{EntityId, Mobility, RenderFrameExtract};

use super::super::culling::{
    is_mesh_visible::is_mesh_visible, mesh_bounds::mesh_bounds,
    visibility_entries::visibility_entries,
};
use super::super::declarations::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBvhInstance, VisibilityContext,
    VisibilityHistoryEntry, VisibilityHistorySnapshot,
};
use super::super::planning::{
    build_bvh_update_plan::build_bvh_update_plan, build_draw_commands::build_draw_commands,
    build_hybrid_gi_plan::build_hybrid_gi_plan,
    build_instance_upload_plan::build_instance_upload_plan,
    build_particle_upload_plan::build_particle_upload_plan,
    build_virtual_geometry_plan::build_virtual_geometry_plan,
};

impl VisibilityContext {
    pub fn from_extract_with_history(
        value: &RenderFrameExtract,
        previous: Option<&VisibilityHistorySnapshot>,
    ) -> Self {
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
        let mut batches = BTreeMap::<VisibilityBatchKey, Vec<EntityId>>::new();
        let mut visible_members = BTreeMap::<VisibilityBatchKey, Vec<EntityId>>::new();
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

        let batches = batches
            .into_iter()
            .map(|(key, entities)| VisibilityBatch { key, entities })
            .collect::<Vec<_>>();
        let visible_batches = visible_members
            .into_iter()
            .map(|(key, entities)| VisibilityBatch { key, entities })
            .collect::<Vec<_>>();
        let (visible_instances, draw_commands) = build_draw_commands(&visible_batches);
        let (
            hybrid_gi_active_probes,
            hybrid_gi_update_plan,
            hybrid_gi_feedback,
            hybrid_gi_requested_probes,
        ) = build_hybrid_gi_plan(
            value.lighting.hybrid_global_illumination.as_ref(),
            &visible_entities,
            &value.view.camera,
            previous,
        );
        let (
            virtual_geometry_visible_clusters,
            virtual_geometry_page_upload_plan,
            virtual_geometry_feedback,
            virtual_geometry_requested_pages,
        ) = build_virtual_geometry_plan(
            value.geometry.virtual_geometry.as_ref(),
            &visible_entities,
            &value.view.camera,
            previous,
        );
        let history_snapshot = VisibilityHistorySnapshot {
            instances: history_entries,
            particle_emitters: value
                .particles
                .emitters
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            hybrid_gi_requested_probes,
            virtual_geometry_requested_pages,
        };
        let bvh_update_plan = build_bvh_update_plan(&history_snapshot, previous);
        let instance_upload_plan = build_instance_upload_plan(&bvh_instances, &bvh_update_plan);
        let particle_upload_plan = build_particle_upload_plan(&history_snapshot, previous);
        let gpu_instancing_candidates = visible_batches
            .iter()
            .filter(|batch| batch.key.mobility == Mobility::Dynamic && batch.entities.len() > 1)
            .cloned()
            .collect::<Vec<_>>();

        Self {
            renderable_entities: renderable_entities.into_iter().collect(),
            static_entities: static_entities.into_iter().collect(),
            dynamic_entities: dynamic_entities.into_iter().collect(),
            visible_entities: visible_entities.into_iter().collect(),
            culled_entities: culled_entities.into_iter().collect(),
            batches,
            visible_batches,
            visible_instances,
            draw_commands,
            bvh_instances,
            bvh_update_plan,
            history_snapshot,
            instance_upload_plan,
            particle_upload_plan,
            hybrid_gi_active_probes,
            hybrid_gi_update_plan,
            hybrid_gi_feedback,
            virtual_geometry_visible_clusters,
            virtual_geometry_page_upload_plan,
            virtual_geometry_feedback,
            gpu_instancing_candidates,
        }
    }
}
