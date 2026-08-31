use crate::ui::layout::{
    fixed_extent_virtual_list_content_extent, fixed_extent_virtual_list_item_offset,
    plan_scrollable_virtual_window,
};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiAxis, UiFrame, UiScrollableBoxConfig, UiSize, UiVirtualListWindow},
    tree::{UiTree, UiTreeError},
};

use super::{arrange_node, hide_subtree_layout, validate_scrollable_children};
use crate::ui::layout::pass::{
    axis::frame_axis_extent, engine::UiLayoutPassEngineContext, slot::UiLayoutSlotIndex,
    virtual_list_layout::UiMaterializedVirtualListLayout,
};

pub(super) fn arrange_materialized_virtual_list_children(
    tree: &mut UiTree,
    node_id: UiNodeId,
    children: &[UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    config: UiScrollableBoxConfig,
    projection: &UiMaterializedVirtualListLayout,
    slot_index: &UiLayoutSlotIndex,
    engine_context: &mut UiLayoutPassEngineContext,
) -> Result<(), UiTreeError> {
    let virtualization = config
        .virtualization
        .expect("materialized virtual-list projection requires virtualization");
    let (previous_content_size, previous_state, previous_window) = {
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
    let content_extent = fixed_extent_virtual_list_content_extent(
        projection.logical_count(),
        virtualization.item_extent,
        config.gap,
    );
    let content_size = size_with_axis_extent(previous_content_size, config.axis, content_extent);
    let plan = plan_scrollable_virtual_window(
        config,
        previous_state,
        previous_window,
        previous_state.offset,
        projection.logical_count(),
        viewport_extent,
        content_extent,
    );
    let visible_window = plan.virtual_window.unwrap_or(UiVirtualListWindow {
        first_visible: 0,
        last_visible_exclusive: projection.logical_count(),
    });

    {
        let node = tree
            .node_mut(node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        node.scroll_state = Some(plan.scroll_state);
        node.layout_cache.content_size = content_size;
        node.layout_cache.virtual_window = Some(visible_window);
        node.dirty.visible_range |= plan.visible_range_changed;
    }

    validate_scrollable_children(tree, children)?;
    for child_id in children.iter().copied() {
        let Some(logical_index) = projection.logical_index_for_child(child_id) else {
            hide_subtree_layout(tree, child_id, slot_index, engine_context)?;
            continue;
        };
        let child_position = fixed_extent_virtual_list_item_offset(
            logical_index,
            virtualization.item_extent,
            config.gap,
        );
        let child_frame = frame_with_axis_extent(
            super::scrollable_child_frame(
                tree,
                child_id,
                frame,
                config.axis,
                child_position,
                plan.scroll_state.offset,
            )?,
            config.axis,
            virtualization.item_extent,
        );
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

fn size_with_axis_extent(mut size: UiSize, axis: UiAxis, extent: f32) -> UiSize {
    match axis {
        UiAxis::Horizontal => size.width = extent,
        UiAxis::Vertical => size.height = extent,
    }
    size
}

fn frame_with_axis_extent(mut frame: UiFrame, axis: UiAxis, extent: f32) -> UiFrame {
    match axis {
        UiAxis::Horizontal => frame.width = extent.max(0.0),
        UiAxis::Vertical => frame.height = extent.max(0.0),
    }
    frame
}
