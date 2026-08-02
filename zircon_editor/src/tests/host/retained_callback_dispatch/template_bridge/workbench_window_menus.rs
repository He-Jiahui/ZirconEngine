use std::collections::BTreeSet;
use std::sync::Arc;

use super::super::support::*;
use super::support::{control_bool, control_float, control_string, control_visibility};
use crate::core::asset::{
    AssetCreationTemplateDescriptor, AssetTypeContribution, AssetTypeId, AssetTypeRegistry,
};
use crate::core::commands::EditorCommandDescriptor;
use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::play::{PlayKind, PlayMode};
use crate::ui::binding::AssetCommand;
use crate::ui::retained_host::menu_popup_contract::menu_popup_content_height;
use crate::ui::retained_host::popup_anchor_metrics::{
    clamp_popup_x_to_bounds, toolbar_popup_render_gap,
};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::tree::UiVisibility;

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
        ("menu.item.open_project", "workbench.project.open"),
        ("menu.item.save_project", "workbench.project.save"),
    ] {
        let binding = bridge
            .main_menu_item_binding("WorkbenchToolbarMainMenu", action_id)
            .unwrap_or_else(|| panic!("{action_id} should resolve a canonical binding"));
        assert!(matches!(
            binding.payload(),
            EditorUiBindingPayload::MenuAction { action_id } if action_id == expected
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
            "Asset Browser|icon=folder",
            "---",
            chrome.asset_browser.creation_menu.entries()[0].raw_item(),
            "---",
            "Open Project|icon=folder",
            "Save Project|icon=save",
            "Command Palette|submenu",
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

#[test]
fn asset_creation_menu_reuses_generation_and_publishes_collision_safe_o1_actions() {
    let shell_size = UiSize::new(900.0, 620.0);
    let create_layout = EditorOperationPath::parse("ui_asset.layout.create").unwrap();
    let create_widget = EditorOperationPath::parse("ui_asset.widget.create").unwrap();
    let layout_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let widget_type = AssetTypeId::from_resource_kind(ResourceKind::UiWidget);
    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    registry
        .apply_contribution(
            "test.ui.creation",
            AssetTypeContribution::augment(layout_type.clone()).with_creation_template(
                AssetCreationTemplateDescriptor::new(
                    "ui_asset.layout",
                    "UI|\nAsset",
                    create_layout,
                ),
            ),
        )
        .unwrap();
    registry
        .apply_contribution(
            "test.ui.creation",
            AssetTypeContribution::augment(widget_type.clone()).with_creation_template(
                AssetCreationTemplateDescriptor::new("ui_asset.widget", "UI Asset", create_widget),
            ),
        )
        .unwrap();

    let generation = registry.creation_menu_generation();
    assert!(Arc::ptr_eq(
        &generation,
        &registry.creation_menu_generation()
    ));
    assert_eq!(generation.entries().len(), 2);
    assert_ne!(
        generation.entries()[0].action_id(),
        generation.entries()[1].action_id()
    );
    assert_ne!(
        generation.entries()[0].raw_item(),
        generation.entries()[1].raw_item()
    );
    for entry in generation.entries() {
        assert_eq!(generation.action(entry.action_id()), Some(entry));
        assert_eq!(entry.raw_item().matches('|').count(), 1);
    }

    let mut chrome = default_preview_fixture().build_chrome();
    chrome.asset_browser.creation_menu = Arc::clone(&generation);
    chrome.asset_browser.selected_folder_id = Some("res://ui".to_string());
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size)
        .expect("componentized workbench template should project");
    for _ in 0..100 {
        bridge
            .recompute_layout_with_workbench_model(
                shell_size,
                &model,
                &WorkbenchChromeMetrics::default(),
            )
            .unwrap();
    }
    assert_eq!(bridge.asset_creation_menu_publish_count(), 1);

    for entry in generation.entries() {
        let request = bridge
            .asset_creation_menu_request(
                &chrome.asset_browser,
                "WorkbenchToolbarMainMenu",
                entry.action_id(),
            )
            .unwrap()
            .unwrap();
        assert_eq!(request.asset_type(), entry.asset_type());
        assert_eq!(request.template_id(), entry.template_id());
    }
}

#[test]
fn asset_creation_menu_keeps_a_compiled_generation_across_scale_matrix() {
    const TEMPLATE_COUNT: usize = 10_000;
    const RESIZE_COUNT: usize = 1_000;
    const ACTION_LOOKUP_COUNT: usize = 1_000_000;

    let shell_size = UiSize::new(900.0, 620.0);
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let create = EditorOperationPath::parse("ui_asset.layout.create").unwrap();
    let mut contribution = AssetTypeContribution::augment(asset_type.clone());
    for ordinal in 0..TEMPLATE_COUNT {
        contribution = contribution.with_creation_template(AssetCreationTemplateDescriptor::new(
            display_colliding_template_id(ordinal),
            "Scale Asset",
            create.clone(),
        ));
    }

    let mut registry = AssetTypeRegistry::with_builtins().unwrap();
    registry
        .apply_contribution("test.asset.creation.scale", contribution)
        .unwrap();
    let generation = registry.creation_menu_generation();
    assert_eq!(generation.entries().len(), TEMPLATE_COUNT);
    assert_eq!(
        generation
            .entries()
            .iter()
            .map(|entry| {
                entry
                    .raw_item()
                    .split_once('|')
                    .expect("compiled menu item should separate its visible label and action")
                    .0
            })
            .collect::<BTreeSet<_>>()
            .len(),
        TEMPLATE_COUNT,
        "display-equivalent template identifiers must still receive unique menu labels"
    );
    assert!(Arc::ptr_eq(
        &generation,
        &registry.creation_menu_generation()
    ));

    let mut chrome = default_preview_fixture().build_chrome();
    chrome.asset_browser.creation_menu = Arc::clone(&generation);
    chrome.asset_browser.selected_folder_id = Some("res://scale".to_string());
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(shell_size)
        .expect("componentized workbench template should project");
    for _ in 0..RESIZE_COUNT {
        bridge
            .recompute_layout_with_workbench_model(
                shell_size,
                &model,
                &WorkbenchChromeMetrics::default(),
            )
            .unwrap();
    }
    assert_eq!(bridge.asset_creation_menu_publish_count(), 1);

    let entry = &generation.entries()[TEMPLATE_COUNT / 2];
    let action_id = entry.action_id().to_owned();
    for _ in 0..ACTION_LOOKUP_COUNT {
        assert!(bridge.is_asset_creation_menu_action("WorkbenchToolbarMainMenu", &action_id));
    }
    let request = bridge
        .asset_creation_menu_request(
            &chrome.asset_browser,
            "WorkbenchToolbarMainMenu",
            &action_id,
        )
        .unwrap()
        .unwrap();
    assert_eq!(request.asset_type(), entry.asset_type());
    assert_eq!(request.template_id(), entry.template_id());
    assert_eq!(request.target_folder(), "res://scale");
}

fn display_colliding_template_id(mut ordinal: usize) -> String {
    let mut id = String::from("test.scale");
    loop {
        id.push(char::from_u32((ordinal % 31 + 1) as u32).unwrap());
        ordinal /= 31;
        if ordinal == 0 {
            return id;
        }
    }
}

#[test]
fn enabled_asset_type_materialization_commits_one_generation_batch() {
    let source = include_str!("../../../../ui/host/editor_extension_registration.rs");
    let implementation = source
        .split("pub(crate) fn materialize_enabled_asset_types(")
        .nth(1)
        .unwrap()
        .split("pub(crate) fn enabled_asset_types_for_shell(")
        .next()
        .unwrap();

    assert!(implementation.contains("asset_types.apply_contributions("));
    assert!(!implementation.contains("asset_types.apply_contribution("));
}

#[test]
fn workbench_main_menu_asset_creation_invokes_registered_operation() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_workbench_main_menu_asset_create");
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let create = EditorOperationPath::parse("ui_asset.layout.create").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_command(
            EditorCommandDescriptor::operation(create.clone(), "Create UI Layout")
                .with_event(EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser)),
        )
        .unwrap();
    extension
        .register_asset_type_contribution(
            AssetTypeContribution::augment(asset_type.clone()).with_creation_template(
                AssetCreationTemplateDescriptor::new(
                    "ui_asset.layout",
                    "UI Layout",
                    create.clone(),
                ),
            ),
        )
        .unwrap();
    harness
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .unwrap();

    let shell_size = UiSize::new(900.0, 620.0);
    let chrome = harness.runtime.chrome_snapshot();
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
        .expect("registered asset creation template should project");
    bridge
        .dispatch_control_state("WorkbenchToolbarMenu", UiEventKind::Click)
        .expect("main menu should dispatch")
        .expect("main menu should expose a click binding");

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchToolbarMainMenu",
        "menu.item.create_u_i_layout",
    )
    .expect("asset creation menu item should be handled")
    .expect("asset creation menu item should dispatch");

    let record = harness.runtime.journal().records().last().unwrap().clone();
    assert_eq!(record.operation_id.as_deref(), Some(create.as_str()));
    assert_eq!(
        record
            .operation_arguments
            .as_ref()
            .and_then(|arguments| arguments.get("asset_type"))
            .and_then(serde_json::Value::as_str),
        Some(asset_type.as_str())
    );
    assert_eq!(
        record
            .operation_arguments
            .as_ref()
            .and_then(|arguments| arguments.get("template_id"))
            .and_then(serde_json::Value::as_str),
        Some("ui_asset.layout")
    );
    assert_eq!(
        record
            .operation_arguments
            .as_ref()
            .and_then(|arguments| arguments.get("target_folder"))
            .and_then(serde_json::Value::as_str),
        Some("res://")
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn workbench_toolbar_window_menus_open_exclusively_and_toggle_closed() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Collapsed)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Collapsed)
    );

    let binding = bridge
        .dispatch_control_state("WorkbenchRunMode", UiEventKind::Click)
        .expect("run mode menu should dispatch")
        .expect("run mode should expose a menu binding");
    assert!(matches!(
        binding.payload(),
        EditorUiBindingPayload::MenuAction { action_id } if action_id == "workbench.run_mode.menu.open"
    ));
    assert!(control_bool(&bridge, "WorkbenchRunMode", "selected"));
    assert!(control_bool(&bridge, "WorkbenchRunMode", "checked"));
    assert!(control_bool(&bridge, "WorkbenchRunModeMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Visible)
    );
    assert!(
        bridge.control_frame("WorkbenchRunModeMenu").is_some(),
        "opened run mode menu should have a native frame"
    );

    bridge
        .dispatch_control_state("WorkbenchLayoutGrid", UiEventKind::Click)
        .expect("layout menu should dispatch")
        .expect("layout menu should expose a menu binding");
    assert!(!control_bool(&bridge, "WorkbenchRunMode", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchRunModeMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Collapsed)
    );
    assert!(control_bool(&bridge, "WorkbenchLayoutGrid", "selected"));
    assert!(control_bool(&bridge, "WorkbenchLayoutMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Visible)
    );

    bridge
        .dispatch_control_state("WorkbenchLayoutGrid", UiEventKind::Click)
        .expect("layout menu should dispatch when toggled closed")
        .expect("layout menu should expose a menu binding");
    assert!(!control_bool(&bridge, "WorkbenchLayoutGrid", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchLayoutMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchLayoutMenu"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn workbench_toolbar_window_menu_item_selection_closes_trigger_and_menu() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
        .expect("componentized workbench template should project");

    bridge
        .dispatch_control_state("WorkbenchRunMode", UiEventKind::Click)
        .expect("run mode menu should dispatch")
        .expect("run mode should expose a menu binding");

    assert_eq!(
        bridge
            .select_popup_menu_item("WorkbenchRunModeMenu", "menu.item.simulate")
            .expect("run mode menu item should select"),
        Some(true)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRunModeMenu", "value").as_deref(),
        Some("Simulate")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRunModeMenu", "value_text").as_deref(),
        Some("Simulate")
    );
    assert_eq!(
        control_string_array(&bridge, "WorkbenchRunModeMenu", "menu_items"),
        vec![
            "Play In Editor|icon=play",
            "Simulate|icon=play,checked",
            "Standalone|disabled,icon=grid",
            "Network Preview|disabled,submenu",
        ]
    );
    assert_eq!(
        bridge
            .select_popup_menu_item("WorkbenchRunModeMenu", "menu.item.standalone")
            .expect("disabled run mode should resolve without mutation"),
        Some(false)
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchRunModeMenu", "value").as_deref(),
        Some("Simulate")
    );
    assert!(!control_bool(&bridge, "WorkbenchRunMode", "selected"));
    assert!(!control_bool(&bridge, "WorkbenchRunMode", "checked"));
    assert!(!control_bool(&bridge, "WorkbenchRunModeMenu", "popup_open"));
    assert_eq!(
        control_visibility(&bridge, "WorkbenchRunModeMenu"),
        Some(UiVisibility::Collapsed)
    );
}

#[test]
fn mixed_command_menu_preserves_its_authored_checked_indicator() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("componentized workbench template should project");

    assert_eq!(
        bridge
            .select_popup_menu_item("WorkbenchLayoutMenu", "menu.item.reset_layout")
            .expect("Reset Layout should select as a command row"),
        Some(true)
    );
    assert_eq!(
        control_string_array(&bridge, "WorkbenchLayoutMenu", "menu_items"),
        vec![
            "Default Layout|checked,disabled,icon=grid",
            "Gameplay Layout|disabled,icon=grid",
            "Rendering Layout|disabled,icon=grid",
            "Reset Layout|danger,icon=trash",
        ]
    );
}

#[test]
fn workbench_run_mode_selection_drives_the_next_play_session() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_workbench_run_mode_simulate");
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("componentized workbench template should project");

    let effects = dispatch_componentized_workbench_menu_item_selected(
        &harness.runtime,
        &mut bridge,
        "WorkbenchRunModeMenu",
        "menu.item.simulate",
    )
    .expect("Simulate menu item should be handled")
    .expect("Simulate menu item should dispatch");

    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::WorkbenchMenu(MenuAction::SelectPlayMode(PlayKind::Simulate))
    );
    assert_eq!(
        harness.runtime.play_sessions().preferred_kind(),
        PlayKind::Simulate
    );
    assert!(effects.presentation_dirty);

    let mut rebuilt_bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
            .expect("rebuilt workbench template should project");
    dispatch_componentized_workbench_control(
        &harness.runtime,
        &mut rebuilt_bridge,
        "WorkbenchRunMode",
        UiEventKind::Click,
    )
    .expect("rebuilt Run Mode should be handled")
    .expect("rebuilt Run Mode should open");
    assert_eq!(
        control_string(&rebuilt_bridge, "WorkbenchRunModeMenu", "value").as_deref(),
        Some("Simulate")
    );
    assert_eq!(
        control_string_array(&rebuilt_bridge, "WorkbenchRunModeMenu", "menu_items"),
        vec![
            "Play In Editor|icon=play",
            "Simulate|icon=play,checked",
            "Standalone|disabled,icon=grid",
            "Network Preview|disabled,submenu",
        ]
    );

    dispatch_componentized_workbench_control(
        &harness.runtime,
        &mut bridge,
        "WorkbenchRunPlay",
        UiEventKind::Click,
    )
    .expect("Play should be handled")
    .expect("Play should enter the selected run mode");

    assert_eq!(
        harness.runtime.play_sessions().mode_snapshot(),
        PlayMode::Playing {
            kind: PlayKind::Simulate,
        }
    );
}

