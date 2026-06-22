use crate::ui::layouts::common::model_rc;

use super::super::super::super::paint_theme::PALETTE;
use super::super::super::template_nodes::paint_template_nodes_for_test;
use super::support::{changed_pixel_count, pixel_at, tree_node};

#[test]
fn selected_tree_row_paints_surface_indent_icon_and_actions() {
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

    assert_ne!(pixel_at(&bytes, 280, 14, 19), [0, 0, 0, 255]);
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
    let bytes = paint_template_nodes_for_test(280, 48, model_rc(vec![node]));

    assert_eq!(pixel_at(&bytes, 280, 38, 16), PALETTE.text_disabled);
}
