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
        "consumed_buttons.contains(button)",
        "consumed_axes.contains",
        "action_context_is_active",
        "InputActionState::from_sets_and_values",
        "binding_axis_consumed",
        "binding_axis_value",
        "binding_axis_transition",
        "dominant_action_value",
    ] {
        assert!(
            evaluator.contains(required_evaluator_anchor)
                || default_action_manager.contains(required_evaluator_anchor),
            "Runtime 12 action evaluator should retain `{required_evaluator_anchor}`"
        );
    }

    for required_registration_anchor in [
        "module_descriptor_with_config",
        "INPUT_ACTION_MANAGER_NAME",
        "InputActionManagerHandle",
        "resolve_input_action_manager",
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
        "rebinding_action_does_not_require_recompilation",
        "action_contexts_filter_gameplay_and_menu_maps_without_rebinding",
        "gamepad_axis_binding_reports_continuous_action_value",
        "consumed_gamepad_axis_does_not_activate_gameplay_action",
        "gamepad_axis_action_reports_deadzone_transition_edges",
        "input_config_builds_action_evaluator_from_serialized_action_map",
        "input_action_manager_resolves_from_runtime_module_descriptor",
        "evaluate_with_consumed_buttons",
        "evaluate_with_consumed_input",
        "evaluate_with_active_contexts",
        "resolve_input_action_manager",
        "value(\"gameplay.move_x\")",
        "action_evaluator()",
        "action_manager",
        "clear_bindings",
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
