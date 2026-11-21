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
        let modes = bridge.control_string_array("UnrealModesMenuPopup", "menu_items");
        assert_eq!(modes.len(), 1);
        assert!(modes[0].contains("viewport.tool.select"));
    }
}

#[test]
fn explicitly_retained_empty_menu_triggers_are_visible_disabled_and_inert() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 800.0))
        .expect("the authored editor should project");
    bridge
        .apply_command_menu_bar(&crate::core::commands::MenuBarModel { menus: Vec::new() })
        .unwrap();
    for (menu, trigger) in [
        ("UnrealHelpMenuPopup", "MenuSlot5"),
        ("UnrealBlueprintsMenuPopup", "UnrealToolbarBlueprints"),
        ("UnrealCinematicsMenuPopup", "UnrealToolbarCinematics"),
    ] {
        let id = bridge
            .control_node_id(trigger)
            .expect("authored trigger node");
        assert!(bridge.surface().tree.node(id).unwrap().is_render_visible());
        assert!(bridge.control_bool(trigger, "disabled"));
        assert!(bridge.control_string_array(menu, "menu_items").is_empty());
        assert!(!bridge.control_bool(menu, "popup_open"));
        assert!(bridge
            .main_menu_item_binding(menu, "viewport.tool.select")
            .is_none());
    }
    for trigger in ["MenuSlot0", "WorkbenchToolSelect"] {
        let id = bridge
            .control_node_id(trigger)
            .expect("authored trigger node");
        assert!(!bridge.surface().tree.node(id).unwrap().is_render_visible());
    }
}

