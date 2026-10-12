use crate::ui::retained_host::host_contract::data::{
    HostWindowPresentationData, TemplateNodeFrameData, TemplatePaneMenuItemData,
    TemplatePaneNodeData,
};

use super::super::{
    hit_test_workbench_window_template_node_for_pointer_move_with_index,
    hit_test_workbench_window_template_node_with_index, HostWorkbenchHitIndex,
    TemplateNodePointerMoveKind,
};
use super::support::{model, option};

#[test]
fn workbench_pointer_move_hit_borrows_generation_owned_node_and_popup_strings() {
    let node_presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "button".into(),
            control_id: "WorkbenchButton".into(),
            role: "Button".into(),
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 20.0,
                width: 120.0,
                height: 32.0,
            },
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };
    let node_index = HostWorkbenchHitIndex::from_presentation(&node_presentation);
    let node = node_presentation.workbench_window_nodes.get(0).unwrap();
    let node_hit = hit_test_workbench_window_template_node_for_pointer_move_with_index(
        &node_presentation,
        &node_index,
        24.0,
        30.0,
    )
    .expect("button should produce a pointer-move hit");
    assert_eq!(node_hit.kind, TemplateNodePointerMoveKind::Node);
    assert!(std::ptr::eq(node_hit.control_id, node.control_id.as_str()));
    let node_owned = hit_test_workbench_window_template_node_with_index(
        &node_presentation,
        &node_index,
        24.0,
        30.0,
    )
    .expect("button should retain the owned press hit");
    assert_eq!(node_hit.control_id, node_owned.control_id.as_str());
    assert_eq!(node_hit.frame, node_owned.frame);

    let menu_presentation = HostWindowPresentationData {
        workbench_window_nodes: model(vec![TemplatePaneNodeData {
            node_id: "menu".into(),
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
            structured_menu_items: model(vec![TemplatePaneMenuItemData {
                action_id: "menu.item.delete".into(),
                label: "Delete".into(),
                ..TemplatePaneMenuItemData::default()
            }]),
            ..TemplatePaneNodeData::default()
        }]),
        ..HostWindowPresentationData::default()
    };
    let menu_index = HostWorkbenchHitIndex::from_presentation(&menu_presentation);
    let menu_node = menu_presentation.workbench_window_nodes.get(0).unwrap();
    let menu_item = menu_node.structured_menu_items.get(0).unwrap();
    let menu_hit = hit_test_workbench_window_template_node_for_pointer_move_with_index(
        &menu_presentation,
        &menu_index,
        24.0,
        30.0,
    )
    .expect("menu row should produce a pointer-move hit");
    assert_eq!(menu_hit.kind, TemplateNodePointerMoveKind::MenuItem);
    assert!(std::ptr::eq(
        menu_hit.control_id,
        menu_node.control_id.as_str()
    ));
    assert!(std::ptr::eq(
        menu_hit.action_id,
        menu_item.action_id.as_str()
    ));
    assert!(std::ptr::eq(menu_hit.value_text, menu_item.label.as_str()));
    let menu_owned = hit_test_workbench_window_template_node_with_index(
        &menu_presentation,
        &menu_index,
        24.0,
        30.0,
    )
    .expect("menu row should retain the owned press hit");
    assert_eq!(menu_hit.control_id, menu_owned.control_id.as_str());
    assert_eq!(menu_hit.action_id, menu_owned.action_id.as_str());
    assert_eq!(menu_hit.value_text, menu_owned.value_text.as_str());
    assert_eq!(menu_hit.frame, menu_owned.frame);

    let option_presentation = HostWindowPresentationData {
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
    let option_index = HostWorkbenchHitIndex::from_presentation(&option_presentation);
    let option_node = option_presentation.workbench_window_nodes.get(0).unwrap();
    let option = option_node.structured_options.get(1).unwrap();
    let option_hit = hit_test_workbench_window_template_node_for_pointer_move_with_index(
        &option_presentation,
        &option_index,
        24.0,
        96.0,
    )
    .expect("option row should produce a pointer-move hit");
    assert_eq!(option_hit.kind, TemplateNodePointerMoveKind::Option);
    assert!(std::ptr::eq(
        option_hit.action_id,
        option_node.edit_action_id.as_str()
    ));
    assert!(std::ptr::eq(option_hit.value_text, option.id.as_str()));
    let option_owned = hit_test_workbench_window_template_node_with_index(
        &option_presentation,
        &option_index,
        24.0,
        96.0,
    )
    .expect("option row should retain the owned press hit");
    assert_eq!(option_hit.control_id, option_owned.control_id.as_str());
    assert_eq!(option_hit.action_id, option_owned.action_id.as_str());
    assert_eq!(option_hit.value_text, option_owned.value_text.as_str());
    assert_eq!(option_hit.frame, option_owned.frame);
}
