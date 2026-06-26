use crate::ui::layouts::common::model_rc;

use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::super::super::template_tree_row_geometry::{tree_disclosure_rect, tree_icon_rect};
use super::super::push_tree_row_commands;
use super::support::{changed_pixel_count, pixel_at, tree_node};

#[test]
fn selected_tree_row_paints_muted_surface_left_indicator_icon_and_actions() {
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

    assert_eq!(pixel_at(&bytes, 280, 4, 19), PALETTE.accent);
    assert_eq!(pixel_at(&bytes, 280, 14, 19), PALETTE.surface_pressed);
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
    let bytes = paint_template_nodes_for_test(280, 48, model_rc(vec![node]));

    assert_eq!(
        pixel_at(
            &bytes,
            280,
            (icon.x + 4.0).round() as u32,
            (icon.y + 5.0).round() as u32
        ),
        PALETTE.text_disabled
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
