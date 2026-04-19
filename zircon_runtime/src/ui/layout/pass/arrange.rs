use crate::ui::tree::UiTreeError;
use crate::ui::{
    layout::UiAxis, layout::UiContainerKind, layout::UiFrame, layout::UiScrollState,
    layout::UiVirtualListWindow, tree::UiTree,
};

use super::axis::{frame_axis_extent, resolve_linear_child_main_extents, size_axis_extent};
use super::child_frame::{free_child_frame, linear_child_frame, scrollable_child_frame};
use super::clip::resolve_clip_frame;

pub(crate) fn arrange_node(
    tree: &mut UiTree,
    node_id: crate::ui::event_ui::UiNodeId,
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
) -> Result<(), UiTreeError> {
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
            for child_id in children {
                let child_frame = free_child_frame(tree, child_id, frame)?;
                arrange_node(tree, child_id, child_frame, next_clip)?;
            }
        }
        UiContainerKind::Space => {
            for child_id in children {
                hide_subtree_layout(tree, child_id)?;
            }
        }
        UiContainerKind::HorizontalBox(config) => {
            arrange_linear_children(
                tree,
                &children,
                frame,
                next_clip,
                UiAxis::Horizontal,
                config.gap,
            )?;
        }
        UiContainerKind::VerticalBox(config) => {
            arrange_linear_children(
                tree,
                &children,
                frame,
                next_clip,
                UiAxis::Vertical,
                config.gap,
            )?;
        }
        UiContainerKind::ScrollableBox(config) => {
            arrange_scrollable_children(tree, node_id, &children, frame, next_clip, config)?;
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

fn arrange_linear_children(
    tree: &mut UiTree,
    children: &[crate::ui::event_ui::UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    axis: UiAxis,
    gap: f32,
) -> Result<(), UiTreeError> {
    let main_extents = resolve_linear_child_main_extents(
        tree,
        children,
        axis,
        frame_axis_extent(frame, axis),
        gap,
    )?;
    let gap = gap.max(0.0);
    let mut cursor = 0.0;

    for (index, child_id) in children.iter().copied().enumerate() {
        let child_frame =
            linear_child_frame(tree, child_id, frame, axis, cursor, main_extents[index])?;
        arrange_node(tree, child_id, child_frame, inherited_clip)?;
        cursor += main_extents[index] + gap;
    }

    Ok(())
}

fn arrange_scrollable_children(
    tree: &mut UiTree,
    node_id: crate::ui::event_ui::UiNodeId,
    children: &[crate::ui::event_ui::UiNodeId],
    frame: UiFrame,
    inherited_clip: Option<UiFrame>,
    config: crate::ui::layout::UiScrollableBoxConfig,
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
    let visible_window = config
        .virtual_window(offset, children.len(), viewport_extent)
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
        arrange_node(tree, child_id, child_frame, inherited_clip)?;
    }

    Ok(())
}

fn child_positions(
    tree: &UiTree,
    children: &[crate::ui::event_ui::UiNodeId],
    axis: UiAxis,
    gap: f32,
) -> Result<Vec<f32>, UiTreeError> {
    let mut positions = Vec::with_capacity(children.len());
    let mut cursor = 0.0;
    let gap = gap.max(0.0);
    for child_id in children {
        positions.push(cursor);
        let node = tree
            .node(*child_id)
            .ok_or(UiTreeError::MissingNode(*child_id))?;
        cursor += match axis {
            UiAxis::Vertical => node.layout_cache.desired_size.height,
            UiAxis::Horizontal => node.layout_cache.desired_size.width,
        } + gap;
    }
    Ok(positions)
}

fn hide_subtree_layout(
    tree: &mut UiTree,
    node_id: crate::ui::event_ui::UiNodeId,
) -> Result<(), UiTreeError> {
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
