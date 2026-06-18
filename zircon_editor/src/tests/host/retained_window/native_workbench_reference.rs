use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use crate::ui::retained_host::callback_dispatch::{
    BuiltinHostWindowTemplateBridge, BuiltinWorkbenchWindowTemplateSurfaceBridge,
};
use crate::ui::retained_host::workbench_popup_actions::WORKBENCH_POPUP_CANCEL_ACTION_ID;
use crate::ui::retained_host::{
    paint_runtime_render_commands_for_test, to_host_contract_workbench_window_nodes,
    HostWindowPresentationData, PaneSurfaceHostContext, TemplatePaneMenuItemData,
    TemplatePaneNodeData, TemplatePaneOptionData, UiHostWindow, WorkbenchContextMenuRequestData,
};
use zircon_runtime_interface::ui::{binding::UiEventKind, layout::UiFrame, layout::UiSize};

const WORKBENCH_REFERENCE_IMAGE_CONTROL_ID: &str = "WorkbenchShellReferenceImage";
const WORKBENCH_REFERENCE_WINDOW_CONTROL_ID: &str = "WorkbenchReferenceImage";
const WORKBENCH_REFERENCE_WIDTH: u32 = 1672;
const WORKBENCH_REFERENCE_HEIGHT: u32 = 941;
const OUTSIDE_WORKBENCH_POPUP_X: f32 = 16.0;
const OUTSIDE_WORKBENCH_POPUP_Y: f32 = 16.0;
const WORKBENCH_PREVIEW_CAPTURE_ENV: &str = "ZIRCON_WRITE_WORKBENCH_PREVIEW";
const WORKBENCH_PREVIEW_CAPTURE_PATH_ENV: &str = "ZIRCON_WORKBENCH_PREVIEW_PATH";
const COMPONENT_LAB_INPUT_TEXT_COMMIT_ACTION_ID: &str = "component_lab.input_text.commit";
const WORKBENCH_ABILITY_NAME_EDIT_ACTION_ID: &str = "workbench.module.ability.name.edit";
const WORKBENCH_ABILITY_NAME_COMMIT_ACTION_ID: &str = "workbench.module.ability.name.commit";
const WORKBENCH_MENU_NEW_ACTION_ID: &str = "menu.item.new";
const WORKBENCH_MENU_MORE_TOOLS_ACTION_ID: &str = "menu.item.more_tools";

#[test]
fn host_window_template_bridge_keeps_workbench_reference_out_of_projection() {
    let bridge = BuiltinHostWindowTemplateBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("builtin workbench host template should project");

    assert!(
        bridge
            .host_projection()
            .node_by_control_id(WORKBENCH_REFERENCE_IMAGE_CONTROL_ID)
            .is_none(),
        "host template must not project the full workbench reference PNG"
    );
    assert!(
        bridge
            .host_projection()
            .node_by_control_id(WORKBENCH_REFERENCE_WINDOW_CONTROL_ID)
            .is_none(),
        "window template must not project the full workbench reference PNG"
    );
}

#[test]
fn componentized_workbench_surface_paints_native_preview_pixels_and_interaction_state() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    let move_frame = bridge
        .control_frame("WorkbenchToolMove")
        .expect("move tool should have an arranged frame");

    let initial = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    assert_eq!(
        initial.len(),
        WORKBENCH_REFERENCE_WIDTH as usize * WORKBENCH_REFERENCE_HEIGHT as usize * 4
    );
    assert!(
        contains_at_least_distinct_non_black_pixels(&initial, 3),
        "native preview capture should contain multiple painted colors"
    );
    maybe_write_workbench_preview_png(&initial);

    bridge
        .dispatch_control_state("WorkbenchToolMove", UiEventKind::Click)
        .expect("tool dispatch should update bridge state")
        .expect("move tool should have a binding");

    let updated = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    let changed_pixels = changed_pixel_count_in_frame(&initial, &updated, move_frame);
    let updated_tool_pixel = first_non_black_pixel_in_frame(&updated, move_frame)
        .expect("selected tool frame should contain visible pixels");

    assert!(
        changed_pixels > 0,
        "state dispatch should repaint at least one pixel inside the selected tool frame"
    );
    assert_ne!(
        updated_tool_pixel,
        [0, 0, 0, 255],
        "selected tool background should not be an empty black placeholder"
    );
}

