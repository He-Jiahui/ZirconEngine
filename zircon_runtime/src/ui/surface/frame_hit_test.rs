use std::collections::{BTreeMap, BTreeSet};

use crate::ui::tree::{
    bounded_cells_for_frame, bounded_hit_grid_dimensions, frame_is_finite_positive,
    hit_grid_capacity_bounds, UiHitTestIndex, UiHitTestResult,
};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiPoint},
    surface::{
        UiArrangedTree, UiHitCoordinateSpace, UiHitRouteNode, UiHitTestCell, UiHitTestDebugDump,
        UiHitTestEntry, UiHitTestGrid, UiHitTestQuery, UiHitTestReject, UiHitTestRejectReason,
        UiPersistentSequenceCowStats, UiSurfaceFrame,
    },
    tree::UiInputPolicy,
};

use super::{arranged_effective_input_policy, is_arranged_child_hit_path_visible, UiSurface};

#[derive(Clone, Debug, Default)]
pub(super) struct UiProjectedHitTestIndex {
    grid: UiHitTestGrid,
    initialized: bool,
    overlay_z_base: i32,
    projection_roots: Vec<UiNodeId>,
    projected_node_ids: BTreeSet<UiNodeId>,
    projected_order_keys: BTreeMap<UiNodeId, (i32, u64)>,
    projected_popup_roots: BTreeMap<UiNodeId, UiNodeId>,
    projected_source_sort_keys: BTreeMap<UiNodeId, (i32, u64)>,
    entry_cells: BTreeMap<UiNodeId, Vec<usize>>,
    entry_indices: BTreeMap<UiNodeId, usize>,
}

// The projected index is a rebuild-owned cache and does not change UiSurface value equality.
impl PartialEq for UiProjectedHitTestIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug)]
struct UiHitTestProjection {
    popup_root: UiNodeId,
    source_frame: UiFrame,
    target_frame: Option<UiFrame>,
    target_clip: Option<UiFrame>,
    stack_order: usize,
}

