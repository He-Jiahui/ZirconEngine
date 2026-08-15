use std::collections::{BTreeMap, BTreeSet};

use crate::ui::tree::{UiHitTestIndex, UiHitTestResult};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiPoint},
    surface::{
        UiArrangedTree, UiHitCoordinateSpace, UiHitTestCell, UiHitTestDebugDump, UiHitTestEntry,
        UiHitTestGrid, UiHitTestQuery, UiHitTestReject, UiHitTestRejectReason, UiSurfaceFrame,
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
    projected_paint_orders: BTreeMap<UiNodeId, u64>,
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

    fn rebuild(&mut self, base_grid: &UiHitTestGrid, projections: &[UiHitTestProjection]) {
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
                    entry,
                    &projection_by_root,
                    self.overlay_z_base,
                    &order_plan.paint_orders,
                );
                if projected {
                    self.projected_node_ids.insert(entry.node_id);
                }
                entry
            })
            .collect();
        self.projected_paint_orders = order_plan.paint_orders;
        self.projected_popup_roots = order_plan.popup_roots;
        self.projected_source_sort_keys = order_plan.source_sort_keys;
        self.grid = build_projected_grid(base_grid, entries);
        self.initialized = true;
        self.reindex_entries();
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
                        projection_for_entry(base_entry, &projection_by_root)
                            .map(|projection| projection.popup_root);
                    if self.projected_popup_roots.get(&node_id).copied() != current_projection_root
                        || current_projection_root.is_some()
                            && self.projected_source_sort_keys.get(&node_id).copied()
                                != Some((base_entry.z_index, base_entry.paint_order))
                    {
                        return Err(());
                    }
                    let (next_entry, _) = project_entry(
                        base_entry,
                        &projection_by_root,
                        self.overlay_z_base,
                        &self.projected_paint_orders,
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
                    let next_cells = projected_cells_for_frame(
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
        for (entry_index, next_entry, previous_cells, next_cells) in updates {
            if self.grid.entries.get(entry_index) == Some(&next_entry)
                && previous_cells == next_cells
            {
                continue;
            }
            for cell_index in previous_cells {
                if let Some(cell) = self.grid.cells.get_mut(cell_index) {
                    cell.entries.retain(|candidate| *candidate != entry_index);
                }
            }
            let node_id = next_entry.node_id;
            if let Some(entry) = self.grid.entries.get_mut(entry_index) {
                *entry = next_entry;
            }
            self.entry_cells.insert(node_id, next_cells.clone());
            for cell_index in next_cells {
                if let Some(cell) = self.grid.cells.get_mut(cell_index) {
                    let insertion_index = cell
                        .entries
                        .partition_point(|candidate| *candidate <= entry_index);
                    cell.entries.insert(insertion_index, entry_index);
                }
            }
        }
        Ok(changed)
    }

    fn synchronize(
        &mut self,
        base_index: &UiHitTestIndex,
        projections: &[UiHitTestProjection],
        changed_node_ids: &BTreeSet<UiNodeId>,
        base_grid_rebuilt: bool,
    ) {
        if base_grid_rebuilt
            || self
                .patch(base_index, projections, changed_node_ids)
                .is_err()
        {
            self.rebuild(&base_index.grid, projections);
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

impl UiSurface {
    pub(super) fn rebuild_projected_hit_test(&mut self) {
        let projections = self.popup_hit_test_projections();
        self.projected_hit_test
            .rebuild(&self.hit_test.grid, &projections);
    }

    pub(super) fn synchronize_projected_hit_test(
        &mut self,
        changed_node_ids: &BTreeSet<UiNodeId>,
        base_grid_rebuilt: bool,
    ) {
        if !base_grid_rebuilt {
            self.hit_test.ensure_entry_lookup();
        }
        let projections = self.popup_hit_test_projections();
        self.projected_hit_test.synchronize(
            &self.hit_test,
            &projections,
            changed_node_ids,
            base_grid_rebuilt,
        );
    }

    fn popup_hit_test_projections(&self) -> Vec<UiHitTestProjection> {
        self.input
            .popup_stack
            .iter()
            .enumerate()
            .filter_map(|(stack_order, popup)| {
                let popup_root = popup.popup_node?;
                let arranged = self.arranged_node(popup_root)?;
                let (target_frame, target_clip) = if self.popup_uses_control_anchor(popup_root) {
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

pub(super) fn hit_test_projected_grid_with_query(
    grid: &UiHitTestGrid,
    arranged_tree: &UiArrangedTree,
    query: UiHitTestQuery,
) -> UiHitTestResult {
    UiHitTestIndex::hit_test_grid_arranged_with_query(grid, arranged_tree, query)
}

pub fn hit_test_surface_frame(surface_frame: &UiSurfaceFrame, point: UiPoint) -> UiHitTestResult {
    hit_test_surface_frame_with_query(surface_frame, UiHitTestQuery::new(point))
}

pub fn hit_test_surface_frame_with_query(
    surface_frame: &UiSurfaceFrame,
    query: UiHitTestQuery,
) -> UiHitTestResult {
    hit_test_projected_grid_with_query(&surface_frame.hit_grid, &surface_frame.arranged_tree, query)
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
    entry: &UiHitTestEntry,
    projection_by_root: &BTreeMap<UiNodeId, &UiHitTestProjection>,
    overlay_z_base: i32,
    projected_paint_orders: &BTreeMap<UiNodeId, u64>,
) -> (UiHitTestEntry, bool) {
    let Some(projection) = projection_for_entry(entry, projection_by_root) else {
        return (entry.clone(), false);
    };
    let Some(target_frame) = projection.target_frame else {
        return (
            inactive_projected_entry(entry, overlay_z_base, projected_paint_orders),
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
            inactive_projected_entry(entry, overlay_z_base, projected_paint_orders),
            true,
        );
    };
    let mut projected = entry.clone();
    projected.frame = projected_frame;
    projected.clip_frame = clip_frame;
    apply_projected_order(&mut projected, overlay_z_base, projected_paint_orders);
    (projected, true)
}

fn inactive_projected_entry(
    entry: &UiHitTestEntry,
    overlay_z_base: i32,
    projected_paint_orders: &BTreeMap<UiNodeId, u64>,
) -> UiHitTestEntry {
    let mut projected = entry.clone();
    projected.frame = UiFrame::default();
    projected.clip_frame = UiFrame::default();
    apply_projected_order(&mut projected, overlay_z_base, projected_paint_orders);
    projected
}

fn apply_projected_order(
    entry: &mut UiHitTestEntry,
    overlay_z_base: i32,
    projected_paint_orders: &BTreeMap<UiNodeId, u64>,
) {
    entry.z_index = overlay_z_base;
    entry.paint_order = projected_paint_orders
        .get(&entry.node_id)
        .copied()
        .unwrap_or(entry.paint_order);
}

struct UiProjectionOrderPlan {
    paint_orders: BTreeMap<UiNodeId, u64>,
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
            projection_for_entry(entry, projection_by_root).map(|projection| {
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
    let mut paint_orders = BTreeMap::new();
    let mut popup_roots = BTreeMap::new();
    let mut source_sort_keys = BTreeMap::new();
    for (rank, (_, source_z, source_paint_order, node_id, popup_root)) in
        projected_entries.into_iter().enumerate()
    {
        paint_orders.insert(
            node_id,
            first_projected_paint_order.saturating_add(u64::try_from(rank).unwrap_or(u64::MAX)),
        );
        popup_roots.insert(node_id, popup_root);
        source_sort_keys.insert(node_id, (source_z, source_paint_order));
    }
    UiProjectionOrderPlan {
        paint_orders,
        popup_roots,
        source_sort_keys,
    }
}

fn projection_for_entry<'a>(
    entry: &UiHitTestEntry,
    projection_by_root: &'a BTreeMap<UiNodeId, &UiHitTestProjection>,
) -> Option<&'a UiHitTestProjection> {
    entry
        .bubble_route
        .iter()
        .find_map(|node_id| projection_by_root.get(node_id).copied())
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
    let bounds = entries
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
    if !frame_has_area(bounds) {
        return UiHitTestGrid {
            bounds,
            cell_size,
            scope: base_grid.scope.clone(),
            entries,
            ..UiHitTestGrid::default()
        };
    }
    let columns = (bounds.width / cell_size).ceil().max(1.0) as u32;
    let rows = (bounds.height / cell_size).ceil().max(1.0) as u32;
    let mut cells = vec![UiHitTestCell::default(); (columns * rows) as usize];
    for (entry_index, entry) in entries.iter().enumerate() {
        for cell_index in
            projected_cells_for_frame(bounds, columns, rows, cell_size, entry.clip_frame)
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
        entries,
        cells,
    }
}

fn projected_cells_for_frame(
    bounds: UiFrame,
    columns: u32,
    rows: u32,
    cell_size: f32,
    frame: UiFrame,
) -> Vec<usize> {
    if columns == 0 || rows == 0 || !frame_has_area(frame) || frame.intersection(bounds).is_none() {
        return Vec::new();
    }
    let left = ((frame.x - bounds.x) / cell_size).floor().max(0.0) as u32;
    let top = ((frame.y - bounds.y) / cell_size).floor().max(0.0) as u32;
    let right = ((frame.right() - bounds.x) / cell_size)
        .floor()
        .max(0.0)
        .min((columns - 1) as f32) as u32;
    let bottom = ((frame.bottom() - bounds.y) / cell_size)
        .floor()
        .max(0.0)
        .min((rows - 1) as f32) as u32;
    let mut cells = Vec::new();
    for row in top..=bottom {
        for column in left..=right {
            cells.push((row * columns + column) as usize);
        }
    }
    cells
}

fn frame_has_area(frame: UiFrame) -> bool {
    frame.width > 0.0 && frame.height > 0.0
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
mod projected_grid_tests {
    use super::*;

    #[test]
    fn incremental_patch_source_does_not_scan_all_base_entries() {
        let source = include_str!("frame_hit_test.rs");
        let patch_body = source
            .split_once("    fn patch(")
            .and_then(|(_, remainder)| remainder.split_once("\n    fn synchronize("))
            .map(|(body, _)| body)
            .expect("projected hit-test patch body should remain source-guardable");
        let compact_patch = patch_body
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        let forbidden_global_scan = ["base_index", ".grid", ".entries", ".iter()"].concat();

        assert!(!compact_patch.contains(&forbidden_global_scan));
    }

    #[test]
    fn affine_projection_maps_frame_and_clip_with_non_uniform_scale() {
        let source = UiFrame::new(10.0, 20.0, 40.0, 20.0);
        let target = UiFrame::new(100.0, 200.0, 80.0, 60.0);

        assert_eq!(
            project_frame(UiFrame::new(20.0, 25.0, 10.0, 5.0), source, target),
            UiFrame::new(120.0, 215.0, 20.0, 15.0)
        );
        assert_eq!(
            project_frame(UiFrame::new(15.0, 22.0, 20.0, 10.0), source, target),
            UiFrame::new(110.0, 206.0, 40.0, 30.0)
        );
    }

    #[test]
    fn incremental_z_crossing_overlay_base_falls_back_to_projected_rebuild() {
        let popup_root = UiNodeId::new(10);
        let popup_entry = hit_entry(UiNodeId::new(11), popup_root, 1, 0);
        let mut ordinary_entry = hit_entry(UiNodeId::new(30), UiNodeId::new(30), 0, 0);
        let base_grid = build_projected_grid(
            &UiHitTestGrid::default(),
            vec![ordinary_entry.clone(), popup_entry.clone()],
        );
        let frame = UiFrame::new(0.0, 0.0, 10.0, 10.0);
        let projections = [UiHitTestProjection {
            popup_root,
            source_frame: frame,
            target_frame: Some(frame),
            target_clip: Some(frame),
            stack_order: 0,
        }];
        let mut projected = UiProjectedHitTestIndex::default();
        projected.rebuild(&base_grid, &projections);
        assert_eq!(projected.overlay_z_base, 2);

        ordinary_entry.z_index = 100;
        let changed_node_ids = BTreeSet::from([ordinary_entry.node_id]);
        let updated_base = UiHitTestIndex::from_grid(build_projected_grid(
            &base_grid,
            vec![ordinary_entry, popup_entry.clone()],
        ));
        projected.synchronize(&updated_base, &projections, &changed_node_ids, false);

        assert_eq!(projected.overlay_z_base, 101);
        let hit = hit_test_projected_grid_with_query(
            &projected.grid,
            &UiArrangedTree::default(),
            UiHitTestQuery::new(UiPoint::new(5.0, 5.0)),
        );
        assert_eq!(hit.top_hit, Some(popup_entry.node_id));
    }

    #[test]
    fn base_full_rebuild_refreshes_same_count_non_projected_entries() {
        let popup_root = UiNodeId::new(10);
        let popup_entry = hit_entry(UiNodeId::new(11), popup_root, 5, 0);
        let mut ordinary_entry = hit_entry(UiNodeId::new(30), UiNodeId::new(30), 0, 0);
        ordinary_entry.frame = UiFrame::new(20.0, 0.0, 10.0, 10.0);
        ordinary_entry.clip_frame = ordinary_entry.frame;
        let base_grid = build_projected_grid(
            &UiHitTestGrid::default(),
            vec![popup_entry.clone(), ordinary_entry.clone()],
        );
        let frame = UiFrame::new(0.0, 0.0, 10.0, 10.0);
        let projections = [UiHitTestProjection {
            popup_root,
            source_frame: frame,
            target_frame: Some(UiFrame::new(100.0, 0.0, 10.0, 10.0)),
            target_clip: Some(UiFrame::new(100.0, 0.0, 10.0, 10.0)),
            stack_order: 0,
        }];
        let mut projected = UiProjectedHitTestIndex::default();
        projected.rebuild(&base_grid, &projections);

        ordinary_entry.frame = UiFrame::new(40.0, 0.0, 10.0, 10.0);
        ordinary_entry.clip_frame = ordinary_entry.frame;
        ordinary_entry.bubble_route = vec![ordinary_entry.node_id, UiNodeId::new(31)];
        let rebuilt_base = UiHitTestIndex::from_grid(build_projected_grid(
            &base_grid,
            vec![popup_entry, ordinary_entry.clone()],
        ));

        projected.synchronize(&rebuilt_base, &projections, &BTreeSet::new(), true);

        let refreshed = projected
            .grid
            .entries
            .iter()
            .find(|entry| entry.node_id == ordinary_entry.node_id)
            .expect("same-count base rebuild must keep the ordinary entry");
        assert_eq!(refreshed.frame, ordinary_entry.frame);
        assert_eq!(refreshed.clip_frame, ordinary_entry.clip_frame);
        assert_eq!(refreshed.bubble_route, ordinary_entry.bubble_route);
    }

    #[test]
    fn incremental_projection_refreshes_rendered_target_clip() {
        let popup_root = UiNodeId::new(10);
        let popup_entry = hit_entry(UiNodeId::new(11), popup_root, 5, 0);
        let base_grid = build_projected_grid(&UiHitTestGrid::default(), vec![popup_entry.clone()]);
        let base_index = UiHitTestIndex::from_grid(base_grid.clone());
        let source_frame = UiFrame::new(0.0, 0.0, 10.0, 10.0);
        let mut projected = UiProjectedHitTestIndex::default();
        projected.rebuild(
            &base_grid,
            &[UiHitTestProjection {
                popup_root,
                source_frame,
                target_frame: Some(source_frame),
                target_clip: Some(source_frame),
                stack_order: 0,
            }],
        );

        let clipped_frame = UiFrame::new(2.0, 2.0, 4.0, 4.0);
        projected.synchronize(
            &base_index,
            &[UiHitTestProjection {
                popup_root,
                source_frame,
                target_frame: Some(source_frame),
                target_clip: Some(clipped_frame),
                stack_order: 0,
            }],
            &BTreeSet::new(),
            false,
        );

        let refreshed = projected
            .grid
            .entries
            .iter()
            .find(|entry| entry.node_id == popup_entry.node_id)
            .expect("projected popup entry should remain indexed");
        assert_eq!(refreshed.frame, source_frame);
        assert_eq!(refreshed.clip_frame, clipped_frame);
        assert_eq!(
            hit_test_projected_grid_with_query(
                &projected.grid,
                &UiArrangedTree::default(),
                UiHitTestQuery::new(UiPoint::new(1.0, 1.0)),
            )
            .top_hit,
            None
        );
        assert_eq!(
            hit_test_projected_grid_with_query(
                &projected.grid,
                &UiArrangedTree::default(),
                UiHitTestQuery::new(UiPoint::new(3.0, 3.0)),
            )
            .top_hit,
            Some(popup_entry.node_id)
        );
    }

    #[test]
    fn projected_order_preserves_inner_z_and_places_next_popup_above_entire_subtree() {
        let first_popup = UiNodeId::new(10);
        let second_popup = UiNodeId::new(20);
        let low_z_high_paint = hit_entry(UiNodeId::new(11), first_popup, 5, 100);
        let high_z_low_paint = hit_entry(UiNodeId::new(12), first_popup, 6, 0);
        let next_popup_low_z = hit_entry(UiNodeId::new(21), second_popup, -100, 0);
        let base_grid = UiHitTestGrid {
            entries: vec![
                low_z_high_paint.clone(),
                high_z_low_paint.clone(),
                next_popup_low_z.clone(),
            ],
            ..UiHitTestGrid::default()
        };
        let frame = UiFrame::new(0.0, 0.0, 10.0, 10.0);
        let projections = [
            UiHitTestProjection {
                popup_root: first_popup,
                source_frame: frame,
                target_frame: Some(frame),
                target_clip: Some(frame),
                stack_order: 0,
            },
            UiHitTestProjection {
                popup_root: second_popup,
                source_frame: frame,
                target_frame: Some(frame),
                target_clip: Some(frame),
                stack_order: 1,
            },
        ];
        let projection_by_root = projection_by_root(&projections);
        let plan = projection_order_plan(&base_grid, &projection_by_root, 7);

        assert!(
            plan.paint_orders[&low_z_high_paint.node_id]
                < plan.paint_orders[&high_z_low_paint.node_id]
        );
        assert!(
            plan.paint_orders[&high_z_low_paint.node_id]
                < plan.paint_orders[&next_popup_low_z.node_id]
        );

        let mut projected = UiProjectedHitTestIndex::default();
        projected.rebuild(&base_grid, &projections);
        let hit = hit_test_projected_grid_with_query(
            &projected.grid,
            &UiArrangedTree::default(),
            UiHitTestQuery::new(UiPoint::new(5.0, 5.0)),
        );
        assert_eq!(hit.top_hit, Some(next_popup_low_z.node_id));
        assert_eq!(
            hit.stacked,
            vec![
                next_popup_low_z.node_id,
                high_z_low_paint.node_id,
                low_z_high_paint.node_id,
            ]
        );
    }

    fn hit_entry(
        node_id: UiNodeId,
        popup_root: UiNodeId,
        z_index: i32,
        paint_order: u64,
    ) -> UiHitTestEntry {
        UiHitTestEntry {
            node_id,
            frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
            clip_frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
            z_index,
            paint_order,
            control_id: None,
            effective_input_policy: Some(UiInputPolicy::Receive),
            bubble_route: vec![node_id, popup_root],
        }
    }
}

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
