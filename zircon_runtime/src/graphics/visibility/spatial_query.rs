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

struct CandidateKeySet {
    membership: Option<HashSet<u64>>,
    order: Vec<u64>,
}

impl CandidateKeySet {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            membership: None,
            order: Vec::with_capacity(capacity),
        }
    }

    fn from_iter(iter: impl IntoIterator<Item = u64>) -> Self {
        let iter = iter.into_iter();
        let (lower_bound, _) = iter.size_hint();
        let mut order = Vec::with_capacity(lower_bound);
        let mut ascending = true;
        let mut descending = true;
        let mut has_duplicate = false;
        let mut last_key = None;
        for key in iter {
            if let Some(last_key) = last_key {
                ascending &= last_key <= key;
                descending &= last_key >= key;
                has_duplicate |= last_key == key;
            }
            last_key = Some(key);
            order.push(key);
        }
        if (ascending || descending) && !has_duplicate {
            return Self {
                membership: None,
                order,
            };
        }
        let mut keys = Self::with_capacity(order.len());
        for key in order {
            keys.insert(key);
        }
        keys
    }

    fn insert(&mut self, key: u64) {
        if let Some(membership) = &mut self.membership {
            if membership.insert(key) {
                self.order.push(key);
            }
            return;
        }
        if self.order.last() == Some(&key) {
            return;
        }
        let ascending = self.order.last().is_none_or(|last| *last <= key);
        let descending = self.order.last().is_none_or(|last| *last >= key);
        if !ascending && !descending {
            let mut membership = HashSet::with_capacity(self.order.len() + 1);
            membership.extend(self.order.iter().copied());
            if !membership.insert(key) {
                self.membership = Some(membership);
                return;
            }
            self.membership = Some(membership);
        }
        self.order.push(key);
    }

    fn len(&self) -> usize {
        self.membership
            .as_ref()
            .map_or(self.order.len(), HashSet::len)
    }

    fn iter(&self) -> impl Iterator<Item = &u64> {
        self.order.iter()
    }
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
                CandidateKeySet::from_iter(
                    static_query
                        .stable_instance_keys
                        .into_iter()
                        .chain(dynamic_query.stable_instance_keys),
                ),
                static_query
                    .visited_node_count
                    .saturating_add(dynamic_query.visited_node_count),
            ),
            _ => (
                CandidateKeySet::from_iter(self.visible_entries.keys().copied()),
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
                CandidateKeySet::from_iter(
                    static_query
                        .stable_instance_keys
                        .into_iter()
                        .chain(dynamic_query.stable_instance_keys),
                ),
                static_query
                    .visited_node_count
                    .saturating_add(dynamic_query.visited_node_count),
            ),
            _ => (
                CandidateKeySet::from_iter(self.visible_entries.keys().copied()),
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
    candidate_keys: &CandidateKeySet,
    visible_entries: &HashMap<u64, VisibleSpatialEntry>,
    mut matches: impl FnMut(VisibilityBounds) -> bool,
) -> Vec<EntityId> {
    let mut entities = Vec::with_capacity(candidate_keys.len());
    let mut ascending = true;
    let mut descending = true;
    let mut last_entity = None;
    for stable_instance_key in candidate_keys.iter() {
        let Some(entry) = visible_entries.get(stable_instance_key) else {
            continue;
        };
        if !matches(entry.bounds) {
            continue;
        }
        if last_entity == Some(entry.entity) {
            continue;
        }
        if let Some(last_entity) = last_entity {
            ascending &= last_entity <= entry.entity;
            descending &= last_entity >= entry.entity;
        }
        last_entity = Some(entry.entity);
        entities.push(entry.entity);
    }
    normalize_entity_ids_with_order(entities, ascending, descending)
}

fn normalize_entity_ids(mut entities: Vec<EntityId>) -> Vec<EntityId> {
    let mut ascending = true;
    let mut descending = true;
    for pair in entities.windows(2) {
        ascending &= pair[0] <= pair[1];
        descending &= pair[0] >= pair[1];
        if !ascending && !descending {
            break;
        }
    }
    normalize_entity_ids_with_order(entities, ascending, descending)
}

fn normalize_entity_ids_with_order(
    mut entities: Vec<EntityId>,
    ascending: bool,
    descending: bool,
) -> Vec<EntityId> {
    if descending && !ascending {
        entities.reverse();
    } else if !ascending {
        entities.sort_unstable();
    }
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
#[path = "tests/spatial_query.rs"]
mod tests;
