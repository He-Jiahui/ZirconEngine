use std::collections::{BTreeMap, BTreeSet};

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
    visible_entries: BTreeMap<u64, VisibleSpatialEntry>,
}

impl VisibleSpatialQuery {
    pub(crate) fn from_context(context: &VisibilityContext) -> Self {
        let visible_stable_instance_keys = context
            .frame_visibility
            .main_view_visible_stable_instance_key_set();
        let visible_entries = context
            .bvh_instances
            .iter()
            .filter(|instance| visible_stable_instance_keys.contains(&instance.stable_instance_key))
            .map(|instance| {
                (
                    instance.stable_instance_key,
                    VisibleSpatialEntry {
                        entity: instance.entity,
                        bounds: instance.bounds,
                    },
                )
            })
            .collect();

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
                    .collect::<BTreeSet<_>>(),
                static_query
                    .visited_node_count
                    .saturating_add(dynamic_query.visited_node_count),
            ),
            _ => (
                self.visible_entries
                    .keys()
                    .copied()
                    .collect::<BTreeSet<_>>(),
                self.visible_entries.len(),
            ),
        };
        let entities = candidate_keys
            .iter()
            .filter_map(|stable_instance_key| self.visible_entries.get(stable_instance_key))
            .filter(|entry| bounds_overlap(entry.bounds, bounds))
            .map(|entry| entry.entity)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

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
                    .collect::<BTreeSet<_>>(),
                static_query
                    .visited_node_count
                    .saturating_add(dynamic_query.visited_node_count),
            ),
            _ => (
                self.visible_entries
                    .keys()
                    .copied()
                    .collect::<BTreeSet<_>>(),
                self.visible_entries.len(),
            ),
        };
        let entities = candidate_keys
            .iter()
            .filter_map(|stable_instance_key| self.visible_entries.get(stable_instance_key))
            .filter(|entry| {
                ray_intersects_bounds(entry.bounds, ray.origin, direction, ray.max_distance)
            })
            .map(|entry| entry.entity)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

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
