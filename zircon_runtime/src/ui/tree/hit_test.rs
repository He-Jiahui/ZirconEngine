use serde::{Deserialize, Serialize};

use crate::ui::surface::{
    arranged_bubble_route, arranged_bubble_route_indexed, arranged_effective_input_policy,
    arranged_effective_input_policy_indexed, arranged_node_indexed, arranged_node_indices,
    build_arranged_tree, is_arranged_child_hit_path_visible,
    is_arranged_child_hit_path_visible_indexed,
};
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

    pub(crate) fn patch_arranged_geometry(
        &mut self,
        arranged_tree: &UiArrangedTree,
        changed_node_ids: &BTreeSet<UiNodeId>,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) -> Result<bool, ()> {
        if changed_node_ids.is_empty() {
            return Ok(false);
        }
        if (self.entry_cells.is_empty() || self.entry_indices.is_empty())
            && !self.grid.entries.is_empty()
        {
            self.reindex_entry_cells();
        }
        let mut updates = Vec::with_capacity(changed_node_ids.len());
        for node_id in changed_node_ids {
            let Some(entry_index) = self.entry_index_by_node_id(*node_id) else {
                // Entries with no positive visible area are omitted by the full
                // build. If layout makes such a node eligible, the local patch
                // cannot insert a new entry without rebuilding the index.
                let Some(node) =
                    arranged_node_for_patch(arranged_tree, arranged_node_indices, *node_id)
                else {
                    return Err(());
                };
                if hit_test_node_is_eligible(arranged_tree, arranged_node_indices, *node_id, node)?
                {
                    return Err(());
                }
                continue;
            };
            let Some(node) =
                arranged_node_for_patch(arranged_tree, arranged_node_indices, *node_id)
            else {
                return Err(());
            };
            if !hit_test_node_is_eligible(arranged_tree, arranged_node_indices, *node_id, node)? {
                return Err(());
            }
            let effective_input_policy = arranged_effective_input_policy_for_patch(
                arranged_tree,
                arranged_node_indices,
                *node_id,
            )?;
            if effective_input_policy == UiInputPolicy::Ignore {
                return Err(());
            }
            let Some(clip_frame) = node.frame.intersection(node.clip_frame) else {
                return Err(());
            };
            if !frame_is_contained(self.grid.bounds, clip_frame) {
                return Err(());
            }
            let Some(previous_cells) = self.entry_cells.get(node_id).cloned() else {
                return Err(());
            };
            let next_cells = cells_for_frame(
                self.grid.bounds,
                self.grid.columns,
                self.grid.rows,
                self.grid.cell_size,
                clip_frame,
            );
            if next_cells
                .iter()
                .any(|cell_index| self.grid.cells.get(*cell_index).is_none())
            {
                return Err(());
            }
            let bubble_route =
                arranged_bubble_route_for_patch(arranged_tree, arranged_node_indices, *node_id)?;
            updates.push((
                entry_index,
                UiHitTestEntry {
                    node_id: *node_id,
                    frame: node.frame,
                    clip_frame,
                    z_index: node.z_index,
                    paint_order: node.paint_order,
                    control_id: node.control_id.clone(),
                    effective_input_policy: Some(effective_input_policy),
                    bubble_route,
                },
                previous_cells,
                next_cells,
            ));
        }

        let changed = !updates.is_empty();
        for (entry_index, entry, previous_cells, next_cells) in updates {
            let entry_node_id = entry.node_id;
            let order_unchanged = self
                .grid
                .entries
                .get(entry_index)
                .is_some_and(|current| entry_sort_key(current) == entry_sort_key(&entry));
            if previous_cells == next_cells && order_unchanged {
                if let Some(current) = self.grid.entries.get_mut(entry_index) {
                    *current = entry;
                }
                continue;
            }
            for cell_index in previous_cells {
                if let Some(cell) = self.grid.cells.get_mut(cell_index) {
                    cell.entries.retain(|candidate| *candidate != entry_index);
                }
            }
            if let Some(current) = self.grid.entries.get_mut(entry_index) {
                *current = entry;
            }
            self.entry_cells.insert(entry_node_id, next_cells.clone());
            for cell_index in &next_cells {
                let insertion_index = self
                    .grid
                    .cells
                    .get(*cell_index)
                    .map(|cell| {
                        let key = self
                            .grid
                            .entries
                            .get(entry_index)
                            .map(entry_sort_key)
                            .unwrap_or_default();
                        cell.entries.partition_point(|candidate| {
                            self.grid
                                .entries
                                .get(*candidate)
                                .map(entry_sort_key)
                                .unwrap_or_default()
                                <= key
                        })
                    })
                    .unwrap_or_default();
                if let Some(cell) = self.grid.cells.get_mut(*cell_index) {
                    cell.entries.insert(insertion_index, entry_index);
                }
            }
        }
        Ok(changed)
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
        Self::hit_test_grid_arranged_with_query(&self.grid, arranged_tree, query)
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
                    let Some((frame, input_policy)) =
                        entry_frame_and_input_policy(entry, arranged_tree)
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
            return hit_result_from_stacked(grid, arranged_tree, &query, stacked, top_entry_index);
        }

        let entry_indices = hit_entry_indices_for_query(grid, point, cursor_radius);
        let mut stacked = Vec::new();
        let mut top_entry_index = None;
        let mut radius_hits = Vec::new();

        for entry_index in entry_indices {
            let Some(entry) = grid.entries.get(entry_index) else {
                continue;
            };
            let Some((frame, input_policy)) = entry_frame_and_input_policy(entry, arranged_tree)
            else {
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
        hit_result_from_stacked(grid, arranged_tree, &query, stacked, top_entry_index)
    }
}

