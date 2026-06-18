use super::super::super::data::TemplateNodeFrameData;
use super::super::super::paint_theme::PALETTE;
use super::super::style_selector::WORKBENCH_TREE_ROW_TEXT_SELECTED as TREE_TEXT_SELECTED;
use super::super::template_nodes::paint_template_nodes_for_test;
use super::*;
use crate::ui::layouts::common::model_rc;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[test]
fn tree_row_kind_matches_roles_and_scene_ids() {
    assert!(is_workbench_tree_row(&tree_node(
        "Custom", "TreeRow", "", "Root", 0, false
    )));
    assert!(is_workbench_tree_row(&tree_node(
        "WorkbenchScenePropsItem",
        "",
        "",
        "Props",
        2,
        true
    )));
    assert!(is_workbench_tree_row(&tree_node(
        "Custom", "", "tree-row", "Node", 0, false
    )));
    assert!(!is_workbench_tree_row(&TemplatePaneNodeData {
        control_id: "WorkbenchListSelected".into(),
        role: "ListRow".into(),
        component_role: "list-row".into(),
        ..TemplatePaneNodeData::default()
    }));
}

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
fn tree_row_style_uses_shared_state_priority() {
    let mut node = tree_node(
        "WorkbenchScenePropsItem",
        "TreeRow",
        "tree-row",
        "Props",
        2,
        true,
    );
    node.hovered = true;
    node.focused = true;
    node.pressed = true;

    let pressed = tree_row_style(&node);
    assert_eq!(pressed.state, UiPainterResolvedState::Pressed);
    assert_eq!(pressed.background, Some(PALETTE.surface_selected));
    assert_eq!(pressed.border, Some(PALETTE.focus_ring));
    assert_eq!(pressed.text, TREE_TEXT_SELECTED);

    node.pressed = false;
    node.selected = false;
    node.checked = false;
    let focused = tree_row_style(&node);
    assert_eq!(focused.state, UiPainterResolvedState::Focused);
    assert_eq!(focused.background, Some(PALETTE.surface_hover));
    assert_eq!(focused.border, Some(PALETTE.focus_ring));

    node.disabled = true;
    let disabled = tree_row_style(&node);
    assert_eq!(disabled.state, UiPainterResolvedState::Disabled);
    assert_eq!(disabled.background, None);
    assert_eq!(disabled.border, None);
    assert_eq!(disabled.text, PALETTE.text_disabled);
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

fn tree_node(
    control_id: &str,
    role: &str,
    component_role: &str,
    text: &str,
    depth: i32,
    selected: bool,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        control_id: control_id.into(),
        role: role.into(),
        component_role: component_role.into(),
        text: text.into(),
        tree_depth: depth,
        tree_indent_px: if selected { 40.0 } else { 0.0 },
        selected,
        checked: selected,
        expanded: !text.contains("Player"),
        frame: TemplateNodeFrameData {
            x: 4.0,
            y: 6.0,
            width: 268.0,
            height: 30.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn changed_pixel_count(
    bytes: &[u8],
    frame_width: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> usize {
    let mut changed = 0;
    for py in y..(y + height) {
        for px in x..(x + width) {
            let index = ((py as usize * frame_width as usize) + px as usize) * 4;
            if bytes[index..index + 4] != [0, 0, 0, 255] {
                changed += 1;
            }
        }
    }
    changed
}

fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
    let index = ((y as usize * frame_width as usize) + x as usize) * 4;
    [
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ]
}
