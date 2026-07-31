use crate::ui::retained_host::host_contract::data::FrameRect;

use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_tree_row_geometry::tree_metrics;
use super::super::push_tree_row_commands;
use super::support::tree_node;

#[test]
fn degenerate_tree_row_does_not_emit_paint_commands() {
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 0.0,
        height: 24.0,
    };
    let mut commands = Vec::new();

    assert!(push_tree_row_commands(
        &mut commands,
        &tree_node(
            "WorkbenchSceneRootItem",
            "TreeRow",
            "tree-row",
            "Root",
            0,
            false
        ),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.is_empty());
}

#[test]
fn collapsed_tree_row_label_slot_does_not_emit_a_one_pixel_text_command() {
    let metrics = tree_metrics();
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: metrics.tree_base_inset_x
            + metrics.tree_disclosure_size
            + metrics.tree_text_gap
            + metrics.tree_icon_size
            + metrics.tree_text_gap
            + metrics.tree_right_inset
            + metrics.tree_action_size * 2.0
            + metrics.tree_action_gap,
        height: 30.0,
    };
    let mut commands = Vec::new();

    assert!(push_tree_row_commands(
        &mut commands,
        &tree_node(
            "WorkbenchSceneRootItem",
            "TreeRow",
            "tree-row",
            "Root",
            0,
            false
        ),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| command.text.is_none()));
}

#[test]
fn narrow_tree_row_elides_actions_that_would_escape_its_frame() {
    let metrics = tree_metrics();
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: metrics.tree_right_inset
            + metrics.tree_action_size
            + metrics.tree_action_size
            + metrics.tree_action_gap
            + (metrics.tree_action_button_size - metrics.tree_action_size) * 0.5
            - 0.25,
        height: metrics.tree_action_button_size,
    };
    let mut commands = Vec::new();

    assert!(push_tree_row_commands(
        &mut commands,
        &tree_node(
            "WorkbenchSceneRootItem",
            "TreeRow",
            "tree-row",
            "Root",
            0,
            false
        ),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| !is_tree_action(command)));
}

#[test]
fn short_tree_row_elides_actions_that_would_escape_its_frame() {
    let metrics = tree_metrics();
    let rect = FrameRect {
        x: 8.0,
        y: 6.0,
        width: 200.0,
        height: metrics.tree_action_button_size - 0.25,
    };
    let mut commands = Vec::new();

    assert!(push_tree_row_commands(
        &mut commands,
        &tree_node(
            "WorkbenchSceneRootItem",
            "TreeRow",
            "tree-row",
            "Root",
            0,
            false
        ),
        &rect,
        &rect,
        4,
        1.0,
    ));

    assert!(commands.iter().all(|command| !is_tree_action(command)));
}

fn is_tree_action(command: &HostPaintCommand) -> bool {
    command.image_pixels.as_ref().is_some_and(|image| {
        image.resource_key.contains("eye.svg")
            || image.resource_key.contains("lock.svg")
            || image.resource_key.contains("more-vertical.svg")
    })
}
