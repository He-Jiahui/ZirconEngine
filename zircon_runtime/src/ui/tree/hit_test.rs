use serde::{Deserialize, Serialize};

mod cell_membership_patch;
mod geometry_patch;
mod query_scratch;
mod route_index;

use query_scratch::UiHitQueryScratchCell;
pub(crate) use route_index::find_bubble_route_value;
use route_index::{
    bubble_route_for_entry, build_route_nodes, patch_route_nodes, route_node_for_entry,
};

use crate::ui::surface::{arranged_node_indices, build_arranged_tree};
use std::collections::{BTreeMap, BTreeSet};
use zircon_runtime_interface::ui::surface::{
    UiArrangedTree, UiHitPath, UiHitRouteNode, UiHitTestCell, UiHitTestEntry, UiHitTestGrid,
    UiHitTestQuery,
};
use zircon_runtime_interface::ui::tree::{UiInputPolicy, UiTree};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiPoint},
};

const HIT_GRID_CELL_SIZE: f32 = 64.0;
const HIT_GRID_MAX_AXIS_CELLS: u32 = 128;
const HIT_GRID_MAX_CELL_COUNT: usize =
    HIT_GRID_MAX_AXIS_CELLS as usize * HIT_GRID_MAX_AXIS_CELLS as usize;
const HIT_GRID_MAX_ENTRY_CELL_COUNT: usize = 4_096;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestResult {
    pub top_hit: Option<UiNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_entry_index: Option<usize>,
    pub stacked: Vec<UiNodeId>,
    pub path: UiHitPath,
}

impl UiHitTestResult {
    pub fn top_entry<'a>(&self, grid: &'a UiHitTestGrid) -> Option<&'a UiHitTestEntry> {
        let entry = grid.entries.get(self.top_entry_index?)?;
        (Some(entry.node_id) == self.top_hit).then_some(entry)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UiHitTestIndex {
    pub grid: UiHitTestGrid,
    #[serde(default, skip_serializing, skip_deserializing)]
    entry_cells: BTreeMap<UiNodeId, Vec<usize>>,
    #[serde(default, skip_serializing, skip_deserializing)]
    entry_indices: BTreeMap<UiNodeId, usize>,
    #[serde(default, skip_serializing, skip_deserializing)]
    query_scratch: UiHitQueryScratchCell,
}

impl PartialEq for UiHitTestIndex {
    fn eq(&self, other: &Self) -> bool {
        self.grid == other.grid
    }
}

impl UiHitTestIndex {
    pub(crate) fn patch_cell_memberships<K: Ord>(
        cells: &mut zircon_runtime_interface::ui::surface::UiPersistentSequence<UiHitTestCell>,
        patches: impl IntoIterator<Item = (usize, Vec<usize>, Vec<usize>)>,
        sort_key: impl FnMut(usize) -> K,
    ) -> Result<
        (
            zircon_runtime_interface::ui::surface::UiPersistentSequenceCowStats,
            usize,
            usize,
            usize,
            usize,
            usize,
            usize,
            usize,
        ),
        (),
    > {
        let mut batch = cell_membership_patch::UiCellMembershipPatches::default();
        for (entry_index, previous_cells, next_cells) in patches {
            batch.stage(entry_index, &previous_cells, &next_cells, false);
        }
        let stats = batch.apply(cells, sort_key)?;
        Ok((
            stats.cell_cow_stats,
            stats.staged_cell_count,
            stats.published_cell_count,
            stats.source_membership_count,
            stats.staged_removal_count,
            stats.staged_addition_count,
            stats.materialized_membership_count,
            stats.replacement_buffer_count,
        ))
    }

    pub fn from_grid(grid: UiHitTestGrid) -> Self {
        let mut index = Self {
            grid,
            entry_cells: BTreeMap::new(),
            entry_indices: BTreeMap::new(),
            query_scratch: UiHitQueryScratchCell::default(),
        };
        index.reindex_entry_cells();
        index
    }

    pub fn rebuild(&mut self, tree: &UiTree) {
        let arranged_tree = build_arranged_tree(tree);
        self.rebuild_arranged(&arranged_tree);
    }

    pub fn rebuild_arranged(&mut self, arranged_tree: &UiArrangedTree) {
        let node_indices = arranged_node_indices(arranged_tree);
        self.rebuild_arranged_indexed(arranged_tree, &node_indices);
    }

