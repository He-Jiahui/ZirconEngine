use serde::{Deserialize, Serialize};

use crate::ui::surface::{
    arranged_bubble_route, arranged_effective_input_policy, build_arranged_tree,
    is_arranged_child_hit_path_visible,
};
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
    pub stacked: Vec<UiNodeId>,
    pub path: UiHitPath,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestIndex {
    pub grid: UiHitTestGrid,
}

impl UiHitTestIndex {
    pub fn from_grid(grid: UiHitTestGrid) -> Self {
        Self { grid }
    }

    pub fn rebuild(&mut self, tree: &UiTree) {
        let arranged_tree = build_arranged_tree(tree);
        self.rebuild_arranged(&arranged_tree);
    }

    pub fn rebuild_arranged(&mut self, arranged_tree: &UiArrangedTree) {
        self.grid = build_hit_grid(arranged_tree);
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
                stacked: Vec::new(),
                path: UiHitPath::from_query(&query),
            };
        }
        let mut stacked = Vec::new();
        let point = query.hit_point();
        let cursor_radius = query.sanitized_cursor_radius();
        let entry_indices = hit_entry_indices_for_query(grid, point, cursor_radius);
        let mut exact_hits = Vec::new();
        let mut radius_hits = Vec::new();

        for entry_index in entry_indices {
            let Some(entry) = grid.entries.get(entry_index) else {
                continue;
            };
            let Some(node) = arranged_tree.get(entry.node_id) else {
                continue;
            };
            let clipped_frame = node
                .frame
                .intersection(entry.clip_frame)
                .unwrap_or(entry.clip_frame);
            if !frame_accepts_point(clipped_frame, point, cursor_radius) {
                continue;
            }
            if arranged_effective_input_policy(arranged_tree, entry.node_id)
                .is_ok_and(|policy| policy == UiInputPolicy::Ignore)
            {
                continue;
            }
            if clipped_frame.contains_point(point) {
                exact_hits.push(entry.node_id);
            } else {
                radius_hits.push((distance_sq_to_frame(clipped_frame, point), entry.node_id));
            }
        }
        radius_hits.sort_by(|left, right| left.0.total_cmp(&right.0));
        stacked.extend(exact_hits);
        stacked.extend(radius_hits.into_iter().map(|(_, node_id)| node_id));

        let top_hit = stacked.first().copied();
        let bubble_route = top_hit
            .and_then(|node_id| arranged_bubble_route(arranged_tree, node_id).ok())
            .unwrap_or_default();
        let mut root_to_leaf = bubble_route.clone();
        root_to_leaf.reverse();

        UiHitTestResult {
            top_hit,
            stacked,
            path: UiHitPath::from_query(&query).with_route(top_hit, root_to_leaf, bubble_route),
        }
    }
}

fn build_hit_grid(arranged_tree: &UiArrangedTree) -> UiHitTestGrid {
    let mut entries: Vec<_> = arranged_tree
        .draw_order
        .iter()
        .filter_map(|node_id| arranged_tree.get(*node_id))
        .filter(|node| node.supports_pointer())
        .filter(|node| {
            is_arranged_child_hit_path_visible(arranged_tree, node.node_id).unwrap_or(false)
        })
        .filter(|node| {
            arranged_effective_input_policy(arranged_tree, node.node_id)
                .is_ok_and(|policy| policy != UiInputPolicy::Ignore)
        })
        .filter_map(|node| {
            let clip_frame = node.frame.intersection(node.clip_frame)?;
            Some(UiHitTestEntry {
                node_id: node.node_id,
                frame: node.frame,
                clip_frame,
                z_index: node.z_index,
                paint_order: node.paint_order,
                control_id: node.control_id.clone(),
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
