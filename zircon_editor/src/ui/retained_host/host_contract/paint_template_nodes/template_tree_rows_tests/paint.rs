use crate::ui::layouts::common::model_rc;

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::template_tree_row_geometry::{tree_disclosure_rect, tree_icon_rect};
use super::super::push_tree_row_commands;
use super::super::style::tree_icon_color;
use super::support::{changed_pixel_count, pixel_at, tree_node};

#[test]
fn selected_tree_row_paints_muted_selected_fill_neutral_outline_icon_and_actions() {
    let bytes = paint_template_nodes_for_test(
        280,
        48,
        model_rc(vec![tree_node(
            "WorkbenchScenePropsItem",
            "TreeRow",
            "tree-row",
            "Props",
            2,
            true,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 280, 4, 19), PALETTE.border);
    assert_eq!(pixel_at(&bytes, 280, 14, 19), PALETTE.surface_pressed);
    assert_ne!(pixel_at(&bytes, 280, 14, 19), PALETTE.surface_selected);
    assert!(changed_pixel_count(&bytes, 280, 50, 10, 40, 24) > 0);
    assert!(changed_pixel_count(&bytes, 280, 230, 13, 40, 18) > 0);
}

#[test]
fn nested_tree_row_draws_indent_guides_without_full_surface() {
    let bytes = paint_template_nodes_for_test(
        240,
        42,
        model_rc(vec![tree_node(
            "WorkbenchSceneEnvironmentItem",
            "TreeRow",
            "tree-row",
            "Environment",
            1,
            false,
        )]),
    );

    assert_eq!(pixel_at(&bytes, 240, 8, 18), [0, 0, 0, 255]);
    assert_ne!(pixel_at(&bytes, 240, 21, 18), [0, 0, 0, 255]);
    assert!(changed_pixel_count(&bytes, 240, 32, 10, 48, 22) > 0);
}

#[test]
fn collapsed_tree_row_paints_right_chevron() {
    let bytes = paint_template_nodes_for_test(
        240,
        42,
        model_rc(vec![tree_node(
            "WorkbenchScenePlayerStartItem",
            "TreeRow",
            "tree-row",
            "PlayerStart",
            0,
            false,
        )]),
    );

    assert!(changed_pixel_count(&bytes, 240, 14, 11, 14, 16) > 0);
    assert!(changed_pixel_count(&bytes, 240, 32, 10, 28, 22) > 0);
}

#[test]
fn loading_player_start_tree_row_mutes_special_icon_color() {
    let mut node = tree_node(
        "WorkbenchScenePlayerStartItem",
        "TreeRow",
        "tree-row",
        "PlayerStart",
        0,
        false,
    );
    node.button_style.loading = true;
    let row_rect = FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    let disclosure = tree_disclosure_rect(&node, &row_rect);
    let icon = tree_icon_rect(&disclosure);
    assert_eq!(tree_icon_color(&node), PALETTE.text_disabled);
    let bytes = paint_template_nodes_for_test(280, 48, model_rc(vec![node]));

    assert!(
        changed_pixel_count(
            &bytes,
            280,
            icon.x.round() as u32,
            icon.y.round() as u32,
            icon.width.round() as u32,
            icon.height.round() as u32,
        ) > 0
    );
}

#[test]
fn tree_row_with_shell_icon_paints_real_asset_pixels() {
    let mut node = tree_node(
        "WorkbenchScenePropsItem",
        "TreeRow",
        "tree-row",
        "Props",
        2,
        true,
    );
    node.icon_name = "zircon_editor_shell/scene/props.svg".into();
    let rect = FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    let mut commands = Vec::new();

    let handled = push_tree_row_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

    assert!(handled);
    let asset_pixels = commands
        .iter()
        .filter_map(|command| command.image_pixels.as_ref())
        .collect::<Vec<_>>();
    assert!(
        !asset_pixels.is_empty(),
        "tree row object icon should render through the shared shell SVG asset path"
    );
    assert!(
        asset_pixels
            .iter()
            .all(|image| !image.resource_key.starts_with("missing-icon:")),
        "tree row object icon should not fall back to missing-icon pixels"
    );
}

#[test]
fn tree_row_disclosure_and_actions_paint_shell_asset_pixels() {
    for node in [
        tree_node(
            "WorkbenchScenePropsItem",
            "TreeRow",
            "tree-row",
            "Props",
            2,
            true,
        ),
        tree_node(
            "WorkbenchScenePlayerStartItem",
            "TreeRow",
            "tree-row",
            "PlayerStart",
            0,
            false,
        ),
    ] {
        let rect = FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        };
        let mut commands = Vec::new();

        let handled = push_tree_row_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

        assert!(handled);
        let asset_pixels = commands
            .iter()
            .filter_map(|command| command.image_pixels.as_ref())
            .collect::<Vec<_>>();
        assert!(
            asset_pixels.len() >= 4,
            "{} should render disclosure, object icon, visibility, and row action through shell icon pixels",
            node.control_id
        );
        assert!(
            asset_pixels
                .iter()
                .all(|image| !image.resource_key.starts_with("missing-icon:")),
            "{} should not use missing-icon pixels for tree row glyphs",
            node.control_id
        );
    }
}

#[test]
fn tree_row_actions_use_standard_icon_button_slots() {
    for (node, action_keys) in [
        (
            tree_node(
                "WorkbenchScenePropsItem",
                "TreeRow",
                "tree-row",
                "Props",
                2,
                true,
            ),
            ["eye.svg", "more-vertical.svg"],
        ),
        (
            tree_node(
                "WorkbenchScenePlayerStartItem",
                "TreeRow",
                "tree-row",
                "PlayerStart",
                0,
                false,
            ),
            ["eye.svg", "lock.svg"],
        ),
    ] {
        let rect = FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        };
        let mut commands = Vec::new();

        let handled = push_tree_row_commands(&mut commands, &node, &rect, &rect, 0, 1.0);

        assert!(handled);
        let action_slots = tree_action_button_slot_commands(&commands);
        assert_eq!(
            action_slots.len(),
            2,
            "{} should paint one button slot per visible action",
            node.control_id
        );
        for resource_key in action_keys {
            let action_image =
                tree_action_image_command(&commands, resource_key).expect(resource_key);
            assert_frame_size(&action_image.frame, 16.0, 16.0);
            let action_slot = tree_action_button_slot_for_image(&commands, &action_image.frame)
                .expect("tree row action image should be centered in a button slot");
            assert_frame_size(&action_slot.frame, 20.0, 20.0);
            assert_eq!(action_slot.background_color, Some(PALETTE.surface_hover));
            assert_eq!(action_slot.border_color, Some(PALETTE.border));
            assert_eq!(action_slot.border_width, 1.0);
            assert_eq!(action_slot.corner_radius, 4.0);
            assert_eq!(action_slot.z_index + 1, action_image.z_index);
        }
    }
}

