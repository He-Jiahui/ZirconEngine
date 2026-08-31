use crate::ui::layout::plan_scrollable_virtual_window;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{
        DesiredSize, UiAxis, UiContainerKind, UiFrame, UiScrollableBoxConfig, UiSize,
        UiVirtualListWindow, UiWrapBoxConfig,
    },
    tree::{UiTree, UiTreeError},
};

mod grid_masonry;
#[cfg(test)]
mod tests;
mod virtual_list;

use self::grid_masonry::{arrange_grid_children, arrange_masonry_children};
use self::virtual_list::arrange_materialized_virtual_list_children;
use super::axis::{frame_axis_extent, resolve_linear_child_main_extents, size_axis_extent};
use super::child_frame::{free_child_frame, linear_child_frame, scrollable_child_frame};
use super::clip::resolve_clip_frame;
use super::engine::UiLayoutPassEngineContext;
use super::measure::measure_wrap_content_size_for_width;
use super::slot::{slot_for_container_child, slot_padding, UiLayoutSlotIndex};
use super::taffy_arrange::try_arrange_taffy_owned_children;
use super::workspace::UiLinearArrangeScratch;

pub(crate) fn arrange_node(
    tree: &mut UiTree,
    node_id: UiNodeId,
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    engine_context.record_arrangement_probe();
    let (
        node_occupies_layout,
        effective_clip,
        previous_frame,
        previous_clip_frame,
        structure_dirty,
        container,
    ) = {
        let node = tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        (
            node.effective_visibility().occupies_layout(),
            node.clip_to_bounds || node.container.clips_to_bounds(),
            node.layout_cache.frame,
            node.layout_cache.clip_frame,
            node.dirty.input,
            node.container,
        )
    };
    if !node_occupies_layout {
        if engine_context.can_reuse_geometry(
            node_id,
            previous_frame,
            previous_clip_frame,
            UiFrame::default(),
            None,
        ) {
            return Ok(());
        }
        hide_subtree_layout(tree, node_id, slot_index, engine_context)?;
        return Ok(());
    }

    let resolved_clip = resolve_clip_frame(inherited_clip, frame, effective_clip);
    let node_clip = resolved_clip.or(inherited_clip);
    if engine_context.can_reuse_geometry(
        node_id,
        previous_frame,
        previous_clip_frame,
        frame,
        node_clip,
    ) {
        return Ok(());
    }
    engine_context.record_geometry(
        node_id,
        previous_frame,
        previous_clip_frame,
        frame,
        node_clip,
    );

    let sparse_children = engine_context.can_sparse_arrange_independent_children(
        node_id,
        container,
        structure_dirty,
        previous_frame,
        previous_clip_frame,
        frame,
        node_clip,
    );
    let mut child_scratch = slot_index.take_arrange_child_scratch();
    let next_clip = {
        let Some(node) = tree.node_mut(node_id) else {
            slot_index.recycle_arrange_child_scratch(child_scratch);
            return Err(UiTreeError::MissingNode(node_id));
        };
        node.layout_cache.frame = frame;
        node.layout_cache.clip_frame = node_clip;
        node.dirty = Default::default();
        if effective_clip {
            resolved_clip
        } else {
            inherited_clip
        }
    };
    if sparse_children {
        // A descendant-only invalidation can skip clean siblings; all structural and parent
        // layout changes were rejected by can_sparse_arrange_independent_children above.
        engine_context.copy_required_children(tree, node_id, &mut child_scratch.children);
        if child_scratch.children.is_empty()
            && tree
                .node(node_id)
                .is_some_and(|node| !node.children.is_empty())
        {
            let ordered_children =
                slot_index.ordered_children_for_container(tree, node_id, container);
            child_scratch.children.extend_from_slice(&ordered_children);
        }
    } else {
        let ordered_children = slot_index.ordered_children_for_container(tree, node_id, container);
        child_scratch.children.extend_from_slice(&ordered_children);
    }

    let arrange_result = (|| -> Result<(), UiTreeError> {
        let children = child_scratch.children.as_slice();
        match container {
            UiContainerKind::Free
            | UiContainerKind::Canvas
            | UiContainerKind::Container
            | UiContainerKind::Overlay => {
                record_zircon_owned_container(engine_context, node_id, container, children);
                for child_id in children.iter().copied() {
                    let child_frame = {
                        let slot = slot_for_container_child(
                            tree, slot_index, node_id, child_id, container,
                        );
                        free_child_frame(tree, child_id, frame, slot)?
                    };
                    arrange_node(
                        tree,
                        child_id,
                        child_frame,
                        next_clip,
                        slot_index,
                        engine_context,
                    )?;
                }
            }
            UiContainerKind::BlockBox => {
                if !try_arrange_taffy_owned_children(
                    tree,
                    node_id,
                    children,
                    frame,
                    next_clip,
                    slot_index,
                    engine_context,
                )? {
                    arrange_block_children(
                        tree,
                        node_id,
                        children,
                        frame,
                        next_clip,
                        slot_index,
                        engine_context,
                    )?;
                }
            }
            UiContainerKind::Space => {
                record_zircon_owned_container(engine_context, node_id, container, children);
                for child_id in children.iter().copied() {
                    hide_subtree_layout(tree, child_id, slot_index, engine_context)?;
                }
            }
            UiContainerKind::SizeBox(config) => {
                record_zircon_owned_container(engine_context, node_id, container, children);
                arrange_size_box_children(
                    tree,
                    node_id,
                    children,
                    frame,
                    next_clip,
                    config,
                    slot_index,
                    engine_context,
                )?;
            }
            UiContainerKind::HorizontalBox(config) => {
                if !try_arrange_taffy_owned_children(
                    tree,
                    node_id,
                    children,
                    frame,
                    next_clip,
                    slot_index,
                    engine_context,
                )? {
                    arrange_linear_children(
                        tree,
                        node_id,
                        children,
                        frame,
                        next_clip,
                        UiAxis::Horizontal,
                        config.gap,
                        &mut child_scratch.linear,
                        slot_index,
                        engine_context,
                    )?;
                }
            }
            UiContainerKind::VerticalBox(config) => {
                if !try_arrange_taffy_owned_children(
                    tree,
                    node_id,
                    children,
                    frame,
                    next_clip,
                    slot_index,
                    engine_context,
                )? {
                    arrange_linear_children(
                        tree,
                        node_id,
                        children,
                        frame,
                        next_clip,
                        UiAxis::Vertical,
                        config.gap,
                        &mut child_scratch.linear,
                        slot_index,
                        engine_context,
                    )?;
                }
            }
            UiContainerKind::ScrollableBox(config) => {
                record_zircon_owned_container(engine_context, node_id, container, children);
                arrange_scrollable_children(
                    tree,
                    node_id,
                    children,
                    frame,
                    next_clip,
                    config,
                    slot_index,
                    engine_context,
                )?;
            }
            UiContainerKind::WrapBox(config) => {
                if !try_arrange_taffy_owned_children(
                    tree,
                    node_id,
                    children,
                    frame,
                    next_clip,
                    slot_index,
                    engine_context,
                )? {
                    arrange_wrap_children(
                        tree,
                        node_id,
                        children,
                        frame,
                        next_clip,
                        config,
                        &mut child_scratch.wrap_row_items,
                        slot_index,
                        engine_context,
                    )?;
                }

                let content_size = wrap_content_size(
                    tree,
                    node_id,
                    children,
                    config,
                    frame.width,
                    &mut child_scratch.wrap_content_desired,
                    slot_index,
                )?;
                let node = tree
                    .node_mut(node_id)
                    .ok_or(UiTreeError::MissingNode(node_id))?;
                node.layout_cache.content_size = content_size;
            }
            UiContainerKind::GridBox(config) => {
                if !try_arrange_taffy_owned_children(
                    tree,
                    node_id,
                    children,
                    frame,
                    next_clip,
                    slot_index,
                    engine_context,
                )? {
                    arrange_grid_children(
                        tree,
                        node_id,
                        children,
                        frame,
                        next_clip,
                        config,
                        slot_index,
                        engine_context,
                    )?;
                }
            }
            UiContainerKind::MasonryBox(config) => {
                record_zircon_owned_container(engine_context, node_id, container, children);
                let content_size = arrange_masonry_children(
                    tree,
                    node_id,
                    children,
                    frame,
                    next_clip,
                    config,
                    &mut child_scratch.masonry,
                    slot_index,
                    engine_context,
                )?;
                let node = tree
                    .node_mut(node_id)
                    .ok_or(UiTreeError::MissingNode(node_id))?;
                node.layout_cache.content_size = content_size;
            }
        }
        Ok(())
    })();
    slot_index.recycle_arrange_child_scratch(child_scratch);
    arrange_result?;

    Ok(())
}

