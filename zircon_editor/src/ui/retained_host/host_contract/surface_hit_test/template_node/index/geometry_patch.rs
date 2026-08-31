use std::collections::BTreeMap;
use std::sync::Arc;

use super::*;

pub(super) fn root_node_origin(node: &TemplatePaneNodeData) -> Option<FrameRect> {
    (node.parent_node_id.is_empty() && node.frame.width > 0.0 && node.frame.height > 0.0).then_some(
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: node
                .frame
                .width
                .max(node.frame.x + node.frame.width)
                .max(1.0),
            height: node
                .frame
                .height
                .max(node.frame.y + node.frame.height)
                .max(1.0),
        },
    )
}

pub(super) fn same_frame_rect(left: &FrameRect, right: &FrameRect) -> bool {
    left.x == right.x
        && left.y == right.y
        && left.width == right.width
        && left.height == right.height
}

impl HostWorkbenchHitIndex {
    pub(crate) fn patch_geometry_presentation(
        &self,
        previous: &HostWindowPresentationData,
        next: &HostWindowPresentationData,
        changed_rows: &[usize],
    ) -> Option<Self> {
        let previous_nodes = &previous.workbench_window_nodes;
        let next_nodes = &next.workbench_window_nodes;
        if !self.indexed_nodes.shares_values_with(previous_nodes)
            || previous_nodes.row_count() != next_nodes.row_count()
            || changed_rows.windows(2).any(|rows| rows[0] >= rows[1])
            || changed_rows.iter().any(|row| {
                previous_nodes
                    .get(*row)
                    .zip(next_nodes.get(*row))
                    .map_or(true, |(previous, next)| {
                        !same_geometry_index_identity(previous, next)
                    })
            })
        {
            return None;
        }

        let previous_origin = self.origin.as_ref()?;
        let origin_row = self.origin_row?;
        let next_origin = root_node_origin(next_nodes.get(origin_row)?)?;
        if previous_nodes
            .get(origin_row)
            .zip(next_nodes.get(origin_row))
            .is_none_or(|(previous, next)| previous.node_id != next.node_id)
        {
            return None;
        }

        let bucket_updates = patch_cell_buckets(
            &self.buckets,
            previous_nodes,
            next_nodes,
            changed_rows,
            previous_origin,
            &next_origin,
            accepts_pointer_move,
        );
        let changed_cell_count = bucket_updates.len();
        let buckets = Arc::new(self.buckets.with_updates(bucket_updates));

        let previous_paint_models = presentation_paint_node_models(previous);
        let next_paint_models = presentation_paint_node_models(next);
        if !self.indexes_paint_models(&previous_paint_models)
            || previous_paint_models.len() != next_paint_models.len()
        {
            return None;
        }
        let mut paint_indices = self.paint_indices.clone();
        let mut rebuilt_paint_model_count = 0usize;
        for (position, (previous_model, next_model)) in previous_paint_models
            .iter()
            .zip(&next_paint_models)
            .enumerate()
        {
            if previous_model.shares_values_with(next_model) {
                continue;
            }
            if previous_model.shares_values_with(previous_nodes)
                && next_model.shares_values_with(next_nodes)
            {
                paint_indices[position] = paint_indices[position].patch_geometry(
                    previous_nodes,
                    next_nodes,
                    changed_rows,
                    previous_origin,
                    &next_origin,
                )?;
            } else {
                paint_indices[position] = HostTemplateNodePaintIndex::new(next_model.clone());
                rebuilt_paint_model_count += 1;
            }
        }

        let mut extension_workspace = self.extension_workspace.clone();
        if let Some(workspace) = extension_workspace.as_mut() {
            if changed_rows.binary_search(&workspace.host_row).is_ok() {
                let host = next_nodes.get(workspace.host_row)?;
                workspace.host_frame = FrameRect {
                    x: host.frame.x,
                    y: host.frame.y,
                    width: host.frame.width,
                    height: host.frame.height,
                };
            }
        }
        zircon_runtime::profile_counter!(
            "editor",
            "ui.window_resize.hit_index_geometry_patch_count",
            1_u8
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.window_resize.hit_index_geometry_patch_row_count",
            changed_rows.len()
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.window_resize.hit_index_geometry_patch_cell_count",
            changed_cell_count
        );
        zircon_runtime::profile_counter!(
            "editor",
            "ui.window_resize.paint_index_geometry_rebuild_model_count",
            rebuilt_paint_model_count
        );
        Some(Self {
            indexed_nodes: next_nodes.clone(),
            origin: Some(next_origin),
            origin_row: Some(origin_row),
            buckets,
            paint_indices,
            popup_rows: Arc::clone(&self.popup_rows),
            parent_rows: Arc::clone(&self.parent_rows),
            extension_workspace,
            #[cfg(test)]
            last_candidate_visit_count: Cell::new(0),
            #[cfg(test)]
            query_count: Cell::new(0),
        })
    }
}

