use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneMenuItemData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::template_geometry::template_nodes_bounds;
use crate::ui::retained_host::host_contract::template_popup_layout::template_option_row_frame_within;

use super::support::{hit_test_workbench_window_template_node, model, option, workbench_node};

#[test]
fn u09_command_palette_hits_its_painted_search_and_candidate_frames() {
    use crate::ui::retained_host::host_contract::template_popup_layout::{
        command_palette_option_frame, command_palette_search_frame,
    };

    let node = u09_palette_node();
    let panel = u09_palette_frame(&node);
    let search = command_palette_search_frame(&panel);
    let row = command_palette_option_frame(&panel, 0);
    let presentation = u09_palette_presentation(node);
    let search_hit = hit_test_workbench_window_template_node(
        &presentation,
        search.x + search.width * 0.5,
        search.y + search.height * 0.5,
    )
    .expect("the painted search field must focus the live palette query");
    assert_eq!(search_hit.control_id.as_str(), "WorkbenchCommandPalette");
    assert_eq!(search_hit.dispatch_kind.as_str(), "surface");
    assert_eq!(
        search_hit.edit_action_id.as_str(),
        "CommandPalette/QueryChanged"
    );
    assert_eq!(
        search_hit.commit_action_id.as_str(),
        "CommandPalette/Commit"
    );
    assert_eq!(search_hit.value_text.as_str(), "关卡 Names");
    assert_eq!(search_hit.frame, search);

    let hit = hit_test_workbench_window_template_node(
        &presentation,
        row.x + row.width * 0.5,
        row.y + row.height * 0.5,
    )
    .expect("the candidate must be selectable inside its painted panel row");
    assert_eq!(hit.control_id.as_str(), "WorkbenchCommandPalette");
    assert_eq!(hit.dispatch_kind.as_str(), "workbench_option");
    assert_eq!(hit.action_id.as_str(), "CommandPalette/Commit");
    assert_eq!(hit.commit_action_id.as_str(), "CommandPalette/Commit");
    assert_eq!(hit.value_text.as_str(), "asset-name-prompt-confirm");
    assert_eq!(hit.frame, row);
    assert!(row.y >= panel.y && row.y + row.height <= panel.y + panel.height);
}

#[test]
fn u09_command_palette_refuses_disabled_empty_closed_and_outside_candidates() {
    use crate::ui::retained_host::host_contract::template_popup_layout::command_palette_option_frame;

    let original = u09_palette_node();
    let panel = u09_palette_frame(&original);
    let row = command_palette_option_frame(&panel, 0);
    let x = row.x + row.width * 0.5;
    let y = row.y + row.height * 0.5;
    for state in ["disabled", "empty", "closed", "missing-commit"] {
        let mut node = original.clone();
        match state {
            "disabled" => {
                node.structured_options = model(vec![option("asset-name-prompt-confirm", true)])
            }
            "empty" => node.structured_options = model(Vec::new()),
            "closed" => node.popup_open = false,
            "missing-commit" => node.commit_action_id = "".into(),
            _ => unreachable!(),
        }
        let hit = hit_test_workbench_window_template_node(&u09_palette_presentation(node), x, y);
        assert!(
            hit.is_none_or(|hit| hit.dispatch_kind.as_str() != "workbench_option"),
            "{state}"
        );
    }
    let presentation = u09_palette_presentation(original);
    assert!(
        hit_test_workbench_window_template_node(&presentation, x, panel.y + panel.height + 8.0,)
            .is_none(),
        "palette rows must never use a phantom dropdown below the panel"
    );
}

#[test]
fn u09_command_palette_hits_only_the_visible_part_of_a_clipped_candidate() {
    use crate::ui::retained_host::host_contract::template_popup_layout::command_palette_option_frame;

    let mut node = u09_palette_node();
    let panel = u09_palette_frame(&node);
    let row = command_palette_option_frame(&panel, 0);
    node.has_clip_frame = true;
    node.clip_frame = TemplateNodeFrameData {
        x: panel.x,
        y: panel.y,
        width: panel.width,
        height: row.y - panel.y + row.height * 0.5,
    };
    let presentation = u09_palette_presentation(node);
    let x = row.x + row.width * 0.5;
    let hit = hit_test_workbench_window_template_node(&presentation, x, row.y + row.height * 0.25)
        .expect("the painted part of a clipped candidate remains selectable");
    assert_eq!(hit.value_text.as_str(), "asset-name-prompt-confirm");
    assert_eq!(hit.frame.height, row.height * 0.5);
    assert!(
        hit_test_workbench_window_template_node(&presentation, x, row.y + row.height * 0.75)
            .is_none()
    );
}

