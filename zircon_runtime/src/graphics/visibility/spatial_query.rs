use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    RenderSpatialBounds, RenderSpatialRay, RenderVisibleSpatialQuery,
    RenderVisibleSpatialQueryResult, RenderVisibleSpatialQueryStats,
};
use crate::core::framework::scene::EntityId;
use crate::core::math::Vec3;

use super::{VisibilityBounds, VisibilityContext, VisibilityStaticIndex};

const MAX_VISIBLE_SPATIAL_QUERY_CELLS_PER_INDEX: usize = 4_096;

#[derive(Clone, Copy, Debug)]
struct VisibleSpatialEntry {
    entity: EntityId,
    bounds: VisibilityBounds,
}

/// Renderer-private acceleration data behind the public immutable query contract.
pub(crate) struct VisibleSpatialQuery {
    static_index: VisibilityStaticIndex,
    dynamic_index: VisibilityStaticIndex,
    visible_entries: HashMap<u64, VisibleSpatialEntry>,
}

impl VisibleSpatialQuery {
    pub(crate) fn from_context(context: &VisibilityContext) -> Self {
        let visible_stable_instance_keys = context
            .frame_visibility
            .main_view_visible_stable_instance_key_set();
        let mut visible_entries = HashMap::with_capacity(visible_stable_instance_keys.len());
        for instance in &context.bvh_instances {
            if visible_stable_instance_keys.contains(&instance.stable_instance_key) {
                visible_entries.insert(
                    instance.stable_instance_key,
                    VisibleSpatialEntry {
                        entity: instance.entity,
                        bounds: instance.bounds,
                    },
                );
            }
        }

        Self {
            static_index: context.static_index().clone(),
            dynamic_index: context.dynamic_index().clone(),
            visible_entries,
        }
    }
}

impl RenderVisibleSpatialQuery for VisibleSpatialQuery {
    fn query_bounds(&self, bounds: RenderSpatialBounds) -> RenderVisibleSpatialQueryResult {
        if !bounds.radius.is_finite() || bounds.radius < 0.0 || !is_finite_vec3(bounds.center) {
            return RenderVisibleSpatialQueryResult::default();
        }

        let bounds = VisibilityBounds {
            center: bounds.center,
            radius: bounds.radius,
        };
        let static_query = self
            .static_index
            .query_bounds_with_stats_limited(bounds, MAX_VISIBLE_SPATIAL_QUERY_CELLS_PER_INDEX);
        let dynamic_query = self
            .dynamic_index
            .query_bounds_with_stats_limited(bounds, MAX_VISIBLE_SPATIAL_QUERY_CELLS_PER_INDEX);
        // An oversized finite tool query must remain correct without materializing a cubic cell
        // range. The fallback visits only already-visible entries, not index cells.
        let (candidate_keys, visited_node_count) = match (static_query, dynamic_query) {
            (Some(static_query), Some(dynamic_query)) => (
                static_query
                    .stable_instance_keys
                    .into_iter()
                    .chain(dynamic_query.stable_instance_keys)
                    .collect::<HashSet<_>>(),
                static_query
                    .visited_node_count
                    .saturating_add(dynamic_query.visited_node_count),
            ),
            _ => (
                self.visible_entries.keys().copied().collect::<HashSet<_>>(),
                self.visible_entries.len(),
            ),
        };
        let entities = matching_entities(&candidate_keys, &self.visible_entries, |entry_bounds| {
            bounds_overlap(entry_bounds, bounds)
        });

        RenderVisibleSpatialQueryResult {
            stats: RenderVisibleSpatialQueryStats {
                visited_node_count,
                candidate_count: candidate_keys.len(),
                hit_count: entities.len(),
            },
            entities,
        }
    }