#[test]
fn menu_contract_preserves_legacy_empty_policy_and_rejects_unknown_policy() {
    let legacy: CommandMenuContract =
        toml::from_str("[menus]\nPopup = []\n[triggers]\nPopup = 'Trigger'\n").unwrap();
    assert!(legacy.empty_trigger_policy.is_empty());
    assert!(toml::from_str::<CommandMenuContract>(
        "[menus]\nPopup = []\n[triggers]\nPopup = 'Trigger'\n[empty_trigger_policy]\nPopup = 'ForceEnabled'\n",
    ).is_err());
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

#[test]
fn file_menu_locale_publishes_the_same_clickable_trigger_and_command() {
    use std::sync::Arc;

    use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};
    use crate::core::i18n::EditorI18nService;
    use crate::core::settings::{
        SettingValue, SettingsAuthority, SettingsKey, SettingsScope, EDITOR_LOCALE_KEY,
    };
    use crate::ui::binding::EditorUiBindingPayload;
    use crate::ui::retained_host::callback_dispatch::load_startup_builtin_template_runtime;
    use crate::ui::retained_host::host_contract::{
        globals::HostContractState, hit_test_workbench_window_template_node_with_index,
    };
    use crate::ui::retained_host::primitives::PhysicalSize;
    use zircon_runtime_interface::ui::layout::UiFrame;

    const TRIGGER: &str = "MenuSlot0";
    const TRIGGER_ROUTE: &str = "workbench.unreal.menu.file.open";
    const OPEN_SCENE: &str = "file.scene.open";
    let settings = SettingsAuthority::with_defaults();
    let key = SettingsKey::parse(EDITOR_LOCALE_KEY).unwrap();
    let i18n = Arc::new(EditorI18nService::default());
    let runtime = Arc::new(load_startup_builtin_template_runtime().unwrap());
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new_mounted_with_runtime_and_i18n(
            runtime,
            UiFrame::new(0.0, 0.0, 1280.0, 800.0),
            Arc::clone(&i18n),
        )
        .unwrap();
    let trigger_id = bridge.control_node_id(TRIGGER).unwrap();
    let popup_id = bridge.control_node_id(FILE_MENU_CONTROL_ID).unwrap();
    let registry = EditorCommandRegistry::default_workbench();
    let context = CommandEvalCtx::interactive().with_project_open(true);
    let command_binding =
        editor_operation_binding(&EditorOperationPath::parse(OPEN_SCENE).unwrap());
    let mut host = HostContractState::new(PhysicalSize::new(1280, 800));
    let mut labels = Vec::new();

    for (step, (tag, root_label, leaf_label)) in [
        ("en", "File", "Open Scene..."),
        ("zh-CN", "文件", "打开场景..."),
        ("en", "File", "Open Scene..."),
    ]
    .into_iter()
    .enumerate()
    {
        if tag == "zh-CN" {
            let change = settings
                .set(
                    SettingsScope::User,
                    &key,
                    SettingValue::Enum(tag.to_owned()),
                )
                .unwrap()
                .expect("selecting Chinese must change the user setting");
            assert!(!change.requires_restart);
        } else if step != 0 {
            assert!(settings.clear(SettingsScope::User, &key).unwrap().is_some());
        }
        assert_eq!(settings.snapshot().locale(), tag);
        assert_eq!(i18n.synchronize_user_locale(&settings).unwrap(), step != 0);
        let locale = i18n.active_locale();
        assert_eq!(locale.as_str(), tag);

        let menus = registry.menu_bar_model(&i18n, &locale, &context);
        let (file, command) = menus
            .menus
            .iter()
            .find_map(|menu| find_menu_command(&menu.items, OPEN_SCENE).map(|item| (menu, item)))
            .expect("the actual File registry projection contains Open Scene");
        assert_eq!(file.label, root_label);
        assert_eq!(command.label, leaf_label);
        assert!(command.enabled);
        assert_eq!(command.shortcut.as_deref(), Some("Ctrl+O"));
        bridge.apply_command_menu_bar(&menus).unwrap();
        bridge.refresh_prepared_state_change().unwrap();

        let closed = file_menu_locale_commit(&mut bridge, &mut host);
        let trigger = file_menu_locale_node(&closed, TRIGGER);
        assert_eq!(trigger.surface_node_id, Some(trigger_id));
        assert_eq!(trigger.role.as_str(), "Button");
        assert!(!trigger.disabled);
        assert!(trigger.frame.width > 0.0 && trigger.frame.height > 0.0);
        assert!(!file_menu_locale_node(&closed, FILE_MENU_CONTROL_ID).popup_open);
        let hit = hit_test_workbench_window_template_node_with_index(
            closed.structure(),
            closed.workbench_hit_index(),
            trigger.frame.x + trigger.frame.width * 0.5,
            trigger.frame.y + trigger.frame.height * 0.5,
        )
        .expect("the published File slot must be a pointer target");
        assert_eq!(hit.control_id.as_str(), TRIGGER);
        assert_eq!(hit.binding_id.as_str(), "UnrealMenu/File");
        assert_eq!(hit.action_id.as_str(), TRIGGER_ROUTE);
        assert!(!hit.disabled);
        let binding = bridge
            .dispatch_control_state(hit.control_id.as_str(), UiEventKind::Click)
            .unwrap()
            .expect("the hit File control must dispatch its real Click binding");
        assert!(matches!(
            binding.payload(),
            EditorUiBindingPayload::MenuAction { action_id } if action_id == TRIGGER_ROUTE
        ));

        let opened = file_menu_locale_commit(&mut bridge, &mut host);
        let popup = file_menu_locale_node(&opened, FILE_MENU_CONTROL_ID);
        assert_eq!(popup.surface_node_id, Some(popup_id));
        assert!(popup.popup_open && popup.frame.width > 0.0 && popup.frame.height > 0.0);
        let row = popup
            .structured_menu_items
            .iter()
            .find(|item| item.action_id.as_str() == OPEN_SCENE)
            .expect("the published popup must retain Open Scene's stable row");
        assert_eq!(row.label.as_str(), leaf_label);
        assert_eq!(row.shortcut.as_str(), "Ctrl+O");
        assert!(!row.disabled);
        assert_eq!(
            bridge.main_menu_item_binding(FILE_MENU_CONTROL_ID, OPEN_SCENE),
            Some(command_binding.clone())
        );
        assert!(bridge.close_popup(FILE_MENU_CONTROL_ID).unwrap());
        eprintln!(
            "FILE_MENU_LOCALE_V1 locale={tag} trigger={:?} command={:?} label={:?} shortcut={:?} disabled={} hit={:?}",
            trigger.text.as_str(),
            row.action_id.as_str(),
            row.label.as_str(),
            row.shortcut.as_str(),
            row.disabled,
            hit.action_id.as_str(),
        );
        labels.push(trigger.text.to_string());
    }

    // Fail after all three stages so the baseline must pass the same route and leaf controls.
    assert_eq!(
        labels,
        ["File", "文件", "File"],
        "committed File trigger text for en -> zh-CN -> en"
    );
}

fn file_menu_locale_commit(
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    host: &mut crate::ui::retained_host::host_contract::globals::HostContractState,
) -> crate::ui::retained_host::host_contract::data::HostPresentationGeneration {
    use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
    use zircon_runtime_interface::ui::layout::UiFrame;

    let frames = bridge.layout_frames();
    let host_frame = |frame: UiFrame| FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    };
    let mut presentation = HostWindowPresentationData {
        workbench_window_nodes: crate::ui::retained_host::to_host_contract_workbench_window_nodes(
            Some(bridge.host_projection()),
        ),
        ..Default::default()
    };
    presentation.host_layout.center_band_frame = host_frame(frames.center_band_frame.unwrap());
    presentation.host_layout.status_bar_frame = host_frame(frames.status_bar_frame.unwrap());
    host.replace_host_presentation(presentation);
    bridge.mark_host_projection_committed();
    host.presentation_generation()
}

fn file_menu_locale_node<'a>(
    generation: &'a crate::ui::retained_host::host_contract::data::HostPresentationGeneration,
    control: &str,
) -> &'a crate::ui::retained_host::host_contract::data::TemplatePaneNodeData {
    let mut nodes = generation
        .structure()
        .workbench_window_nodes
        .iter()
        .filter(|node| node.control_id.as_str() == control);
    let node = nodes
        .next()
        .expect("the mounted source must publish this control");
    assert!(nodes.next().is_none(), "{control} must have one live owner");
    node
}
