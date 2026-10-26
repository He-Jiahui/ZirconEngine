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

#[test]
fn tree_row_content_icon_and_label_keep_the_row_center_when_icon_exceeds_disclosure() {
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_at_scale_for_test,
    };

    for scale in [1.0, 1.25, 1.5] {
        let _theme =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));
        let metrics = tree_metrics();
        assert!(metrics.tree_icon_size > metrics.tree_disclosure_size);
        let rect = FrameRect {
            x: 8.25 * scale,
            y: 6.25 * scale,
            width: 319.0 * scale,
            height: 28.0 * scale,
        };
        let mut node = tree_node(
            "NativeHierarchyEntity:2",
            "TreeRow",
            "tree-row",
            "Sun",
            0,
            false,
        );
        node.component_variant = "content".into();
        let mut commands = Vec::new();
        assert!(push_tree_row_commands(
            &mut commands,
            &node,
            &rect,
            &rect,
            0,
            1.0
        ));
        let images = commands
            .iter()
            .filter(|command| command.image_pixels.is_some())
            .collect::<Vec<_>>();
        let labels = commands
            .iter()
            .filter(|command| command.text.is_some())
            .collect::<Vec<_>>();
        assert_eq!(images.len(), 1);
        assert_eq!(labels.len(), 1);
        let center = rect.y + rect.height * 0.5;
        for command in [images[0], labels[0]] {
            assert!((command.frame.y + command.frame.height * 0.5 - center).abs() < 0.01);
            assert!(command.frame.y >= rect.y);
            assert!(command.frame.y + command.frame.height <= rect.y + rect.height);
        }
    }
}

#[test]
fn tree_row_content_icon_outside_a_short_row_is_not_painted() {
    let metrics = tree_metrics();
    assert!(metrics.tree_icon_size > metrics.tree_disclosure_size);
    let rect = FrameRect {
        x: 8.25,
        y: 6.25,
        width: 319.0,
        height: metrics.tree_icon_size - 0.25,
    };
    let mut node = tree_node(
        "NativeHierarchyEntity:2",
        "TreeRow",
        "tree-row",
        "Sun",
        0,
        false,
    );
    node.component_variant = "content".into();
    let mut commands = Vec::new();
    assert!(push_tree_row_commands(
        &mut commands,
        &node,
        &rect,
        &rect,
        0,
        1.0
    ));
    assert!(commands
        .iter()
        .all(|command| command.image_pixels.is_none()));
    assert!(commands.iter().all(|command| command.text.is_none()));
}

fn is_tree_action(command: &HostPaintCommand) -> bool {
    command.image_pixels.as_ref().is_some_and(|image| {
        image.resource_key.contains("eye.svg")
            || image.resource_key.contains("lock.svg")
            || image.resource_key.contains("more-vertical.svg")
    })
}