impl UiProjectedHitTestIndex {
    pub(super) fn authoritative_grid<'a>(
        &'a self,
        base_grid: &'a UiHitTestGrid,
    ) -> &'a UiHitTestGrid {
        if self.initialized {
            &self.grid
        } else {
            base_grid
        }
    }

    pub(super) fn authoritative_entry<'a>(
        &'a self,
        base_grid: &'a UiHitTestGrid,
        node_id: UiNodeId,
    ) -> Option<&'a UiHitTestEntry> {
        if self.initialized {
            self.entry_indices
                .get(&node_id)
                .and_then(|entry_index| self.grid.entries.get(*entry_index))
        } else {
            base_grid
                .entries
                .iter()
                .find(|entry| entry.node_id == node_id)
        }
    }

    fn rebuild(&mut self, base_grid: &UiHitTestGrid, projections: &[UiHitTestProjection]) -> bool {
        #[cfg(feature = "profiling")]
        let rebuild_start = std::time::Instant::now();
        self.overlay_z_base = base_grid
            .entries
            .iter()
            .map(|entry| entry.z_index)
            .max()
            .unwrap_or_default()
            .saturating_add(1);
        self.projection_roots = projection_roots(projections);
        self.projected_node_ids.clear();
        let projection_by_root = projection_by_root(projections);
        let order_plan = projection_order_plan(base_grid, &projection_by_root, self.overlay_z_base);
        let entries = base_grid
            .entries
            .iter()
            .map(|entry| {
                let (entry, projected) = project_entry(
                    base_grid,
                    entry,
                    &projection_by_root,
                    self.overlay_z_base,
                    &order_plan.order_keys,
                );
                if projected {
                    self.projected_node_ids.insert(entry.node_id);
                }
                entry
            })
            .collect();
        self.projected_order_keys = order_plan.order_keys;
        self.projected_popup_roots = order_plan.popup_roots;
        self.projected_source_sort_keys = order_plan.source_sort_keys;
        let next_grid = build_projected_grid(base_grid, entries);
        let changed = !self.initialized || self.grid != next_grid;
        self.grid = next_grid;
        self.initialized = true;
        self.reindex_entries();
        #[cfg(feature = "profiling")]
        crate::core::diagnostics::profiling::record_counter(
            "runtime",
            "ui.surface_projected_hit.rebuild_elapsed_us",
            rebuild_start.elapsed().as_micros() as f64,
        );
        changed
    }

    fn patch(
        &mut self,
        base_index: &UiHitTestIndex,
        projections: &[UiHitTestProjection],
        changed_node_ids: &BTreeSet<UiNodeId>,
    ) -> Result<bool, ()> {
        if !self.initialized
            || self.projection_roots != projection_roots(projections)
            || self.grid.scope != base_index.grid.scope
            || self.grid.entries.len() != base_index.grid.entries.len()
            || self.grid.route_nodes.len() != base_index.grid.route_nodes.len()
        {
            return Err(());
        }
        if (self.entry_cells.is_empty() || self.entry_indices.is_empty())
            && !self.grid.entries.is_empty()
        {
            self.reindex_entries();
        }

        // Keep incremental work bounded to changed entries plus the projected popup subtree.
        // A base-wide invariant change is repaired through the rebuild fallback below.
        let projection_by_root = projection_by_root(projections);
        self.grid.route_nodes = base_index.grid.route_nodes.clone();
        let mut affected_node_ids = changed_node_ids.clone();
        affected_node_ids.extend(self.projected_node_ids.iter().copied());
        let mut updates = Vec::with_capacity(affected_node_ids.len());
        for node_id in affected_node_ids {
            let base_entry = base_index.entry_by_node_id(node_id);
            let current_entry_index = self.entry_indices.get(&node_id).copied();
            match (base_entry, current_entry_index) {
                (None, None) => continue,
                (Some(base_entry), Some(entry_index)) => {
                    if base_entry.z_index >= self.overlay_z_base {
                        return Err(());
                    }
                    let current_projection_root =
                        projection_for_entry(&base_index.grid, base_entry, &projection_by_root)
                            .map(|projection| projection.popup_root);
                    if self.projected_popup_roots.get(&node_id).copied() != current_projection_root
                        || current_projection_root.is_some()
                            && self.projected_source_sort_keys.get(&node_id).copied()
                                != Some((base_entry.z_index, base_entry.paint_order))
                    {
                        return Err(());
                    }
                    let (next_entry, _) = project_entry(
                        &base_index.grid,
                        base_entry,
                        &projection_by_root,
                        self.overlay_z_base,
                        &self.projected_order_keys,
                    );
                    let Some(current_entry) = self.grid.entries.get(entry_index) else {
                        return Err(());
                    };
                    if projected_entry_sort_key(current_entry)
                        != projected_entry_sort_key(&next_entry)
                    {
                        return Err(());
                    }
                    if frame_has_area(next_entry.clip_frame)
                        && !frame_is_contained(self.grid.bounds, next_entry.clip_frame)
                    {
                        return Err(());
                    }
                    let previous_cells =
                        self.entry_cells.get(&node_id).cloned().unwrap_or_default();
                    let next_cells = bounded_cells_for_frame(
                        self.grid.bounds,
                        self.grid.columns,
                        self.grid.rows,
                        self.grid.cell_size,
                        next_entry.clip_frame,
                    );
                    if next_cells
                        .iter()
                        .any(|cell_index| self.grid.cells.get(*cell_index).is_none())
                    {
                        return Err(());
                    }
                    updates.push((entry_index, next_entry, previous_cells, next_cells));
                }
                _ => return Err(()),
            }
        }

        let changed = updates
            .iter()
            .any(|(entry_index, next, previous_cells, next_cells)| {
                self.grid.entries.get(*entry_index) != Some(next) || previous_cells != next_cells
            });
        let mut entry_cow_stats = UiPersistentSequenceCowStats::default();
        let mut cell_cow_stats = UiPersistentSequenceCowStats::default();
        let mut cell_membership_clone_count = 0_usize;
        for (entry_index, next_entry, previous_cells, next_cells) in updates {
            if self.grid.entries.get(entry_index) == Some(&next_entry)
                && previous_cells == next_cells
            {
                continue;
            }
            for cell_index in previous_cells {
                if let Some((cell, stats)) = self.grid.cells.get_mut_with_stats(cell_index) {
                    cell_cow_stats.accumulate(stats);
                    cell_membership_clone_count = cell_membership_clone_count
                        .saturating_add(cell.entries.retain(|candidate| *candidate != entry_index));
                }
            }
            let node_id = next_entry.node_id;
            if let Some((entry, stats)) = self.grid.entries.get_mut_with_stats(entry_index) {
                entry_cow_stats.accumulate(stats);
                *entry = next_entry;
            }
            self.entry_cells.insert(node_id, next_cells.clone());
            for cell_index in next_cells {
                if let Some((cell, stats)) = self.grid.cells.get_mut_with_stats(cell_index) {
                    cell_cow_stats.accumulate(stats);
                    let insertion_index = cell
                        .entries
                        .partition_point(|candidate| *candidate <= entry_index);
                    cell_membership_clone_count = cell_membership_clone_count
                        .saturating_add(cell.entries.insert(insertion_index, entry_index));
                }
            }
        }
        record_projected_hit_persistent_cow(
            entry_cow_stats,
            cell_cow_stats,
            cell_membership_clone_count,
        );
        Ok(changed)
    }

    fn synchronize(
        &mut self,
        base_index: &UiHitTestIndex,
        projections: &[UiHitTestProjection],
        changed_node_ids: &BTreeSet<UiNodeId>,
        base_grid_rebuilt: bool,
    ) -> bool {
        if base_grid_rebuilt {
            return self.rebuild(&base_index.grid, projections);
        }
        #[cfg(feature = "profiling")]
        let patch_start = std::time::Instant::now();
        let patch_result = self.patch(base_index, projections, changed_node_ids);
        #[cfg(feature = "profiling")]
        let patch_elapsed_us = patch_start.elapsed().as_micros() as f64;
        #[cfg(feature = "profiling")]
        let affected_entry_count = changed_node_ids.union(&self.projected_node_ids).count() as f64;
        #[cfg(feature = "profiling")]
        crate::core::diagnostics::profiling::record_counter_batch(
            "runtime",
            &[
                (
                    "ui.surface_projected_hit.patch_elapsed_us",
                    patch_elapsed_us,
                ),
                (
                    "ui.surface_projected_hit.affected_entry_count",
                    affected_entry_count,
                ),
            ],
        );
        match patch_result {
            Ok(changed) => changed,
            Err(()) => {
                crate::profile_counter!(
                    "runtime",
                    "ui.surface_projected_hit.patch_fallback_count",
                    1
                );
                self.rebuild(&base_index.grid, projections)
            }
        }
    }

    fn reindex_entries(&mut self) {
        self.entry_cells.clear();
        self.entry_indices.clear();
        for (entry_index, entry) in self.grid.entries.iter().enumerate() {
            self.entry_indices.insert(entry.node_id, entry_index);
        }
        for (cell_index, cell) in self.grid.cells.iter().enumerate() {
            for entry_index in &cell.entries {
                if let Some(entry) = self.grid.entries.get(*entry_index) {
                    self.entry_cells
                        .entry(entry.node_id)
                        .or_default()
                        .push(cell_index);
                }
            }
        }
    }
}