fn hit_test_node_is_eligible(
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
    node: &zircon_runtime_interface::ui::surface::UiArrangedNode,
) -> Result<bool, ()> {
    if !node.supports_pointer() {
        return Ok(false);
    }
    if !is_arranged_child_hit_path_visible_for_patch(arranged_tree, arranged_node_indices, node_id)?
    {
        return Ok(false);
    }
    let effective_input_policy =
        arranged_effective_input_policy_for_patch(arranged_tree, arranged_node_indices, node_id)?;
    if effective_input_policy == UiInputPolicy::Ignore {
        return Ok(false);
    }
    Ok(node
        .frame
        .intersection(node.clip_frame)
        .is_some_and(|frame| frame.width > 0.0 && frame.height > 0.0))
}

fn arranged_node_for_patch<'a>(
    arranged_tree: &'a UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Option<&'a zircon_runtime_interface::ui::surface::UiArrangedNode> {
    let index = arranged_node_indices.get(&node_id).copied()?;
    arranged_tree
        .nodes
        .get(index)
        .filter(|node| node.node_id == node_id)
}

fn is_arranged_child_hit_path_visible_for_patch(
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Result<bool, ()> {
    let node = arranged_node_for_patch(arranged_tree, arranged_node_indices, node_id).ok_or(())?;
    if !node.allows_self_pointer_hit_test() {
        return Ok(false);
    }
    let mut current = node.parent;
    while let Some(ancestor_id) = current {
        let ancestor =
            arranged_node_for_patch(arranged_tree, arranged_node_indices, ancestor_id).ok_or(())?;
        if !ancestor.allows_child_pointer_hit_test() {
            return Ok(false);
        }
        current = ancestor.parent;
    }
    Ok(true)
}

fn arranged_effective_input_policy_for_patch(
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Result<UiInputPolicy, ()> {
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arranged_node_for_patch(arranged_tree, arranged_node_indices, id).ok_or(())?;
        match node.input_policy {
            UiInputPolicy::Inherit => current = node.parent,
            explicit => return Ok(explicit),
        }
    }
    Ok(UiInputPolicy::Receive)
}

fn arranged_bubble_route_for_patch(
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Result<Vec<UiNodeId>, ()> {
    let mut route = Vec::new();
    let mut current = Some(node_id);
    while let Some(id) = current {
        let node = arranged_node_for_patch(arranged_tree, arranged_node_indices, id).ok_or(())?;
        route.push(id);
        current = node.parent;
    }
    Ok(route)
}

fn hit_result_from_stacked(
    grid: &UiHitTestGrid,
    arranged_tree: &UiArrangedTree,
    query: &UiHitTestQuery,
    stacked: Vec<UiNodeId>,
    top_entry_index: Option<usize>,
) -> UiHitTestResult {
    let top_hit = stacked.first().copied();
    let bubble_route = top_entry_index
        .and_then(|entry_index| grid.entries.get(entry_index))
        .filter(|entry| Some(entry.node_id) == top_hit)
        .and_then(cached_bubble_route)
        .map(<[_]>::to_vec)
        .or_else(|| top_hit.and_then(|node_id| arranged_bubble_route(arranged_tree, node_id).ok()))
        .unwrap_or_default();
    let mut root_to_leaf = bubble_route.clone();
    root_to_leaf.reverse();

    UiHitTestResult {
        top_hit,
        top_entry_index,
        stacked,
        path: UiHitPath::from_query(query).with_route(top_hit, root_to_leaf, bubble_route),
    }
}

fn entry_frame_and_input_policy(
    entry: &UiHitTestEntry,
    arranged_tree: &UiArrangedTree,
) -> Option<(UiFrame, UiInputPolicy)> {
    if cached_bubble_route(entry).is_some() {
        if let Some(input_policy) = entry.effective_input_policy {
            return Some((entry.frame, input_policy));
        }
    }

    let node = arranged_tree.get(entry.node_id)?;
    let input_policy = entry.effective_input_policy.unwrap_or_else(|| {
        arranged_effective_input_policy(arranged_tree, entry.node_id)
            .unwrap_or(UiInputPolicy::Receive)
    });
    Some((node.frame, input_policy))
}

fn cached_bubble_route(entry: &UiHitTestEntry) -> Option<&[UiNodeId]> {
    entry
        .bubble_route
        .first()
        .is_some_and(|node_id| *node_id == entry.node_id)
        .then_some(entry.bubble_route.as_slice())
}

fn build_hit_grid(
    arranged_tree: &UiArrangedTree,
    node_indices: &BTreeMap<UiNodeId, usize>,
) -> UiHitTestGrid {
    let mut entries: Vec<_> = arranged_tree
        .draw_order
        .iter()
        .filter_map(|node_id| arranged_node_indexed(arranged_tree, node_indices, *node_id).ok())
        .filter(|node| node.supports_pointer())
        .filter(|node| {
            is_arranged_child_hit_path_visible_indexed(arranged_tree, node_indices, node.node_id)
                .unwrap_or(false)
        })
        .filter_map(|node| {
            let effective_input_policy =
                arranged_effective_input_policy_indexed(arranged_tree, node_indices, node.node_id)
                    .ok()?;
            if effective_input_policy == UiInputPolicy::Ignore {
                return None;
            }
            let bubble_route =
                arranged_bubble_route_indexed(arranged_tree, node_indices, node.node_id).ok()?;
            let clip_frame = node.frame.intersection(node.clip_frame)?;
            Some(UiHitTestEntry {
                node_id: node.node_id,
                frame: node.frame,
                clip_frame,
                z_index: node.z_index,
                paint_order: node.paint_order,
                control_id: node.control_id.clone(),
                effective_input_policy: Some(effective_input_policy),
                bubble_route,
            })
        })
        .collect();

    entries.sort_by_key(|entry| (entry.z_index, entry.paint_order, entry.node_id));
    let bounds = union_entry_bounds(&entries).unwrap_or_default();
    if entries.is_empty() || bounds.width <= 0.0 || bounds.height <= 0.0 {
        return UiHitTestGrid {
            bounds,
            cell_size: HIT_GRID_CELL_SIZE,
            columns: 0,
            rows: 0,
            scope: Default::default(),
            entries,
            cells: Vec::new(),
            ..UiHitTestGrid::default()
        };
    }

    let columns = (bounds.width / HIT_GRID_CELL_SIZE).ceil().max(1.0) as u32;
    let rows = (bounds.height / HIT_GRID_CELL_SIZE).ceil().max(1.0) as u32;
    let mut cells = vec![UiHitTestCell::default(); (columns * rows) as usize];
    for (entry_index, entry) in entries.iter().enumerate() {
        for cell_index in
            cells_for_frame(bounds, columns, rows, HIT_GRID_CELL_SIZE, entry.clip_frame)
        {
            cells[cell_index].entries.push(entry_index);
        }
    }

    UiHitTestGrid {
        bounds,
        cell_size: HIT_GRID_CELL_SIZE,
        columns,
        rows,
        scope: Default::default(),
        entries,
        cells,
        ..UiHitTestGrid::default()
    }
}

fn hit_entry_indices_for_query(
    grid: &UiHitTestGrid,
    point: UiPoint,
    cursor_radius: f32,
) -> Vec<usize> {
    let cell_indices = cell_indices_for_query(grid, point, cursor_radius);
    let mut entries = Vec::new();
    for cell_index in cell_indices {
        let Some(cell) = grid.cells.get(cell_index) else {
            continue;
        };
        for entry_index in &cell.entries {
            if !entries.contains(entry_index) {
                entries.push(*entry_index);
            }
        }
    }
    entries.sort_by(|left, right| {
        let left_entry = grid.entries.get(*left);
        let right_entry = grid.entries.get(*right);
        match (left_entry, right_entry) {
            (Some(left_entry), Some(right_entry)) => {
                entry_sort_key(right_entry).cmp(&entry_sort_key(left_entry))
            }
            _ => right.cmp(left),
        }
    });
    entries
}

fn cell_indices_for_query(grid: &UiHitTestGrid, point: UiPoint, cursor_radius: f32) -> Vec<usize> {
    if grid.columns == 0 || grid.rows == 0 {
        return Vec::new();
    }
    if cursor_radius <= 0.0 {
        return cell_index_for_point(grid, point).into_iter().collect();
    }
    let query_frame = UiFrame::new(
        point.x - cursor_radius,
        point.y - cursor_radius,
        cursor_radius * 2.0,
        cursor_radius * 2.0,
    );
    if query_frame.intersection(grid.bounds).is_none() {
        return Vec::new();
    }
    cells_for_frame(
        grid.bounds,
        grid.columns,
        grid.rows,
        grid.cell_size,
        query_frame,
    )
}

fn cell_index_for_point(grid: &UiHitTestGrid, point: UiPoint) -> Option<usize> {
    if grid.columns == 0 || grid.rows == 0 || !grid.bounds.contains_point(point) {
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

fn cells_for_frame(
    bounds: UiFrame,
    columns: u32,
    rows: u32,
    cell_size: f32,
    frame: UiFrame,
) -> Vec<usize> {
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
    let mut indices = Vec::new();
    for row in top..=bottom {
        for column in left..=right {
            indices.push((row * columns + column) as usize);
        }
    }
    indices
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
            roots: vec![node_id],
            nodes: vec![pointer_node(node_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0))],
            draw_order: vec![node_id],
            canvas_layers: Vec::new(),
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
            roots: vec![moving_id, anchor_id],
            nodes: vec![
                pointer_node(moving_id, 0, UiFrame::new(0.0, 0.0, 20.0, 20.0)),
                pointer_node(anchor_id, 1, UiFrame::new(100.0, 0.0, 20.0, 20.0)),
            ],
            draw_order: vec![moving_id, anchor_id],
            canvas_layers: Vec::new(),
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
            roots: vec![parent_id],
            nodes: vec![parent, child],
            draw_order: vec![parent_id, child_id],
            canvas_layers: Vec::new(),
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
            roots: vec![parent_id],
            nodes: vec![parent, child],
            draw_order: vec![parent_id, child_id],
            canvas_layers: Vec::new(),
        };

        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged(&arranged_tree);

        assert!(index.grid.entries.is_empty());
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
    let mut iter = entries.iter();
    let first = iter.next()?.clip_frame;
    let (mut left, mut top, mut right, mut bottom) =
        (first.x, first.y, first.right(), first.bottom());
    for entry in iter {
        left = left.min(entry.clip_frame.x);
        top = top.min(entry.clip_frame.y);
        right = right.max(entry.clip_frame.right());
        bottom = bottom.max(entry.clip_frame.bottom());
    }
    Some(UiFrame::new(left, top, right - left, bottom - top))
}

fn frame_is_contained(bounds: UiFrame, frame: UiFrame) -> bool {
    frame.width >= 0.0
        && frame.height >= 0.0
        && frame.x >= bounds.x
        && frame.y >= bounds.y
        && frame.right() <= bounds.right()
        && frame.bottom() <= bounds.bottom()
}
