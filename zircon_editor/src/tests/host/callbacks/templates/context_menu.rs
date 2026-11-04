use super::super::support::*;
use crate::core::editor_event::MenuAction;
use crate::ui::binding::{AssetCommand, EditorUiBindingPayload};
use crate::ui::workbench::event::{dispatch_editor_host_binding, EditorHostEvent};

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
fn asset_delete_context_item_retains_uuid_with_open_locator_query() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    bridge
        .open_context_menu(&WorkbenchContextMenuRequestData {
            target_control_id: "AssetContent:browser".into(),
            target_value_text: "Runtime Material".into(),
            target_path: "workbench://asset/asset-runtime-material?open_locator=7265733a2f2f6d6174657269616c732f72756e74696d652e7a6d6174".into(),
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
fn asset_open_context_item_retains_the_generation_locator() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");
    bridge
        .open_context_menu(&WorkbenchContextMenuRequestData {
            target_control_id: "AssetContent:browser".into(),
            target_value_text: "Runtime Material".into(),
            target_path: "workbench://asset/asset-runtime-material?open_locator=7265733a2f2f6d6174657269616c732f72756e74696d652e7a6d6174".into(),
            menu_items: vec!["Open|action=menu.item.asset.open,icon=folder".into()],
            ..WorkbenchContextMenuRequestData::default()
        })
        .expect("asset context menu should open");

    let binding = bridge
        .context_menu_item_binding(WORKBENCH_CONTEXT_MENU_CONTROL_ID, "menu.item.asset.open")
        .expect("asset open row should retain a typed binding");

    assert_eq!(
        binding.payload(),
        &EditorUiBindingPayload::AssetCommand(AssetCommand::OpenAsset {
            asset_locator: "res://materials/runtime.zmat".to_string(),
        })
    );
}

#[test]
fn u09_native_asset_context_menu_retains_owner_pointer_and_target() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let root_node_id = bridge
        .surface()
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata.as_ref().is_some_and(|metadata| {
                metadata.control_id.as_deref() == Some("WorkbenchWindowRoot")
            })
        })
        .unwrap()
        .node_id;
    for (target_control, target_path) in [
        ("AssetContent:browser", "workbench://asset/test-asset"),
        ("AssetContent:activity", "workbench://asset/test-asset"),
        (
            "AssetFolder:browser",
            "workbench://asset-folder/7265733a2f2f",
        ),
        (
            "AssetFolder:activity",
            "workbench://asset-folder/7265733a2f2f",
        ),
    ] {
        assert!(bridge
            .open_context_menu(&WorkbenchContextMenuRequestData {
                target_control_id: target_control.into(),
                target_path: target_path.into(),
                popup_anchor_x: 128.0,
                popup_anchor_y: 256.0,
                menu_items: vec!["Rename|action=menu.item.asset.rename,icon=edit".into()],
                ..WorkbenchContextMenuRequestData::default()
            })
            .unwrap());
        assert_eq!(control_bool_attribute(&bridge, "popup_open"), Some(true));
        assert_eq!(
            control_string_attribute(&bridge, "context_target").as_deref(),
            Some(target_control)
        );
        assert_eq!(
            control_string_attribute(&bridge, "context_target_path").as_deref(),
            Some(target_path)
        );
        assert_eq!(
            control_string_attribute(&bridge, "popup_owner").as_deref(),
            Some("WorkbenchWindowRoot")
        );
        let popup = bridge.surface().input.popup_stack.last().unwrap();
        assert_eq!(popup.owner, Some(root_node_id));
        assert_eq!(popup.anchor, Some(UiPoint::new(128.0, 256.0)));
        assert_eq!(
            bridge
                .surface()
                .input
                .popup_anchor_points
                .get(&popup.popup_node.unwrap())
                .copied(),
            Some(UiPoint::new(128.0, 256.0))
        );
        assert!(
            bridge
                .host_projection()
                .node_by_control_id(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
                .unwrap()
                .popup_open
        );
    }
}

#[test]
fn u09_context_menu_refuses_hidden_in_surface_owner() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert!(!bridge
        .open_context_menu(&WorkbenchContextMenuRequestData {
            target_control_id: "WorkbenchScenePropsItem".into(),
            target_path: "workbench://scene/props".into(),
            popup_anchor_x: 128.0,
            popup_anchor_y: 256.0,
            menu_items: vec!["Rename|icon=edit".into()],
            ..WorkbenchContextMenuRequestData::default()
        })
        .unwrap());
    assert_eq!(control_bool_attribute(&bridge, "popup_open"), Some(false));
    assert_eq!(
        control_string_attribute(&bridge, "popup_owner").as_deref(),
        Some("WorkbenchScenePropsItem")
    );
    assert!(bridge.surface().input.popup_stack.is_empty());
    assert!(bridge.surface().input.popup_anchor_points.is_empty());
}

