use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::core::math::{Real, Vec3};

use super::declarations::{
    VisibilityBounds, VisibilityBvhInstance, VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy,
};

const DEFAULT_CELL_SIZE: Real = 16.0;
const MIN_CELL_SIZE: Real = 0.001;
const MAX_CELLS_PER_INDEXED_INSTANCE: usize = 4_096;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityStaticIndexReport {
    pub full_rebuild_count: u32,
    pub incremental_update_count: u32,
    pub frame_full_rebuild_count: u32,
    pub frame_incremental_update_count: u32,
    pub inserted_count: usize,
    pub updated_count: usize,
    pub removed_count: usize,
    pub indexed_entity_count: usize,
    pub occupied_cell_count: usize,
    pub main_view_prefilter_used: bool,
    pub main_view_static_input_count: usize,
    pub main_view_static_candidate_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct VisibilityStaticIndex {
    cell_size: Real,
    // Persistent visibility history and published query snapshots may share these maps.
    // Mutations use copy-on-write, so pointer queries never retain mutable renderer state.
    entries: Arc<BTreeMap<u64, StaticIndexEntry>>,
    cells: Arc<BTreeMap<StaticCellCoord, BTreeSet<u64>>>,
    overflow_instance_keys: Arc<BTreeSet<u64>>,
    report: VisibilityStaticIndexReport,
}

#[derive(Clone, Debug, PartialEq)]
struct StaticIndexEntry {
    bounds: VisibilityBounds,
    cells: Vec<StaticCellCoord>,
    overflow: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct StaticCellCoord {
    x: i32,
    y: i32,
    z: i32,
}

impl Default for VisibilityStaticIndex {
    fn default() -> Self {
        Self::new(DEFAULT_CELL_SIZE)
    }
}

impl VisibilityStaticIndex {
    pub(crate) fn new(cell_size: Real) -> Self {
        Self {
            cell_size: cell_size.max(MIN_CELL_SIZE),
            entries: Arc::new(BTreeMap::new()),
            cells: Arc::new(BTreeMap::new()),
            overflow_instance_keys: Arc::new(BTreeSet::new()),
            report: VisibilityStaticIndexReport::default(),
        }
    }

    pub(crate) fn rebuild(
        &mut self,
        instances: &[VisibilityBvhInstance],
    ) -> VisibilityStaticIndexReport {
        Arc::make_mut(&mut self.entries).clear();
        Arc::make_mut(&mut self.cells).clear();
        Arc::make_mut(&mut self.overflow_instance_keys).clear();

        for instance in instances {
            self.insert_or_replace(instance.stable_instance_key, instance.bounds);
        }

        self.report = VisibilityStaticIndexReport {
            full_rebuild_count: self.report.full_rebuild_count.saturating_add(1),
            incremental_update_count: self.report.incremental_update_count,
            frame_full_rebuild_count: 1,
            frame_incremental_update_count: 0,
            inserted_count: instances.len(),
            indexed_entity_count: self.entries.len(),
            occupied_cell_count: self.cells.len(),
            ..VisibilityStaticIndexReport::default()
        };
        self.report.clone()
    }

    pub(crate) fn apply_update_plan(
        &mut self,
        instances: &[VisibilityBvhInstance],
        plan: &VisibilityBvhUpdatePlan,
    ) -> VisibilityStaticIndexReport {
        if matches!(plan.strategy, VisibilityBvhUpdateStrategy::FullRebuild) {
            return self.rebuild(instances);
        }

        let instances_by_stable_instance_key = instances
            .iter()
            .map(|instance| (instance.stable_instance_key, instance))
            .collect::<BTreeMap<_, _>>();

        for stable_instance_key in &plan.removed_stable_instance_keys {
            self.remove(*stable_instance_key);
        }
        for stable_instance_key in plan
            .inserted_stable_instance_keys
            .iter()
            .chain(plan.updated_stable_instance_keys.iter())
        {
            if let Some(instance) = instances_by_stable_instance_key.get(stable_instance_key) {
                self.insert_or_replace(instance.stable_instance_key, instance.bounds);
            } else {
                self.remove(*stable_instance_key);
            }
        }

        self.report = VisibilityStaticIndexReport {
            full_rebuild_count: self.report.full_rebuild_count,
            incremental_update_count: self.report.incremental_update_count.saturating_add(1),
            frame_full_rebuild_count: 0,
            frame_incremental_update_count: 1,
            inserted_count: plan.inserted_stable_instance_keys.len(),
            updated_count: plan.updated_stable_instance_keys.len(),
            removed_count: plan.removed_stable_instance_keys.len(),
            indexed_entity_count: self.entries.len(),
            occupied_cell_count: self.cells.len(),
            ..VisibilityStaticIndexReport::default()
        };
        self.report.clone()
    }

    pub(crate) fn query_bounds(&self, bounds: VisibilityBounds) -> Vec<u64> {
        self.query_bounds_with_stats_limited(bounds, MAX_CELLS_PER_INDEXED_INSTANCE)
            .map(|query| query.stable_instance_keys)
            .unwrap_or_else(|| self.entries.keys().copied().collect())
    }

    /// `unit_direction` is normalized by the query owner once before both
    /// static and dynamic indexes traverse the same ray.
    pub(crate) fn query_ray_with_stats_limited(
        &self,
        origin: Vec3,
        unit_direction: Vec3,
        max_distance: Real,
        max_cell_count: usize,
    ) -> Option<VisibilityStaticIndexQuery> {
        let cells =
            self.cells_for_ray_limited(origin, unit_direction, max_distance, max_cell_count)?;
        let mut stable_instance_keys = self
            .overflow_instance_keys
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for cell in &cells {
            if let Some(cell_instance_keys) = self.cells.get(cell) {
                stable_instance_keys.extend(cell_instance_keys.iter().copied());
            }
        }
        Some(VisibilityStaticIndexQuery {
            stable_instance_keys: stable_instance_keys.into_iter().collect(),
            visited_node_count: cells.len(),
        })
    }

    pub(crate) fn query_bounds_with_stats_limited(
        &self,
        bounds: VisibilityBounds,
        max_cell_count: usize,
    ) -> Option<VisibilityStaticIndexQuery> {
        let cells = self.cells_for_bounds_limited(bounds, max_cell_count)?;
        let mut stable_instance_keys = self
            .overflow_instance_keys
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for cell in &cells {
            if let Some(cell_instance_keys) = self.cells.get(cell) {
                stable_instance_keys.extend(cell_instance_keys.iter().copied());
            }
        }
        Some(VisibilityStaticIndexQuery {
            stable_instance_keys: stable_instance_keys.into_iter().collect(),
            visited_node_count: cells.len(),
        })
    }

    pub(crate) fn report(&self) -> VisibilityStaticIndexReport {
        self.report.clone()
    }

    fn insert_or_replace(&mut self, stable_instance_key: u64, bounds: VisibilityBounds) {
        self.remove(stable_instance_key);
        let Some(cells) = self.cells_for_bounds_limited(bounds, MAX_CELLS_PER_INDEXED_INSTANCE)
        else {
            Arc::make_mut(&mut self.overflow_instance_keys).insert(stable_instance_key);
            Arc::make_mut(&mut self.entries).insert(
                stable_instance_key,
                StaticIndexEntry {
                    bounds,
                    cells: Vec::new(),
                    overflow: true,
                },
            );
            return;
        };
        let indexed_cells = Arc::make_mut(&mut self.cells);
        for cell in &cells {
            indexed_cells
                .entry(*cell)
                .or_default()
                .insert(stable_instance_key);
        }
        Arc::make_mut(&mut self.entries).insert(
            stable_instance_key,
            StaticIndexEntry {
                bounds,
                cells,
                overflow: false,
            },
        );
    }

    fn remove(&mut self, stable_instance_key: u64) {
        let Some(entry) = Arc::make_mut(&mut self.entries).remove(&stable_instance_key) else {
            return;
        };
        if entry.overflow {
            Arc::make_mut(&mut self.overflow_instance_keys).remove(&stable_instance_key);
            return;
        }
        let indexed_cells = Arc::make_mut(&mut self.cells);
        for cell in entry.cells {
            let should_remove = indexed_cells
                .get_mut(&cell)
                .is_some_and(|cell_instance_keys| {
                    cell_instance_keys.remove(&stable_instance_key);
                    cell_instance_keys.is_empty()
                });
            if should_remove {
                indexed_cells.remove(&cell);
            }
        }
    }

    fn cells_for_bounds_limited(
        &self,
        bounds: VisibilityBounds,
        max_cell_count: usize,
    ) -> Option<Vec<StaticCellCoord>> {
        let radius = bounds.radius.max(0.0);
        let min = bounds.center - Vec3::splat(radius);
        let max = bounds.center + Vec3::splat(radius);
        let min_cell = self.cell_for_point(min);
        let max_cell = self.cell_for_point(max);
        let cell_count = cell_axis_span(min_cell.x, max_cell.x)?
            .checked_mul(cell_axis_span(min_cell.y, max_cell.y)?)?
            .checked_mul(cell_axis_span(min_cell.z, max_cell.z)?)?;
        if cell_count > max_cell_count {
            return None;
        }

        let mut cells = Vec::with_capacity(cell_count);
        for z in min_cell.z..=max_cell.z {
            for y in min_cell.y..=max_cell.y {
                for x in min_cell.x..=max_cell.x {
                    cells.push(StaticCellCoord { x, y, z });
                }
            }
        }
        Some(cells)
    }

    fn cells_for_ray_limited(
        &self,
        origin: Vec3,
        unit_direction: Vec3,
        max_distance: Real,
        max_cell_count: usize,
    ) -> Option<Vec<StaticCellCoord>> {
        if max_cell_count == 0 {
            return None;
        }
        let mut cell = self.cell_for_point(origin);
        let (step_x, mut next_x, delta_x) =
            ray_axis_step(origin.x, unit_direction.x, cell.x, self.cell_size);
        let (step_y, mut next_y, delta_y) =
            ray_axis_step(origin.y, unit_direction.y, cell.y, self.cell_size);
        let (step_z, mut next_z, delta_z) =
            ray_axis_step(origin.z, unit_direction.z, cell.z, self.cell_size);
        let mut cells = Vec::new();

        loop {
            if cells.len() >= max_cell_count {
                return None;
            }
            cells.push(cell);
            let next = next_x.min(next_y).min(next_z);
            if !next.is_finite() || next > max_distance {
                break;
            }

            let crosses_x = approximately_equal(next_x, next);
            let crosses_y = approximately_equal(next_y, next);
            let crosses_z = approximately_equal(next_z, next);
            if !append_ray_boundary_neighbors(
                &mut cells,
                cell,
                (crosses_x, step_x),
                (crosses_y, step_y),
                (crosses_z, step_z),
                max_cell_count,
            ) {
                return None;
            }

            // Advance every crossed axis after preserving the side cells that
            // meet at an edge or corner. A ray touching that boundary may hit
            // an entry whose conservative bounds live in any of those cells.
            if crosses_x {
                cell.x = cell.x.saturating_add(step_x);
                next_x += delta_x;
            }
            if crosses_y {
                cell.y = cell.y.saturating_add(step_y);
                next_y += delta_y;
            }
            if crosses_z {
                cell.z = cell.z.saturating_add(step_z);
                next_z += delta_z;
            }
        }

        Some(cells)
    }

    fn cell_for_point(&self, point: Vec3) -> StaticCellCoord {
        StaticCellCoord {
            x: cell_axis(point.x, self.cell_size),
            y: cell_axis(point.y, self.cell_size),
            z: cell_axis(point.z, self.cell_size),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct VisibilityStaticIndexQuery {
    pub(crate) stable_instance_keys: Vec<u64>,
    pub(crate) visited_node_count: usize,
}

fn cell_axis(value: Real, cell_size: Real) -> i32 {
    (value / cell_size).floor() as i32
}

fn cell_axis_span(min_cell: i32, max_cell: i32) -> Option<usize> {
    let span = i64::from(max_cell)
        .checked_sub(i64::from(min_cell))?
        .checked_add(1)?;
    usize::try_from(span).ok()
}

fn ray_axis_step(origin: Real, direction: Real, cell: i32, cell_size: Real) -> (i32, Real, Real) {
    if direction > 0.0 {
        let boundary = (i64::from(cell) + 1) as Real * cell_size;
        return (
            1,
            ((boundary - origin) / direction).max(0.0),
            cell_size / direction,
        );
    }
    if direction < 0.0 {
        let boundary = cell as Real * cell_size;
        return (
            -1,
            ((boundary - origin) / direction).max(0.0),
            cell_size / -direction,
        );
    }
    (0, Real::INFINITY, Real::INFINITY)
}

fn approximately_equal(left: Real, right: Real) -> bool {
    (left - right).abs() <= Real::EPSILON * 8.0
}

fn append_ray_boundary_neighbors(
    cells: &mut Vec<StaticCellCoord>,
    cell: StaticCellCoord,
    x: (bool, i32),
    y: (bool, i32),
    z: (bool, i32),
    max_cell_count: usize,
) -> bool {
    let axes = [x, y, z];
    let crossed_count = axes.iter().filter(|(crosses, _)| *crosses).count();
    if crossed_count <= 1 {
        return true;
    }

    // Exclude the full advance: it becomes the next loop's primary cell.
    for selection in 1..((1_usize << crossed_count) - 1) {
        if cells.len() >= max_cell_count {
            return false;
        }
        let mut neighbor = cell;
        let mut crossed_index = 0_usize;
        for (axis, (crosses, step)) in axes.iter().enumerate() {
            if !*crosses {
                continue;
            }
            if selection & (1_usize << crossed_index) != 0 {
                match axis {
                    0 => neighbor.x = neighbor.x.saturating_add(*step),
                    1 => neighbor.y = neighbor.y.saturating_add(*step),
                    2 => neighbor.z = neighbor.z.saturating_add(*step),
                    _ => {}
                }
            }
            crossed_index += 1;
        }
        cells.push(neighbor);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::RenderLayerSet;
    use crate::core::framework::scene::{EntityId, Mobility};
    use crate::core::resource::ResourceId;
    use crate::graphics::visibility::{VisibilityBatchKey, VisibilityBvhUpdateStrategy};

    #[test]
    fn visibility_static_index_incremental_update_matches_full_rebuild_queries() {
        let first_frame = vec![
            instance(1, Vec3::new(0.0, 0.0, 0.0), 1.0),
            instance(2, Vec3::new(40.0, 0.0, 0.0), 1.0),
        ];
        let second_frame = vec![
            instance(2, Vec3::new(48.0, 0.0, 0.0), 1.0),
            instance(3, Vec3::new(0.0, 20.0, 0.0), 1.0),
        ];
        let plan = VisibilityBvhUpdatePlan {
            strategy: VisibilityBvhUpdateStrategy::Incremental,
            inserted_stable_instance_keys: vec![3],
            updated_stable_instance_keys: vec![2],
            removed_stable_instance_keys: vec![1],
        };
        let mut incremental = VisibilityStaticIndex::new(16.0);
        incremental.rebuild(&first_frame);

        let update_report = incremental.apply_update_plan(&second_frame, &plan);

        let mut full = VisibilityStaticIndex::new(16.0);
        let rebuild_report = full.rebuild(&second_frame);
        let full_scene_query = VisibilityBounds {
            center: Vec3::new(24.0, 10.0, 0.0),
            radius: 64.0,
        };
        let removed_entity_query = VisibilityBounds {
            center: Vec3::ZERO,
            radius: 2.0,
        };

        assert_eq!(update_report.full_rebuild_count, 1);
        assert_eq!(update_report.incremental_update_count, 1);
        assert_eq!(update_report.frame_full_rebuild_count, 0);
        assert_eq!(update_report.frame_incremental_update_count, 1);
        assert_eq!(update_report.inserted_count, 1);
        assert_eq!(update_report.updated_count, 1);
        assert_eq!(update_report.removed_count, 1);
        assert_eq!(update_report.indexed_entity_count, 2);
        assert_eq!(rebuild_report.full_rebuild_count, 1);
        assert_eq!(
            incremental.query_bounds(full_scene_query),
            full.query_bounds(full_scene_query)
        );
        assert_eq!(
            incremental.query_bounds(removed_entity_query),
            Vec::<EntityId>::new()
        );
    }

    #[test]
    fn visibility_static_index_full_rebuild_strategy_replaces_existing_rows() {
        let mut index = VisibilityStaticIndex::new(16.0);
        index.rebuild(&[instance(1, Vec3::ZERO, 1.0)]);
        let report = index.apply_update_plan(
            &[instance(2, Vec3::new(32.0, 0.0, 0.0), 1.0)],
            &VisibilityBvhUpdatePlan {
                strategy: VisibilityBvhUpdateStrategy::FullRebuild,
                inserted_stable_instance_keys: vec![2],
                updated_stable_instance_keys: Vec::new(),
                removed_stable_instance_keys: Vec::new(),
            },
        );

        assert_eq!(report.full_rebuild_count, 2);
        assert_eq!(report.incremental_update_count, 0);
        assert_eq!(report.frame_full_rebuild_count, 1);
        assert_eq!(report.frame_incremental_update_count, 0);
        assert_eq!(index.report().indexed_entity_count, 1);
        assert_eq!(
            index.query_bounds(VisibilityBounds {
                center: Vec3::new(32.0, 0.0, 0.0),
                radius: 2.0,
            }),
            vec![2]
        );
    }

    #[test]
    fn visibility_static_index_clone_shares_persistent_storage_until_a_mutation() {
        let mut index = VisibilityStaticIndex::new(16.0);
        index.rebuild(&[instance(1, Vec3::ZERO, 1.0)]);
        let snapshot = index.clone();

        assert!(Arc::ptr_eq(&index.entries, &snapshot.entries));
        assert!(Arc::ptr_eq(&index.cells, &snapshot.cells));
        assert!(Arc::ptr_eq(
            &index.overflow_instance_keys,
            &snapshot.overflow_instance_keys,
        ));

        index.rebuild(&[instance(2, Vec3::new(32.0, 0.0, 0.0), 1.0)]);

        assert_eq!(
            snapshot.query_bounds(VisibilityBounds {
                center: Vec3::ZERO,
                radius: 2.0,
            }),
            vec![1]
        );
        assert_eq!(
            index.query_bounds(VisibilityBounds {
                center: Vec3::new(32.0, 0.0, 0.0),
                radius: 2.0,
            }),
            vec![2]
        );
    }

    #[test]
    fn visibility_static_index_bounded_query_refuses_extreme_cell_volume() {
        let mut index = VisibilityStaticIndex::new(16.0);
        index.rebuild(&[instance(1, Vec3::ZERO, 1.0)]);

        assert_eq!(
            index.query_bounds_with_stats_limited(
                VisibilityBounds {
                    center: Vec3::ZERO,
                    radius: f32::MAX,
                },
                4_096,
            ),
            None,
        );
    }

    #[test]
    fn visibility_static_index_keeps_extreme_bounds_in_conservative_overflow() {
        let mut index = VisibilityStaticIndex::new(16.0);
        let report = index.rebuild(&[instance(1, Vec3::ZERO, f32::MAX)]);

        let query = index
            .query_bounds_with_stats_limited(
                VisibilityBounds {
                    center: Vec3::new(1.0, 0.0, 0.0),
                    radius: 1.0,
                },
                MAX_CELLS_PER_INDEXED_INSTANCE,
            )
            .expect("small query stays inside the cell budget");

        assert_eq!(report.indexed_entity_count, 1);
        assert_eq!(report.occupied_cell_count, 0);
        assert_eq!(query.stable_instance_keys, vec![1]);
    }

    #[test]
    fn visibility_static_index_large_internal_query_keeps_all_entries_conservative() {
        let mut index = VisibilityStaticIndex::new(16.0);
        index.rebuild(&[
            instance(1, Vec3::ZERO, 1.0),
            instance(2, Vec3::new(64.0, 0.0, 0.0), 1.0),
        ]);

        assert_eq!(
            index.query_bounds(VisibilityBounds {
                center: Vec3::ZERO,
                radius: f32::MAX,
            }),
            vec![1, 2],
        );
    }

    fn instance(entity: EntityId, center: Vec3, radius: Real) -> VisibilityBvhInstance {
        VisibilityBvhInstance {
            entity,
            stable_instance_key: entity,
            key: VisibilityBatchKey {
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                material_id: ResourceId::from_stable_label("tests/material"),
                model_id: ResourceId::from_stable_label("tests/model"),
                mobility: Mobility::Static,
            },
            bounds: VisibilityBounds { center, radius },
        }
    }
}