fn record_projected_hit_persistent_cow(
    entry_stats: UiPersistentSequenceCowStats,
    cell_stats: UiPersistentSequenceCowStats,
    cell_membership_clone_count: usize,
) {
    crate::profile_counter!(
        "runtime",
        "ui.surface_projected_hit.persistent_entry_item_clone_count",
        entry_stats.cloned_item_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.surface_projected_hit.persistent_entry_segment_clone_count",
        entry_stats.cloned_segment_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.surface_projected_hit.persistent_cell_item_clone_count",
        cell_stats.cloned_item_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.surface_projected_hit.persistent_cell_segment_clone_count",
        cell_stats.cloned_segment_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.surface_projected_hit.persistent_cell_membership_clone_count",
        cell_membership_clone_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.surface_projected_hit.persistent_directory_node_clone_count",
        entry_stats
            .cloned_directory_node_count
            .saturating_add(cell_stats.cloned_directory_node_count)
    );
}

impl UiSurface {
    pub(super) fn rebuild_projected_hit_test(&mut self) -> bool {
        let projections = self.popup_hit_test_projections();
        self.projected_hit_test
            .rebuild(&self.hit_test.grid, &projections)
    }

    pub(super) fn synchronize_projected_hit_test(
        &mut self,
        changed_node_ids: &BTreeSet<UiNodeId>,
        base_grid_rebuilt: bool,
    ) -> bool {
        if !base_grid_rebuilt {
            self.hit_test.ensure_entry_lookup();
        }
        let projections = self.popup_hit_test_projections();
        self.projected_hit_test.synchronize(
            &self.hit_test,
            &projections,
            changed_node_ids,
            base_grid_rebuilt,
        )
    }