#[test]
fn native_workbench_text_input_focuses_edits_and_commits_from_keyboard() {
    let ui = host_with_componentized_workbench_nodes();
    let edited_values = Rc::new(RefCell::new(Vec::new()));
    let edited_values_for_callback = edited_values.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_edited(move |control_id, action_id, value| {
            edited_values_for_callback.borrow_mut().push((
                control_id.to_string(),
                action_id.to_string(),
                value.to_string(),
            ));
        });

    let before = ui.get_host_presentation();
    let input = workbench_node(&before, "WorkbenchInputText");
    assert_eq!(input.value_text.as_str(), "Text field");
    assert_eq!(
        input.edit_action_id.as_str(),
        "component_lab.input_text.edit"
    );
    assert_eq!(
        input.commit_action_id.as_str(),
        "component_lab.input_text.commit"
    );

    let (x, y) = node_center(&input);
    let focus_result = ui.dispatch_native_primary_press_for_test(x, y);
    assert!(focus_result.request_redraw());
    assert!(ui.text_input_focus_active());

    let edit_result = ui.dispatch_native_text_for_test("!");
    assert!(edit_result.request_redraw());
    let commit_result = ui.dispatch_native_enter_for_test();
    assert!(commit_result.request_redraw());

    assert_eq!(
        edited_values.borrow().as_slice(),
        [
            (
                "WorkbenchInputText".to_string(),
                "component_lab.input_text.edit".to_string(),
                "Text field!".to_string()
            ),
            (
                "WorkbenchInputText".to_string(),
                COMPONENT_LAB_INPUT_TEXT_COMMIT_ACTION_ID.to_string(),
                "Text field!".to_string()
            )
        ]
    );
}

#[test]
fn native_workbench_module_field_focuses_edits_and_commits_from_keyboard() {
    let ui = host_with_selected_workbench_module_nodes("WorkbenchModuleAbility");
    let edited_values = Rc::new(RefCell::new(Vec::new()));
    let edited_values_for_callback = edited_values.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_edited(move |control_id, action_id, value| {
            edited_values_for_callback.borrow_mut().push((
                control_id.to_string(),
                action_id.to_string(),
                value.to_string(),
            ));
        });

    let before = ui.get_host_presentation();
    let input = workbench_node(&before, "WorkbenchAbilityNameField");
    assert_eq!(input.value_text.as_str(), "GA_DashAttack");
    assert_eq!(
        input.edit_action_id.as_str(),
        WORKBENCH_ABILITY_NAME_EDIT_ACTION_ID
    );
    assert_eq!(
        input.commit_action_id.as_str(),
        WORKBENCH_ABILITY_NAME_COMMIT_ACTION_ID
    );
    assert!(
        input.frame.width > 0.0 && input.frame.height > 0.0,
        "selected module field should have a hittable native frame"
    );

    let (x, y) = node_center(&input);
    let focus_result = ui.dispatch_native_primary_press_for_test(x, y);
    assert!(focus_result.request_redraw());
    assert!(ui.text_input_focus_active());

    let edit_result = ui.dispatch_native_text_for_test("_Preview");
    assert!(edit_result.request_redraw());
    let commit_result = ui.dispatch_native_enter_for_test();
    assert!(commit_result.request_redraw());

    assert_eq!(
        edited_values.borrow().as_slice(),
        [
            (
                "WorkbenchAbilityNameField".to_string(),
                WORKBENCH_ABILITY_NAME_EDIT_ACTION_ID.to_string(),
                "GA_DashAttack_Preview".to_string()
            ),
            (
                "WorkbenchAbilityNameField".to_string(),
                WORKBENCH_ABILITY_NAME_COMMIT_ACTION_ID.to_string(),
                "GA_DashAttack_Preview".to_string()
            )
        ]
    );
}

