use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiAlignment, UiAxis, UiFrame, UiMargin, UiSlot},
    tree::{UiTree, UiTreeError},
};

use super::axis::{arranged_axis_extent, stacked_axis_extent};
use super::slot::{has_slot_frame_policy, slot_padding};

pub(crate) fn free_child_frame(
    tree: &UiTree,
    node_id: UiNodeId,
    parent_frame: UiFrame,
    slot: Option<&UiSlot>,
) -> Result<UiFrame, UiTreeError> {
    let node = tree
        .node(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;

    if has_slot_frame_policy(slot) {
        let slot = slot.expect("slot policy requires a slot");
        return Ok(aligned_child_frame(
            node.constraints,
            node.layout_cache.desired_size.width,
            node.layout_cache.desired_size.height,
            node.position.x,
            node.position.y,
            inset_frame(parent_frame, slot_padding(Some(slot))),
            slot.alignment.horizontal,
            slot.alignment.vertical,
        ));
    }

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
    node_id: UiNodeId,
    parent_frame: UiFrame,
    axis: UiAxis,
    start: f32,
    main_extent: f32,
    slot: Option<&UiSlot>,
) -> Result<UiFrame, UiTreeError> {
    let node = tree
        .node(node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;

    if has_slot_frame_policy(slot) {
        let slot = slot.expect("slot policy requires a slot");
        let outer_frame = match axis {
            UiAxis::Horizontal => UiFrame::new(
                parent_frame.x + start,
                parent_frame.y,
                main_extent.max(0.0),
                parent_frame.height,
            ),
            UiAxis::Vertical => UiFrame::new(
                parent_frame.x,
                parent_frame.y + start,
                parent_frame.width,
                main_extent.max(0.0),
            ),
        };
        return Ok(aligned_child_frame(
            node.constraints,
            node.layout_cache.desired_size.width,
            node.layout_cache.desired_size.height,
            node.position.x,
            node.position.y,
            inset_frame(outer_frame, slot_padding(Some(slot))),
            slot.alignment.horizontal,
            slot.alignment.vertical,
        ));
    }

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

fn aligned_child_frame(
    constraints: zircon_runtime_interface::ui::layout::BoxConstraints,
    desired_width: f32,
    desired_height: f32,
    offset_x: f32,
    offset_y: f32,
    content_frame: UiFrame,
    horizontal_alignment: UiAlignment,
    vertical_alignment: UiAlignment,
) -> UiFrame {
    let width = arranged_axis_extent(constraints.width, desired_width, content_frame.width);
    let height = arranged_axis_extent(constraints.height, desired_height, content_frame.height);
    UiFrame::new(
        aligned_axis_origin(
            content_frame.x,
            content_frame.width,
            width,
            horizontal_alignment,
            offset_x,
        ),
        aligned_axis_origin(
            content_frame.y,
            content_frame.height,
            height,
            vertical_alignment,
            offset_y,
        ),
        width,
        height,
    )
}

fn aligned_axis_origin(
    start: f32,
    available: f32,
    extent: f32,
    alignment: UiAlignment,
    offset: f32,
) -> f32 {
    let aligned_start = match alignment {
        UiAlignment::Start | UiAlignment::Fill => start,
        UiAlignment::Center => start + (available - extent) * 0.5,
        UiAlignment::End => start + available - extent,
    };
    aligned_start + offset
}

fn inset_frame(frame: UiFrame, padding: UiMargin) -> UiFrame {
    UiFrame::new(
        frame.x + padding.left,
        frame.y + padding.top,
        (frame.width - padding.horizontal()).max(0.0),
        (frame.height - padding.vertical()).max(0.0),
    )
}

pub(crate) fn scrollable_child_frame(
    tree: &UiTree,
    node_id: UiNodeId,
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
