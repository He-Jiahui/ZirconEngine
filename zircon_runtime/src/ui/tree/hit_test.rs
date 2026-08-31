use serde::{Deserialize, Serialize};

mod geometry_patch;
mod query_scratch;
mod route_index;

use query_scratch::UiHitQueryScratchCell;
pub(crate) use route_index::find_bubble_route_value;
use route_index::{
    bubble_route_for_entry, build_route_nodes, patch_route_nodes, route_node_for_entry,
    route_node_index_for_node,
};

use crate::ui::surface::{arranged_node_indexed, arranged_node_indices, build_arranged_tree};
use std::collections::{BTreeMap, BTreeSet};
use zircon_runtime_interface::ui::surface::{
    UiArrangedTree, UiHitPath, UiHitTestCell, UiHitTestEntry, UiHitTestGrid, UiHitTestQuery,
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
        let route_changed = patch_route_nodes(
            &mut self.grid.route_nodes,
            arranged_tree,
            changed_node_ids,
            arranged_node_indices,
        )?;
        let entry_changed =
            self.patch_arranged_geometry(arranged_tree, changed_node_ids, arranged_node_indices)?;
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
        self.entry_cells.clear();
        self.entry_indices.clear();
        self.entry_indices.extend(
            self.grid
                .entries
                .iter()
                .enumerate()
                .map(|(index, entry)| (entry.node_id, index)),
        );
        self.entry_cells.extend(
            self.grid
                .entries
                .iter()
                .map(|entry| (entry.node_id, Vec::new())),
        );
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
            let mut stacked = Vec::new();
            let mut top_entry_index = None;
            if let Some(cell) =
                cell_index_for_point(grid, point).and_then(|cell_index| grid.cells.get(cell_index))
            {
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

        let query_scratch = query_scratch.collect(grid, point, cursor_radius);
        let mut stacked = Vec::new();
        let mut top_entry_index = None;
        let mut radius_hits = Vec::new();

        for entry_index in query_scratch.candidates.iter().copied() {
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
                .into_iter()
                .map(|(_, node_id, _entry_index)| node_id),
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
    let Some((top_entry_index, bubble_route)) = top_entry_index.and_then(|entry_index| {
        let entry = grid.entries.get(entry_index)?;
        (entry.node_id == top_hit)
            .then(|| bubble_route_for_entry(grid, entry))
            .flatten()
            .map(|bubble_route| (entry_index, bubble_route))
    }) else {
        return UiHitTestResult {
            top_hit: None,
            top_entry_index: None,
            stacked: Vec::new(),
            path: UiHitPath::from_query(query),
        };
    };

    UiHitTestResult {
        top_hit: Some(top_hit),
        top_entry_index: Some(top_entry_index),
        stacked,
        path: UiHitPath::from_bubble_route(query, Some(top_hit), bubble_route),
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
    let mut entries: Vec<_> = arranged_tree
        .draw_order
        .iter()
        .filter_map(|node_id| arranged_node_indexed(arranged_tree, node_indices, *node_id).ok())
        .filter_map(|node| {
            let route_node_index = route_node_index_for_node(node_indices, node.node_id)?;
            stable_geometry_entry(&route_nodes, node, route_node_index)
        })
        .collect();

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
) -> Vec<usize> {
    let Some((left, right, top, bottom)) =
        cell_span_for_frame(bounds, columns, rows, cell_size, frame)
    else {
        return Vec::new();
    };
    let capacity = (right - left + 1) as usize * (bottom - top + 1) as usize;
    let mut indices = Vec::with_capacity(capacity);
    for row in top..=bottom {
        for column in left..=right {
            indices.push((row * columns + column) as usize);
        }
    }
    indices
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
mod incremental_patch_tests {
    use super::*;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodePath, UiTreeId},
        surface::UiArrangedNode,
        tree::{UiPointerEvents, UiVisibility},
    };

    #[test]
    fn missing_ephemeral_lookup_requires_explicit_reindex() {
        let node_id = UiNodeId::new(1);
        let arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.lookup-reindex"),
            roots: vec![node_id].into(),
            nodes: vec![pointer_node(node_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0))].into(),
            draw_order: vec![node_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged(&arranged_tree);
        index.entry_cells.clear();
        index.entry_indices.clear();

        assert!(index.entry_by_node_id(node_id).is_none());
        index.ensure_entry_lookup();
        assert_eq!(
            index.entry_by_node_id(node_id).map(|entry| entry.node_id),
            Some(node_id)
        );
    }

    #[test]
    fn moving_entry_across_cells_keeps_one_hit_index() {
        let moving_id = UiNodeId::new(1);
        let anchor_id = UiNodeId::new(2);
        let mut arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.incremental.cross-cell"),
            roots: vec![moving_id, anchor_id].into(),
            nodes: vec![
                pointer_node(moving_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0)),
                pointer_node(anchor_id, 1, UiFrame::new(100.0, 0.0, 20.0, 20.0)),
            ]
            .into(),
            draw_order: vec![moving_id, anchor_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let node_indices = BTreeMap::from([(moving_id, 0), (anchor_id, 1)]);
        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged(&arranged_tree);

        arranged_tree.nodes[0].frame = UiFrame::new(70.0, 0.0, 20.0, 20.0);
        arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;
        assert_eq!(
            index.patch_arranged_geometry(
                &arranged_tree,
                &BTreeSet::from([moving_id]),
                &node_indices,
            ),
            Ok(true)
        );

        assert_eq!(
            index
                .hit_test_arranged(&arranged_tree, UiPoint::new(5.0, 5.0))
                .top_hit,
            None
        );
        let moved_hit = index.hit_test_arranged(&arranged_tree, UiPoint::new(75.0, 5.0));
        assert_eq!(moved_hit.top_hit, Some(moving_id));
        assert_eq!(moved_hit.stacked, vec![moving_id]);
        let moving_entry_index = index.entry_indices[&moving_id];
        assert_eq!(
            index
                .grid
                .cells
                .iter()
                .flat_map(|cell| cell.entries.iter())
                .filter(|entry_index| **entry_index == moving_entry_index)
                .count(),
            1
        );
    }

    #[test]
    fn geometry_patch_reuses_route_table() {
        let node_id = UiNodeId::new(5);
        let mut arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.geometry-route-reuse"),
            roots: vec![node_id].into(),
            nodes: vec![pointer_node(node_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0))].into(),
            draw_order: vec![node_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let node_indices = BTreeMap::from([(node_id, 0)]);
        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged(&arranged_tree);
        let route_nodes = index.grid.route_nodes.clone();

        arranged_tree.nodes[0].frame = UiFrame::new(10.0, 0.0, 20.0, 20.0);
        arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;
        assert_eq!(
            index.patch_arranged_geometry(
                &arranged_tree,
                &BTreeSet::from([node_id]),
                &node_indices,
            ),
            Ok(true)
        );
        assert!(std::sync::Arc::ptr_eq(
            &route_nodes,
            &index.grid.route_nodes
        ));
    }

    #[test]
    fn malformed_parent_route_fails_closed() {
        let parent_id = UiNodeId::new(6);
        let child_id = UiNodeId::new(7);
        let frame = UiFrame::new(0.0, 0.0, 20.0, 20.0);
        let mut parent = pointer_node(parent_id, 0, frame);
        parent.children.push(child_id);
        let mut child = pointer_node(child_id, 1, frame);
        child.parent = Some(parent_id);
        let arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.malformed-parent-route"),
            roots: vec![parent_id].into(),
            nodes: vec![parent, child].into(),
            draw_order: vec![parent_id, child_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged(&arranged_tree);
        let parent_route_index = index.grid.entries[0].route_node_index as usize;
        std::sync::Arc::make_mut(&mut index.grid.route_nodes)[parent_route_index].route_valid =
            false;

        let hit = index.hit_test_arranged(&arranged_tree, UiPoint::new(5.0, 5.0));

        assert_eq!(hit.top_hit, None);
        assert_eq!(hit.top_entry_index, None);
        assert!(hit.stacked.is_empty());
        assert!(hit.path.has_consistent_route());
    }

    #[test]
    fn self_none_excludes_the_node_but_keeps_pointer_children() {
        let parent_id = UiNodeId::new(10);
        let child_id = UiNodeId::new(11);
        let frame = UiFrame::new(0.0, 0.0, 20.0, 20.0);
        let mut parent = pointer_node(parent_id, 0, frame);
        parent.children.push(child_id);
        parent.pointer_events = UiPointerEvents::SelfNone;
        let mut child = pointer_node(child_id, 1, frame);
        child.parent = Some(parent_id);
        let arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.pointer-events.self-none"),
            roots: vec![parent_id].into(),
            nodes: vec![parent, child].into(),
            draw_order: vec![parent_id, child_id].into(),
            canvas_layers: Vec::new().into(),
        };

        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged(&arranged_tree);

        assert_eq!(
            index
                .grid
                .entries
                .iter()
                .map(|entry| entry.node_id)
                .collect::<Vec<_>>(),
            vec![child_id]
        );
    }

    #[test]
    fn none_excludes_the_entire_pointer_subtree() {
        let parent_id = UiNodeId::new(20);
        let child_id = UiNodeId::new(21);
        let frame = UiFrame::new(0.0, 0.0, 20.0, 20.0);
        let mut parent = pointer_node(parent_id, 0, frame);
        parent.children.push(child_id);
        parent.pointer_events = UiPointerEvents::None;
        let mut child = pointer_node(child_id, 1, frame);
        child.parent = Some(parent_id);
        let arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.pointer-events.none"),
            roots: vec![parent_id].into(),
            nodes: vec![parent, child].into(),
            draw_order: vec![parent_id, child_id].into(),
            canvas_layers: Vec::new().into(),
        };

        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged(&arranged_tree);

        assert!(index.grid.entries.is_empty());
    }

    #[test]
    fn hit_grid_bounds_geometry_and_cell_count_are_bounded() {
        let valid_id = UiNodeId::new(30);
        let invalid_id = UiNodeId::new(31);
        let huge_id = UiNodeId::new(32);
        let valid_frame = UiFrame::new(0.0, 0.0, 20.0, 20.0);
        let huge_frame = UiFrame::new(0.0, 0.0, 1_000_000.0, 1_000_000.0);
        let arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.bounded-grid"),
            roots: vec![valid_id, invalid_id, huge_id].into(),
            nodes: vec![
                pointer_node(valid_id, 0, valid_frame),
                pointer_node(invalid_id, 1, UiFrame::new(f32::NAN, 0.0, 20.0, 20.0)),
                pointer_node(huge_id, 2, huge_frame),
            ]
            .into(),
            draw_order: vec![valid_id, invalid_id, huge_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let node_indices = BTreeMap::from([(valid_id, 0), (invalid_id, 1), (huge_id, 2)]);
        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

        assert!(index.grid.columns > 0);
        assert!(index.grid.rows > 0);
        assert!(
            (index.grid.columns as usize) * (index.grid.rows as usize) <= HIT_GRID_MAX_CELL_COUNT
        );
        assert_eq!(index.grid.columns, 1);
        assert_eq!(index.grid.rows, 1);
        assert!(index
            .grid
            .entries
            .iter()
            .all(|entry| entry.node_id != invalid_id));
        assert_eq!(
            index
                .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
                .top_hit,
            Some(huge_id)
        );
    }

    #[test]
    fn ordinary_bounds_keep_fine_grained_cell_partitioning() {
        let first_id = UiNodeId::new(40);
        let second_id = UiNodeId::new(41);
        let arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.fine-grid"),
            roots: vec![first_id, second_id].into(),
            nodes: vec![
                pointer_node(first_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0)),
                pointer_node(second_id, 1, UiFrame::new(128.0, 0.0, 20.0, 20.0)),
            ]
            .into(),
            draw_order: vec![first_id, second_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let node_indices = BTreeMap::from([(first_id, 0), (second_id, 1)]);
        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

        assert!(index.grid.columns >= 2);
        assert_eq!(
            index
                .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
                .top_hit,
            Some(first_id)
        );
        assert_eq!(
            index
                .hit_test_arranged(&arranged_tree, UiPoint::new(138.0, 10.0))
                .top_hit,
            Some(second_id)
        );
    }

    #[test]
    fn capacity_envelope_absorbs_small_growth_and_regrids_only_at_geometric_boundaries() {
        let root_id = UiNodeId::new(50);
        let mut arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.capacity-envelope"),
            roots: vec![root_id].into(),
            nodes: vec![pointer_node(
                root_id,
                0,
                UiFrame::new(0.0, 0.0, 120.0, 60.0),
            )]
            .into(),
            draw_order: vec![root_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let node_indices = BTreeMap::from([(root_id, 0)]);
        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

        assert_eq!(index.grid.bounds, UiFrame::new(0.0, 0.0, 128.0, 64.0));
        arranged_tree.nodes[0].frame = UiFrame::new(0.0, 0.0, 121.0, 60.0);
        arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;
        assert_eq!(
            index.patch_arranged_geometry(
                &arranged_tree,
                &BTreeSet::from([root_id]),
                &node_indices,
            ),
            Ok(true)
        );
        assert_eq!(index.grid.bounds, UiFrame::new(0.0, 0.0, 128.0, 64.0));

        arranged_tree.nodes[0].frame = UiFrame::new(0.0, 0.0, 129.0, 60.0);
        arranged_tree.nodes[0].clip_frame = arranged_tree.nodes[0].frame;
        assert_eq!(
            index.patch_arranged_geometry(
                &arranged_tree,
                &BTreeSet::from([root_id]),
                &node_indices,
            ),
            Err(())
        );
    }

    fn pointer_node(node_id: UiNodeId, paint_order: u64, frame: UiFrame) -> UiArrangedNode {
        UiArrangedNode {
            node_id,
            node_path: UiNodePath::new(format!("root/{}", node_id.0)),
            parent: None,
            children: Vec::new(),
            frame,
            clip_frame: frame,
            z_index: 0,
            paint_order,
            visibility: UiVisibility::Visible,
            input_policy: UiInputPolicy::Receive,
            pointer_events: Default::default(),
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: false,
            clip_to_bounds: false,
            control_id: None,
            slot: None,
        }
    }
}

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