#[test]
fn componentized_workbench_module_command_feedback_paints_native_preview_pixels() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchModuleAbility", UiEventKind::Click)
        .expect("ability module tab should dispatch")
        .expect("ability module tab should expose a preview binding");
    bridge
        .dispatch_control_state("WorkbenchAbilityPlaytestButton", UiEventKind::Click)
        .expect("ability playtest button should dispatch")
        .expect("ability playtest button should expose a preview binding");

    let output_frame = bridge
        .control_frame("WorkbenchAbilityOutputRow")
        .expect("ability output row should have a native frame");
    let status_frame = bridge
        .control_frame("WorkbenchStatusReady")
        .expect("status ready item should have a native frame");
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchAbilityOutputRow")
            .expect("ability output row projection after command")
            .value_text
            .as_deref(),
        Some("Playtest queued   predicted activation   GA_DashAttack")
    );

    let pixels = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    maybe_write_workbench_preview_png(&pixels);

    assert!(
        first_non_black_pixel_in_frame(&pixels, output_frame).is_some(),
        "command feedback output row should render visible native pixels"
    );
    assert!(
        first_non_black_pixel_in_frame(&pixels, status_frame).is_some(),
        "command feedback status item should render visible native pixels"
    );
}

#[test]
fn componentized_workbench_module_dropdown_open_paints_native_preview_pixels() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
        .expect("material module tab should dispatch")
        .expect("material module tab should expose a preview binding");
    let dropdown_frame = bridge
        .control_frame("WorkbenchMaterialDomainDropdown")
        .expect("material domain dropdown should have a native frame");
    let closed = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );

    bridge
        .dispatch_control_state("WorkbenchMaterialDomainDropdown", UiEventKind::Change)
        .expect("material domain dropdown should dispatch")
        .expect("material domain dropdown should expose a field binding");
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchMaterialDomainDropdown")
            .expect("material domain dropdown projection after open")
            .value_text
            .as_deref(),
        Some("Surface")
    );

    let opened = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    maybe_write_workbench_preview_png(&opened);

    assert!(
        changed_pixel_count_in_frame(&closed, &opened, dropdown_frame) > 0,
        "opening the module dropdown should repaint the native dropdown frame"
    );
    assert!(
        first_non_black_pixel_in_frame(&opened, dropdown_frame).is_some(),
        "opened module dropdown should render visible native pixels"
    );
}

#[test]
fn componentized_workbench_module_dropdown_selection_paints_native_preview_pixels() {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchModuleMaterial", UiEventKind::Click)
        .expect("material module tab should dispatch")
        .expect("material module tab should expose a preview binding");
    let dropdown_frame = bridge
        .control_frame("WorkbenchMaterialDomainDropdown")
        .expect("material domain dropdown should have a native frame");
    let before = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );

    assert!(bridge
        .select_dropdown_option("WorkbenchMaterialDomainDropdown", "post_process")
        .expect("material domain dropdown option selection should apply"));
    assert_eq!(
        bridge
            .host_projection()
            .node_by_control_id("WorkbenchMaterialDomainDropdown")
            .expect("material domain dropdown projection after option selection")
            .value_text
            .as_deref(),
        Some("post_process")
    );

    let after = paint_runtime_render_commands_for_test(
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        &bridge.surface().render_extract.list.commands,
    );
    maybe_write_workbench_preview_png(&after);

    assert!(
        changed_pixel_count_in_frame(&before, &after, dropdown_frame) > 0,
        "selecting a module dropdown option should repaint the native dropdown frame"
    );
    assert!(
        first_non_black_pixel_in_frame(&after, dropdown_frame).is_some(),
        "selected module dropdown should render visible native pixels"
    );
}

#[test]
fn native_workbench_dropdown_option_row_hover_updates_structured_row_state() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(dropdown.popup_open);
    assert!(structured_option(&dropdown, 1).hovered);

    let (x, y) = dropdown_option_row_point(&dropdown, 0);
    let result = ui.dispatch_native_pointer_move_for_test(x, y);

    assert!(result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .as_str(),
        "workbench_option"
    );
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );
    let dropdown = workbench_node(&after, "WorkbenchInputDropdown");
    let hovered = structured_option(&dropdown, 0);
    let previous = structured_option(&dropdown, 1);

    assert!(hovered.hovered);
    assert!(!previous.hovered);
    assert!(!previous.focused);
}

