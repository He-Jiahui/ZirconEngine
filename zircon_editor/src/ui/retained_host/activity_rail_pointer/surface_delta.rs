use zircon_runtime_interface::ui::{event_ui::UiNodeId, layout::UiFrame};

use super::constants::{LEFT_STRIP_NODE_ID, RIGHT_STRIP_NODE_ID, ROOT_NODE_ID};
use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;
use super::host_activity_rail_pointer_side::HostActivityRailPointerSide;
use super::root_frame::root_frame;
use super::strip_button_node_id::strip_button_node_id;

#[derive(Debug)]
pub(super) enum ActivityRailSurfaceDelta {
    NoChange,
    Geometry(Vec<ActivityRailNodeFrameChange>),
    Topology,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ActivityRailNodeFrameChange {
    pub(super) node_id: UiNodeId,
    pub(super) frame: UiFrame,
}

pub(super) fn surface_delta_for_layout(
    previous: &HostActivityRailPointerLayout,
    next: &HostActivityRailPointerLayout,
) -> ActivityRailSurfaceDelta {
    if strip_node_count(previous, HostActivityRailPointerSide::Left)
        != strip_node_count(next, HostActivityRailPointerSide::Left)
        || strip_node_count(previous, HostActivityRailPointerSide::Right)
            != strip_node_count(next, HostActivityRailPointerSide::Right)
    {
        return ActivityRailSurfaceDelta::Topology;
    }

    let mut changes = Vec::new();
    push_change(
        &mut changes,
        ROOT_NODE_ID,
        root_frame(previous),
        root_frame(next),
    );
    if strip_is_visible(next.left_strip_frame) {
        push_change(
            &mut changes,
            LEFT_STRIP_NODE_ID,
            previous.left_strip_frame,
            next.left_strip_frame,
        );
        for index in 0..previous.left_tabs.len() {
            push_change(
                &mut changes,
                strip_button_node_id(HostActivityRailPointerSide::Left, index),
                button_frame(previous.left_strip_frame, index),
                button_frame(next.left_strip_frame, index),
            );
        }
    }
    if strip_is_visible(next.right_strip_frame) {
        push_change(
            &mut changes,
            RIGHT_STRIP_NODE_ID,
            previous.right_strip_frame,
            next.right_strip_frame,
        );
        for index in 0..previous.right_tabs.len() {
            push_change(
                &mut changes,
                strip_button_node_id(HostActivityRailPointerSide::Right, index),
                button_frame(previous.right_strip_frame, index),
                button_frame(next.right_strip_frame, index),
            );
        }
    }

    if changes.is_empty() {
        ActivityRailSurfaceDelta::NoChange
    } else {
        ActivityRailSurfaceDelta::Geometry(changes)
    }
}

fn strip_node_count(
    layout: &HostActivityRailPointerLayout,
    side: HostActivityRailPointerSide,
) -> usize {
    let (frame, tabs) = match side {
        HostActivityRailPointerSide::Left => (layout.left_strip_frame, layout.left_tabs.len()),
        HostActivityRailPointerSide::Right => (layout.right_strip_frame, layout.right_tabs.len()),
    };
    if strip_is_visible(frame) {
        1 + tabs
    } else {
        0
    }
}

fn strip_is_visible(frame: UiFrame) -> bool {
    frame.width > 0.0 && frame.height > 0.0
}

pub(super) fn button_frame(frame: UiFrame, index: usize) -> UiFrame {
    UiFrame::new(
        frame.x + super::constants::STRIP_X_INSET,
        frame.y
            + super::constants::STRIP_Y_INSET
            + index as f32 * (super::constants::BUTTON_EXTENT + super::constants::BUTTON_GAP),
        super::constants::BUTTON_EXTENT,
        super::constants::BUTTON_EXTENT,
    )
}

fn push_change(
    changes: &mut Vec<ActivityRailNodeFrameChange>,
    node_id: UiNodeId,
    before: UiFrame,
    after: UiFrame,
) {
    if before != after {
        changes.push(ActivityRailNodeFrameChange {
            node_id,
            frame: after,
        });
    }
}