#[test]
fn workbench_toolbar_window_menus_anchor_to_toolbar_controls_across_widths() {
    for width in [900.0, 1260.0, 1672.0] {
        let mut bridge =
            BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(width, 620.0))
                .unwrap_or_else(|error| {
                    panic!("workbench {width}px bridge should build: {error:?}")
                });
        assert_toolbar_menu_anchor(
            &mut bridge,
            "WorkbenchToolbarMenu",
            "WorkbenchToolbarMainMenu",
            ToolbarMenuAlign::Start,
        );
    }

    let mut compact = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(900.0, 620.0))
        .expect("compact workbench bridge should build");
    assert_toolbar_menu_anchor(
        &mut compact,
        "WorkbenchModuleMore",
        "WorkbenchModuleOverflowMenu",
        ToolbarMenuAlign::Start,
    );

    for (trigger_id, menu_id) in [
        ("WorkbenchRunMode", "WorkbenchRunModeMenu"),
        ("WorkbenchLayoutGrid", "WorkbenchLayoutMenu"),
    ] {
        let mut wide = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0))
            .expect("wide workbench bridge should build");
        assert_toolbar_menu_anchor(&mut wide, trigger_id, menu_id, ToolbarMenuAlign::End);
    }
}

#[test]
fn workbench_toolbar_window_menu_anchor_metrics_stay_shared() {
    let source = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "src/ui/retained_host/callback_dispatch/template_bridge/workbench/window_menu_state.rs",
    ))
    .expect("window menu state source should be readable");

    assert!(source.contains("popup_anchor_metrics"));
    assert!(
        !source.contains("const TOOLBAR_POPUP_EDGE_MARGIN"),
        "toolbar popup edge margin must stay in popup_anchor_metrics"
    );
    assert!(
        !source.contains("const TOOLBAR_POPUP_RENDER_GAP"),
        "toolbar popup render gap must stay in popup_anchor_metrics"
    );
}