#[test]
fn native_workbench_popup_menu_row_hover_updates_structured_row_state() {
    let ui = host_with_componentized_workbench_nodes();
    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    assert!(structured_menu_item(&menu, 3).hovered);

    let (x, y) = menu_item_row_point(&menu, 0);
    let result = ui.dispatch_native_pointer_move_for_test(x, y);

    assert!(result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .as_str(),
        "workbench_menu_item"
    );
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_NEW_ACTION_ID
    );
    let menu = workbench_node(&after, "WorkbenchPopupMenu");
    let hovered = structured_menu_item(&menu, 0);
    let previous = structured_menu_item(&menu, 3);

    assert!(hovered.hovered);
    assert!(!previous.hovered);
}

#[test]
fn native_workbench_dropdown_option_primary_press_keeps_selection_path() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let selected_options = Rc::new(RefCell::new(Vec::new()));
    let selected_options_for_callback = selected_options.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_component_showcase_option_selected(move |control_id, action_id, option_id| {
            selected_options_for_callback.borrow_mut().push((
                control_id.to_string(),
                action_id.to_string(),
                option_id.to_string(),
            ));
        });
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    let (x, y) = dropdown_option_row_point(&dropdown, 0);
    let result = ui.dispatch_native_primary_press_for_test(x, y);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        selected_options.borrow().as_slice(),
        [(
            "WorkbenchInputDropdown".to_string(),
            "component_lab.input_dropdown.select".to_string(),
            "dropdown".to_string()
        )]
    );
    assert!(cancelled.borrow().is_empty());
}

#[test]
fn native_workbench_popup_menu_item_primary_press_keeps_menu_selection_path() {
    let ui = host_with_componentized_workbench_nodes();
    let clicked_items = Rc::new(RefCell::new(Vec::new()));
    let clicked_items_for_callback = clicked_items.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            clicked_items_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    let (x, y) = menu_item_row_point(&menu, 0);
    let result = ui.dispatch_native_primary_press_for_test(x, y);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        clicked_items.borrow().as_slice(),
        [(
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_MENU_NEW_ACTION_ID.to_string()
        )]
    );
}

#[test]
fn native_workbench_secondary_press_requests_scene_context_menu() {
    let ui = host_with_componentized_workbench_nodes();
    let requests = Rc::new(RefCell::new(Vec::<WorkbenchContextMenuRequestData>::new()));
    let requests_for_callback = requests.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_workbench_context_menu_requested(move |request| {
            requests_for_callback.borrow_mut().push(request);
        });

    let before = ui.get_host_presentation();
    let scene_node = workbench_node(&before, "WorkbenchScenePropsItem");
    let (x, y) = node_right_center(&scene_node);
    let result = ui.dispatch_native_secondary_press_for_test(x, y);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    let requests = requests.borrow();
    let request = requests
        .first()
        .expect("scene row secondary press should request a context menu");
    assert_eq!(
        request.target_control_id.as_str(),
        "WorkbenchScenePropsItem"
    );
    assert_eq!(
        request.target_path.as_str(),
        "workbench://scene/workbenchscenepropsitem"
    );
    assert_eq!(request.popup_anchor_x, x);
    assert_eq!(request.popup_anchor_y, y);
    assert!(request
        .menu_items
        .iter()
        .any(|item| item.as_str() == "Rename|icon=edit"));
    assert!(request
        .menu_items
        .iter()
        .any(|item| item.as_str() == "Delete|danger,icon=trash"));
}

#[test]
fn native_workbench_disabled_dropdown_option_primary_press_is_ignored_without_cancel() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let selected_options = Rc::new(RefCell::new(Vec::new()));
    let selected_options_for_callback = selected_options.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_component_showcase_option_selected(move |control_id, action_id, option_id| {
            selected_options_for_callback.borrow_mut().push((
                control_id.to_string(),
                action_id.to_string(),
                option_id.to_string(),
            ));
        });
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(structured_option(&dropdown, 2).disabled);
    let (x, y) = dropdown_option_row_point(&dropdown, 2);
    let result = ui.dispatch_native_primary_press_for_test(x, y);

    assert!(!result.requires_frame_update());
    assert!(selected_options.borrow().is_empty());
    assert!(cancelled.borrow().is_empty());
    let after = ui.get_host_presentation();
    assert!(workbench_node(&after, "WorkbenchInputDropdown").popup_open);
}