impl HostTemplateNodePaintIndex {
    fn patch_geometry(
        &self,
        previous_nodes: &ModelRc<TemplatePaneNodeData>,
        next_nodes: &ModelRc<TemplatePaneNodeData>,
        changed_rows: &[usize],
        previous_origin: &FrameRect,
        next_origin: &FrameRect,
    ) -> Option<Self> {
        if !self.indexed_nodes.shares_values_with(previous_nodes)
            || previous_nodes.row_count() != next_nodes.row_count()
            || changed_rows.iter().any(|row| {
                previous_nodes
                    .get(*row)
                    .zip(next_nodes.get(*row))
                    .is_none_or(|(previous, next)| previous.z_index != next.z_index)
            })
        {
            return None;
        }
        let updates = patch_cell_buckets(
            &self.buckets,
            previous_nodes,
            next_nodes,
            changed_rows,
            previous_origin,
            next_origin,
            |_| true,
        );
        Some(Self {
            indexed_nodes: next_nodes.clone(),
            origin: Some(next_origin.clone()),
            buckets: Arc::new(self.buckets.with_updates(updates)),
            paint_order_rows: Arc::clone(&self.paint_order_rows),
            query_scratch: Arc::clone(&self.query_scratch),
            #[cfg(test)]
            query_sort_count: Cell::new(0),
        })
    }
}

fn patch_cell_buckets(
    buckets: &PersistentCellBuckets,
    previous_nodes: &ModelRc<TemplatePaneNodeData>,
    next_nodes: &ModelRc<TemplatePaneNodeData>,
    changed_rows: &[usize],
    previous_origin: &FrameRect,
    next_origin: &FrameRect,
    includes: fn(&TemplatePaneNodeData) -> bool,
) -> BTreeMap<(i32, i32), Option<Vec<usize>>> {
    let mut deltas = BTreeMap::<(i32, i32), CellMembershipDelta>::new();
    for row in changed_rows {
        let previous = previous_nodes
            .get(*row)
            .expect("validated geometry patch previous row");
        if includes(previous) {
            if let Some(frame) = indexed_node_frame(previous, previous_origin) {
                visit_frame_cells(&frame, &mut |cell| {
                    deltas.entry(cell).or_default().removed_rows.push(*row);
                });
            }
        }
        let next = next_nodes
            .get(*row)
            .expect("validated geometry patch next row");
        if includes(next) {
            if let Some(frame) = indexed_node_frame(next, next_origin) {
                visit_frame_cells(&frame, &mut |cell| {
                    deltas.entry(cell).or_default().added_rows.push(*row);
                });
            }
        }
    }
    deltas
        .into_iter()
        .map(|(cell, delta)| {
            let mut rows = buckets.get(&cell).cloned().unwrap_or_default();
            rows.retain(|row| delta.removed_rows.binary_search(row).is_err());
            rows.extend(delta.added_rows);
            sort_rows_in_paint_order(next_nodes, &mut rows);
            rows.dedup();
            let rows = (!rows.is_empty()).then_some(rows);
            (cell, rows)
        })
        .collect()
}

#[derive(Default)]
struct CellMembershipDelta {
    removed_rows: Vec<usize>,
    added_rows: Vec<usize>,
}

fn visit_frame_cells(frame: &FrameRect, visit: &mut dyn FnMut((i32, i32))) {
    let min_x = cell_coordinate(frame.x);
    let max_x = cell_coordinate(frame.x + frame.width - f32::EPSILON);
    let min_y = cell_coordinate(frame.y);
    let max_y = cell_coordinate(frame.y + frame.height - f32::EPSILON);
    for cell_y in min_y..=max_y {
        for cell_x in min_x..=max_x {
            visit((cell_x, cell_y));
        }
    }
}

fn same_geometry_index_identity(
    previous: &TemplatePaneNodeData,
    next: &TemplatePaneNodeData,
) -> bool {
    previous.node_id == next.node_id
        && previous.parent_node_id == next.parent_node_id
        && previous.control_id == next.control_id
        && previous.popup_open == next.popup_open
        && previous.disabled == next.disabled
        && accepts_pointer_move(previous) == accepts_pointer_move(next)
}
