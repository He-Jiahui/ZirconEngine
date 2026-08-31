use super::support::*;

#[test]
fn workbench_main_menu_business_items_resolve_canonical_bindings() {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("componentized workbench template should project");

    let asset_browser = bridge
        .main_menu_item_binding("WorkbenchToolbarMainMenu", "menu.item.asset_browser")
        .expect("Asset Browser should resolve a canonical binding");
    assert_eq!(
        asset_browser.payload(),
        &EditorUiBindingPayload::asset_command(AssetCommand::OpenAssetBrowser)
    );
    for (action_id, expected) in [
        ("menu.item.open_project", "file.project.open"),
        ("menu.item.save_project", "file.project.save"),
    ] {
        let binding = bridge
            .main_menu_item_binding("WorkbenchToolbarMainMenu", action_id)
            .unwrap_or_else(|| panic!("{action_id} should resolve a canonical binding"));
        assert!(matches!(
            binding.payload(),
            EditorUiBindingPayload::EditorCommand { command_id } if command_id == expected
        ));
    }
    let command_palette = bridge
        .main_menu_item_binding("WorkbenchToolbarMainMenu", "menu.item.command_palette")
        .expect("Command Palette should resolve a canonical binding");
    assert!(matches!(
        command_palette.payload(),
        EditorUiBindingPayload::EditorCommand { command_id }
            if command_id == "editor.command.palette"
    ));
    assert!(bridge
        .main_menu_item_binding("WorkbenchRunModeMenu", "menu.item.asset_browser")
        .is_none());
    let reset_layout = bridge
        .layout_menu_item_binding("WorkbenchLayoutMenu", "menu.item.reset_layout")
        .expect("Reset Layout should resolve a canonical binding");
    assert_eq!(
        reset_layout.payload(),
        &EditorUiBindingPayload::menu_action("workbench.layout.reset")
    );
    assert!(bridge
        .layout_menu_item_binding("WorkbenchLayoutMenu", "menu.item.gameplay_layout")
        .is_none());
}

#[test]
fn workbench_layout_menu_dispatches_reset_and_rejects_unimplemented_presets() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_workbench_layout_menu_reset");
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("componentized workbench template should project");
    let record_count = harness.runtime.journal().records().len();

    for action_id in [
        "menu.item.default_layout",
        "menu.item.gameplay_layout",
        "menu.item.rendering_layout",
    ] {
        let disabled_effects = dispatch_componentized_workbench_menu_item_selected(
            &harness.runtime,
            &mut bridge,
            "WorkbenchLayoutMenu",
            action_id,
        )
        .unwrap_or_else(|| panic!("{action_id} row should be recognized"))
        .unwrap_or_else(|error| panic!("disabled {action_id} should not fail: {error}"));
        assert!(!disabled_effects.layout_dirty);
    }
    assert_eq!(harness.runtime.journal().records().len(), record_count);
    assert_eq!(
        control_string(&bridge, "WorkbenchLayoutMenu", "value").as_deref(),
        Some("Default Layout")
    );

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchLayoutMenu",
        "menu.item.reset_layout",
    )
    .expect("Reset Layout row should be handled")
    .expect("Reset Layout row should dispatch");
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        control_string(&bridge, "WorkbenchLayoutMenu", "value").as_deref(),
        Some("Default Layout")
    );
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchLayoutMenu")
            .expect("layout menu host projection after reset")
            .value_text
            .as_deref(),
        Some("Default Layout")
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn workbench_main_menu_asset_browser_item_dispatches_real_asset_event() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_workbench_main_menu_asset_browser");
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchToolbarMenu", UiEventKind::Click)
        .expect("main menu should dispatch")
        .expect("main menu should expose a click binding");

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchToolbarMainMenu",
        "menu.item.asset_browser",
    )
    .expect("Asset Browser menu item should be handled")
    .expect("Asset Browser menu item should dispatch");

    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser)
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert_eq!(
        control_string(&bridge, "WorkbenchToolbarMainMenu", "value").as_deref(),
        Some("Asset Browser")
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchToolbarMainMenu"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn workbench_main_menu_projects_registered_asset_creation_templates() {
    let shell_size = UiSize::new(900.0, 620.0);
    let mut chrome = default_preview_fixture().build_chrome();
    chrome.asset_browser.selected_folder_id = Some("res://ui".to_string());
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let create = EditorOperationPath::parse("ui_asset.layout.create").unwrap();
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    registry
        .apply_contribution(
            "test.ui.layout",
            AssetTypeContribution::augment(asset_type).with_creation_template(
                AssetCreationTemplateDescriptor::new("ui_asset.layout", "UI Layout", create),
            ),
        )
        .unwrap();
    chrome.asset_browser.creation_menu = registry.creation_menu_generation();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size)
        .expect("componentized workbench template should project");

    bridge
        .recompute_layout_with_workbench_model(
            shell_size,
            &model,
            &WorkbenchChromeMetrics::default(),
        )
        .expect("asset creation menu should project from the Workbench model");

    assert_eq!(
        control_string_array(&bridge, "WorkbenchToolbarMainMenu", "menu_items"),
        vec![
            "Asset Browser|action=menu.item.asset_browser,icon=folder",
            "---",
            chrome.asset_browser.creation_menu.entries()[0].raw_item(),
            "---",
            "Open Project|action=menu.item.open_project,icon=folder|Ctrl+O",
            "Save Project|action=menu.item.save_project,icon=save|Ctrl+S",
            "Command Palette|action=menu.item.command_palette,icon=search|Ctrl+Shift+P",
        ]
    );
    assert_near(
        "asset creation menu content-derived height",
        bridge
            .control_frame("WorkbenchToolbarMainMenu")
            .expect("asset creation menu should expose its frame")
            .height,
        menu_popup_content_height(7),
    );
    let request = bridge
        .asset_creation_menu_request(
            &chrome.asset_browser,
            "WorkbenchToolbarMainMenu",
            chrome.asset_browser.creation_menu.entries()[0].action_id(),
        )
        .expect("projected creation item should resolve")
        .expect("projected creation request should be valid");
    assert_eq!(request.target_folder(), "res://ui");
}