#[test]
fn native_workbench_popup_menu_submenu_primary_press_keeps_menu_selection_path() {
    let ui = host_with_componentized_workbench_nodes();
    let clicked_items = Rc::new(RefCell::new(Vec::new()));
    let clicked_items_for_callback = clicked_items.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            clicked_items_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    assert_eq!(
        structured_menu_item(&menu, 4).action_id.as_str(),
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );
    let (x, y) = menu_item_row_point(&menu, 4);
    let result = ui.dispatch_native_primary_press_for_test(x, y);

    assert!(result.request_redraw());
    assert!(result.requires_frame_update());
    assert_eq!(
        clicked_items.borrow().as_slice(),
        [(
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_MENU_MORE_TOOLS_ACTION_ID.to_string()
        )]
    );
}

#[test]
fn native_workbench_dropdown_keyboard_moves_row_hover_and_enter_dispatches_option() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let selected_options = Rc::new(RefCell::new(Vec::new()));
    let selected_options_for_callback = selected_options.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_component_showcase_option_selected(move |control_id, action_id, option_id| {
            selected_options_for_callback.borrow_mut().push((
                control_id.to_string(),
                action_id.to_string(),
                option_id.to_string(),
            ));
        });

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(dropdown.popup_open);
    assert!(structured_option(&dropdown, 1).hovered);

    let move_result = ui.dispatch_native_popup_arrow_down_for_test();

    assert!(move_result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .as_str(),
        "workbench_option"
    );
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );
    let dropdown = workbench_node(&after, "WorkbenchInputDropdown");
    assert!(structured_option(&dropdown, 0).hovered);
    assert!(!structured_option(&dropdown, 1).hovered);

    let enter_result = ui.dispatch_native_popup_enter_for_test();

    assert!(enter_result.request_redraw());
    assert!(enter_result.requires_frame_update());
    assert_eq!(
        selected_options.borrow().as_slice(),
        [(
            "WorkbenchInputDropdown".to_string(),
            "component_lab.input_dropdown.select".to_string(),
            "dropdown".to_string()
        )]
    );
}

#[test]
fn native_workbench_dropdown_home_jumps_to_first_enabled_option() {
    let ui = host_with_open_workbench_dropdown_nodes();

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(dropdown.popup_open);
    assert!(structured_option(&dropdown, 1).hovered);

    let home_result = ui.dispatch_native_popup_home_for_test();

    assert!(home_result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .as_str(),
        "workbench_option"
    );
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );
    let dropdown = workbench_node(&after, "WorkbenchInputDropdown");
    assert!(structured_option(&dropdown, 0).hovered);
    assert!(!structured_option(&dropdown, 1).hovered);
}

#[test]
fn native_workbench_dropdown_text_search_jumps_to_matching_enabled_option() {
    let ui = host_with_open_workbench_dropdown_nodes();

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    assert!(dropdown.popup_open);
    assert!(structured_option(&dropdown, 1).hovered);

    let search_result = ui.dispatch_native_popup_text_for_test("d");

    assert!(search_result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );
    let dropdown = workbench_node(&after, "WorkbenchInputDropdown");
    assert!(structured_option(&dropdown, 0).hovered);
    assert!(!structured_option(&dropdown, 1).hovered);
}

#[test]
fn native_workbench_dropdown_escape_dispatches_popup_cancel() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let move_result = ui.dispatch_native_popup_arrow_down_for_test();

    assert!(move_result.request_redraw());
    assert_eq!(
        ui.get_pane_interaction_state()
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );

    let escape_result = ui.dispatch_native_popup_escape_for_test();

    assert!(escape_result.request_redraw());
    assert!(escape_result.requires_frame_update());
    assert_eq!(
        cancelled.borrow().as_slice(),
        [(
            "WorkbenchInputDropdown".to_string(),
            WORKBENCH_POPUP_CANCEL_ACTION_ID.to_string()
        )]
    );
    let interaction = ui.get_pane_interaction_state();
    assert!(interaction.hovered_template_control_id.is_empty());
    assert!(interaction.hovered_template_action_id.is_empty());
    assert!(interaction.hovered_template_value_text.is_empty());
}

