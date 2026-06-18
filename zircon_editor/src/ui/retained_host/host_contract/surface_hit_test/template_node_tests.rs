use std::rc::Rc;

use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::retained_host::host_contract::data::{
    HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneMenuItemData,
    TemplatePaneNodeData, TemplatePaneOptionData,
};
use crate::ui::retained_host::host_contract::template_component_family::TemplateComponentFamily;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use crate::ui::retained_host::to_host_contract_workbench_window_nodes;
use zircon_runtime_interface::ui::binding::UiEventKind;
use zircon_runtime_interface::ui::layout::UiSize;

use super::hit_test_workbench_window_template_node;

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
    assert_eq!(hit.frame.y, 88.0);
}

#[test]
fn workbench_hit_test_routes_componentized_text_input_center() {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: to_host_contract_workbench_window_nodes(Some(
            bridge.host_projection(),
        )),
        ..HostWindowPresentationData::default()
    };
    let input = workbench_node(&presentation, "WorkbenchInputText");
    let hit = hit_test_workbench_window_template_node(
        &presentation,
        input.frame.x + input.frame.width * 0.5,
        input.frame.y + input.frame.height * 0.5,
    )
    .expect("input center should hit a componentized workbench node");

    assert_eq!(
        hit.control_id.as_str(),
        "WorkbenchInputText",
        "input center routed to {} with kind {} and role {}",
        hit.control_id,
        hit.dispatch_kind,
        hit.component_role
    );
    assert_eq!(hit.edit_action_id.as_str(), "component_lab.input_text.edit");
}

#[test]
fn text_field_family_without_legacy_input_role_is_hit_tested() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "text".into(),
            control_id: "GenericTextField".into(),
            role: "TextField".into(),
            component_category: "input".into(),
            component_role: "text-field".into(),
            component_layout_role: "leaf".into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 12.0,
                width: 120.0,
                height: 28.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };

    let hit = hit_test_workbench_window_template_node(&presentation, 24.0, 20.0)
        .expect("TextInput component family should enter the template hit surface");

    assert_eq!(hit.control_id.as_str(), "GenericTextField");
    assert_eq!(
        hit.component_family,
        Some(TemplateComponentFamily::TextInput)
    );
}

#[test]
fn workbench_hit_test_ignores_decorative_viewport_scene_layers() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchModuleScene", UiEventKind::Click)
        .expect("scene module state dispatch should succeed")
        .expect("scene module should expose a preview binding");
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: to_host_contract_workbench_window_nodes(Some(
            bridge.host_projection(),
        )),
        ..HostWindowPresentationData::default()
    };
    let scene_layer = workbench_node(&presentation, "WorkbenchViewportFloorGrateRight");
    let x = scene_layer.frame.x + scene_layer.frame.width * 0.5;
    let y = scene_layer.frame.y + scene_layer.frame.height * 0.5;

    let hit = hit_test_workbench_window_template_node(&presentation, x, y);

    assert!(
        hit.is_none(),
        "decorative viewport scene layer should not capture pointer hit, routed to {:?}",
        hit.as_ref().map(|hit| hit.control_id.to_string())
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
    assert_eq!(hit.frame.y, 60.0);
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

fn option(id: &str, disabled: bool) -> TemplatePaneOptionData {
    TemplatePaneOptionData {
        id: id.into(),
        label: id.into(),
        disabled,
        ..TemplatePaneOptionData::default()
    }
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

fn workbench_node(
    presentation: &HostWindowPresentationData,
    control_id: &str,
) -> TemplatePaneNodeData {
    (0..presentation.workbench_window_nodes.row_count())
        .filter_map(|row| presentation.workbench_window_nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("{control_id} should project to native host nodes"))
}

fn model<T: Clone>(values: Vec<T>) -> ModelRc<T> {
    Rc::new(VecModel::from(values)).into()
}