    pub(super) fn patch_projected_hit_test_strict(
        &mut self,
        changed_node_ids: &BTreeSet<UiNodeId>,
    ) -> Result<bool, ()> {
        self.hit_test.ensure_entry_lookup();
        let projections = self.popup_hit_test_projections();
        self.projected_hit_test
            .patch(&self.hit_test, &projections, changed_node_ids)
    }

    fn popup_hit_test_projections(&self) -> Vec<UiHitTestProjection> {
        self.input
            .popup_stack
            .iter()
            .enumerate()
            .filter_map(|(stack_order, popup)| {
                let popup_root = popup.popup_node?;
                let arranged = self.arranged_node(popup_root)?;
                let (target_frame, target_clip) = if self.popup_uses_runtime_anchor(popup_root) {
                    self.rendered_popup_background(popup_root, arranged)
                        .map(|(_, command)| (Some(command.frame), command.clip_frame))
                        .unwrap_or((None, None))
                } else {
                    (Some(arranged.frame), Some(arranged.clip_frame))
                };
                Some(UiHitTestProjection {
                    popup_root,
                    source_frame: arranged.frame,
                    target_frame,
                    target_clip,
                    stack_order,
                })
            })
            .collect()
    }
}

pub(super) fn hit_test_surface_frame_with_query_using_index(
    surface_frame: &UiSurfaceFrame,
    query: UiHitTestQuery,
    hit_test_index: &UiHitTestIndex,
) -> UiHitTestResult {
    hit_test_index.hit_test_owned_grid_arranged_with_query(
        &surface_frame.hit_grid,
        &surface_frame.arranged_tree,
        query,
    )
}

pub fn hit_test_surface_frame(surface_frame: &UiSurfaceFrame, point: UiPoint) -> UiHitTestResult {
    hit_test_surface_frame_with_query(surface_frame, UiHitTestQuery::new(point))
}

pub fn hit_test_surface_frame_with_query(
    surface_frame: &UiSurfaceFrame,
    query: UiHitTestQuery,
) -> UiHitTestResult {
    let hit_test_index = UiHitTestIndex::default();
    hit_test_surface_frame_with_query_using_index(surface_frame, query, &hit_test_index)
}

fn projection_roots(projections: &[UiHitTestProjection]) -> Vec<UiNodeId> {
    projections
        .iter()
        .map(|projection| projection.popup_root)
        .collect()
}

fn projection_by_root(
    projections: &[UiHitTestProjection],
) -> BTreeMap<UiNodeId, &UiHitTestProjection> {
    projections
        .iter()
        .map(|projection| (projection.popup_root, projection))
        .collect()
}

