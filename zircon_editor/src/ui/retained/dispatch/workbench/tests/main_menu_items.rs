use super::super::super::popup_primitives::template_popup_menu_item_state;
use super::*;
use crate::ui::template_runtime::RetainedUiHostValue;

const FILE_MENU_CONTROL_ID: &str = "UnrealFileMenuPopup";
use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue, layout::UiSize};

#[test]
fn file_menu_click_opens_measured_popup_and_background_has_no_action() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1200.0, 800.0))
        .expect("workbench bridge should build");
    assert!(!bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    assert!(bridge
        .dispatch_control_state(FILE_MENU_CONTROL_ID, UiEventKind::Click)
        .expect("background dispatch should be harmless")
        .is_none());
    assert!(!bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    bridge.mutate_control_property(FILE_MENU_CONTROL_ID, "menu_items",
        UiValue::Array(vec![UiValue::String(
            "Open a scene with a deliberately long project owned display label|action=menu.item.open_scene,icon=folder|Ctrl+O".to_string()
        )])).unwrap();
    assert!(bridge
        .dispatch_control_state("MenuSlot0", UiEventKind::Click)
        .expect("actual toolbar click should open its menu")
        .is_some());
    assert!(bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    assert!(bridge.control_frame(FILE_MENU_CONTROL_ID).unwrap().width > 240.0);
    assert!(bridge
        .main_menu_item_binding(FILE_MENU_CONTROL_ID, "")
        .is_none());
    assert!(bridge
        .main_menu_item_binding(FILE_MENU_CONTROL_ID, "unknown")
        .is_none());
}

#[test]
fn window_menus_switch_exclusively_and_cancel_clears_the_trigger() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored editor should project");
    bridge
        .dispatch_control_state("MenuSlot0", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    bridge
        .dispatch_control_state("MenuSlot1", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(!bridge.control_bool(FILE_MENU_CONTROL_ID, "popup_open"));
    assert!(bridge.control_bool("UnrealEditMenuPopup", "popup_open"));
    assert!(bridge.control_bool("MenuSlot1", "selected"));
    assert!(bridge.close_popup("UnrealEditMenuPopup").unwrap());
    assert!(!bridge.control_bool("UnrealEditMenuPopup", "popup_open"));
    assert!(!bridge.control_bool("MenuSlot1", "selected"));
    bridge
        .dispatch_control_state("MenuSlot1", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(bridge.control_bool("UnrealEditMenuPopup", "popup_open"));
    bridge
        .dispatch_control_state("MenuSlot1", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(!bridge.control_bool("UnrealEditMenuPopup", "popup_open"));
}

#[test]
fn projected_popup_items_use_registered_commands_and_current_enablement() {
    use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};
    use crate::core::i18n::EditorI18nService;
    let registry = EditorCommandRegistry::default_workbench();
    let i18n = EditorI18nService::default();
    for project_open in [false, true] {
        let context = CommandEvalCtx::interactive().with_project_open(project_open);
        let menus = registry.menu_bar_model(&i18n, &i18n.active_locale(), &context);
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0)).unwrap();
        bridge.apply_command_menu_bar(&menus).unwrap();
        for (menu, commands) in &command_menu_contract().menus {
            for id in commands {
                assert!(
                    registry.command(id.as_str()).is_some(),
                    "unknown command {id}"
                );
            }
            for raw in bridge.control_string_array(menu, "menu_items") {
                let item = template_popup_menu_item_state(&raw).unwrap();
                let command = registry.command(item.action_id.as_str()).unwrap();
                assert_eq!(!item.disabled, command.is_enabled(&context));
                assert_eq!(
                    bridge
                        .main_menu_item_binding(menu, &item.action_id)
                        .is_some(),
                    !item.disabled
                );
            }
        }
        let add = bridge.control_string_array("UnrealAddMenuPopup", "menu_items");
        assert_eq!(add.len(), 7);
        assert!(add
            .iter()
            .any(|item| item.contains("scene.node.create_point_light")));
    }
}

#[test]
fn content_drawer_toggle_and_close_keep_the_trigger_state_in_sync() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored editor should project");
    assert!(!bridge.control_bool("UnrealContentDrawerToggle", "selected"));
    bridge
        .dispatch_control_state("UnrealContentDrawerToggle", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(bridge.control_bool("UnrealContentDrawerToggle", "selected"));
    assert!(bridge.control_frame("ContentDrawerClose").unwrap().height > 0.0);
    bridge
        .dispatch_control_state("ContentDrawerClose", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(!bridge.control_bool("UnrealContentDrawerToggle", "selected"));
    bridge
        .dispatch_control_state("UnrealContentDrawerToggle", UiEventKind::Click)
        .unwrap()
        .unwrap();
    assert!(bridge.control_bool("UnrealContentDrawerToggle", "selected"));
}

#[test]
fn closed_authored_menus_are_not_visible_or_pointer_targets_at_startup() {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored editor should project");
    for menu in [
        "UnrealFileMenuPopup",
        "UnrealEditMenuPopup",
        "UnrealWindowMenuPopup",
        "UnrealToolsMenuPopup",
        "UnrealBuildMenuPopup",
        "UnrealHelpMenuPopup",
        "UnrealModesMenuPopup",
        "UnrealAddMenuPopup",
        "UnrealBlueprintsMenuPopup",
        "UnrealCinematicsMenuPopup",
        "UnrealSettingsMenuPopup",
        "WorkbenchRunModeMenu",
    ] {
        assert!(
            !bridge.control_bool(menu, "popup_open"),
            "menu {menu} starts open"
        );
        let id = bridge.control_node_id(menu).expect("authored menu node");
        let node = bridge.surface().tree.node(id).expect("menu tree node");
        assert!(
            !node.is_render_visible(),
            "closed menu {menu} participates in drawing"
        );
        assert!(
            !node.is_self_hit_test_visible(),
            "closed menu {menu} intercepts pointer input"
        );
    }
}

#[test]
fn native_add_popup_projects_the_current_arranged_trigger_anchor() {
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .mutate_control_property(
            "UnrealAddMenuPopup",
            "menu_items",
            UiValue::Array(vec![UiValue::String(
                "Create Cube|action=scene.node.create_cube|".into(),
            )]),
        )
        .unwrap();
    bridge
        .dispatch_control_state("WorkbenchToolbarAddActor", UiEventKind::Click)
        .unwrap()
        .unwrap();
    let trigger = bridge
        .host_projection()
        .node_by_control_id("WorkbenchToolbarAddActor")
        .unwrap();
    let popup = bridge
        .host_projection()
        .node_by_control_id("UnrealAddMenuPopup")
        .unwrap();
    assert!(popup.has_popup_anchor);
    assert_eq!(popup.popup_anchor_x, trigger.frame.x as f64);
    assert_eq!(popup.popup_anchor_y, trigger.frame.y as f64);
    assert!(popup.menu_items[0].contains("action=scene.node.create_cube"));
    let mut projection = bridge.host_projection().clone();
    // The runtime's first-row text summary must not become a second native label.
    let popup = projection
        .nodes
        .iter_mut()
        .find(|node| node.control_id.as_deref() == Some("UnrealAddMenuPopup"))
        .unwrap();
    popup.text = Some("Create Cube".into());
    popup.properties.insert(
        "text".into(),
        RetainedUiHostValue::String("Create Cube".into()),
    );
    let nodes =
        crate::ui::retained_host::ui::to_host_contract_workbench_window_nodes(Some(&projection));
    let native_popup = nodes
        .iter()
        .find(|node| node.control_id.as_str() == "UnrealAddMenuPopup")
        .unwrap();
    assert_eq!(native_popup.component_role.as_str(), "menu");
    assert_eq!(native_popup.frame.x, trigger.frame.x);
    assert_eq!(native_popup.frame.y, trigger.frame.y + trigger.frame.height);
    eprintln!("native_add_popup role={} open={} frame=({}, {}, {}, {}) clip=({}, {}, {}, {}) surface={} rows={}",
        native_popup.component_role, native_popup.popup_open,
        native_popup.frame.x, native_popup.frame.y, native_popup.frame.width, native_popup.frame.height,
        native_popup.clip_frame.x, native_popup.clip_frame.y, native_popup.clip_frame.width, native_popup.clip_frame.height,
        native_popup.surface_variant, native_popup.structured_menu_items.row_count());
    let mut popup_with_summary = native_popup.clone();
    popup_with_summary.options_text = std::rc::Rc::new("Create Cube".to_owned());
    let summary =
        crate::ui::retained_host::template_node_command_summary_for_test(&popup_with_summary);
    assert_eq!(
        summary.text_count, 1,
        "the native popup must paint its real command row"
    );
}
