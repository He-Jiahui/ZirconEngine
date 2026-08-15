#[test]
fn runtime_12_action_mapping_keeps_ui_filtered_evaluation_path() {
    let runtime_12_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let input_doc = include_str!("../../../../../docs/zircon_runtime/input/input_state.md");
    let action = include_str!("../../../core/framework/input/input_action.rs");
    let action_context = include_str!("../../../core/framework/input/input_action_context.rs");
    let binding = include_str!("../../../core/framework/input/input_binding.rs");
    let action_map = include_str!("../../../core/framework/input/input_action_map.rs");
    let action_manager = include_str!("../../../core/framework/input/input_action_manager.rs");
    let action_state = include_str!("../../../core/framework/input/input_action_state.rs");
    let config = include_str!("../../../input/module/config.rs");
    let descriptor = include_str!("../../../input/module/descriptor.rs");
    let evaluator = include_str!("../../../input/runtime/action_evaluator.rs");
    let evaluator_generation =
        include_str!("../../../input/runtime/action_evaluator/generation.rs");
    let evaluator_workspace = include_str!("../../../input/runtime/action_evaluator/workspace.rs");
    let consumed_input_index =
        include_str!("../../../input/runtime/action_evaluator/consumed_input_index.rs");
    let frame_axis_index =
        include_str!("../../../input/runtime/action_evaluator/frame_axis_index.rs");
    let default_action_manager =
        include_str!("../../../input/runtime/default_input_action_manager.rs");
    let core_manager = include_str!("../../../core/manager/mod.rs");
    let core_manager_resolver = include_str!("../../../core/manager/resolver.rs");
    let core_manager_service_names = include_str!("../../../core/manager/service_names.rs");
    let action_tests = include_str!("../../../input/tests/action_mapping.rs");
    let action_axis_transition_tests =
        include_str!("../../../input/tests/action_axis_transitions.rs");

    for required_contract_anchor in [
        "pub struct InputAction",
        "pub struct InputActionContext",
        "pub trait InputActionManager",
        "pub struct InputBinding",
        "pub struct InputAxisBinding",
        "pub enum InputAxisDirection",
        "pub struct InputActionMap",
        "pub struct InputActionState",
        "pub struct InputConfig",
        "Serialize, Deserialize",
    ] {
        assert!(
            action.contains(required_contract_anchor)
                || action_context.contains(required_contract_anchor)
                || action_manager.contains(required_contract_anchor)
                || binding.contains(required_contract_anchor)
                || action_map.contains(required_contract_anchor)
                || action_state.contains(required_contract_anchor)
                || config.contains(required_contract_anchor),
            "Runtime 12 action contract should retain `{required_contract_anchor}`"
        );
    }

    for required_evaluator_anchor in [
        "pub struct InputActionEvaluator",
        "DefaultInputActionManager",
        "set_action_map",
        "evaluate_with_consumed_buttons",
        "evaluate_with_consumed_input",
        "evaluate_with_active_contexts",
        "evaluate_with_active_contexts_and_consumed_buttons",
        "evaluate_with_active_contexts_and_consumed_input",
        "ActionEvaluationGeneration::from_action_map",
        "workspace.consumed_inputs()",
        "button_is_consumed",
        "axis_is_consumed",
        "action_context_is_active",
        "InputActionState::from_sets_and_values",
        "evaluate_binding_axes",
        "BindingAxisEvaluation",
        "dominant_action_value",
        "compiled.binding_indices(generation)",
        "workspace.frame_axes()",
        "evaluate_while_manager_locked",
        "evaluation_consumed_input_source_visit_count",
        "binary_search_by_key",
    ] {
        assert!(
            evaluator.contains(required_evaluator_anchor)
                || evaluator_generation.contains(required_evaluator_anchor)
                || evaluator_workspace.contains(required_evaluator_anchor)
                || consumed_input_index.contains(required_evaluator_anchor)
                || frame_axis_index.contains(required_evaluator_anchor)
                || default_action_manager.contains(required_evaluator_anchor),
            "Runtime 12 action evaluator should retain `{required_evaluator_anchor}`"
        );
    }

    assert_eq!(
        evaluator
            .matches("let axis = evaluate_binding_axes(")
            .count(),
        1,
        "Runtime 12 should evaluate each binding's axis value and transition state through one owner call"
    );
    assert!(
        evaluator.contains("generation.context_enabled(context_slot)"),
        "Runtime 12 should resolve context enabled-state from the compiled generation"
    );
    for stale_hot_path_anchor in [
        "self.action_map.context_enabled(context)",
        "fn binding_axis_value(",
        "fn binding_axis_transition(",
        "binding_axis_value(&frame_axes",
        "binding_axis_transition(&frame_axes",
        "mod binding_index;",
        "ActionBindingIndex",
        "binding_index.indices_for_action",
        "FrameAxisIndex::from_frame",
        "consumed_buttons.contains(button)",
        "consumed_axes.contains(&axis_input)",
    ] {
        assert!(
            !evaluator.contains(stale_hot_path_anchor),
            "Runtime 12 action evaluation must not restore stale hot-path anchor `{stale_hot_path_anchor}`"
        );
    }

    for required_registration_anchor in [
        "module_descriptor_with_config",
        "INPUT_ACTION_MANAGER_NAME",
        "RegisteredManagerService::<dyn InputActionManager>",
        "input_action_manager_handle",
        "InputModule.Manager.InputActionManager",
    ] {
        assert!(
            descriptor.contains(required_registration_anchor)
                || core_manager.contains(required_registration_anchor)
                || core_manager_resolver.contains(required_registration_anchor)
                || core_manager_service_names.contains(required_registration_anchor),
            "Runtime 12 action manager registration should retain `{required_registration_anchor}`"
        );
    }

    for required_test_anchor in [
        "action_map_resolves_chords_and_reports_just_activated",
        "replacing_action_map_rebuilds_bindings_automatically",
        "action_contexts_filter_gameplay_and_menu_maps_without_rebinding",
        "gamepad_axis_binding_reports_continuous_action_value",
        "consumed_gamepad_axis_does_not_activate_gameplay_action",
        "gamepad_axis_action_reports_deadzone_transition_edges",
        "input_config_builds_action_evaluator_from_serialized_action_map",
        "input_action_manager_resolves_from_runtime_module_descriptor",
        "evaluate_with_consumed_buttons",
        "evaluate_with_consumed_input",
        "evaluate_with_active_contexts",
        "input_actions_handle",
        "value(\"gameplay.move_x\")",
        "action_evaluator()",
        "action_manager",
        "clear_bindings",
        "action_evaluator_indexes_10_100_1000_and_10000_bindings_once",
        "action_evaluator_indexes_axis_frame_sources_once_for_10_100_1000_and_10000_bindings",
        "action_evaluator_records_generation_builds_and_distinct_projected_actions",
        "action_evaluator_reuses_consumed_button_index_at_10000_bindings",
        "action_evaluator_reuses_consumed_axis_index_at_10000_bindings",
    ] {
        assert!(
            action_tests.contains(required_test_anchor)
                || action_axis_transition_tests.contains(required_test_anchor)
                || config.contains(required_test_anchor),
            "Runtime 12 action mapping tests/config should retain `{required_test_anchor}`"
        );
    }

    for required_plan_anchor in [
        "UI surface/pointer capture/popup/focus 优先",
        "玩法/action mapping 只消费 UI 未处理",
        "action_contract_static_passed_cargo_pending",
        "action_evaluator_static_passed_cargo_pending",
        "action_context_static_passed_cargo_pending",
        "action_axis_value_static_passed_cargo_deferred",
        "action_config_static_passed_cargo_deferred",
        "action_manager_registration_static_passed_cargo_deferred",
    ] {
        assert!(
            runtime_12_plan.contains(required_plan_anchor)
                || runtime_index.contains(required_plan_anchor)
                || input_doc.contains(required_plan_anchor),
            "Runtime 12 docs/index should retain action arbitration anchor `{required_plan_anchor}`"
        );
    }
}
