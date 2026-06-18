use super::super::support::*;

#[test]
fn workbench_context_menu_open_state_populates_visible_overlay() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    assert!(bridge.has_control(WORKBENCH_CONTEXT_MENU_CONTROL_ID));
    assert_eq!(
        control_string_attribute(&bridge, "visibility").as_deref(),
        Some("collapsed")
    );

    let opened = bridge
        .open_context_menu(&WorkbenchContextMenuRequestData {
            target_control_id: "WorkbenchScenePropsItem".into(),
            target_action_id: "workbench.hierarchy.select_props".into(),
            target_dispatch_kind: "workbench".into(),
            target_role: "tree-row".into(),
            target_value_text: "Props".into(),
            target_path: "workbench://scene/props".into(),
            popup_anchor_x: 128.0,
            popup_anchor_y: 256.0,
            menu_items: vec![
                "Open|icon=folder".into(),
                "Rename|icon=edit".into(),
                "---".into(),
                "Delete|danger,icon=trash".into(),
            ],
        })
        .expect("context menu should open");

    assert!(opened);
    assert_eq!(
        control_string_attribute(&bridge, "visibility").as_deref(),
        Some("visible")
    );
    assert_eq!(control_bool_attribute(&bridge, "popup_open"), Some(true));
    assert_eq!(
        control_string_attribute(&bridge, "context_target").as_deref(),
        Some("WorkbenchScenePropsItem")
    );
    assert_eq!(
        control_string_attribute(&bridge, "context_target_path").as_deref(),
        Some("workbench://scene/props")
    );
    assert_eq!(
        control_float_attribute(&bridge, "popup_anchor_x"),
        Some(128.0)
    );
    assert_eq!(
        control_float_attribute(&bridge, "popup_anchor_y"),
        Some(256.0)
    );
    assert_eq!(
        control_string_list_attribute(&bridge, "menu_items"),
        vec![
            "Open|icon=folder",
            "Rename|icon=edit",
            "---",
            "Delete|danger,icon=trash"
        ]
    );

    let host_nodes = crate::ui::retained_host::to_host_contract_workbench_window_nodes(Some(
        bridge.host_projection(),
    ));
    let context_node = (0..host_nodes.row_count())
        .filter_map(|row| host_nodes.row_data(row))
        .find(|node| node.control_id.as_str() == WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        .expect("opened context menu should project to native host nodes");
    assert!(context_node.popup_open);
    assert_eq!(context_node.structured_menu_items.row_count(), 4);
    assert_eq!(
        context_node
            .structured_menu_items
            .row_data(1)
            .expect("rename row should project")
            .action_id
            .as_str(),
        "menu.item.rename"
    );

    assert!(bridge
        .close_popup(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        .expect("context menu should close via shared popup cancel"));
    assert_eq!(
        control_string_attribute(&bridge, "visibility").as_deref(),
        Some("collapsed")
    );
    assert!(!control_bool_attribute(&bridge, "popup_open").unwrap_or(false));
}

fn control_string_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Option<String> {
    control_attribute(bridge, property)
        .and_then(toml::Value::as_str)
        .map(str::to_string)
}

fn control_bool_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Option<bool> {
    control_attribute(bridge, property).and_then(toml::Value::as_bool)
}

fn control_float_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Option<f64> {
    control_attribute(bridge, property).and_then(|value| {
        value
            .as_float()
            .or_else(|| value.as_integer().map(|integer| integer as f64))
    })
}

fn control_string_list_attribute(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Vec<String> {
    control_attribute(bridge, property)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_string)
        .collect()
}

fn control_attribute<'a>(
    bridge: &'a BuiltinWorkbenchWindowTemplateSurfaceBridge,
    property: &str,
) -> Option<&'a toml::Value> {
    bridge.surface().tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .filter(|metadata| {
                metadata.control_id.as_deref() == Some(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
            })
            .and_then(|metadata| metadata.attributes.get(property))
    })
}