fn tree_action_image_command<'a>(
    commands: &'a [HostPaintCommand],
    resource_key: &str,
) -> Option<&'a HostPaintCommand> {
    commands.iter().find(|command| {
        command
            .image_pixels
            .as_ref()
            .is_some_and(|image| image.resource_key.contains(resource_key))
    })
}

fn tree_action_button_slot_commands(commands: &[HostPaintCommand]) -> Vec<&HostPaintCommand> {
    commands
        .iter()
        .filter(|command| is_tree_action_button_slot_command(command))
        .collect()
}

fn tree_action_button_slot_for_image<'a>(
    commands: &'a [HostPaintCommand],
    image: &FrameRect,
) -> Option<&'a HostPaintCommand> {
    commands.iter().find(|command| {
        is_tree_action_button_slot_command(command) && rect_contains(&command.frame, image)
    })
}

fn is_tree_action_button_slot_command(command: &HostPaintCommand) -> bool {
    command.frame.width == 20.0
        && command.frame.height == 20.0
        && command.background_color == Some(PALETTE.surface_hover)
        && command.border_color == Some(PALETTE.border)
}

fn assert_frame_size(frame: &FrameRect, width: f32, height: f32) {
    assert_eq!(frame.width, width);
    assert_eq!(frame.height, height);
}

fn rect_contains(outer: &FrameRect, inner: &FrameRect) -> bool {
    outer.x <= inner.x
        && outer.y <= inner.y
        && outer.x + outer.width >= inner.x + inner.width
        && outer.y + outer.height >= inner.y + inner.height
}
