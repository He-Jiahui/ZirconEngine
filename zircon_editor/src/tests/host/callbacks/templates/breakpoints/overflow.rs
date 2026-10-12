use super::super::super::support::{
    dispatch_componentized_workbench_menu_item_selected, env_lock,
    BuiltinWorkbenchWindowTemplateSurfaceBridge, EditorUiBindingPayload, EventRuntimeHarness,
    UiEventKind, UiSize,
};
use super::super::support::{control_bool, control_visibility};
use super::support::{workbench_window_node, COMPACT_WORKBENCH_HEIGHT, COMPACT_WORKBENCH_WIDTH};
use crate::ui::binding::AssetCommand;
use crate::ui::retained_host::{
    HostInvalidationMask, TemplatePaneMenuItemData, TemplatePaneNodeData,
};
use zircon_runtime_interface::ui::tree::UiVisibility;

#[test]
fn ultra_toolbar_main_menu_keeps_hidden_file_commands_reachable() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(420.0, 360.0))
        .unwrap_or_else(|error| panic!("ultra workbench bridge should build: {error:?}"));

    bridge
        .dispatch_control_state("WorkbenchToolbarMenu", UiEventKind::Click)
        .expect("ultra toolbar main menu should dispatch")
        .expect("ultra toolbar main menu should expose a binding");
    assert!(control_bool(
        &bridge,
        "WorkbenchToolbarMainMenu",
        "popup_open"
    ));

    let asset_browser = bridge
        .main_menu_item_binding("WorkbenchToolbarMainMenu", "menu.item.asset_browser")
        .expect("the main menu should retain the hidden Asset Browser command");
    assert_eq!(
        asset_browser.payload(),
        &EditorUiBindingPayload::asset_command(AssetCommand::OpenAssetBrowser)
    );

    for (action_id, expected_command) in [
        ("menu.item.open_project", "file.project.open"),
        ("menu.item.save_project", "file.project.save"),
    ] {
        let binding = bridge
            .main_menu_item_binding("WorkbenchToolbarMainMenu", action_id)
            .unwrap_or_else(|| panic!("the main menu should retain {expected_command}"));
        assert!(matches!(
            binding.payload(),
            EditorUiBindingPayload::EditorCommand { command_id }
                if command_id == expected_command
        ));
    }
}

#[test]
fn compact_workbench_module_more_opens_overflow_menu_and_selects_hidden_module() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let harness = EventRuntimeHarness::new("zircon_workbench_module_overflow_menu");
    let mut bridge = match BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        COMPACT_WORKBENCH_WIDTH as f32,
        COMPACT_WORKBENCH_HEIGHT as f32,
    )) {
        Ok(bridge) => bridge,
        Err(error) => panic!("workbench bridge should build: {error:?}"),
    };

    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleOverflowMenu"),
        Some(UiVisibility::Collapsed)
    );

    let binding = bridge
        .dispatch_control_state("WorkbenchModuleMore", UiEventKind::Click)
        .expect("module overflow should dispatch")
        .expect("module overflow should expose a binding");
    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id } if action_id == "workbench.module.more.open"
    ));
    assert!(control_bool(&bridge, "WorkbenchModuleMore", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleMore", "checked"));
    assert!(control_bool(
        &bridge,
        "WorkbenchModuleOverflowMenu",
        "popup_open"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleOverflowMenu"),
        Some(UiVisibility::Visible)
    );

    let opened_menu = workbench_window_node(&bridge, "WorkbenchModuleOverflowMenu");
    assert!(opened_menu.popup_open);
    assert_eq!(opened_menu.structured_menu_items.row_count(), 7);
    assert_eq!(
        structured_menu_item(&opened_menu, 0).label.as_str(),
        "Behavior"
    );
    assert_eq!(
        structured_menu_item(&opened_menu, 0).action_id.as_str(),
        "menu.item.behavior"
    );
    assert_eq!(
        structured_menu_item(&opened_menu, 1).label.as_str(),
        "Render"
    );
    assert_eq!(
        structured_menu_item(&opened_menu, 2).label.as_str(),
        "Assets"
    );
    assert_eq!(structured_menu_item(&opened_menu, 3).label.as_str(), "VFX");
    assert_eq!(structured_menu_item(&opened_menu, 4).label.as_str(), "HUD");
    assert_eq!(structured_menu_item(&opened_menu, 5).label.as_str(), "Diff");
    assert_eq!(structured_menu_item(&opened_menu, 6).label.as_str(), "Sim");

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchModuleOverflowMenu",
        "menu.item.behavior",
    )
    .expect("module overflow menu item should be handled")
    .expect("module overflow menu item should dispatch");

    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
    assert!(!control_bool(&bridge, "WorkbenchModuleMore", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchModuleMore", "checked"));
    assert!(!control_bool(
        &bridge,
        "WorkbenchModuleOverflowMenu",
        "popup_open"
    ));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleOverflowMenu"),
        Some(UiVisibility::Collapsed)
    );
    assert!(control_bool(&bridge, "WorkbenchModuleBehavior", "selected"));
    assert!(control_bool(&bridge, "WorkbenchModuleBehavior", "checked"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleBehaviorWorkspace"),
        Some(UiVisibility::Visible)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchModuleEffectWorkspace"),
        Some(UiVisibility::Collapsed)
    );

    bridge
        .dispatch_control_state("WorkbenchModuleMore", UiEventKind::Click)
        .expect("module overflow should reopen")
        .expect("module overflow should expose a binding");
    let reopened_menu = workbench_window_node(&bridge, "WorkbenchModuleOverflowMenu");
    assert!(
        structured_menu_item(&reopened_menu, 0).checked,
        "reopened overflow menu should reflect the active hidden module"
    );
}

#[test]
fn ultra_module_overflow_exposes_every_hidden_module_and_secondary_command() {
    let _guard = match env_lock().lock() {
        Ok(guard) => guard,
        Err(error) => panic!("test environment lock is poisoned: {error}"),
    };

    let harness = EventRuntimeHarness::new("zircon_workbench_ultra_module_overflow_menu");
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(420.0, 360.0))
        .unwrap_or_else(|error| panic!("ultra workbench bridge should build: {error:?}"));
    bridge
        .dispatch_control_state("WorkbenchModuleMore", UiEventKind::Click)
        .expect("ultra module overflow should dispatch")
        .expect("ultra module overflow should expose a binding");

    let opened_menu = workbench_window_node(&bridge, "WorkbenchModuleOverflowMenu");
    assert_eq!(opened_menu.structured_menu_items.row_count(), 9);
    let labels = (0..9)
        .map(|row| structured_menu_item(&opened_menu, row).label.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        labels.iter().map(String::as_str).collect::<Vec<_>>(),
        vec![
            "Perception",
            "Material",
            "Behavior",
            "Render",
            "Assets",
            "VFX",
            "HUD",
            "Diff",
            "Sim",
        ]
    );
    assert!(
        opened_menu.frame.y + opened_menu.frame.height <= 360.0,
        "ultra overflow menu should remain inside the 360px shell"
    );

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchModuleOverflowMenu",
        "menu.item.diff",
    )
    .expect("hidden Diff menu item should be handled")
    .expect("hidden Diff menu item should dispatch through the source binding");
    assert!(effects
        .dirty_domains()
        .contains(HostInvalidationMask::PAINT_ONLY));
}

fn structured_menu_item(node: &TemplatePaneNodeData, row: usize) -> TemplatePaneMenuItemData {
    node.structured_menu_items
        .row_data(row)
        .unwrap_or_else(|| panic!("structured menu item row {row} should exist"))
}