#[test]
fn native_workbench_dropdown_outside_primary_press_dispatches_popup_cancel() {
    let ui = host_with_open_workbench_dropdown_nodes();
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let dropdown = workbench_node(&before, "WorkbenchInputDropdown");
    let (x, y) = dropdown_option_row_point(&dropdown, 0);
    let move_result = ui.dispatch_native_pointer_move_for_test(x, y);

    assert!(move_result.request_redraw());
    assert_eq!(
        ui.get_pane_interaction_state()
            .hovered_template_value_text
            .as_str(),
        "dropdown"
    );

    let outside_result = ui.dispatch_native_primary_press_for_test(
        OUTSIDE_WORKBENCH_POPUP_X,
        OUTSIDE_WORKBENCH_POPUP_Y,
    );

    assert!(outside_result.request_redraw());
    assert!(outside_result.requires_frame_update());
    assert_eq!(
        cancelled.borrow().as_slice(),
        [(
            "WorkbenchInputDropdown".to_string(),
            WORKBENCH_POPUP_CANCEL_ACTION_ID.to_string()
        )]
    );
    let interaction = ui.get_pane_interaction_state();
    assert!(interaction.hovered_template_control_id.is_empty());
    assert!(interaction.hovered_template_action_id.is_empty());
    assert!(interaction.hovered_template_value_text.is_empty());
}

#[test]
fn native_workbench_popup_menu_keyboard_moves_row_hover_and_enter_dispatches_menu_item() {
    let ui = host_with_componentized_workbench_nodes();
    let clicked_items = Rc::new(RefCell::new(Vec::new()));
    let clicked_items_for_callback = clicked_items.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            clicked_items_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    assert!(menu.popup_open);
    assert!(structured_menu_item(&menu, 3).hovered);

    let move_result = ui.dispatch_native_popup_arrow_down_for_test();

    assert!(move_result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .as_str(),
        "workbench_menu_item"
    );
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );
    let menu = workbench_node(&after, "WorkbenchPopupMenu");
    assert!(structured_menu_item(&menu, 4).hovered);
    assert!(!structured_menu_item(&menu, 3).hovered);

    let enter_result = ui.dispatch_native_popup_enter_for_test();

    assert!(enter_result.request_redraw());
    assert!(enter_result.requires_frame_update());
    assert_eq!(
        clicked_items.borrow().as_slice(),
        [(
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_MENU_MORE_TOOLS_ACTION_ID.to_string()
        )]
    );
}

#[test]
fn native_workbench_popup_menu_home_end_jump_to_boundary_rows() {
    let ui = host_with_componentized_workbench_nodes();

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    assert!(menu.popup_open);
    assert!(structured_menu_item(&menu, 3).hovered);

    let home_result = ui.dispatch_native_popup_home_for_test();

    assert!(home_result.request_redraw());
    let after_home = ui.get_host_presentation();
    assert_eq!(
        after_home
            .pane_interaction_state
            .hovered_template_dispatch_kind
            .as_str(),
        "workbench_menu_item"
    );
    assert_eq!(
        after_home
            .pane_interaction_state
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_NEW_ACTION_ID
    );
    let menu = workbench_node(&after_home, "WorkbenchPopupMenu");
    assert!(structured_menu_item(&menu, 0).hovered);
    assert!(!structured_menu_item(&menu, 3).hovered);

    let end_result = ui.dispatch_native_popup_end_for_test();

    assert!(end_result.request_redraw());
    let after_end = ui.get_host_presentation();
    assert_eq!(
        after_end
            .pane_interaction_state
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );
    let menu = workbench_node(&after_end, "WorkbenchPopupMenu");
    assert!(!structured_menu_item(&menu, 0).hovered);
    assert!(structured_menu_item(&menu, 4).hovered);
}

#[test]
fn native_workbench_popup_menu_text_search_jumps_to_matching_item() {
    let ui = host_with_componentized_workbench_nodes();

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    assert!(menu.popup_open);
    assert!(structured_menu_item(&menu, 3).hovered);

    let search_result = ui.dispatch_native_popup_text_for_test("m");

    assert!(search_result.request_redraw());
    let after = ui.get_host_presentation();
    assert_eq!(
        after
            .pane_interaction_state
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );
    let menu = workbench_node(&after, "WorkbenchPopupMenu");
    assert!(!structured_menu_item(&menu, 3).hovered);
    assert!(structured_menu_item(&menu, 4).hovered);
}

