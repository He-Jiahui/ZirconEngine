use crate::ui::{layout::virtual_window_for_scrollable_box, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        DesiredSize, UiAxis, UiContainerKind, UiFrame, UiGridBoxConfig, UiMasonryBoxConfig,
        UiScrollState, UiScrollableBoxConfig, UiSize, UiVirtualListWindow, UiWrapBoxConfig,
    },
    tree::{UiTree, UiTreeError},
};

use super::axis::{frame_axis_extent, resolve_linear_child_main_extents, size_axis_extent};
use super::child_frame::{free_child_frame, linear_child_frame, scrollable_child_frame};
use super::clip::resolve_clip_frame;
use super::engine::UiLayoutPassEngineContext;
use super::measure::measure_wrap_content_size_for_width;
use super::slot::{ordered_children_for_container, slot_for_container_child, slot_padding};
use super::taffy_arrange::try_arrange_taffy_owned_children;

pub(crate) fn arrange_node(
    tree: &mut UiTree,
    node_id: UiNodeId,
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    if tree
        .node(node_id)
        .is_some_and(|node| !node.effective_visibility().occupies_layout())
    {
        hide_subtree_layout(tree, node_id)?;
        return Ok(());
    }

    let (children, clip_frame, next_clip, container) = {
        let node = tree
            .node_mut(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        let effective_clip = node.clip_to_bounds || node.container.clips_to_bounds();
        node.layout_cache.frame = frame;
        node.layout_cache.clip_frame = resolve_clip_frame(inherited_clip, frame, effective_clip);
        node.dirty = Default::default();
        (
            node.children.clone(),
            node.layout_cache.clip_frame,
            if effective_clip {
                node.layout_cache.clip_frame
            } else {
                inherited_clip
            },
            node.container,
        )
    };

    match container {
        UiContainerKind::Free | UiContainerKind::Container | UiContainerKind::Overlay => {
            record_zircon_owned_container(engine_context, node_id, container, &children);
            let children = ordered_children_for_container(tree, node_id, &children, container);
            for child_id in children {
                let child_frame = {
                    let slot = slot_for_container_child(tree, node_id, child_id, container);
                    free_child_frame(tree, child_id, frame, slot)?
                };
                arrange_node(tree, child_id, child_frame, next_clip, engine_context)?;
            }
        }
        UiContainerKind::Space => {
            record_zircon_owned_container(engine_context, node_id, container, &children);
            for child_id in children {
                hide_subtree_layout(tree, child_id)?;
            }
        }
        UiContainerKind::SizeBox(config) => {
            record_zircon_owned_container(engine_context, node_id, container, &children);
            arrange_size_box_children(
                tree,
                node_id,
                &children,
                frame,
                next_clip,
                config,
                engine_context,
            )?;
        }
        UiContainerKind::HorizontalBox(config) => {
            if !try_arrange_taffy_owned_children(
                tree,
                node_id,
                &children,
                frame,
                next_clip,
                engine_context,
            )? {
                arrange_linear_children(
                    tree,
                    node_id,
                    &children,
                    frame,
                    next_clip,
                    UiAxis::Horizontal,
                    config.gap,
                    engine_context,
                )?;
            }
        }
        UiContainerKind::VerticalBox(config) => {
            if !try_arrange_taffy_owned_children(
                tree,
                node_id,
                &children,
                frame,
                next_clip,
                engine_context,
            )? {
                arrange_linear_children(
                    tree,
                    node_id,
                    &children,
                    frame,
                    next_clip,
                    UiAxis::Vertical,
                    config.gap,
                    engine_context,
                )?;
            }
        }
        UiContainerKind::ScrollableBox(config) => {
            record_zircon_owned_container(engine_context, node_id, container, &children);
            arrange_scrollable_children(
                tree,
                node_id,
                &children,
                frame,
                next_clip,
                config,
                engine_context,
            )?;
        }
        UiContainerKind::WrapBox(config) => {
            if !try_arrange_taffy_owned_children(
                tree,
                node_id,
                &children,
                frame,
                next_clip,
                engine_context,
            )? {
                arrange_wrap_children(
                    tree,
                    node_id,
                    &children,
                    frame,
                    next_clip,
                    config,
                    engine_context,
                )?;
            }

            let content_size = wrap_content_size(tree, node_id, &children, config, frame.width)?;
            let node = tree
                .node_mut(node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            node.layout_cache.content_size = content_size;
        }
        UiContainerKind::GridBox(config) => {
            if !try_arrange_taffy_owned_children(
                tree,
                node_id,
                &children,
                frame,
                next_clip,
                engine_context,
            )? {
                arrange_grid_children(
                    tree,
                    node_id,
                    &children,
                    frame,
                    next_clip,
                    config,
                    engine_context,
                )?;
            }
        }
        UiContainerKind::MasonryBox(config) => {
            record_zircon_owned_container(engine_context, node_id, container, &children);
            let content_size = arrange_masonry_children(
                tree,
                node_id,
                &children,
                frame,
                next_clip,
                config,
                engine_context,
            )?;
            let node = tree
                .node_mut(node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            node.layout_cache.content_size = content_size;
        }
    }

    if clip_frame.is_none() && inherited_clip.is_some() {
        let node = tree
            .node_mut(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        node.layout_cache.clip_frame = inherited_clip;
    }

    Ok(())
}

fn record_zircon_owned_container(
    engine_context: &mut UiLayoutPassEngineContext,
    node_id: UiNodeId,
    container: UiContainerKind,
    children: &[UiNodeId],
) {
    if !children.is_empty() {
        engine_context.record_zircon_owned(node_id, container);
    }
}

fn arrange_size_box_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    config: zircon_runtime_interface::ui::layout::UiSizeBoxConfig,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = UiContainerKind::SizeBox(config);
    let content_frame = size_box_content_frame(frame, config.aspect_ratio);
    let children = ordered_children_for_container(tree, parent_id, children, container);
    for child_id in children {
        let child_frame = {
            let slot = slot_for_container_child(tree, parent_id, child_id, container);
            free_child_frame(tree, child_id, content_frame, slot)?
        };
        arrange_node(tree, child_id, child_frame, inherited_clip, engine_context)?;
    }
    Ok(())
}

fn size_box_content_frame(frame: UiFrame, aspect_ratio: f32) -> UiFrame {
    let Some(aspect_ratio) = normalized_aspect_ratio(aspect_ratio) else {
        return frame;
    };
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return frame;
    }

    let height_from_width = frame.width / aspect_ratio;
    if height_from_width <= frame.height {
        return UiFrame::new(
            frame.x,
            frame.y + (frame.height - height_from_width) * 0.5,
            frame.width,
            height_from_width,
        );
    }

    let width_from_height = frame.height * aspect_ratio;
    UiFrame::new(
        frame.x + (frame.width - width_from_height) * 0.5,
        frame.y,
        width_from_height,
        frame.height,
    )
}

fn normalized_aspect_ratio(aspect_ratio: f32) -> Option<f32> {
    aspect_ratio
        .is_finite()
        .then_some(aspect_ratio)
        .filter(|ratio| *ratio > 0.0)
}

fn wrap_content_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    config: UiWrapBoxConfig,
    available_width: f32,
) -> Result<UiSize, UiTreeError> {
    let child_desired = children
        .iter()
        .filter_map(|child_id| {
            tree.node(*child_id).map(|child| {
                child
                    .effective_visibility()
                    .occupies_layout()
                    .then_some((*child_id, child.layout_cache.desired_size))
            })
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(UiTreeError::MissingNode(parent_id))?;
    Ok(measure_wrap_content_size_for_width(
        tree,
        parent_id,
        UiContainerKind::WrapBox(config),
        config,
        &child_desired,
        available_width,
    ))
}

fn arrange_linear_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    axis: UiAxis,
    gap: f32,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = match axis {
        UiAxis::Horizontal => UiContainerKind::HorizontalBox(Default::default()),
        UiAxis::Vertical => UiContainerKind::VerticalBox(Default::default()),
    };
    let children = ordered_children_for_container(tree, parent_id, children, container);
    let main_extents = resolve_linear_child_main_extents(
        tree,
        parent_id,
        &children,
        axis,
        frame_axis_extent(frame, axis),
        gap,
    )?;
    let gap = gap.max(0.0);
    let mut cursor = 0.0;
    let mut placed_count = 0usize;

    for (index, child_id) in children.iter().copied().enumerate() {
        let occupies_layout = tree
            .node(child_id)
            .is_some_and(|node| node.effective_visibility().occupies_layout());
        if occupies_layout && placed_count > 0 {
            cursor += gap;
        }
        let child_frame = {
            let slot = slot_for_container_child(tree, parent_id, child_id, container);
            linear_child_frame(
                tree,
                child_id,
                frame,
                axis,
                cursor,
                main_extents[index],
                slot,
            )?
        };
        arrange_node(tree, child_id, child_frame, inherited_clip, engine_context)?;
        if occupies_layout {
            cursor += main_extents[index];
            placed_count += 1;
        }
    }

    Ok(())
}

fn arrange_scrollable_children(
    tree: &mut UiTree,
    node_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    config: UiScrollableBoxConfig,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let (content_size, previous_offset) = {
        let node = tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        (
            node.layout_cache.content_size,
            node.scroll_state.unwrap_or_default().offset,
        )
    };

    let viewport_extent = frame_axis_extent(frame, config.axis);
    let content_extent = size_axis_extent(content_size, config.axis);
    let max_offset = (content_extent - viewport_extent).max(0.0);
    let offset = previous_offset.max(0.0).min(max_offset);
    let visible_window =
        virtual_window_for_scrollable_box(config, offset, children.len(), viewport_extent)
            .unwrap_or(UiVirtualListWindow {
                first_visible: 0,
                last_visible_exclusive: children.len(),
            });

    {
        let node = tree
            .node_mut(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        node.scroll_state = Some(UiScrollState {
            offset,
            viewport_extent,
            content_extent,
        });
        node.layout_cache.virtual_window = Some(visible_window);
    }

    let positions = child_positions(tree, children, config.axis, config.gap)?;
    for (index, child_id) in children.iter().copied().enumerate() {
        if index < visible_window.first_visible || index >= visible_window.last_visible_exclusive {
            hide_subtree_layout(tree, child_id)?;
            continue;
        }

        let child_frame =
            scrollable_child_frame(tree, child_id, frame, config.axis, positions[index], offset)?;
        arrange_node(tree, child_id, child_frame, inherited_clip, engine_context)?;
    }

    Ok(())
}

fn arrange_wrap_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    config: UiWrapBoxConfig,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = UiContainerKind::WrapBox(config);
    let children = ordered_children_for_container(tree, parent_id, children, container);
    let horizontal_gap = config.horizontal_gap.max(0.0);
    let vertical_gap = config.vertical_gap.max(0.0);
    let available_width = frame.width.max(0.0);
    let mut cursor_x = 0.0_f32;
    let mut cursor_y = 0.0_f32;
    let mut row_height = 0.0_f32;
    let mut row_items: Vec<(UiNodeId, f32)> = Vec::new();

    for child_id in children.iter().copied() {
        let Some(node) = tree.node(child_id) else {
            return Err(UiTreeError::MissingNode(child_id));
        };
        if !node.effective_visibility().occupies_layout() {
            hide_subtree_layout(tree, child_id)?;
            continue;
        }

        let item_size = wrap_item_outer_size(tree, parent_id, child_id, config)?;
        let item_width = item_size.width;
        let item_height = item_size.height;
        let next_width = if row_items.is_empty() {
            item_width
        } else {
            cursor_x + horizontal_gap + item_width
        };
        if !row_items.is_empty() && next_width > available_width {
            arrange_wrap_row(
                tree,
                parent_id,
                &row_items,
                frame,
                inherited_clip,
                cursor_y,
                row_height,
                config,
                engine_context,
            )?;
            cursor_y += row_height + vertical_gap;
            cursor_x = 0.0;
            row_height = 0.0;
            row_items.clear();
        }

        if !row_items.is_empty() {
            cursor_x += horizontal_gap;
        }
        cursor_x += item_width;
        row_height = row_height.max(item_height);
        row_items.push((child_id, item_width));
    }

    if !row_items.is_empty() {
        arrange_wrap_row(
            tree,
            parent_id,
            &row_items,
            frame,
            inherited_clip,
            cursor_y,
            row_height,
            config,
            engine_context,
        )?;
    }

    Ok(())
}

fn arrange_grid_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    config: UiGridBoxConfig,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = UiContainerKind::GridBox(config);
    let children = ordered_children_for_container(tree, parent_id, children, container);
    let (columns, rows) = grid_dimensions(tree, parent_id, &children, config);

    for (index, child_id) in children.iter().copied().enumerate() {
        let slot = slot_for_container_child(tree, parent_id, child_id, container);
        let placement = grid_placement_for_child(slot, index, columns);
        let child_frame = free_child_frame(
            tree,
            child_id,
            grid_cell_frame(frame, config, columns, rows, placement),
            slot,
        )?;
        arrange_node(tree, child_id, child_frame, inherited_clip, engine_context)?;
    }

    Ok(())
}

fn arrange_masonry_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    config: UiMasonryBoxConfig,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<UiSize, UiTreeError> {
    let container = UiContainerKind::MasonryBox(config);
    let children = ordered_children_for_container(tree, parent_id, children, container);
    let columns = config.columns.max(1);
    let gap = config.gap.max(0.0);
    let column_width =
        ((frame.width - gap * columns.saturating_sub(1) as f32) / columns as f32).max(0.0);
    let mut column_heights = vec![0.0_f32; columns];
    let mut column_counts = vec![0usize; columns];
    let mut visible_index = 0usize;

    for child_id in children {
        let Some(node) = tree.node(child_id) else {
            return Err(UiTreeError::MissingNode(child_id));
        };
        if !node.effective_visibility().occupies_layout() {
            hide_subtree_layout(tree, child_id)?;
            continue;
        }

        let slot = slot_for_container_child(tree, parent_id, child_id, container);
        let outer_height = masonry_child_outer_height(tree, child_id, slot)?;
        let column = masonry_target_column(visible_index, config.sequential, &column_heights);
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
        arrange_node(tree, child_id, child_frame, inherited_clip, engine_context)?;

        column_heights[column] += outer_height;
        column_counts[column] += 1;
        visible_index += 1;
    }

    Ok(UiSize::new(
        frame.width.max(0.0),
        column_heights.iter().copied().fold(0.0_f32, f32::max),
    ))
}

fn wrap_item_outer_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    config: UiWrapBoxConfig,
) -> Result<UiSize, UiTreeError> {
    let node = tree
        .node(child_id)
        .ok_or(UiTreeError::MissingNode(child_id))?;
    let padding = slot_padding(slot_for_container_child(
        tree,
        parent_id,
        child_id,
        UiContainerKind::WrapBox(config),
    ));
    Ok(wrap_item_outer_size_from_desired(
        node.layout_cache.desired_size,
        padding,
        config,
    ))
}