fn u09_palette_node() -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: "palette".into(),
        control_id: "WorkbenchCommandPalette".into(),
        role: "CommandPalette".into(),
        component_role: "command-palette".into(),
        popup_open: true,
        edit_action_id: "CommandPalette/QueryChanged".into(),
        commit_action_id: "CommandPalette/Commit".into(),
        search_query: "关卡 Names".into(),
        structured_options: model(vec![option("asset-name-prompt-confirm", false)]),
        frame: TemplateNodeFrameData {
            x: 20.25,
            y: 80.5,
            width: 360.5,
            height: 240.0,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn u09_palette_frame(node: &TemplatePaneNodeData) -> FrameRect {
    FrameRect {
        x: node.frame.x,
        y: node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    }
}

fn u09_palette_presentation(node: TemplatePaneNodeData) -> HostWindowPresentationData {
    HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "root".into(),
                control_id: "WorkbenchWindowRoot".into(),
                role: "Panel".into(),
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 800.0,
                    height: 600.0,
                },
                ..TemplatePaneNodeData::default()
            },
            node,
        ]),
        ..HostWindowPresentationData::default()
    }
}

#[test]
fn workbench_hit_test_routes_open_dropdown_option_rows() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "root".into(),
                control_id: "WorkbenchRoot".into(),
                role: "Panel".into(),
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 240.0,
                    height: 240.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "dropdown".into(),
                control_id: "WorkbenchInputDropdown".into(),
                role: "Dropdown".into(),
                component_role: "dropdown".into(),
                edit_action_id: "component_lab.input_dropdown.select".into(),
                popup_open: true,
                frame: TemplateNodeFrameData {
                    x: 10.0,
                    y: 20.0,
                    width: 120.0,
                    height: 32.0,
                },
                structured_options: model(vec![
                    option("dropdown", false),
                    option("option_a", false),
                    option("option_b", true),
                ]),
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 96.0)
        .expect("open dropdown option row should be hit-tested");

    assert_eq!(hit.control_id.as_str(), "WorkbenchInputDropdown");
    assert_eq!(hit.dispatch_kind.as_str(), "workbench_option");
    assert_eq!(
        hit.action_id.as_str(),
        "component_lab.input_dropdown.select"
    );
    assert_eq!(hit.value_text.as_str(), "option_a");
    assert_eq!(
        hit.frame.y,
        expected_option_row_frame(&presentation, "WorkbenchInputDropdown", 1).y
    );
}

#[test]
fn workbench_hit_test_routes_dropdown_option_rows_above_control_when_bottom_clipped() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![
            TemplatePaneNodeData {
                node_id: "root".into(),
                control_id: "WorkbenchRoot".into(),
                role: "Panel".into(),
                frame: TemplateNodeFrameData {
                    x: 0.0,
                    y: 0.0,
                    width: 160.0,
                    height: 160.0,
                },
                ..TemplatePaneNodeData::default()
            },
            TemplatePaneNodeData {
                node_id: "dropdown".into(),
                control_id: "WorkbenchInputDropdown".into(),
                role: "Dropdown".into(),
                component_role: "dropdown".into(),
                edit_action_id: "component_lab.input_dropdown.select".into(),
                popup_open: true,
                frame: TemplateNodeFrameData {
                    x: 20.0,
                    y: 120.0,
                    width: 100.0,
                    height: 28.0,
                },
                structured_options: model(vec![
                    option("dropdown", false),
                    option("option_a", false),
                    option("option_b", false),
                ]),
                ..TemplatePaneNodeData::default()
            },
        ]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 28.0, 74.0)
        .expect("clipped dropdown option row should be hit-tested above the control");

    assert_eq!(hit.control_id.as_str(), "WorkbenchInputDropdown");
    assert_eq!(hit.dispatch_kind.as_str(), "workbench_option");
    assert_eq!(hit.value_text.as_str(), "option_a");
    assert_eq!(
        hit.frame.y,
        expected_option_row_frame(&presentation, "WorkbenchInputDropdown", 1).y
    );
}