#[test]
fn native_workbench_popup_menu_escape_dispatches_popup_cancel() {
    let ui = host_with_componentized_workbench_nodes();
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let move_result = ui.dispatch_native_popup_arrow_down_for_test();

    assert!(move_result.request_redraw());
    assert_eq!(
        ui.get_pane_interaction_state()
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_MORE_TOOLS_ACTION_ID
    );

    let escape_result = ui.dispatch_native_popup_escape_for_test();

    assert!(escape_result.request_redraw());
    assert!(escape_result.requires_frame_update());
    assert_eq!(
        cancelled.borrow().as_slice(),
        [(
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_POPUP_CANCEL_ACTION_ID.to_string()
        )]
    );
    let interaction = ui.get_pane_interaction_state();
    assert!(interaction.hovered_template_control_id.is_empty());
    assert!(interaction.hovered_template_action_id.is_empty());
    assert!(interaction.hovered_template_value_text.is_empty());
}

#[test]
fn native_workbench_popup_menu_outside_primary_press_dispatches_popup_cancel() {
    let ui = host_with_componentized_workbench_nodes();
    let cancelled = Rc::new(RefCell::new(Vec::new()));
    let cancelled_for_callback = cancelled.clone();
    ui.global::<PaneSurfaceHostContext>()
        .on_surface_control_clicked(move |control_id, action_id| {
            cancelled_for_callback
                .borrow_mut()
                .push((control_id.to_string(), action_id.to_string()));
        });

    let before = ui.get_host_presentation();
    let menu = workbench_node(&before, "WorkbenchPopupMenu");
    let (x, y) = menu_item_row_point(&menu, 0);
    let move_result = ui.dispatch_native_pointer_move_for_test(x, y);

    assert!(move_result.request_redraw());
    assert_eq!(
        ui.get_pane_interaction_state()
            .hovered_template_action_id
            .as_str(),
        WORKBENCH_MENU_NEW_ACTION_ID
    );

    let outside_result = ui.dispatch_native_primary_press_for_test(
        OUTSIDE_WORKBENCH_POPUP_X,
        OUTSIDE_WORKBENCH_POPUP_Y,
    );

    assert!(outside_result.request_redraw());
    assert!(outside_result.requires_frame_update());
    assert_eq!(
        cancelled.borrow().as_slice(),
        [(
            "WorkbenchPopupMenu".to_string(),
            WORKBENCH_POPUP_CANCEL_ACTION_ID.to_string()
        )]
    );
    let interaction = ui.get_pane_interaction_state();
    assert!(interaction.hovered_template_control_id.is_empty());
    assert!(interaction.hovered_template_action_id.is_empty());
    assert!(interaction.hovered_template_value_text.is_empty());
}

fn host_with_componentized_workbench_nodes() -> UiHostWindow {
    let bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    host_with_workbench_bridge(bridge)
}

fn host_with_selected_workbench_module_nodes(module_control_id: &str) -> UiHostWindow {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state(module_control_id, UiEventKind::Click)
        .expect("module tab click should dispatch")
        .expect("module tab should have a selection binding");
    host_with_workbench_bridge(bridge)
}

fn host_with_open_workbench_dropdown_nodes() -> UiHostWindow {
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(
        WORKBENCH_REFERENCE_WIDTH as f32,
        WORKBENCH_REFERENCE_HEIGHT as f32,
    ))
    .expect("componentized workbench template should project");
    bridge
        .dispatch_control_state("WorkbenchInputDropdown", UiEventKind::Click)
        .expect("dropdown click should dispatch")
        .expect("dropdown should have an open binding");
    host_with_workbench_bridge(bridge)
}