#[test]
fn u09_folder_context_items_retain_the_right_clicked_folder_uri() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    assert!(bridge
        .open_context_menu(&WorkbenchContextMenuRequestData {
            target_control_id: "AssetFolder:browser".into(),
            target_path: "workbench://asset-folder/7265733a2f2f6d6f64656c73".into(),
            menu_items: vec![
                "Open|action=menu.item.asset.folder.open,icon=folder".into(),
                "New Folder|action=menu.item.asset.folder.create,icon=folder-plus".into(),
                "Rename|action=menu.item.asset.folder.rename,icon=edit".into(),
                "Delete Empty Folder|action=menu.item.asset.folder.delete,danger,icon=trash".into(),
            ],
            ..WorkbenchContextMenuRequestData::default()
        })
        .unwrap());
    for (action, expected) in [
        (
            "menu.item.asset.folder.create",
            AssetCommand::CreateFolder {
                parent_folder_id: "res://models".to_owned(),
            },
        ),
        (
            "menu.item.asset.folder.rename",
            AssetCommand::RenameFolder {
                folder_id: "res://models".to_owned(),
            },
        ),
        (
            "menu.item.asset.folder.delete",
            AssetCommand::DeleteFolder {
                folder_id: "res://models".to_owned(),
            },
        ),
        (
            "menu.item.asset.folder.open",
            AssetCommand::SelectFolder {
                folder_id: "res://models".to_owned(),
            },
        ),
    ] {
        assert_eq!(
            bridge
                .context_menu_item_binding(WORKBENCH_CONTEXT_MENU_CONTROL_ID, action)
                .unwrap()
                .payload(),
            &EditorUiBindingPayload::AssetCommand(expected)
        );
    }
}

#[test]
fn u09_asset_context_rename_and_reimport_use_the_context_uuid() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let uuid = "00112233-4455-6677-8899-aabbccddeeff";
    assert!(bridge
        .open_context_menu(&WorkbenchContextMenuRequestData {
            target_control_id: "AssetContent:browser".into(),
            target_path: format!(
                "workbench://asset/{uuid}?open_locator=7265733a2f2f6d6f64656c732f736869702e676c62"
            )
            .into(),
            menu_items: vec![
                "Rename|action=menu.item.asset.rename,icon=edit".into(),
                "Reimport|action=menu.item.asset.reimport,icon=refresh-cw".into(),
                "New Folder|action=menu.item.asset.folder.create,icon=folder-plus".into(),
            ],
            ..WorkbenchContextMenuRequestData::default()
        })
        .unwrap());
    for (action, expected) in [
        (
            "menu.item.asset.rename",
            AssetCommand::RenameAsset {
                asset_uuid: uuid.to_owned(),
            },
        ),
        (
            "menu.item.asset.reimport",
            AssetCommand::ReimportAsset {
                asset_uuid: uuid.to_owned(),
            },
        ),
        (
            "menu.item.asset.folder.create",
            AssetCommand::CreateFolder {
                parent_folder_id: "res://models".to_owned(),
            },
        ),
    ] {
        assert_eq!(
            bridge
                .context_menu_item_binding(WORKBENCH_CONTEXT_MENU_CONTROL_ID, action)
                .unwrap()
                .payload(),
            &EditorUiBindingPayload::AssetCommand(expected)
        );
    }
}

#[test]
fn u09_folder_context_rejects_a_malformed_target_encoding() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    for target in [
        "workbench://asset-folder/not-hex",
        "workbench://asset-folder/",
        "workbench://control/selected",
    ] {
        assert!(bridge
            .open_context_menu(&WorkbenchContextMenuRequestData {
                target_control_id: "WorkbenchWindowRoot".into(),
                target_path: target.into(),
                menu_items: vec![
                    "Delete Empty Folder|action=menu.item.asset.folder.delete,danger,icon=trash"
                        .into(),
                ],
                ..WorkbenchContextMenuRequestData::default()
            })
            .unwrap());
        assert!(bridge
            .context_menu_item_binding(
                WORKBENCH_CONTEXT_MENU_CONTROL_ID,
                "menu.item.asset.folder.delete"
            )
            .is_none());
    }
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
            target_control_id: "WorkbenchMainBandSceneTreePanel".into(),
            target_action_id: "Hierarchy/ContextMenu".into(),
            target_dispatch_kind: "workbench".into(),
            target_role: "tree".into(),
            target_value_text: "Scene".into(),
            target_path: "workbench://control/scene-tree".into(),
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
        Some("WorkbenchMainBandSceneTreePanel")
    );
    assert_eq!(
        control_string_attribute(&bridge, "context_target_path").as_deref(),
        Some("workbench://control/scene-tree")
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
                    metadata.control_id.as_deref() == Some("WorkbenchMainBandSceneTreePanel")
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

    assert!(bridge
        .close_popup(WORKBENCH_CONTEXT_MENU_CONTROL_ID)
        .expect("context menu should close via shared popup cancel"));
    assert_eq!(
        control_string_attribute(&bridge, "visibility").as_deref(),
        Some("collapsed")
    );
    assert!(!control_bool_attribute(&bridge, "popup_open").unwrap_or(false));
    assert!(!bridge
        .surface()
        .input
        .popup_anchor_points
        .contains_key(&context_menu_node_id));
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