    fn query_ray(&self, ray: RenderSpatialRay) -> RenderVisibleSpatialQueryResult {
        if !ray.max_distance.is_finite()
            || ray.max_distance < 0.0
            || !is_finite_vec3(ray.origin)
            || !is_finite_vec3(ray.direction)
        {
            return RenderVisibleSpatialQueryResult::default();
        }
        let direction_length = ray.direction.length();
        if !direction_length.is_finite() || direction_length <= f32::EPSILON {
            return RenderVisibleSpatialQueryResult::default();
        }
        let direction = ray.direction / direction_length;
        let static_query = self.static_index.query_ray_with_stats_limited(
            ray.origin,
            direction,
            ray.max_distance,
            MAX_VISIBLE_SPATIAL_QUERY_CELLS_PER_INDEX,
        );
        let dynamic_query = self.dynamic_index.query_ray_with_stats_limited(
            ray.origin,
            direction,
            ray.max_distance,
            MAX_VISIBLE_SPATIAL_QUERY_CELLS_PER_INDEX,
        );
        let (candidate_keys, visited_node_count) = match (static_query, dynamic_query) {
            (Some(static_query), Some(dynamic_query)) => (
                static_query
                    .stable_instance_keys
                    .into_iter()
                    .chain(dynamic_query.stable_instance_keys)
                    .collect::<HashSet<_>>(),
                static_query
                    .visited_node_count
                    .saturating_add(dynamic_query.visited_node_count),
            ),
            _ => (
                self.visible_entries.keys().copied().collect::<HashSet<_>>(),
                self.visible_entries.len(),
            ),
        };
        let entities = matching_entities(&candidate_keys, &self.visible_entries, |entry_bounds| {
            ray_intersects_bounds(entry_bounds, ray.origin, direction, ray.max_distance)
        });

        RenderVisibleSpatialQueryResult {
            stats: RenderVisibleSpatialQueryStats {
                visited_node_count,
                candidate_count: candidate_keys.len(),
                hit_count: entities.len(),
            },
            entities,
        }
    }
}

fn matching_entities(
    candidate_keys: &HashSet<u64>,
    visible_entries: &HashMap<u64, VisibleSpatialEntry>,
    mut matches: impl FnMut(VisibilityBounds) -> bool,
) -> Vec<EntityId> {
    let mut entities = candidate_keys
        .iter()
        .filter_map(|stable_instance_key| visible_entries.get(stable_instance_key))
        .filter(|entry| matches(entry.bounds))
        .map(|entry| entry.entity)
        .collect::<Vec<_>>();
    entities.sort_unstable();
    entities.dedup();
    entities
}

fn bounds_overlap(left: VisibilityBounds, right: VisibilityBounds) -> bool {
    let radius = left.radius.max(0.0) + right.radius.max(0.0);
    left.center.distance_squared(right.center) <= radius * radius
}

