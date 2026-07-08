type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 scene-tests ECS systems guard child-owner split",
        &[
            "runtime_15_scene_tests_ecs_systems_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/scene_tests/ecs_systems.rs",
            "runtime_15_scene_tests_ecs_systems_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 Net HTTP policy guard child-owner split",
        &[
            "runtime_15_net_http_policy_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/net.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/net/http1_client_policy.rs",
            "runtime_15_net_http_policy_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 Hub raw-text policy guard child-owner split",
        &[
            "runtime_15_hub_raw_text_policy_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/hub.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/hub/raw_text_policy.rs",
            "runtime_15_hub_raw_text_policy_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 input mouse-wheel line-delta guard child-owner split",
        &[
            "runtime_15_input_mouse_wheel_line_delta_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/input.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/input/mouse_wheel_line_delta.rs",
            "runtime_15_input_mouse_wheel_line_delta_guard_is_child_owner",
        ],
    ),
    (
        "Runtime 15 M3 UI platform-input guard child-owner split",
        &[
            "runtime_15_ui_platform_input_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/ui.rs",
            "tests/runtime_absorption/naming_boundary/runtime_15_m2/ui/platform_input.rs",
            "runtime_15_ui_platform_input_guards_are_child_owner",
        ],
    ),
];