fn wrap_item_outer_size_from_desired(
    desired: DesiredSize,
    padding: zircon_runtime_interface::ui::layout::UiMargin,
    config: UiWrapBoxConfig,
) -> UiSize {
    UiSize::new(
        desired.width.max(config.item_min_width) + padding.horizontal(),
        desired.height + padding.vertical(),
    )
}

fn masonry_child_outer_height(
    tree: &UiTree,
    child_id: UiNodeId,
    slot: Option<&zircon_runtime_interface::ui::layout::UiSlot>,
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

fn arrange_wrap_row(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    row_items: &[(UiNodeId, f32)],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    row_y: f32,
    row_height: f32,
    config: UiWrapBoxConfig,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = UiContainerKind::WrapBox(config);
    let row_frame = UiFrame::new(frame.x, frame.y + row_y, frame.width, row_height.max(0.0));
    let mut cursor_x = 0.0_f32;
    let horizontal_gap = config.horizontal_gap.max(0.0);
    for (index, (child_id, item_width)) in row_items.iter().copied().enumerate() {
        if index > 0 {
            cursor_x += horizontal_gap;
        }
        let child_frame = {
            let slot = slot_for_container_child(tree, parent_id, child_id, container);
            linear_child_frame(
                tree,
                child_id,
                row_frame,
                UiAxis::Horizontal,
                cursor_x,
                item_width,
                slot,
            )?
        };
        arrange_node(tree, child_id, child_frame, inherited_clip, engine_context)?;
        cursor_x += item_width;
    }
    Ok(())
}

fn grid_dimensions(
    tree: &UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    config: UiGridBoxConfig,
) -> (usize, usize) {
    let mut columns = config.columns.max(1);
    let mut rows = config.rows.max(1);
    for (index, child_id) in children.iter().copied().enumerate() {
        let slot =
            slot_for_container_child(tree, parent_id, child_id, UiContainerKind::GridBox(config));
        let placement = grid_placement_for_child(slot, index, columns);
        columns = columns.max(placement.column + placement.column_span.max(1));
        rows = rows.max(placement.row + placement.row_span.max(1));
    }
    (columns, rows)
}

fn grid_placement_for_child(
    slot: Option<&zircon_runtime_interface::ui::layout::UiSlot>,
    index: usize,
    columns: usize,
) -> zircon_runtime_interface::ui::layout::UiGridSlotPlacement {
    if let Some(placement) = slot.and_then(|slot| slot.grid_placement) {
        return placement.with_span(placement.column_span, placement.row_span);
    }

    let columns = columns.max(1);
    zircon_runtime_interface::ui::layout::UiGridSlotPlacement::new(index % columns, index / columns)
}

fn grid_cell_frame(
    frame: UiFrame,
    config: UiGridBoxConfig,
    columns: usize,
    rows: usize,
    placement: zircon_runtime_interface::ui::layout::UiGridSlotPlacement,
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

fn child_positions(
    tree: &UiTree,
    children: &[UiNodeId],
    axis: UiAxis,
    gap: f32,
) -> Result<Vec<f32>, UiTreeError> {
    let mut positions = Vec::with_capacity(children.len());
    let mut cursor = 0.0;
    let gap = gap.max(0.0);
    let mut placed_count = 0usize;
    for child_id in children {
        let node = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        let occupies_layout = node.effective_visibility().occupies_layout();
        if occupies_layout && placed_count > 0 {
            cursor += gap;
        }
        positions.push(cursor);
        if occupies_layout {
            cursor += match axis {
                UiAxis::Vertical => node.layout_cache.desired_size.height,
                UiAxis::Horizontal => node.layout_cache.desired_size.width,
            };
            placed_count += 1;
        }
    }
    Ok(positions)
}

fn hide_subtree_layout(tree: &mut UiTree, node_id: UiNodeId) -> Result<(), UiTreeError> {
    let children = {
        let node = tree
            .node_mut(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        node.layout_cache.frame = UiFrame::default();
        node.layout_cache.clip_frame = None;
        node.children.clone()
    };
    for child_id in children {
        hide_subtree_layout(tree, child_id)?;
    }
    Ok(())
}