#[derive(Clone, Copy)]
enum ToolbarMenuAlign {
    Start,
    End,
}

fn assert_toolbar_menu_anchor(
    bridge: &mut BuiltinWorkbenchWindowTemplateSurfaceBridge,
    trigger_id: &str,
    menu_id: &str,
    align: ToolbarMenuAlign,
) {
    let trigger = bridge
        .control_frame(trigger_id)
        .unwrap_or_else(|| panic!("{trigger_id} should expose a visible trigger frame"));
    let toolbar = bridge
        .control_frame("WorkbenchWindowTopToolbarRegion")
        .expect("workbench should expose the top toolbar frame");
    let root = bridge
        .control_frame("WorkbenchWindowRoot")
        .expect("workbench should expose the root frame");

    bridge
        .dispatch_control_state(trigger_id, UiEventKind::Click)
        .unwrap_or_else(|error| panic!("{trigger_id} should dispatch: {error:?}"))
        .unwrap_or_else(|| panic!("{trigger_id} should expose a click binding"));

    let menu = bridge
        .control_frame(menu_id)
        .unwrap_or_else(|| panic!("{menu_id} should open with a visible menu frame"));
    let authored_x = match align {
        ToolbarMenuAlign::Start => trigger.x,
        ToolbarMenuAlign::End => trigger.right() - menu.width,
    };
    let expected_x = clamped_toolbar_menu_x(authored_x, menu.width, root.width);
    assert_near(&format!("{menu_id} x"), menu.x, expected_x);
    assert_near(&format!("{menu_id} y"), menu.y, toolbar.bottom());
    assert_near(
        &format!("{menu_id} popup_anchor_x"),
        control_float(bridge, menu_id, "popup_anchor_x")
            .unwrap_or_else(|| panic!("{menu_id} should store popup_anchor_x")) as f32,
        menu.x,
    );
    assert_near(
        &format!("{menu_id} popup_anchor_y"),
        control_float(bridge, menu_id, "popup_anchor_y")
            .unwrap_or_else(|| panic!("{menu_id} should store popup_anchor_y")) as f32,
        menu.y,
    );
    assert_near(
        &format!("{menu_id} popup_offset_y"),
        control_float(bridge, menu_id, "popup_offset_y")
            .unwrap_or_else(|| panic!("{menu_id} should store popup_offset_y")) as f32,
        -toolbar_popup_render_gap(),
    );
    assert_eq!(
        control_string(bridge, menu_id, "placement").as_deref(),
        Some("bottom-start")
    );
}

fn clamped_toolbar_menu_x(authored_x: f32, menu_width: f32, root_width: f32) -> f32 {
    clamp_popup_x_to_bounds(authored_x, 0.0, root_width, menu_width)
}

fn assert_near(label: &str, actual: f32, expected: f32) {
    const EPSILON: f32 = 0.01;
    assert!(
        (actual - expected).abs() <= EPSILON,
        "{label} should be {expected}, got {actual}"
    );
}

fn control_string_array(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    property: &str,
) -> Vec<String> {
    bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
                .and_then(|metadata| metadata.attributes.get(property))
                .and_then(toml::Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(toml::Value::as_str)
                        .map(str::to_string)
                        .collect()
                })
        })
        .unwrap_or_default()
}