fn project_entry(
    grid: &UiHitTestGrid,
    entry: &UiHitTestEntry,
    projection_by_root: &BTreeMap<UiNodeId, &UiHitTestProjection>,
    overlay_z_base: i32,
    projected_order_keys: &BTreeMap<UiNodeId, (i32, u64)>,
) -> (UiHitTestEntry, bool) {
    let Some(projection) = projection_for_entry(grid, entry, projection_by_root) else {
        return (entry.clone(), false);
    };
    let Some(target_frame) = projection.target_frame else {
        return (
            inactive_projected_entry(entry, overlay_z_base, projected_order_keys),
            true,
        );
    };
    let projected_frame = project_frame(entry.frame, projection.source_frame, target_frame);
    let projected_clip = project_frame(entry.clip_frame, projection.source_frame, target_frame);
    let target_clip = projection.target_clip.unwrap_or(target_frame);
    let Some(clip_frame) = projected_frame
        .intersection(projected_clip)
        .and_then(|clip| clip.intersection(target_clip))
        .filter(|clip| frame_has_area(*clip))
    else {
        return (
            inactive_projected_entry(entry, overlay_z_base, projected_order_keys),
            true,
        );
    };
    let mut projected = entry.clone();
    projected.frame = projected_frame;
    projected.clip_frame = clip_frame;
    apply_projected_order(&mut projected, overlay_z_base, projected_order_keys);
    (projected, true)
}

fn inactive_projected_entry(
    entry: &UiHitTestEntry,
    overlay_z_base: i32,
    projected_order_keys: &BTreeMap<UiNodeId, (i32, u64)>,
) -> UiHitTestEntry {
    let mut projected = entry.clone();
    projected.frame = UiFrame::default();
    projected.clip_frame = UiFrame::default();
    apply_projected_order(&mut projected, overlay_z_base, projected_order_keys);
    projected
}

fn apply_projected_order(
    entry: &mut UiHitTestEntry,
    overlay_z_base: i32,
    projected_order_keys: &BTreeMap<UiNodeId, (i32, u64)>,
) {
    let (z_index, paint_order) = projected_order_keys
        .get(&entry.node_id)
        .copied()
        .unwrap_or((overlay_z_base, entry.paint_order));
    entry.z_index = z_index;
    entry.paint_order = paint_order;
}

struct UiProjectionOrderPlan {
    order_keys: BTreeMap<UiNodeId, (i32, u64)>,
    popup_roots: BTreeMap<UiNodeId, UiNodeId>,
    source_sort_keys: BTreeMap<UiNodeId, (i32, u64)>,
}

fn projection_order_plan(
    base_grid: &UiHitTestGrid,
    projection_by_root: &BTreeMap<UiNodeId, &UiHitTestProjection>,
    overlay_z_base: i32,
) -> UiProjectionOrderPlan {
    let mut projected_entries = base_grid
        .entries
        .iter()
        .filter_map(|entry| {
            projection_for_entry(base_grid, entry, projection_by_root).map(|projection| {
                (
                    projection.stack_order,
                    entry.z_index,
                    entry.paint_order,
                    entry.node_id,
                    projection.popup_root,
                )
            })
        })
        .collect::<Vec<_>>();
    projected_entries.sort_unstable();
    let first_projected_paint_order = base_grid
        .entries
        .iter()
        .filter(|entry| entry.z_index == overlay_z_base)
        .map(|entry| entry.paint_order)
        .max()
        .unwrap_or_default()
        .saturating_add(1);
    let mut order_keys = BTreeMap::new();
    let mut popup_roots = BTreeMap::new();
    let mut source_sort_keys = BTreeMap::new();
    for (rank, (_, source_z, source_paint_order, node_id, popup_root)) in
        projected_entries.into_iter().enumerate()
    {
        let rank = i32::try_from(rank).unwrap_or(i32::MAX);
        order_keys.insert(
            node_id,
            (
                overlay_z_base.saturating_add(rank),
                first_projected_paint_order.saturating_add(u64::try_from(rank).unwrap_or(u64::MAX)),
            ),
        );
        popup_roots.insert(node_id, popup_root);
        source_sort_keys.insert(node_id, (source_z, source_paint_order));
    }
    UiProjectionOrderPlan {
        order_keys,
        popup_roots,
        source_sort_keys,
    }
}