pub(crate) fn arrange_resized_root(
    tree: &mut UiTree,
    node_id: UiNodeId,
    frame: UiFrame,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let (container, has_children, can_use_dependency_index) = {
        let node = tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        (
            node.container,
            !node.children.is_empty(),
            node.effective_visibility().occupies_layout()
                && !node.clip_to_bounds
                && !node.container.clips_to_bounds()
                && matches!(
                    node.container,
                    UiContainerKind::Free
                        | UiContainerKind::Canvas
                        | UiContainerKind::Container
                        | UiContainerKind::Overlay
                ),
        )
    };
    if !can_use_dependency_index {
        return arrange_node(tree, node_id, frame, None, slot_index, engine_context);
    }

    engine_context.record_arrangement_probe();
    let (previous_frame, previous_clip_frame) = {
        let node = tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        (node.layout_cache.frame, node.layout_cache.clip_frame)
    };
    engine_context.record_geometry(node_id, previous_frame, previous_clip_frame, frame, None);
    let node = tree
        .node_mut(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    node.layout_cache.frame = frame;
    node.layout_cache.clip_frame = None;
    node.dirty = Default::default();
    if has_children {
        engine_context.record_zircon_owned(node_id, container);
    }

    let mut child_scratch = slot_index.take_arrange_child_scratch();
    slot_index.copy_parent_size_dependent_children(tree, node_id, &mut child_scratch.children);
    let arrange_result = (|| -> Result<(), UiTreeError> {
        for child_id in child_scratch.children.iter().copied() {
            let child_frame = {
                let slot = slot_for_container_child(tree, slot_index, node_id, child_id, container);
                free_child_frame(tree, child_id, frame, slot)?
            };
            arrange_node(
                tree,
                child_id,
                child_frame,
                None,
                slot_index,
                engine_context,
            )?;
        }
        Ok(())
    })();
    slot_index.recycle_arrange_child_scratch(child_scratch);
    arrange_result
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
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = UiContainerKind::SizeBox(config);
    let content_frame = size_box_content_frame(frame, config.aspect_ratio);
    for child_id in children.iter().copied() {
        let child_frame = {
            let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
            free_child_frame(tree, child_id, content_frame, slot)?
        };
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

fn arrange_block_children(
    tree: &mut UiTree,
    parent_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = UiContainerKind::BlockBox;
    let mut cursor = 0.0;

    for child_id in children.iter().copied() {
        let Some(node) = tree.node(child_id) else {
            return Err(UiTreeError::MissingNode(child_id));
        };
        if !node.effective_visibility().occupies_layout() {
            hide_subtree_layout(tree, child_id, slot_index, engine_context)?;
            continue;
        }

        let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
        let main_extent = block_child_outer_height(tree, child_id, slot)?;
        let child_frame = linear_child_frame(
            tree,
            child_id,
            frame,
            UiAxis::Vertical,
            cursor,
            main_extent,
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
        cursor += main_extent;
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
    wrap_content_desired: &mut Vec<(UiNodeId, DesiredSize)>,
    slot_index: &UiLayoutSlotIndex,
) -> Result<UiSize, UiTreeError> {
    wrap_content_desired.clear();
    for child_id in children.iter().copied() {
        let Some(child) = tree.node(child_id) else {
            return Err(UiTreeError::MissingNode(parent_id));
        };
        if child.effective_visibility().occupies_layout() {
            wrap_content_desired.push((child_id, child.layout_cache.desired_size));
        }
    }
    Ok(measure_wrap_content_size_for_width(
        tree,
        parent_id,
        UiContainerKind::WrapBox(config),
        config,
        wrap_content_desired,
        available_width,
        slot_index,
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
    linear_scratch: &mut UiLinearArrangeScratch,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = match axis {
        UiAxis::Horizontal => UiContainerKind::HorizontalBox(Default::default()),
        UiAxis::Vertical => UiContainerKind::VerticalBox(Default::default()),
    };
    resolve_linear_child_main_extents(
        tree,
        parent_id,
        children,
        axis,
        frame_axis_extent(frame, axis),
        gap,
        slot_index,
        linear_scratch,
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
            let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
            linear_child_frame(
                tree,
                child_id,
                frame,
                axis,
                cursor,
                linear_scratch.resolved[index].resolved,
                slot,
            )?
        };
        arrange_node(
            tree,
            child_id,
            child_frame,
            inherited_clip,
            slot_index,
            engine_context,
        )?;
        if occupies_layout {
            cursor += linear_scratch.resolved[index].resolved;
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
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    if config.virtualization.is_some() {
        let materialized = slot_index.with_materialized_virtual_list(node_id, |projection| {
            projection.map(|projection| {
                arrange_materialized_virtual_list_children(
                    tree,
                    node_id,
                    children,
                    frame,
                    inherited_clip,
                    config,
                    projection,
                    slot_index,
                    engine_context,
                )
            })
        });
        if let Some(result) = materialized {
            return result;
        }
    }

    let (content_size, previous_state, previous_window) = {
        let node = tree
            .node(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        (
            node.layout_cache.content_size,
            node.scroll_state.unwrap_or_default(),
            node.layout_cache.virtual_window,
        )
    };

    let viewport_extent = frame_axis_extent(frame, config.axis);
    let content_extent = size_axis_extent(content_size, config.axis);
    let plan = plan_scrollable_virtual_window(
        config,
        previous_state,
        previous_window,
        previous_state.offset,
        children.len(),
        viewport_extent,
        content_extent,
    );
    let visible_window = plan.virtual_window.unwrap_or(UiVirtualListWindow {
        first_visible: 0,
        last_visible_exclusive: children.len(),
    });

    {
        let node = tree
            .node_mut(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        node.scroll_state = Some(plan.scroll_state);
        node.layout_cache.virtual_window = Some(visible_window);
        node.dirty.visible_range |= plan.visible_range_changed;
    }

    validate_scrollable_children(tree, children)?;
    let mut cursor = 0.0;
    let gap = config.gap.max(0.0);
    let mut placed_count = 0usize;
    for (index, child_id) in children.iter().copied().enumerate() {
        let (occupies_layout, desired_extent) = {
            let node = tree
                .node(child_id)
                .ok_or(UiTreeError::MissingNode(child_id))?;
            (
                node.effective_visibility().occupies_layout(),
                size_axis_extent(
                    UiSize::new(
                        node.layout_cache.desired_size.width,
                        node.layout_cache.desired_size.height,
                    ),
                    config.axis,
                ),
            )
        };
        if occupies_layout && placed_count > 0 {
            cursor += gap;
        }
        let child_position = cursor;
        if index < visible_window.first_visible || index >= visible_window.last_visible_exclusive {
            hide_subtree_layout(tree, child_id, slot_index, engine_context)?;
        } else {
            let child_frame = scrollable_child_frame(
                tree,
                child_id,
                frame,
                config.axis,
                child_position,
                plan.scroll_state.offset,
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
        if occupies_layout {
            cursor += desired_extent;
            placed_count += 1;
        }
    }

    Ok(())
}

pub(super) fn validate_scrollable_children(
    tree: &UiTree,
    children: &[UiNodeId],
) -> Result<(), UiTreeError> {
    for child_id in children.iter().copied() {
        if tree.node(child_id).is_none() {
            return Err(UiTreeError::MissingNode(child_id));
        }
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
    wrap_row_items: &mut Vec<(UiNodeId, f32)>,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let container = UiContainerKind::WrapBox(config);
    let horizontal_gap = config.horizontal_gap.max(0.0);
    let vertical_gap = config.vertical_gap.max(0.0);
    let available_width = frame.width.max(0.0);
    let mut cursor_x = 0.0_f32;
    let mut cursor_y = 0.0_f32;
    let mut row_height = 0.0_f32;
    wrap_row_items.clear();

    for child_id in children.iter().copied() {
        let Some(node) = tree.node(child_id) else {
            return Err(UiTreeError::MissingNode(child_id));
        };
        if !node.effective_visibility().occupies_layout() {
            hide_subtree_layout(tree, child_id, slot_index, engine_context)?;
            continue;
        }

        let item_size = wrap_item_outer_size(tree, slot_index, parent_id, child_id, config)?;
        let item_width = item_size.width;
        let item_height = item_size.height;
        let next_width = if wrap_row_items.is_empty() {
            item_width
        } else {
            cursor_x + horizontal_gap + item_width
        };
        if !wrap_row_items.is_empty() && next_width > available_width {
            arrange_wrap_row(
                tree,
                parent_id,
                wrap_row_items,
                frame,
                inherited_clip,
                cursor_y,
                row_height,
                config,
                slot_index,
                engine_context,
            )?;
            cursor_y += row_height + vertical_gap;
            cursor_x = 0.0;
            row_height = 0.0;
            wrap_row_items.clear();
        }

        if !wrap_row_items.is_empty() {
            cursor_x += horizontal_gap;
        }
        cursor_x += item_width;
        row_height = row_height.max(item_height);
        wrap_row_items.push((child_id, item_width));
    }

    if !wrap_row_items.is_empty() {
        arrange_wrap_row(
            tree,
            parent_id,
            wrap_row_items,
            frame,
            inherited_clip,
            cursor_y,
            row_height,
            config,
            slot_index,
            engine_context,
        )?;
    }

    Ok(())
}

fn wrap_item_outer_size(
    tree: &UiTree,
    slot_index: &UiLayoutSlotIndex,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    config: UiWrapBoxConfig,
) -> Result<UiSize, UiTreeError> {
    let node = tree
        .node(child_id)
        .ok_or(UiTreeError::MissingNode(child_id))?;
    let padding = slot_padding(slot_for_container_child(
        tree,
        slot_index,
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

fn block_child_outer_height(
    tree: &UiTree,
    child_id: UiNodeId,
    slot: Option<&zircon_runtime_interface::ui::layout::UiSlot>,
) -> Result<f32, UiTreeError> {
    let node = tree
        .node(child_id)
        .ok_or(UiTreeError::MissingNode(child_id))?;
    Ok((node.layout_cache.desired_size.height + slot_padding(slot).vertical()).max(0.0))
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
    slot_index: &UiLayoutSlotIndex,
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
            let slot = slot_for_container_child(tree, slot_index, parent_id, child_id, container);
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
        arrange_node(
            tree,
            child_id,
            child_frame,
            inherited_clip,
            slot_index,
            engine_context,
        )?;
        cursor_x += item_width;
    }
    Ok(())
}

pub(super) fn hide_subtree_layout(
    tree: &mut UiTree,
    node_id: UiNodeId,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let node = tree
        .node(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    if engine_context.can_reuse_geometry(
        node_id,
        node.layout_cache.frame,
        node.layout_cache.clip_frame,
        UiFrame::default(),
        None,
    ) {
        return Ok(());
    }

    let mut pending = slot_index.take_hidden_subtree_stack();
    pending.push(node_id);
    let hide_result = (|| -> Result<(), UiTreeError> {
        while let Some(current_id) = pending.pop() {
            let node = tree
                .node_mut(current_id)
                .ok_or(UiTreeError::MissingNode(current_id))?;
            engine_context.record_geometry(
                current_id,
                node.layout_cache.frame,
                node.layout_cache.clip_frame,
                UiFrame::default(),
                None,
            );
            node.layout_cache.frame = UiFrame::default();
            node.layout_cache.clip_frame = None;
            pending.extend(node.children.iter().rev().copied());
        }
        Ok(())
    })();
    slot_index.recycle_hidden_subtree_stack(pending);
    hide_result
}