#[test]
fn workbench_hit_test_routes_open_popup_menu_rows() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "popup".into(),
            control_id: "WorkbenchPopupMenu".into(),
            role: "Menu".into(),
            component_role: "menu".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 140.0,
                height: 120.0,
            },
            structured_menu_items: model(vec![
                menu_item("New", false, false),
                menu_item("Open", false, false),
                menu_item("Save", false, false),
                menu_item("", true, true),
                menu_item("Delete", false, false),
            ]),
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 128.0)
        .expect("open popup menu item row should be hit-tested");

    assert_eq!(hit.control_id.as_str(), "WorkbenchPopupMenu");
    assert_eq!(hit.dispatch_kind.as_str(), "workbench_menu_item");
    assert_eq!(hit.action_id.as_str(), "menu.item.delete");
    assert_eq!(hit.value_text.as_str(), "Delete");
    assert_eq!(hit.frame.y, 116.0);
}

#[test]
fn native_popup_hit_preserves_registered_editor_command_identity() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "add-popup".into(),
            control_id: "UnrealAddMenuPopup".into(),
            role: "Menu".into(),
            component_role: "menu".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 72.0,
                y: 92.0,
                width: 200.0,
                height: 32.0,
            },
            structured_menu_items: model(vec![TemplatePaneMenuItemData {
                action_id: "scene.node.create_cube".into(),
                label: "Create Cube".into(),
                ..TemplatePaneMenuItemData::default()
            }]),
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };
    let hit = hit_test_workbench_window_template_node(&presentation, 100.0, 108.0).unwrap();
    assert_eq!(hit.action_id.as_str(), "scene.node.create_cube");
    assert_eq!(hit.dispatch_kind.as_str(), "workbench_menu_item");
}

#[test]
fn workbench_hit_test_blocks_popup_menu_separator_row() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "popup".into(),
            control_id: "WorkbenchPopupMenu".into(),
            role: "Menu".into(),
            component_role: "menu".into(),
            action_id: "workbench.component.menu.open".into(),
            popup_open: true,
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 140.0,
                height: 120.0,
            },
            structured_menu_items: model(vec![
                menu_item("New", false, false),
                menu_item("Open", false, false),
                menu_item("Save", false, false),
                menu_item("", true, true),
                menu_item("Delete", false, false),
            ]),
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };

    assert!(
        hit_test_workbench_window_template_node(&presentation, 24.0, 104.0).is_none(),
        "separator rows should block parent/underlay hit fallback while staying inside the popup"
    );
}

fn menu_item(action_id: &str, disabled: bool, separator: bool) -> TemplatePaneMenuItemData {
    TemplatePaneMenuItemData {
        action_id: action_id.into(),
        label: action_id.into(),
        disabled,
        separator,
        ..TemplatePaneMenuItemData::default()
    }
}

fn expected_option_row_frame(
    presentation: &HostWindowPresentationData,
    control_id: &str,
    row: usize,
) -> FrameRect {
    let node = workbench_node(presentation, control_id);
    let origin = workbench_template_origin(presentation);
    let control_frame = FrameRect {
        x: origin.x + node.frame.x,
        y: origin.y + node.frame.y,
        width: node.frame.width,
        height: node.frame.height,
    };
    template_option_row_frame_within(
        &node,
        &control_frame,
        node.structured_options.row_count(),
        row,
        &origin,
    )
    .unwrap_or_else(|| panic!("{control_id} option row {row} should project to a popup frame"))
}

fn workbench_template_origin(presentation: &HostWindowPresentationData) -> FrameRect {
    let bounds = template_nodes_bounds(&presentation.workbench_window_nodes)
        .expect("workbench template should expose non-empty bounds");
    FrameRect {
        x: 0.0,
        y: 0.0,
        width: bounds.width.max(bounds.x + bounds.width).max(1.0),
        height: bounds.height.max(bounds.y + bounds.height).max(1.0),
    }
}