fn projection_for_entry<'a>(
    grid: &UiHitTestGrid,
    entry: &UiHitTestEntry,
    projection_by_root: &'a BTreeMap<UiNodeId, &UiHitTestProjection>,
) -> Option<&'a UiHitTestProjection> {
    find_bubble_route_value(grid, entry, projection_by_root)
}

fn project_frame(frame: UiFrame, source: UiFrame, target: UiFrame) -> UiFrame {
    let (x, width) = project_axis(
        frame.x,
        frame.width,
        source.x,
        source.width,
        target.x,
        target.width,
    );
    let (y, height) = project_axis(
        frame.y,
        frame.height,
        source.y,
        source.height,
        target.y,
        target.height,
    );
    UiFrame::new(x, y, width, height)
}

fn project_axis(
    origin: f32,
    extent: f32,
    source_origin: f32,
    source_extent: f32,
    target_origin: f32,
    target_extent: f32,
) -> (f32, f32) {
    if source_extent > 0.0 && target_extent >= 0.0 {
        let scale = target_extent / source_extent;
        (
            target_origin + (origin - source_origin) * scale,
            extent * scale,
        )
    } else {
        (origin - source_origin + target_origin, extent)
    }
}

fn build_projected_grid(
    base_grid: &UiHitTestGrid,
    mut entries: Vec<UiHitTestEntry>,
) -> UiHitTestGrid {
    entries.sort_by_key(projected_entry_sort_key);
    let content_bounds = entries
        .iter()
        .filter(|entry| frame_has_area(entry.clip_frame))
        .map(|entry| entry.clip_frame)
        .reduce(union_frames)
        .unwrap_or_default();
    let cell_size = if base_grid.cell_size.is_finite() && base_grid.cell_size > 0.0 {
        base_grid.cell_size
    } else {
        UiHitTestGrid::default().cell_size
    };
    let combined_bounds = match (
        frame_has_area(base_grid.bounds),
        frame_has_area(content_bounds),
    ) {
        (true, true) => union_frames(base_grid.bounds, content_bounds),
        (true, false) => base_grid.bounds,
        (false, true) => content_bounds,
        (false, false) => UiFrame::default(),
    };
    let bounds = hit_grid_capacity_bounds(combined_bounds, cell_size);
    if !frame_has_area(bounds) {
        return UiHitTestGrid {
            bounds,
            cell_size,
            scope: base_grid.scope.clone(),
            route_nodes: base_grid.route_nodes.clone(),
            entries: entries.into(),
            ..UiHitTestGrid::default()
        };
    }
    let (columns, rows, cell_size) = bounded_hit_grid_dimensions(bounds, &entries, cell_size);
    let cell_count = (columns as usize)
        .checked_mul(rows as usize)
        .expect("hit grid dimensions are bounded");
    let mut cells = vec![UiHitTestCell::default(); cell_count];
    for (entry_index, entry) in entries.iter().enumerate() {
        for cell_index in
            bounded_cells_for_frame(bounds, columns, rows, cell_size, entry.clip_frame)
        {
            cells[cell_index].entries.push(entry_index);
        }
    }
    UiHitTestGrid {
        bounds,
        cell_size,
        columns,
        rows,
        scope: base_grid.scope.clone(),
        route_nodes: base_grid.route_nodes.clone(),
        entries: entries.into(),
        cells: cells.into(),
    }
}

fn frame_has_area(frame: UiFrame) -> bool {
    frame_is_finite_positive(frame)
}

