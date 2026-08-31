use std::collections::{BTreeMap, HashSet};

use crate::core::framework::render::{
    PrimitiveRelevance, RenderFrameExtract, RenderMaterialAlphaMode,
};
use crate::core::framework::scene::Mobility;

use super::super::super::culling::{
    mesh_bounds::mesh_bounds, visibility_entries::visibility_mesh_indices,
};
use super::super::super::declarations::{
    VisibilityBatch, VisibilityBatchKey, VisibilityBvhInstance, VisibilityHistoryEntry,
    VisibilityRelevanceEntry,
};
use super::batching_result::BatchingResult;

pub(super) fn collect_batching_result(value: &RenderFrameExtract) -> BatchingResult {
    let mut material_alpha_modes =
        vec![RenderMaterialAlphaMode::Opaque; value.geometry.meshes.len()];
    for phase_input in &value.geometry.phase_inputs {
        if let Some(alpha_mode) = material_alpha_modes.get_mut(phase_input.mesh_index) {
            *alpha_mode = phase_input.material_alpha_mode;
        }
    }
    let mesh_indices = visibility_mesh_indices(value);
    let mut renderable_entities = HashSet::new();
    let mut static_entities = HashSet::new();
    let mut dynamic_entities = HashSet::new();
    let mut primitive_relevance = Vec::new();
    let mut batches = BTreeMap::<VisibilityBatchKey, Vec<(u64, _)>>::new();
    let mut bvh_instances = Vec::new();
    let mut history_entries = Vec::new();

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
        let bounds = mesh_bounds(mesh);
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
mod tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::scene::{EntityId, Mobility};

    use super::super::batching_result::sorted_entity_ids;

    #[test]
    fn batching_computes_bounds_once_without_intermediate_lookup() {
        let source = include_str!("collect_batching_result.rs");

        assert!(!source.contains(concat!("frustum_", "candidates")));
        assert!(!source.contains(concat!("bounds_by_", "entity")));
        assert!(!source.contains(concat!("mesh_lookup", " =")));
        assert!(!source.contains(concat!("phase_inputs_by_", "entity")));
    }

    #[test]
    fn optimization_batch_20260826n_runtime09b_batching_uses_hash_membership_sets() {
        let collect_source = include_str!("collect_batching_result.rs");
        let collect_production = collect_source
            .split("#[cfg(test)]")
            .next()
            .expect("batching collection production source");
        let result_source = include_str!("batching_result.rs");
        let result_production = result_source
            .split("#[cfg(test)]")
            .next()
            .expect("batching result production source");
        let construct_source = include_str!("construct.rs");
        let construct_production = construct_source
            .split("#[cfg(test)]")
            .next()
            .expect("visibility context construction source");

        assert!(!collect_production.contains("let mut renderable_entities = BTreeSet::new()"));
        assert_eq!(collect_production.matches("HashSet::new()").count(), 3);
        assert_eq!(result_production.matches("HashSet<EntityId>").count(), 4);
        assert!(result_production.contains("fn sorted_entity_ids"));
        assert_eq!(
            construct_production.matches("sorted_entity_ids(").count(),
            3
        );
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826n_runtime09b_batching_hash_entity_set_performance_evidence() {
        fn legacy_entity_sets(
            inputs: &[(EntityId, Mobility)],
        ) -> (Vec<EntityId>, Vec<EntityId>, Vec<EntityId>) {
            let mut renderable = BTreeSet::new();
            let mut static_entities = BTreeSet::new();
            let mut dynamic_entities = BTreeSet::new();
            for (entity, mobility) in inputs {
                renderable.insert(*entity);
                match mobility {
                    Mobility::Static => {
                        static_entities.insert(*entity);
                    }
                    Mobility::Dynamic => {
                        dynamic_entities.insert(*entity);
                    }
                }
            }
            (
                renderable.into_iter().collect(),
                static_entities.into_iter().collect(),
                dynamic_entities.into_iter().collect(),
            )
        }

        fn hash_entity_sets(
            inputs: &[(EntityId, Mobility)],
        ) -> (Vec<EntityId>, Vec<EntityId>, Vec<EntityId>) {
            let mut renderable = HashSet::new();
            let mut static_entities = HashSet::new();
            let mut dynamic_entities = HashSet::new();
            for (entity, mobility) in inputs {
                renderable.insert(*entity);
                match mobility {
                    Mobility::Static => {
                        static_entities.insert(*entity);
                    }
                    Mobility::Dynamic => {
                        dynamic_entities.insert(*entity);
                    }
                }
            }
            (
                sorted_entity_ids(renderable),
                sorted_entity_ids(static_entities),
                sorted_entity_ids(dynamic_entities),
            )
        }

        let inputs = (0..100_000_u64)
            .map(|entity| {
                let mobility = if entity % 2 == 0 {
                    Mobility::Static
                } else {
                    Mobility::Dynamic
                };
                (entity, mobility)
            })
            .collect::<Vec<_>>();
        assert_eq!(legacy_entity_sets(&inputs), hash_entity_sets(&inputs));

        let mut legacy_samples = Vec::with_capacity(17);
        let mut hash_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            black_box(legacy_entity_sets(black_box(&inputs)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(hash_entity_sets(black_box(&inputs)));
            hash_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        hash_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let hash_p95 = hash_samples[16];
        println!(
            "RUNTIME09B_FRAME_BATCHING_HASH_ENTITY_SETS_BENCH_V1 mesh_entities={} output_entity_values={} legacy_p95_ns={} hash_p95_ns={} legacy_tree_admissions={} hash_admissions={} target_ratio_bp=6000",
            inputs.len(),
            inputs.len() * 2,
            legacy_p95,
            hash_p95,
            inputs.len() * 2,
            inputs.len() * 2,
        );
        assert!(
            hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "frame batching hash entity-set P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }
}
