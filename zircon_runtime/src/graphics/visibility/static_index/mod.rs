use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::scene::EntityId;
use crate::core::math::{Real, Vec3};

use super::declarations::{
    VisibilityBounds, VisibilityBvhInstance, VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy,
};

const DEFAULT_CELL_SIZE: Real = 16.0;
const MIN_CELL_SIZE: Real = 0.001;

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
    entries: BTreeMap<EntityId, StaticIndexEntry>,
    cells: BTreeMap<StaticCellCoord, BTreeSet<EntityId>>,
    report: VisibilityStaticIndexReport,
}

#[derive(Clone, Debug, PartialEq)]
struct StaticIndexEntry {
    bounds: VisibilityBounds,
    cells: Vec<StaticCellCoord>,
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
            entries: BTreeMap::new(),
            cells: BTreeMap::new(),
            report: VisibilityStaticIndexReport::default(),
        }
    }

    pub(crate) fn rebuild(
        &mut self,
        instances: &[VisibilityBvhInstance],
    ) -> VisibilityStaticIndexReport {
        self.entries.clear();
        self.cells.clear();

        for instance in instances {
            self.insert_or_replace(instance.entity, instance.bounds);
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

        let instances_by_entity = instances
            .iter()
            .map(|instance| (instance.entity, instance))
            .collect::<BTreeMap<_, _>>();

        for entity in &plan.removed_entities {
            self.remove(*entity);
        }
        for entity in plan
            .inserted_entities
            .iter()
            .chain(plan.updated_entities.iter())
        {
            if let Some(instance) = instances_by_entity.get(entity) {
                self.insert_or_replace(instance.entity, instance.bounds);
            } else {
                self.remove(*entity);
            }
        }

        self.report = VisibilityStaticIndexReport {
            full_rebuild_count: self.report.full_rebuild_count,
            incremental_update_count: self.report.incremental_update_count.saturating_add(1),
            frame_full_rebuild_count: 0,
            frame_incremental_update_count: 1,
            inserted_count: plan.inserted_entities.len(),
            updated_count: plan.updated_entities.len(),
            removed_count: plan.removed_entities.len(),
            indexed_entity_count: self.entries.len(),
            occupied_cell_count: self.cells.len(),
            ..VisibilityStaticIndexReport::default()
        };
        self.report.clone()
    }

    pub(crate) fn query_bounds(&self, bounds: VisibilityBounds) -> Vec<EntityId> {
        let mut entities = BTreeSet::new();
        for cell in self.cells_for_bounds(bounds) {
            if let Some(cell_entities) = self.cells.get(&cell) {
                entities.extend(cell_entities.iter().copied());
            }
        }
        entities.into_iter().collect()
    }

    pub(crate) fn report(&self) -> VisibilityStaticIndexReport {
        self.report.clone()
    }

    fn insert_or_replace(&mut self, entity: EntityId, bounds: VisibilityBounds) {
        self.remove(entity);
        let cells = self.cells_for_bounds(bounds);
        for cell in &cells {
            self.cells.entry(*cell).or_default().insert(entity);
        }
        self.entries
            .insert(entity, StaticIndexEntry { bounds, cells });
    }

    fn remove(&mut self, entity: EntityId) {
        let Some(entry) = self.entries.remove(&entity) else {
            return;
        };
        for cell in entry.cells {
            let should_remove = self.cells.get_mut(&cell).is_some_and(|entities| {
                entities.remove(&entity);
                entities.is_empty()
            });
            if should_remove {
                self.cells.remove(&cell);
            }
        }
    }

    fn cells_for_bounds(&self, bounds: VisibilityBounds) -> Vec<StaticCellCoord> {
        let radius = bounds.radius.max(0.0);
        let min = bounds.center - Vec3::splat(radius);
        let max = bounds.center + Vec3::splat(radius);
        let min_cell = self.cell_for_point(min);
        let max_cell = self.cell_for_point(max);
        let mut cells = Vec::new();
        for z in min_cell.z..=max_cell.z {
            for y in min_cell.y..=max_cell.y {
                for x in min_cell.x..=max_cell.x {
                    cells.push(StaticCellCoord { x, y, z });
                }
            }
        }
        cells
    }

    fn cell_for_point(&self, point: Vec3) -> StaticCellCoord {
        StaticCellCoord {
            x: cell_axis(point.x, self.cell_size),
            y: cell_axis(point.y, self.cell_size),
            z: cell_axis(point.z, self.cell_size),
        }
    }
}

fn cell_axis(value: Real, cell_size: Real) -> i32 {
    (value / cell_size).floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::RenderLayerSet;
    use crate::core::framework::scene::Mobility;
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
            inserted_entities: vec![3],
            updated_entities: vec![2],
            removed_entities: vec![1],
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
                inserted_entities: vec![2],
                updated_entities: Vec::new(),
                removed_entities: Vec::new(),
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

    fn instance(entity: EntityId, center: Vec3, radius: Real) -> VisibilityBvhInstance {
        VisibilityBvhInstance {
            entity,
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
