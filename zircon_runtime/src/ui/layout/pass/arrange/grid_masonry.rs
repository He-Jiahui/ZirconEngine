use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        UiContainerKind, UiFrame, UiGridBoxConfig, UiGridSlotPlacement, UiMasonryBoxConfig, UiSize,
        UiSlot,
    },
    tree::{UiTree, UiTreeError},
};

use super::{arrange_node, hide_subtree_layout};
use crate::ui::layout::pass::{
    child_frame::free_child_frame,
    engine::UiLayoutPassEngineContext,
    slot::{slot_for_container_child, slot_padding, UiLayoutSlotIndex},
    workspace::UiMasonryArrangeScratch,
};

pub(super) fn arrange_grid_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    config: UiGridBoxConfig,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = UiContainerKind::GridBox(config);
    let (columns, rows) = grid_dimensions(tree, slot_index, parent_id, children, config);

    for (index, child_id) in children.iter().copied().enumerate() {
        let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
        let placement = grid_placement_for_child(slot, index, columns);
        let child_frame = free_child_frame(
            tree,
            child_id,
            grid_cell_frame(frame, config, columns, rows, placement),
            slot,
        )?;
        arrange_node(
            tree,
            child_id,
            child_frame,
            inherited_clip,
            slot_index,
            engine_context,
        )?;
    }

    Ok(())
}

pub(super) fn arrange_masonry_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    config: UiMasonryBoxConfig,
    scratch: &mut UiMasonryArrangeScratch,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<UiSize, UiTreeError> {
    let container = UiContainerKind::MasonryBox(config);
    let columns = config.columns.max(1);
    let gap = config.gap.max(0.0);
    let column_width =
        ((frame.width - gap * columns.saturating_sub(1) as f32) / columns as f32).max(0.0);
    scratch.column_heights.clear();
    scratch.column_heights.resize(columns, 0.0);
    scratch.column_counts.clear();
    scratch.column_counts.resize(columns, 0);
    let column_heights = &mut scratch.column_heights;
    let column_counts = &mut scratch.column_counts;
    let mut visible_index = 0usize;

    for child_id in children.iter().copied() {
        let Some(node) = tree.node(child_id) else {
            return Err(UiTreeError::MissingNode(child_id));
        };
        if !node.effective_visibility().occupies_layout() {
            hide_subtree_layout(tree, child_id, slot_index, engine_context)?;
            continue;
        }

        let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
        let outer_height = masonry_child_outer_height(tree, child_id, slot)?;
        let column = masonry_target_column(visible_index, config.sequential, column_heights);
        if column_counts[column] > 0 {
            column_heights[column] += gap;
        }
        let cell_frame = UiFrame::new(
            frame.x + column as f32 * (column_width + gap),
            frame.y + column_heights[column],
            column_width,
            outer_height,
        );
        let child_frame = free_child_frame(tree, child_id, cell_frame, slot)?;
        arrange_node(
            tree,
            child_id,
            child_frame,
            inherited_clip,
            slot_index,
            engine_context,
        )?;

        column_heights[column] += outer_height;
        column_counts[column] += 1;
        visible_index += 1;
    }

    Ok(UiSize::new(
        frame.width.max(0.0),
        column_heights.iter().copied().fold(0.0_f32, f32::max),
    ))
}

fn masonry_child_outer_height(
    tree: &UiTree,
    child_id: UiNodeId,
    slot: Option<&UiSlot>,
) -> Result<f32, UiTreeError> {
    let node = tree
        .node(child_id)
        .ok_or(UiTreeError::MissingNode(child_id))?;
    Ok((node.layout_cache.desired_size.height + slot_padding(slot).vertical()).max(0.0))
}

fn masonry_target_column(index: usize, sequential: bool, column_heights: &[f32]) -> usize {
    if sequential {
        return index % column_heights.len().max(1);
    }

    column_heights
        .iter()
        .copied()
        .enumerate()
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(column, _)| column)
        .unwrap_or(0)
}

fn grid_dimensions(
    tree: &UiTree,
    slot_index: &UiLayoutSlotIndex,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    config: UiGridBoxConfig,
) -> (usize, usize) {
    let mut columns = config.columns.max(1);
    let mut rows = config.rows.max(1);
    for (index, child_id) in children.iter().copied().enumerate() {
        let slot = slot_for_container_child(
            tree,
            slot_index,
            parent_id,
            child_id,
            UiContainerKind::GridBox(config),
        );
        let placement = grid_placement_for_child(slot, index, columns);
        columns = columns.max(placement.column + placement.column_span.max(1));
        rows = rows.max(placement.row + placement.row_span.max(1));
    }
    (columns, rows)
}

fn grid_placement_for_child(
    slot: Option<&UiSlot>,
    index: usize,
    columns: usize,
) -> UiGridSlotPlacement {
    if let Some(placement) = slot.and_then(|slot| slot.grid_placement) {
        return placement.with_span(placement.column_span, placement.row_span);
    }

    let columns = columns.max(1);
    UiGridSlotPlacement::new(index % columns, index / columns)
}

fn grid_cell_frame(
    frame: UiFrame,
    config: UiGridBoxConfig,
    columns: usize,
    rows: usize,
    placement: UiGridSlotPlacement,
) -> UiFrame {
    let columns = columns.max(1);
    let rows = rows.max(1);
    let column_gap = config.column_gap.max(0.0);
    let row_gap = config.row_gap.max(0.0);
    let cell_width =
        ((frame.width - column_gap * columns.saturating_sub(1) as f32) / columns as f32).max(0.0);
    let cell_height =
        ((frame.height - row_gap * rows.saturating_sub(1) as f32) / rows as f32).max(0.0);
    let column = placement.column.min(columns - 1);
    let row = placement.row.min(rows - 1);
    let column_span = placement.column_span.max(1).min(columns - column);
    let row_span = placement.row_span.max(1).min(rows - row);

    UiFrame::new(
        frame.x + column as f32 * (cell_width + column_gap),
        frame.y + row as f32 * (cell_height + row_gap),
        cell_width * column_span as f32 + column_gap * column_span.saturating_sub(1) as f32,
        cell_height * row_span as f32 + row_gap * row_span.saturating_sub(1) as f32,
    )
}
