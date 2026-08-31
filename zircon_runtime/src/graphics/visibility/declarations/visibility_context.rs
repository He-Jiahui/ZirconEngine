use std::collections::BTreeSet;

use crate::core::framework::scene::EntityId;

use super::super::view_context::FrameVisibility;
use super::super::{VisibilityStaticIndex, VisibilityStaticIndexReport};
use super::{
    visibility_batch::VisibilityBatch, visibility_bvh_instance::VisibilityBvhInstance,
    visibility_bvh_update_plan::VisibilityBvhUpdatePlan,
    visibility_draw_command::VisibilityDrawCommand,
    visibility_history_snapshot::VisibilityHistorySnapshot,
    visibility_hybrid_gi_feedback::VisibilityHybridGiFeedback,
    visibility_hybrid_gi_probe::VisibilityHybridGiProbe,
    visibility_hybrid_gi_update_plan::VisibilityHybridGiUpdatePlan,
    visibility_instance_upload_plan::VisibilityInstanceUploadPlan,
    visibility_particle_upload_plan::VisibilityParticleUploadPlan,
    visibility_relevance_entry::VisibilityRelevanceEntry,
    visibility_virtual_geometry_cluster::VisibilityVirtualGeometryCluster,
    visibility_virtual_geometry_draw_segment::VisibilityVirtualGeometryDrawSegment,
    visibility_virtual_geometry_feedback::VisibilityVirtualGeometryFeedback,
    visibility_virtual_geometry_page_upload_plan::VisibilityVirtualGeometryPageUploadPlan,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibilityContext {
    pub frame_visibility: FrameVisibility,
    pub renderable_entities: Vec<EntityId>,
    pub static_entities: Vec<EntityId>,
    pub dynamic_entities: Vec<EntityId>,
    pub primitive_relevance: Vec<VisibilityRelevanceEntry>,
    pub batches: Vec<VisibilityBatch>,
    /// Stable render-instance keys selected for the main view draw commands.
    pub visible_instances: Vec<u64>,
    pub draw_commands: Vec<VisibilityDrawCommand>,
    pub bvh_instances: Vec<VisibilityBvhInstance>,
    pub bvh_update_plan: VisibilityBvhUpdatePlan,
    pub static_index_report: VisibilityStaticIndexReport,
    pub history_snapshot: VisibilityHistorySnapshot,
    pub instance_upload_plan: VisibilityInstanceUploadPlan,
    pub particle_upload_plan: VisibilityParticleUploadPlan,
    pub hybrid_gi_active_probes: Vec<VisibilityHybridGiProbe>,
    pub hybrid_gi_update_plan: VisibilityHybridGiUpdatePlan,
    pub hybrid_gi_feedback: VisibilityHybridGiFeedback,
    pub virtual_geometry_visible_clusters: Vec<VisibilityVirtualGeometryCluster>,
    pub virtual_geometry_draw_segments: Vec<VisibilityVirtualGeometryDrawSegment>,
    pub virtual_geometry_page_upload_plan: VisibilityVirtualGeometryPageUploadPlan,
    pub virtual_geometry_feedback: VisibilityVirtualGeometryFeedback,
    pub gpu_instancing_candidates: Vec<VisibilityBatch>,
    pub(crate) static_index: VisibilityStaticIndex,
    pub(crate) dynamic_index: VisibilityStaticIndex,
}

impl VisibilityContext {
    pub(crate) fn static_index(&self) -> &VisibilityStaticIndex {
        &self.static_index
    }

    pub(crate) fn dynamic_index(&self) -> &VisibilityStaticIndex {
        &self.dynamic_index
    }

    /// Main-view visibility is derived from `FrameVisibility` so there is only one
    /// authoritative per-view visibility store in the context.
    pub fn main_view_visible_entities(&self) -> Vec<EntityId> {
        self.main_view_visible_entity_set().into_iter().collect()
    }

    pub fn main_view_visible_entity_set(&self) -> BTreeSet<EntityId> {
        self.frame_visibility.main_view_visible_entity_set()
    }

    pub fn main_view_visible_stable_instance_keys(&self) -> Vec<u64> {
        self.frame_visibility
            .main_view_visible_stable_instance_key_set()
            .into_iter()
            .collect()
    }

    pub fn main_view_culled_entities(&self) -> Vec<EntityId> {
        let visible_entities = self.main_view_visible_entity_set();
        let mut culled_entities = Vec::with_capacity(self.renderable_entities.len());
        culled_entities.extend(
            self.renderable_entities
                .iter()
                .copied()
                .filter(|entity| !visible_entities.contains(entity)),
        );
        culled_entities
    }

    pub fn main_view_culled_stable_instance_keys(&self) -> Vec<u64> {
        let visible_stable_instance_keys = self
            .frame_visibility
            .main_view_visible_stable_instance_key_set();
        let mut culled_stable_instance_keys = Vec::with_capacity(self.bvh_instances.len());
        culled_stable_instance_keys.extend(
            self.bvh_instances
                .iter()
                .map(|instance| instance.stable_instance_key)
                .filter(|stable_instance_key| {
                    !visible_stable_instance_keys.contains(stable_instance_key)
                }),
        );
        culled_stable_instance_keys
    }

    pub fn main_view_visible_batches(&self) -> Vec<VisibilityBatch> {
        Self::visible_batches_for_stable_instance_keys(
            &self.batches,
            &self
                .frame_visibility
                .main_view_visible_stable_instance_key_set(),
        )
    }

    pub(crate) fn visible_batches_for_stable_instance_keys(
        batches: &[VisibilityBatch],
        visible_stable_instance_keys: &BTreeSet<u64>,
    ) -> Vec<VisibilityBatch> {
        let mut visible_batches = Vec::with_capacity(batches.len());
        for batch in batches {
            let member_capacity = batch.stable_instance_keys.len().min(batch.entities.len());
            let mut stable_instance_keys = Vec::with_capacity(member_capacity);
            let mut entities = Vec::with_capacity(member_capacity);
            for (stable_instance_key, entity) in
                batch.stable_instance_keys.iter().zip(batch.entities.iter())
            {
                if visible_stable_instance_keys.contains(stable_instance_key) {
                    stable_instance_keys.push(*stable_instance_key);
                    entities.push(*entity);
                }
            }
            if !stable_instance_keys.is_empty() {
                visible_batches.push(VisibilityBatch {
                    key: batch.key.clone(),
                    stable_instance_keys,
                    entities,
                });
            }
        }
        visible_batches
    }
}

#[cfg(test)]
mod optimization_batch_20260830cg_runtime_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::render::RenderLayerSet;
    use crate::core::framework::scene::Mobility;
    use crate::core::resource::ResourceId;

    use super::super::visibility_batch_key::VisibilityBatchKey;
    use super::*;

    const SAMPLE_PAIRS: usize = 17;
    const BATCHES_PER_SAMPLE: usize = 64;
    const MEMBERS_PER_BATCH: usize = 128;

    #[test]
    fn visible_batch_projection_preserves_order_and_zip_truncation() {
        let batches = vec![
            batch("first", vec![1, 2, 3], vec![10, 20]),
            batch("empty", vec![4], vec![40]),
            batch("second", vec![2, 5], vec![22, 50]),
        ];
        let visible = BTreeSet::from([2, 3]);

        let projected =
            VisibilityContext::visible_batches_for_stable_instance_keys(&batches, &visible);

        assert_eq!(projected.len(), 2);
        assert_eq!(projected[0].key.material_id, batches[0].key.material_id);
        assert_eq!(projected[0].stable_instance_keys, vec![2]);
        assert_eq!(projected[0].entities, vec![20]);
        assert_eq!(projected[1].key.material_id, batches[2].key.material_id);
        assert_eq!(projected[1].stable_instance_keys, vec![2]);
        assert_eq!(projected[1].entities, vec![22]);
    }

    #[test]
    fn visibility_projection_reserves_known_input_bounds() {
        let source = include_str!("visibility_context.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("visibility context implementation");

        assert!(implementation.contains("Vec::with_capacity(self.renderable_entities.len())"));
        assert!(implementation.contains("Vec::with_capacity(self.bvh_instances.len())"));
        assert!(implementation.contains("Vec::with_capacity(batches.len())"));
        assert!(
            implementation.contains("batch.stable_instance_keys.len().min(batch.entities.len())")
        );
        assert!(!implementation.contains("members.into_iter().unzip()"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cg_runtime_visibility_projection_capacity_p95() {
        let batches = benchmark_batches();
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(&batches, false));
                optimized.push(measure(&batches, true));
            } else {
                optimized.push(measure(&batches, true));
                legacy.push(measure(&batches, false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME385_VISIBILITY_PROJECTION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} batches_per_sample={BATCHES_PER_SAMPLE} members_per_batch={MEMBERS_PER_BATCH} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn batch(label: &str, stable_instance_keys: Vec<u64>, entities: Vec<u64>) -> VisibilityBatch {
        VisibilityBatch {
            key: VisibilityBatchKey {
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                material_id: ResourceId::from_stable_label(&format!("tests/{label}/material")),
                model_id: ResourceId::from_stable_label(&format!("tests/{label}/model")),
                mobility: Mobility::Dynamic,
            },
            stable_instance_keys,
            entities,
        }
    }

    fn benchmark_batches() -> Vec<Vec<u64>> {
        (0..BATCHES_PER_SAMPLE)
            .map(|batch| {
                (0..MEMBERS_PER_BATCH)
                    .map(|member| (batch * MEMBERS_PER_BATCH + member) as u64)
                    .collect()
            })
            .collect()
    }

    fn measure(batches: &[Vec<u64>], use_capacity: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..32 {
            let mut projected = if use_capacity {
                Vec::with_capacity(batches.len())
            } else {
                Vec::new()
            };
            for batch in black_box(batches) {
                if use_capacity {
                    let mut keys = Vec::with_capacity(batch.len());
                    let mut entities = Vec::with_capacity(batch.len());
                    for (entity, key) in batch.iter().enumerate() {
                        if key % 3 != 0 {
                            keys.push(*key);
                            entities.push(entity as u64);
                        }
                    }
                    projected.push((keys, entities));
                } else {
                    let members = batch
                        .iter()
                        .enumerate()
                        .filter(|(_, key)| *key % 3 != 0)
                        .map(|(entity, key)| (*key, entity as u64))
                        .collect::<Vec<_>>();
                    projected.push(members.into_iter().unzip());
                }
            }
            checksum ^= projected.iter().map(|(keys, _)| keys.len()).sum::<usize>();
            black_box(projected);
        }
        black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], p: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
