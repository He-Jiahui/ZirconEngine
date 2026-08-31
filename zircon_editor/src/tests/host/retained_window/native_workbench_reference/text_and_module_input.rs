use super::support::*;

#[test]
fn native_workbench_pointer_move_queries_the_hit_index_once() {
    let ui = host_with_componentized_workbench_nodes();
    let generation = ui.get_host_presentation_generation();
    let tool = workbench_node(generation.structure(), "WorkbenchToolMove");
    let (x, y) = node_center(&tool);
    let query_count_before = generation.workbench_hit_index().query_count_for_test();

    let _ = ui.dispatch_native_pointer_move_for_test(x, y);

    assert_eq!(
        generation.workbench_hit_index().query_count_for_test() - query_count_before,
        1,
        "one native pointer move must share one workbench hit query across popup and normal routing"
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
        Some("Playtest queued   activation phase   GA_DashAttack")
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