fn host_with_workbench_bridge(bridge: BuiltinWorkbenchWindowTemplateSurfaceBridge) -> UiHostWindow {
    let mut presentation = HostWindowPresentationData::default();
    presentation.workbench_window_nodes =
        to_host_contract_workbench_window_nodes(Some(bridge.host_projection()));
    let ui = UiHostWindow::new().expect("host window should construct for workbench hover test");
    ui.set_host_presentation(presentation);
    ui
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

fn structured_option(node: &TemplatePaneNodeData, row: usize) -> TemplatePaneOptionData {
    node.structured_options
        .row_data(row)
        .unwrap_or_else(|| panic!("structured option row {row} should exist"))
}

fn structured_menu_item(node: &TemplatePaneNodeData, row: usize) -> TemplatePaneMenuItemData {
    node.structured_menu_items
        .row_data(row)
        .unwrap_or_else(|| panic!("structured menu item row {row} should exist"))
}

fn dropdown_option_row_point(node: &TemplatePaneNodeData, row: usize) -> (f32, f32) {
    let row_height = node.frame.height.max(24.0);
    (
        node.frame.x + 8.0,
        node.frame.y + node.frame.height + 4.0 + row as f32 * row_height + row_height * 0.5,
    )
}

fn menu_item_row_point(node: &TemplatePaneNodeData, row: usize) -> (f32, f32) {
    let row_count = node.structured_menu_items.row_count().max(1);
    let row_height = (node.frame.height / row_count as f32).max(24.0);
    (
        node.frame.x + 8.0,
        node.frame.y + row as f32 * row_height + row_height * 0.5,
    )
}

fn node_center(node: &TemplatePaneNodeData) -> (f32, f32) {
    (
        node.frame.x + node.frame.width * 0.5,
        node.frame.y + node.frame.height * 0.5,
    )
}

fn node_right_center(node: &TemplatePaneNodeData) -> (f32, f32) {
    (
        node.frame.x + (node.frame.width - 16.0).max(1.0),
        node.frame.y + node.frame.height * 0.5,
    )
}

fn changed_pixel_count_in_frame(before: &[u8], after: &[u8], frame: UiFrame) -> usize {
    frame_points(frame)
        .filter(|(x, y)| pixel(before, *x, *y) != pixel(after, *x, *y))
        .count()
}

fn first_non_black_pixel_in_frame(bytes: &[u8], frame: UiFrame) -> Option<[u8; 4]> {
    frame_points(frame)
        .map(|(x, y)| pixel(bytes, x, y))
        .find(|pixel| *pixel != [0, 0, 0, 255])
}

fn frame_points(frame: UiFrame) -> impl Iterator<Item = (u32, u32)> {
    let start_x = frame.x.floor().max(0.0) as u32;
    let start_y = frame.y.floor().max(0.0) as u32;
    let end_x = (frame.x + frame.width)
        .ceil()
        .min(WORKBENCH_REFERENCE_WIDTH as f32) as u32;
    let end_y = (frame.y + frame.height)
        .ceil()
        .min(WORKBENCH_REFERENCE_HEIGHT as f32) as u32;
    (start_y..end_y).flat_map(move |y| (start_x..end_x).map(move |x| (x, y)))
}

fn pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * WORKBENCH_REFERENCE_WIDTH + x) * 4) as usize;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}

fn contains_at_least_distinct_non_black_pixels(bytes: &[u8], minimum: usize) -> bool {
    let mut distinct = Vec::<[u8; 4]>::new();
    for chunk in bytes.chunks_exact(4) {
        let pixel = [chunk[0], chunk[1], chunk[2], chunk[3]];
        if pixel == [0, 0, 0, 255] || distinct.contains(&pixel) {
            continue;
        }
        distinct.push(pixel);
        if distinct.len() >= minimum {
            return true;
        }
    }
    false
}

fn maybe_write_workbench_preview_png(bytes: &[u8]) {
    if std::env::var_os(WORKBENCH_PREVIEW_CAPTURE_ENV).is_none() {
        return;
    }

    let path = std::env::var_os(WORKBENCH_PREVIEW_CAPTURE_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from("target")
                .join("editor-workbench-visual-check")
                .join("editor-workbench-native-1672x941.png")
        });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("workbench preview output directory should exist");
    }

    image::save_buffer_with_format(
        &path,
        bytes,
        WORKBENCH_REFERENCE_WIDTH,
        WORKBENCH_REFERENCE_HEIGHT,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )
    .expect("workbench preview PNG should be written");
}