fn frame_is_contained(bounds: UiFrame, frame: UiFrame) -> bool {
    frame.x >= bounds.x
        && frame.y >= bounds.y
        && frame.right() <= bounds.right()
        && frame.bottom() <= bounds.bottom()
}

fn union_frames(left: UiFrame, right: UiFrame) -> UiFrame {
    let x = left.x.min(right.x);
    let y = left.y.min(right.y);
    let right_edge = left.right().max(right.right());
    let bottom_edge = left.bottom().max(right.bottom());
    UiFrame::new(x, y, right_edge - x, bottom_edge - y)
}

fn projected_entry_sort_key(entry: &UiHitTestEntry) -> (i32, u64, UiNodeId) {
    (entry.z_index, entry.paint_order, entry.node_id)
}

#[cfg(test)]
#[path = "frame_hit_test/tests.rs"]
mod projected_grid_tests;

pub fn debug_hit_test_surface_frame(
    surface_frame: &UiSurfaceFrame,
    point: UiPoint,
) -> UiHitTestDebugDump {
    debug_hit_test_surface_frame_with_query(surface_frame, UiHitTestQuery::new(point))
}

pub fn debug_hit_test_surface_frame_with_query(
    surface_frame: &UiSurfaceFrame,
    query: UiHitTestQuery,
) -> UiHitTestDebugDump {
    let point = query.hit_point();
    let hit = hit_test_surface_frame_with_query(surface_frame, query.clone());
    let mut rejected = Vec::new();
    if query.coordinate_space == UiHitCoordinateSpace::World && !query.has_projected_world_hit() {
        rejected.push(UiHitTestReject {
            node_id: Default::default(),
            control_id: None,
            reason: UiHitTestRejectReason::WorldHitUnavailable,
            message: "world hit query has no finite ray plus surface-local projection".to_string(),
        });
    } else if !query.uses_surface_coordinates() {
        rejected.push(UiHitTestReject {
            node_id: Default::default(),
            control_id: None,
            reason: UiHitTestRejectReason::UnsupportedCoordinateSpace,
            message: "hit query was not projected into surface coordinates".to_string(),
        });
    } else if !surface_frame.hit_grid.scope.accepts_query(&query.scope) {
        rejected.push(UiHitTestReject {
            node_id: Default::default(),
            control_id: None,
            reason: UiHitTestRejectReason::ScopeMismatch,
            message: "hit query scope does not match the surface hit grid scope".to_string(),
        });
    }
    for node in &surface_frame.arranged_tree.nodes {
        if !node.frame.contains_point(point) {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::OutsideFrame,
                message: "point is outside the arranged frame".to_string(),
            });
        } else if !node.clip_frame.contains_point(point) {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::OutsideClip,
                message: "point is outside the effective clip frame".to_string(),
            });
        } else if !is_arranged_child_hit_path_visible(&surface_frame.arranged_tree, node.node_id)
            .unwrap_or(false)
        {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::VisibilityFiltered,
                message: "node or ancestor visibility excludes hit testing".to_string(),
            });
        } else if !node.enabled {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::Disabled,
                message: "node is disabled".to_string(),
            });
        } else if arranged_effective_input_policy(&surface_frame.arranged_tree, node.node_id)
            .is_ok_and(|policy| policy == UiInputPolicy::Ignore)
        {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::InputPolicyIgnore,
                message: "effective input policy ignores pointer input".to_string(),
            });
        } else if !node.supports_pointer() {
            rejected.push(UiHitTestReject {
                node_id: node.node_id,
                control_id: node.control_id.clone(),
                reason: UiHitTestRejectReason::NotPointerTarget,
                message: "node does not declare pointer interaction support".to_string(),
            });
        }
    }
    UiHitTestDebugDump {
        tree_id: surface_frame.tree_id.clone(),
        point,
        hit_stack: hit.stacked,
        hit_path: hit.path,
        inspected: surface_frame.arranged_tree.nodes.len(),
        rejected,
    }
}
