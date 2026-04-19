use crate::tree::UiTreeError;
use crate::{UiAxis, UiFrame, UiTree};

use super::axis::{arranged_axis_extent, stacked_axis_extent};

pub(crate) fn free_child_frame(
    tree: &UiTree,
    node_id: crate::event_ui::UiNodeId,
    parent_frame: UiFrame,
) -> Result<UiFrame, UiTreeError> {
    let node = tree
        .node(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;

    let width = arranged_axis_extent(
        node.constraints.width,
        node.layout_cache.desired_size.width,
        parent_frame.width,
    );
    let height = arranged_axis_extent(
        node.constraints.height,
        node.layout_cache.desired_size.height,
        parent_frame.height,
    );
    let x = parent_frame.x + parent_frame.width * node.anchor.x + node.position.x
        - width * node.pivot.x;
    let y = parent_frame.y + parent_frame.height * node.anchor.y + node.position.y
        - height * node.pivot.y;

    Ok(UiFrame::new(x, y, width, height))
}

pub(crate) fn linear_child_frame(
    tree: &UiTree,
    node_id: crate::event_ui::UiNodeId,
    parent_frame: UiFrame,
    axis: UiAxis,
    start: f32,
    main_extent: f32,
) -> Result<UiFrame, UiTreeError> {
    let node = tree
        .node(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    match axis {
        UiAxis::Vertical => {
            let width = arranged_axis_extent(
                node.constraints.width,
                node.layout_cache.desired_size.width,
                parent_frame.width,
            );
            let x = parent_frame.x + parent_frame.width * node.anchor.x + node.position.x
                - width * node.pivot.x;
            let y = parent_frame.y + start + node.position.y;
            Ok(UiFrame::new(x, y, width, main_extent.max(0.0)))
        }
        UiAxis::Horizontal => {
            let height = arranged_axis_extent(
                node.constraints.height,
                node.layout_cache.desired_size.height,
                parent_frame.height,
            );
            let x = parent_frame.x + start + node.position.x;
            let y = parent_frame.y + parent_frame.height * node.anchor.y + node.position.y
                - height * node.pivot.y;
            Ok(UiFrame::new(x, y, main_extent.max(0.0), height))
        }
    }
}

pub(crate) fn scrollable_child_frame(
    tree: &UiTree,
    node_id: crate::event_ui::UiNodeId,
    parent_frame: UiFrame,
    axis: UiAxis,
    start: f32,
    offset: f32,
) -> Result<UiFrame, UiTreeError> {
    let node = tree
        .node(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    match axis {
        UiAxis::Vertical => {
            let width = arranged_axis_extent(
                node.constraints.width,
                node.layout_cache.desired_size.width,
                parent_frame.width,
            );
            let height = stacked_axis_extent(
                node.constraints.height,
                node.layout_cache.desired_size.height,
            );
            let x = parent_frame.x + parent_frame.width * node.anchor.x + node.position.x
                - width * node.pivot.x;
            let y = parent_frame.y + start - offset + node.position.y;
            Ok(UiFrame::new(x, y, width, height))
        }
        UiAxis::Horizontal => {
            let width =
                stacked_axis_extent(node.constraints.width, node.layout_cache.desired_size.width);
            let height = arranged_axis_extent(
                node.constraints.height,
                node.layout_cache.desired_size.height,
                parent_frame.height,
            );
            let x = parent_frame.x + start - offset + node.position.x;
            let y = parent_frame.y + parent_frame.height * node.anchor.y + node.position.y
                - height * node.pivot.y;
            Ok(UiFrame::new(x, y, width, height))
        }
    }
}