    pub(crate) fn rebuild_arranged_indexed(
        &mut self,
        arranged_tree: &UiArrangedTree,
        node_indices: &BTreeMap<UiNodeId, usize>,
    ) {
        self.grid = build_hit_grid(arranged_tree, node_indices);
        self.reindex_entry_cells();
    }

    pub(crate) fn patch_arranged_input(
        &mut self,
        arranged_tree: &UiArrangedTree,
        changed_node_ids: &BTreeSet<UiNodeId>,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) -> Result<bool, ()> {
        let mut next_route_nodes = self.grid.route_nodes.clone();
        let route_changed = patch_route_nodes(
            &mut next_route_nodes,
            arranged_tree,
            changed_node_ids,
            arranged_node_indices,
        )?;
        let entry_changed = self.patch_arranged_geometry_with_routes(
            arranged_tree,
            changed_node_ids,
            arranged_node_indices,
            next_route_nodes.as_slice(),
        )?;
        self.grid.route_nodes = next_route_nodes;
        Ok(route_changed || entry_changed)
    }

    fn entry_index_by_node_id(&self, node_id: UiNodeId) -> Option<usize> {
        self.entry_indices.get(&node_id).copied()
    }

    pub(crate) fn entry_by_node_id(&self, node_id: UiNodeId) -> Option<&UiHitTestEntry> {
        self.entry_index_by_node_id(node_id)
            .and_then(|entry_index| self.grid.entries.get(entry_index))
    }

    pub(crate) fn ensure_entry_lookup(&mut self) {
        if self.entry_indices.len() != self.grid.entries.len() {
            self.reindex_entry_cells();
        }
    }

