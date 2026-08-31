use super::super::support::*;
use crate::core::editor_event::MenuAction;
use crate::ui::binding::{AssetCommand, EditorUiBindingPayload};
use crate::ui::workbench::event::{EditorHostEvent, dispatch_editor_host_binding};

#[test]
fn keep_play_changes_context_item_routes_to_the_typed_menu_action() {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    let binding = bridge
        .context_menu_item_binding(
            WORKBENCH_CONTEXT_MENU_CONTROL_ID,
            "menu.item.keep_play_changes",
        )
        .expect("the play context action should have a binding");

    assert_eq!(
        dispatch_editor_host_binding(&binding).unwrap(),
        EditorHostEvent::Menu(MenuAction::KeepPlayChanges)
    );
}

#[test]
fn asset_delete_context_item_retains_the_generation_target_uuid() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    bridge
        .open_context_menu(&WorkbenchContextMenuRequestData {
            target_control_id: "AssetContent:browser".into(),
            target_value_text: "Runtime Material".into(),
            target_path: "workbench://asset/asset-runtime-material".into(),
            menu_items: vec!["Delete|action=menu.item.asset.delete,danger,icon=trash".into()],
            ..WorkbenchContextMenuRequestData::default()
        })
        .expect("asset context menu should open");

    let binding = bridge
        .context_menu_item_binding(WORKBENCH_CONTEXT_MENU_CONTROL_ID, "menu.item.asset.delete")
        .expect("asset delete row should retain a typed binding");

    assert_eq!(
        binding.payload(),
        &EditorUiBindingPayload::AssetCommand(AssetCommand::DeleteAsset {
            asset_uuid: "asset-runtime-material".to_string(),
        })
    );
}

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
                "Open Referenced Asset in External Editor|icon=folder|Ctrl+Shift+O".into(),
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
    let context_menu_node_id = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| {
                    metadata.control_id.as_deref() == Some(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
                })
                .map(|_| node.node_id)
        })
        .expect("context menu should retain its node identity");
    assert_eq!(
        bridge
            .surface()
            .input
            .popup_anchor_points
            .get(&context_menu_node_id)
            .copied(),
        Some(UiPoint::new(128.0, 256.0))
    );
    let target_node_id = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| {
                    metadata.control_id.as_deref() == Some("WorkbenchScenePropsItem")
                })
                .map(|_| node.node_id)
        })
        .expect("context target should retain its node identity");
    assert_eq!(
        bridge
            .surface()
            .input
            .popup_stack
            .iter()
            .find(|popup| popup.popup_node == Some(context_menu_node_id))
            .and_then(|popup| popup.owner),
        Some(target_node_id)
    );
    assert_eq!(
        control_string_list_attribute(&bridge, "menu_items"),
        vec![
            "Open Referenced Asset in External Editor|icon=folder|Ctrl+Shift+O",
            "Rename|icon=edit",
            "---",
            "Delete|danger,icon=trash"
        ]
    );
    let context_menu_frame = bridge
        .control_frame(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        .expect("opened context menu should expose its measured frame");
    assert!(
        context_menu_frame.width > 220.0,
        "long runtime rows should grow beyond the authored fallback width"
    );
    assert!(
        (context_menu_frame.height
            - crate::ui::retained_host::menu_popup_contract::menu_popup_content_height(4))
        .abs()
            <= f32::EPSILON,
        "runtime row count should determine context-menu height"
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

    assert!(
        bridge
            .close_popup(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
            .expect("context menu should close via shared popup cancel")
    );
    assert_eq!(
        control_string_attribute(&bridge, "visibility").as_deref(),
        Some("collapsed")
    );
    assert!(!control_bool_attribute(&bridge, "popup_open").unwrap_or(false));
    assert!(
        !bridge
            .surface()
            .input
            .popup_anchor_points
            .contains_key(&context_menu_node_id)
    );
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