fn ray_intersects_bounds(
    bounds: VisibilityBounds,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> bool {
    let offset = bounds.center - origin;
    let nearest_distance = offset.dot(direction).clamp(0.0, max_distance);
    let nearest_point = origin + direction * nearest_distance;
    bounds.center.distance_squared(nearest_point) <= bounds.radius.max(0.0) * bounds.radius.max(0.0)
}

fn is_finite_vec3(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::render::{RenderLayerSet, RenderSpatialBounds, RenderSpatialRay};
    use crate::core::framework::scene::Mobility;
    use crate::core::resource::ResourceId;
    use crate::graphics::{
        ViewVisibilityContext, VisibilityBatchKey, VisibilityBvhInstance, VisibilityContext,
        VisibilityViewKey,
    };

    use super::*;

    #[test]
    fn visible_spatial_query_keeps_static_and_dynamic_hits_sorted_and_deduplicated() {
        let static_instance = instance(4, 4 << 16, Mobility::Static, Vec3::ZERO);
        let dynamic_instance = instance(2, 2 << 16, Mobility::Dynamic, Vec3::new(1.0, 0.0, 0.0));
        let sibling_primitive =
            instance(4, (4 << 16) | 1, Mobility::Static, Vec3::new(0.5, 0.0, 0.0));
        let hidden = instance(7, 7 << 16, Mobility::Dynamic, Vec3::ZERO);
        let mut context = VisibilityContext::default();
        context.bvh_instances = vec![
            static_instance.clone(),
            dynamic_instance.clone(),
            sibling_primitive.clone(),
            hidden,
        ];
        context
            .static_index
            .rebuild(&[static_instance, sibling_primitive]);
        context.dynamic_index.rebuild(&[dynamic_instance]);
        context.frame_visibility.stable_instance_keys =
            vec![4 << 16, 2 << 16, (4 << 16) | 1, 7 << 16];
        context.frame_visibility.views = vec![ViewVisibilityContext {
            view: VisibilityViewKey::MainCamera,
            visible: vec![0, 1, 2],
            ..Default::default()
        }];

        let result = VisibleSpatialQuery::from_context(&context)
            .query_bounds(RenderSpatialBounds::new(Vec3::ZERO, 2.0));

        assert_eq!(result.entities, vec![2, 4]);
        assert_eq!(result.stats.hit_count, 2);
        assert_eq!(result.stats.candidate_count, 3);
        assert!(result.stats.visited_node_count > 0);
    }

    #[test]
    fn visible_spatial_query_rejects_non_finite_input_without_visiting_indexes() {
        let result = VisibleSpatialQuery::from_context(&VisibilityContext::default())
            .query_bounds(RenderSpatialBounds::new(Vec3::splat(f32::NAN), 1.0));

        assert_eq!(result, RenderVisibleSpatialQueryResult::default());
    }

    #[test]
    fn visible_spatial_query_uses_linear_fallback_for_extreme_finite_bounds() {
        let context = dynamic_context(2);

        let result = VisibleSpatialQuery::from_context(&context)
            .query_bounds(RenderSpatialBounds::new(Vec3::ZERO, f32::MAX));

        assert_eq!(result.entities, vec![1, 2]);
        assert_eq!(result.stats.visited_node_count, 2);
        assert_eq!(result.stats.candidate_count, 2);
        assert_eq!(result.stats.hit_count, 2);
    }

    #[test]
    fn visible_spatial_query_candidate_cost_follows_intersected_cells_at_scale() {
        for instance_count in [1_u64, 1_000, 10_000] {
            let context = dynamic_context(instance_count);
            let result = VisibleSpatialQuery::from_context(&context)
                .query_bounds(RenderSpatialBounds::new(Vec3::ZERO, 1.0));

            assert_eq!(result.entities, vec![1], "instance_count={instance_count}");
            assert_eq!(
                result.stats.candidate_count, 1,
                "instance_count={instance_count}"
            );
            assert!(
                result.stats.visited_node_count <= 16,
                "instance_count={instance_count}, stats={:?}",
                result.stats
            );
        }
    }

    #[test]
    fn visible_spatial_ray_query_visits_only_crossed_cells_at_scale() {
        for instance_count in [1_u64, 1_000, 10_000] {
            let context = dynamic_context(instance_count);
            let result = VisibleSpatialQuery::from_context(&context).query_ray(
                RenderSpatialRay::new(Vec3::new(0.0, 0.0, 10.0), Vec3::NEG_Z, 100.0),
            );

            assert_eq!(result.entities, vec![1], "instance_count={instance_count}");
            assert_eq!(
                result.stats.candidate_count, 1,
                "instance_count={instance_count}"
            );
            assert!(
                result.stats.visited_node_count <= 16,
                "instance_count={instance_count}, stats={:?}",
                result.stats
            );
        }
    }

    #[test]
    fn optimization_batch_20260826k_runtime09b_hash_dedup_preserves_sorted_fallback_hits() {
        let context = dynamic_context(256);

        let result = VisibleSpatialQuery::from_context(&context)
            .query_bounds(RenderSpatialBounds::new(Vec3::ZERO, f32::MAX));

        assert_eq!(result.entities, (1..=256).collect::<Vec<_>>());
        assert_eq!(result.stats.candidate_count, 256);
        assert_eq!(result.stats.hit_count, 256);
    }

    #[test]
    fn optimization_batch_20260826k_runtime09b_spatial_query_uses_hash_then_vec_dedup() {
        let source = include_str!("spatial_query.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("visible spatial query production source");

        assert!(!production.contains("BTreeMap"));
        assert!(production.contains("visible_entries: HashMap"));
        assert!(production.contains("collect::<HashSet<_>>()"));
        assert!(production.contains("fn matching_entities"));
        assert!(production.contains("entities.sort_unstable()"));
        assert!(production.contains("entities.dedup()"));
    }

    #[test]
    fn optimization_batch_gz_runtime581_hash_lookup_preserves_sorted_hits() {
        let context = dynamic_context(512);
        let result = VisibleSpatialQuery::from_context(&context)
            .query_bounds(RenderSpatialBounds::new(Vec3::ZERO, f32::MAX));

        assert_eq!(result.entities, (1..=512).collect::<Vec<_>>());
        assert_eq!(result.stats.candidate_count, 512);
        assert_eq!(result.stats.hit_count, 512);
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_gz_runtime581_hash_lookup_performance_evidence() {
        fn legacy_matching_entities(
            candidate_keys: &HashSet<u64>,
            visible_entries: &BTreeMap<u64, VisibleSpatialEntry>,
        ) -> Vec<EntityId> {
            let mut entities = candidate_keys
                .iter()
                .filter_map(|key| visible_entries.get(key))
                .map(|entry| entry.entity)
                .collect::<Vec<_>>();
            entities.sort_unstable();
            entities.dedup();
            entities
        }

        let visible_entries = (0..32_768_u64)
            .map(|key| {
                (
                    key,
                    VisibleSpatialEntry {
                        entity: key / 2,
                        bounds: VisibilityBounds {
                            center: Vec3::new(key as f32, 0.0, 0.0),
                            radius: 1.0,
                        },
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let hash_entries = visible_entries
            .iter()
            .map(|(key, entry)| (*key, *entry))
            .collect::<HashMap<_, _>>();
        let candidate_keys = (0..32_768_u64).rev().collect::<HashSet<_>>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut hash_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            black_box(legacy_matching_entities(
                black_box(&candidate_keys),
                black_box(&visible_entries),
            ));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(matching_entities(
                black_box(&candidate_keys),
                black_box(&hash_entries),
                |_| true,
            ));
            hash_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        hash_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let hash_p95 = hash_samples[16];
        println!(
            "RUNTIME581_SPATIAL_QUERY_HASH_LOOKUP_BENCH_V1 candidate_keys={} unique_entities={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
            candidate_keys.len(),
            visible_entries.len() / 2,
            legacy_p95,
            hash_p95,
        );
        assert!(
            hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
            "visible spatial HashMap lookup P95 {hash_p95} ns exceeded 70% of legacy {legacy_p95} ns"
        );
    }

    #[test]
    #[ignore = "release performance evidence; run through the validation coordinator"]
    fn optimization_batch_20260826k_runtime09b_spatial_query_hash_dedup_performance_evidence() {
        fn legacy_matching_entities(
            candidate_keys: &[u64],
            visible_entries: &BTreeMap<u64, VisibleSpatialEntry>,
        ) -> Vec<EntityId> {
            candidate_keys
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .iter()
                .filter_map(|key| visible_entries.get(key))
                .map(|entry| entry.entity)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        }

        let visible_entries = (0..32_768_u64)
            .map(|key| {
                (
                    key,
                    VisibleSpatialEntry {
                        entity: key / 2,
                        bounds: VisibilityBounds {
                            center: Vec3::new(key as f32, 0.0, 0.0),
                            radius: 1.0,
                        },
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let candidate_input = (0..32_768_u64).rev().collect::<Vec<_>>();
        let mut legacy_samples = Vec::with_capacity(17);
        let mut hash_samples = Vec::with_capacity(17);
        for _ in 0..17 {
            let started = Instant::now();
            black_box(legacy_matching_entities(
                black_box(&candidate_input),
                black_box(&visible_entries),
            ));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            let candidate_keys = candidate_input.iter().copied().collect::<HashSet<_>>();
            black_box(matching_entities(
                black_box(&candidate_keys),
                black_box(&visible_entries),
                |_| true,
            ));
            hash_samples.push(started.elapsed().as_nanos());
        }

        legacy_samples.sort_unstable();
        hash_samples.sort_unstable();
        let legacy_p95 = legacy_samples[16];
        let hash_p95 = hash_samples[16];
        println!(
            "RUNTIME09B_VISIBLE_SPATIAL_QUERY_HASH_DEDUP_BENCH_V1 candidate_keys={} unique_entities={} legacy_p95_ns={} hash_p95_ns={} legacy_ordered_set_admissions={} hash_candidate_admissions={} vec_entity_values={} target_ratio_bp=6000",
            candidate_input.len(),
            visible_entries.len() / 2,
            legacy_p95,
            hash_p95,
            candidate_input.len() + visible_entries.len() / 2,
            candidate_input.len(),
            candidate_input.len(),
        );
        assert!(
            hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
            "visible spatial hash dedup P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
        );
    }

    fn instance(
        entity: u64,
        stable_instance_key: u64,
        mobility: Mobility,
        center: Vec3,
    ) -> VisibilityBvhInstance {
        VisibilityBvhInstance {
            entity,
            stable_instance_key,
            key: VisibilityBatchKey {
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                material_id: ResourceId::from_stable_label("tests/material"),
                model_id: ResourceId::from_stable_label("tests/model"),
                mobility,
            },
            bounds: VisibilityBounds {
                center,
                radius: 1.0,
            },
        }
    }

    fn dynamic_context(instance_count: u64) -> VisibilityContext {
        let instances = (0..instance_count)
            .map(|index| {
                instance(
                    index + 1,
                    (index + 1) << 16,
                    Mobility::Dynamic,
                    Vec3::new(index as f32 * 32.0, 0.0, 0.0),
                )
            })
            .collect::<Vec<_>>();
        let mut context = VisibilityContext::default();
        context.frame_visibility.stable_instance_keys = instances
            .iter()
            .map(|instance| instance.stable_instance_key)
            .collect();
        context.frame_visibility.views = vec![ViewVisibilityContext {
            view: VisibilityViewKey::MainCamera,
            visible: (0..instance_count)
                .map(|index| u32::try_from(index).expect("test primitive index fits u32"))
                .collect(),
            ..Default::default()
        }];
        context.dynamic_index.rebuild(&instances);
        context.bvh_instances = instances;
        context
    }
}