    fn reindex_entry_cells(&mut self) {
        self.entry_indices.clear();
        self.entry_indices.extend(
            self.grid
                .entries
                .iter()
                .enumerate()
                .map(|(index, entry)| (entry.node_id, index)),
        );
        let entry_indices = &self.entry_indices;
        self.entry_cells.retain(|node_id, cells| {
            if entry_indices.contains_key(node_id) {
                cells.clear();
                true
            } else {
                false
            }
        });
        for entry in self.grid.entries.iter() {
            self.entry_cells.entry(entry.node_id).or_default();
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

    pub fn hit_test(&self, tree: &UiTree, point: UiPoint) -> UiHitTestResult {
        let arranged_tree = build_arranged_tree(tree);
        self.hit_test_arranged(&arranged_tree, point)
    }

    pub fn hit_test_arranged(
        &self,
        arranged_tree: &UiArrangedTree,
        point: UiPoint,
    ) -> UiHitTestResult {
        self.hit_test_arranged_with_query(arranged_tree, UiHitTestQuery::new(point))
    }

    pub fn hit_test_arranged_with_query(
        &self,
        arranged_tree: &UiArrangedTree,
        query: UiHitTestQuery,
    ) -> UiHitTestResult {
        self.hit_test_owned_grid_arranged_with_query(&self.grid, arranged_tree, query)
    }

    pub(crate) fn hit_test_owned_grid_arranged_with_query(
        &self,
        grid: &UiHitTestGrid,
        arranged_tree: &UiArrangedTree,
        query: UiHitTestQuery,
    ) -> UiHitTestResult {
        Self::hit_test_grid_arranged_with_query_using_scratch(
            grid,
            arranged_tree,
            query,
            &self.query_scratch,
        )
    }

    #[cfg(test)]
    pub(crate) fn query_scratch_stats(&self) -> query_scratch::UiHitQueryScratchStats {
        self.query_scratch.stats()
    }

    pub fn hit_test_grid_arranged(
        grid: &UiHitTestGrid,
        arranged_tree: &UiArrangedTree,
        point: UiPoint,
    ) -> UiHitTestResult {
        Self::hit_test_grid_arranged_with_query(grid, arranged_tree, UiHitTestQuery::new(point))
    }

    pub fn hit_test_grid_arranged_with_query(
        grid: &UiHitTestGrid,
        arranged_tree: &UiArrangedTree,
        query: UiHitTestQuery,
    ) -> UiHitTestResult {
        let query_scratch = UiHitQueryScratchCell::default();
        Self::hit_test_grid_arranged_with_query_using_scratch(
            grid,
            arranged_tree,
            query,
            &query_scratch,
        )
    }

    fn hit_test_grid_arranged_with_query_using_scratch(
        grid: &UiHitTestGrid,
        _arranged_tree: &UiArrangedTree,
        query: UiHitTestQuery,
        query_scratch: &UiHitQueryScratchCell,
    ) -> UiHitTestResult {
        if !query.uses_surface_coordinates() || !grid.scope.accepts_query(&query.scope) {
            return UiHitTestResult {
                top_hit: None,
                top_entry_index: None,
                stacked: Vec::new(),
                path: UiHitPath::from_query(&query),
            };
        }
        let point = query.hit_point();
        let cursor_radius = query.sanitized_cursor_radius();
        if cursor_radius <= 0.0 {
            let cell =
                cell_index_for_point(grid, point).and_then(|cell_index| grid.cells.get(cell_index));
            let mut stacked = Vec::with_capacity(cell.map_or(0, |cell| cell.entries.len()));
            let mut top_entry_index = None;
            if let Some(cell) = cell {
                for entry_index in cell.entries.iter().rev() {
                    let Some(entry) = grid.entries.get(*entry_index) else {
                        continue;
                    };
                    let Some((frame, input_policy)) = entry_frame_and_input_policy(grid, entry)
                    else {
                        continue;
                    };
                    let clipped_frame = frame
                        .intersection(entry.clip_frame)
                        .unwrap_or(entry.clip_frame);
                    if !clipped_frame.contains_point(point) {
                        continue;
                    }
                    if input_policy == UiInputPolicy::Ignore {
                        continue;
                    }
                    top_entry_index.get_or_insert(*entry_index);
                    stacked.push(entry.node_id);
                }
            }
            return hit_result_from_stacked(grid, &query, stacked, top_entry_index);
        }

        let mut query_scratch = query_scratch.collect(grid, point, cursor_radius);
        let query_scratch = &mut *query_scratch;
        let (candidates, radius_hits) = (&query_scratch.candidates, &mut query_scratch.radius_hits);
        let mut stacked = Vec::with_capacity(candidates.len());
        let mut top_entry_index = None;

        for entry_index in candidates.iter().copied() {
            let Some(entry) = grid.entries.get(entry_index) else {
                continue;
            };
            let Some((frame, input_policy)) = entry_frame_and_input_policy(grid, entry) else {
                continue;
            };
            let clipped_frame = frame
                .intersection(entry.clip_frame)
                .unwrap_or(entry.clip_frame);
            if !frame_accepts_point(clipped_frame, point, cursor_radius) {
                continue;
            }
            if input_policy == UiInputPolicy::Ignore {
                continue;
            }
            if clipped_frame.contains_point(point) {
                top_entry_index.get_or_insert(entry_index);
                stacked.push(entry.node_id);
            } else {
                radius_hits.push((
                    distance_sq_to_frame(clipped_frame, point),
                    entry.node_id,
                    entry_index,
                ));
            }
        }
        radius_hits.sort_by(|left, right| left.0.total_cmp(&right.0));
        if top_entry_index.is_none() {
            top_entry_index = radius_hits.first().map(|(_, _, entry_index)| *entry_index);
        }
        stacked.extend(
            radius_hits
                .iter()
                .map(|(_, node_id, _entry_index)| *node_id),
        );
        hit_result_from_stacked(grid, &query, stacked, top_entry_index)
    }
}

fn stable_geometry_entry(
    route_nodes: &[zircon_runtime_interface::ui::surface::UiHitRouteNode],
    node: &zircon_runtime_interface::ui::surface::UiArrangedNode,
    route_node_index: u32,
) -> Option<UiHitTestEntry> {
    if !node.supports_pointer() {
        return None;
    }
    if !frame_is_finite_positive(node.frame) {
        crate::profile_counter!("runtime", "ui.hit_grid.invalid_geometry_entry_count", 1);
        return None;
    }
    let route = route_nodes
        .get(route_node_index as usize)
        .filter(|route| route.node_id == node.node_id)?;
    if !route.route_valid
        || !route.pointer_path_visible
        || route.effective_input_policy == UiInputPolicy::Ignore
    {
        return None;
    }
    let clip_frame = node
        .frame
        .intersection(node.clip_frame)
        .filter(|frame| frame_is_finite_positive(*frame))
        .unwrap_or_default();
    Some(UiHitTestEntry {
        node_id: node.node_id,
        frame: node.frame,
        clip_frame,
        z_index: node.z_index,
        paint_order: node.paint_order,
        control_id: node.control_id.clone(),
        route_node_index,
    })
}

fn hit_result_from_stacked(
    grid: &UiHitTestGrid,
    query: &UiHitTestQuery,
    stacked: Vec<UiNodeId>,
    top_entry_index: Option<usize>,
) -> UiHitTestResult {
    let Some(top_hit) = stacked.first().copied() else {
        return UiHitTestResult {
            top_hit: None,
            top_entry_index: None,
            stacked,
            path: UiHitPath::from_query(query),
        };
    };

    // Try to build a full propagation path for the top hit.
    if let Some((resolved_entry_index, bubble_route)) = top_entry_index.and_then(|entry_index| {
        let entry = grid.entries.get(entry_index)?;
        (entry.node_id == top_hit)
            .then(|| bubble_route_for_entry(grid, entry))
            .flatten()
            .map(|bubble_route| (entry_index, bubble_route))
    }) {
        return UiHitTestResult {
            top_hit: Some(top_hit),
            top_entry_index: Some(resolved_entry_index),
            stacked,
            path: UiHitPath::from_bubble_route(query, Some(top_hit), bubble_route),
        };
    }

    // No valid bubble route (stale route state or missing entry index); still
    // report the hit with a flat path so pointer dispatch can reach the node.
    UiHitTestResult {
        top_hit: Some(top_hit),
        top_entry_index,
        stacked,
        path: UiHitPath::from_query(query),
    }
}

fn entry_frame_and_input_policy(
    grid: &UiHitTestGrid,
    entry: &UiHitTestEntry,
) -> Option<(UiFrame, UiInputPolicy)> {
    let route = route_node_for_entry(grid, entry)?;
    Some((entry.frame, route.effective_input_policy))
}

fn build_hit_grid(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
) -> UiHitTestGrid {
    let route_nodes = build_route_nodes(arranged_tree, node_indices);
    let mut entries = Vec::with_capacity(arranged_tree.draw_order.len());
    for node_id in arranged_tree.draw_order.iter().copied() {
        let Some(node_index) = node_indices.get(&node_id).copied() else {
            continue;
        };
        let Some(node) = arranged_tree
            .nodes
            .get(node_index)
            .filter(|node| node.node_id == node_id)
        else {
            continue;
        };
        let Some(route_node_index) = u32::try_from(node_index)
            .ok()
            .filter(|index| *index != UiHitRouteNode::NO_PARENT_INDEX)
        else {
            continue;
        };
        if let Some(entry) = stable_geometry_entry(&route_nodes, node, route_node_index) {
            entries.push(entry);
        }
    }

    entries.sort_by_key(|entry| (entry.z_index, entry.paint_order, entry.node_id));
    let bounds = union_entry_bounds(&entries)
        .map(|bounds| hit_grid_capacity_bounds(bounds, HIT_GRID_CELL_SIZE))
        .unwrap_or_default();
    if entries.is_empty() || bounds.width <= 0.0 || bounds.height <= 0.0 {
        return UiHitTestGrid {
            bounds,
            cell_size: HIT_GRID_CELL_SIZE,
            columns: 0,
            rows: 0,
            scope: Default::default(),
            route_nodes,
            entries: entries.into(),
            cells: Vec::new().into(),
            ..UiHitTestGrid::default()
        };
    }

    let (columns, rows, cell_size) =
        bounded_hit_grid_dimensions(bounds, &entries, HIT_GRID_CELL_SIZE);
    let mut cells = vec![
        UiHitTestCell::default();
        (columns as usize)
            .checked_mul(rows as usize)
            .expect("hit grid dimensions are bounded")
    ];
    for (entry_index, entry) in entries.iter().enumerate() {
        if entry.clip_frame.width <= 0.0 || entry.clip_frame.height <= 0.0 {
            continue;
        }
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
        scope: Default::default(),
        route_nodes,
        entries: entries.into(),
        cells: cells.into(),
        ..UiHitTestGrid::default()
    }
}

pub(crate) fn bounded_hit_grid_dimensions(
    bounds: UiFrame,
    entries: &[UiHitTestEntry],
    minimum_cell_size: f32,
) -> (u32, u32, f32) {
    let minimum_cell_size = if minimum_cell_size.is_finite() && minimum_cell_size > 0.0 {
        minimum_cell_size.max(HIT_GRID_CELL_SIZE)
    } else {
        HIT_GRID_CELL_SIZE
    };
    let requested_cell_size = minimum_cell_size
        .max(bounds.width / HIT_GRID_MAX_AXIS_CELLS as f32)
        .max(bounds.height / HIT_GRID_MAX_AXIS_CELLS as f32);
    let columns = (bounds.width / requested_cell_size)
        .ceil()
        .clamp(1.0, HIT_GRID_MAX_AXIS_CELLS as f32) as u32;
    let rows = (bounds.height / requested_cell_size)
        .ceil()
        .clamp(1.0, HIT_GRID_MAX_AXIS_CELLS as f32) as u32;
    debug_assert!((columns as usize) * (rows as usize) <= HIT_GRID_MAX_CELL_COUNT);

    let has_wide_entry = entries.iter().any(|entry| {
        cell_count_for_frame(bounds, columns, rows, requested_cell_size, entry.clip_frame)
            > HIT_GRID_MAX_ENTRY_CELL_COUNT
    });
    if has_wide_entry {
        // Doubling a grid already capped at 128 cells per axis yields at most 64x64
        // memberships for any one entry without collapsing unrelated local geometry.
        let coarsened_cell_size = requested_cell_size * 2.0;
        let coarsened_columns = (bounds.width / coarsened_cell_size)
            .ceil()
            .clamp(1.0, HIT_GRID_MAX_AXIS_CELLS as f32) as u32;
        let coarsened_rows = (bounds.height / coarsened_cell_size)
            .ceil()
            .clamp(1.0, HIT_GRID_MAX_AXIS_CELLS as f32) as u32;
        debug_assert!(entries.iter().all(|entry| {
            cell_count_for_frame(
                bounds,
                coarsened_columns,
                coarsened_rows,
                coarsened_cell_size,
                entry.clip_frame,
            ) <= HIT_GRID_MAX_ENTRY_CELL_COUNT
        }));
        crate::profile_counter!("runtime", "ui.hit_grid.adaptive_coarsening_count", 1);
        crate::profile_counter!("runtime", "ui.hit_grid.coarse_fallback_count", 1);
        return (coarsened_columns, coarsened_rows, coarsened_cell_size);
    }
    (columns, rows, requested_cell_size)
}

fn cell_bounds_for_query(
    grid: &UiHitTestGrid,
    point: UiPoint,
    cursor_radius: f32,
) -> Option<(u32, u32, u32, u32)> {
    if grid.columns == 0 || grid.rows == 0 {
        return None;
    }
    let query_frame = UiFrame::new(
        point.x - cursor_radius,
        point.y - cursor_radius,
        cursor_radius * 2.0,
        cursor_radius * 2.0,
    );
    if query_frame.intersection(grid.bounds).is_none() {
        return None;
    }
    Some(cell_bounds_for_frame(
        grid.bounds,
        grid.columns,
        grid.rows,
        grid.cell_size,
        query_frame,
    ))
}

fn cell_index_for_point(grid: &UiHitTestGrid, point: UiPoint) -> Option<usize> {
    if grid.columns == 0
        || grid.rows == 0
        || grid.columns > HIT_GRID_MAX_AXIS_CELLS
        || grid.rows > HIT_GRID_MAX_AXIS_CELLS
        || grid.cells.len() > HIT_GRID_MAX_CELL_COUNT
        || !grid.bounds.contains_point(point)
    {
        return None;
    }
    let column = ((point.x - grid.bounds.x) / grid.cell_size).floor() as i32;
    let row = ((point.y - grid.bounds.y) / grid.cell_size).floor() as i32;
    if column < 0 || row < 0 {
        return None;
    }
    let column = (column as u32).min(grid.columns - 1);
    let row = (row as u32).min(grid.rows - 1);
    Some((row * grid.columns + column) as usize)
}

pub(crate) fn bounded_cells_for_frame(
    bounds: UiFrame,
    columns: u32,
    rows: u32,
    cell_size: f32,
    frame: UiFrame,
) -> impl Iterator<Item = usize> {
    let span = cell_span_for_frame(bounds, columns, rows, cell_size, frame);
    span.into_iter()
        .flat_map(move |(left, right, top, bottom)| {
            (top..=bottom).flat_map(move |row| {
                (left..=right).map(move |column| (row * columns + column) as usize)
            })
        })
}

fn cell_count_for_frame(
    bounds: UiFrame,
    columns: u32,
    rows: u32,
    cell_size: f32,
    frame: UiFrame,
) -> usize {
    let Some((left, right, top, bottom)) =
        cell_span_for_frame(bounds, columns, rows, cell_size, frame)
    else {
        return 0;
    };
    (right - left + 1) as usize * (bottom - top + 1) as usize
}

fn cell_span_for_frame(
    bounds: UiFrame,
    columns: u32,
    rows: u32,
    cell_size: f32,
    frame: UiFrame,
) -> Option<(u32, u32, u32, u32)> {
    if columns == 0
        || rows == 0
        || columns > HIT_GRID_MAX_AXIS_CELLS
        || rows > HIT_GRID_MAX_AXIS_CELLS
        || !cell_size.is_finite()
        || cell_size <= 0.0
        || !frame_is_finite_positive(frame)
        || frame.intersection(bounds).is_none()
    {
        return None;
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
    (left <= right && top <= bottom).then_some((left, right, top, bottom))
}

fn cell_bounds_for_frame(
    bounds: UiFrame,
    columns: u32,
    rows: u32,
    cell_size: f32,
    frame: UiFrame,
) -> (u32, u32, u32, u32) {
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
    (left, right, top, bottom)
}

fn frame_accepts_point(frame: UiFrame, point: UiPoint, radius: f32) -> bool {
    if radius <= 0.0 {
        frame.contains_point(point)
    } else {
        distance_sq_to_frame(frame, point) <= radius * radius
    }
}

fn entry_sort_key(entry: &UiHitTestEntry) -> (i32, u64, UiNodeId) {
    (entry.z_index, entry.paint_order, entry.node_id)
}

#[cfg(test)]
#[path = "tests/hit_test_incremental_patch_tests.rs"]
mod incremental_patch_tests;

fn distance_sq_to_frame(frame: UiFrame, point: UiPoint) -> f32 {
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return f32::INFINITY;
    }
    let closest_x = point.x.clamp(frame.x, frame.right());
    let closest_y = point.y.clamp(frame.y, frame.bottom());
    let dx = point.x - closest_x;
    let dy = point.y - closest_y;
    dx * dx + dy * dy
}

fn union_entry_bounds(entries: &[UiHitTestEntry]) -> Option<UiFrame> {
    let mut iter = entries
        .iter()
        .filter(|entry| frame_is_finite_positive(entry.clip_frame));
    let first = iter.next()?.clip_frame;
    let (mut left, mut top, mut right, mut bottom) =
        (first.x, first.y, first.right(), first.bottom());
    for entry in iter {
        left = left.min(entry.clip_frame.x);
        top = top.min(entry.clip_frame.y);
        right = right.max(entry.clip_frame.right());
        bottom = bottom.max(entry.clip_frame.bottom());
    }
    let bounds = UiFrame::new(left, top, right - left, bottom - top);
    frame_is_finite_positive(bounds).then_some(bounds)
}

pub(crate) fn hit_grid_capacity_bounds(bounds: UiFrame, quantum: f32) -> UiFrame {
    if !frame_is_finite_positive(bounds) || !quantum.is_finite() || quantum <= 0.0 {
        return bounds;
    }
    let (x, width) = hit_grid_capacity_axis(bounds.x, bounds.right(), quantum);
    let (y, height) = hit_grid_capacity_axis(bounds.y, bounds.bottom(), quantum);
    UiFrame::new(x, y, width, height)
}

fn hit_grid_capacity_axis(origin: f32, end: f32, quantum: f32) -> (f32, f32) {
    let capacity_origin = (origin / quantum).floor() * quantum;
    let required = (end - capacity_origin).max(quantum);
    let mut capacity = quantum;
    while capacity < required && capacity <= f32::MAX / 2.0 {
        capacity *= 2.0;
    }
    if capacity.is_finite() {
        (capacity_origin, capacity)
    } else {
        (origin, end - origin)
    }
}

pub(crate) fn frame_is_finite_positive(frame: UiFrame) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.right().is_finite()
        && frame.bottom().is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn frame_is_contained(bounds: UiFrame, frame: UiFrame) -> bool {
    frame.width >= 0.0
        && frame.height >= 0.0
        && frame.x >= bounds.x
        && frame.y >= bounds.y
        && frame.right() <= bounds.right()
        && frame.bottom() <= bounds.bottom()
}

#[cfg(test)]
#[path = "tests/hit_test_output_capacity_tests.rs"]
mod output_capacity_tests;
