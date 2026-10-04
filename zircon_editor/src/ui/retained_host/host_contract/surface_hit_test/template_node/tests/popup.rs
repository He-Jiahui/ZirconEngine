use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneMenuItemData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::template_geometry::template_nodes_bounds;
use crate::ui::retained_host::host_contract::template_popup_layout::template_option_row_frame_within;

use super::support::{hit_test_workbench_window_template_node, model, option, workbench_node};

#[test]
fn workbench_hit_test_routes_open_dropdown_option_rows() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
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
        }]),
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
