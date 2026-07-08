type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 UI template test folder split",
        &[
            "runtime_15_ui_template_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/template.rs",
            "ui/tests/template/interaction_bindings.rs",
            "ui/tests/template/slot_contracts.rs",
            "runtime_15_ui_template_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI event routing test folder split",
        &[
            "runtime_15_ui_event_routing_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/event_routing.rs",
            "ui/tests/event_routing/pointer_state.rs",
            "ui/tests/event_routing/shared_input.rs",
            "runtime_15_ui_event_routing_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI runtime input reply routes test folder split",
        &[
            "runtime_15_ui_runtime_input_reply_routes_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_input_reply_routes.rs",
            "ui/tests/runtime_input_reply_routes/route_trace_routes.rs",
            "ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs",
            "runtime_15_ui_runtime_input_reply_routes_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI runtime input reply route child folder split",
        &[
            "runtime_15_ui_runtime_input_reply_route_children_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs",
            "ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs",
            "ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs",
            "runtime_15_ui_runtime_input_reply_route_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI runtime input reply table pointer route folder split",
        &[
            "runtime_15_ui_runtime_input_reply_table_pointer_routes_folder_split_static_passed_cargo_deferred",
            "ui/tests/runtime_input_reply_routes/table_pointer_routes.rs",
            "ui/tests/runtime_input_reply_routes/table_pointer_routes/resize.rs",
            "ui/tests/runtime_input_reply_routes/table_pointer_routes/virtualization.rs",
            "runtime_15_ui_runtime_input_reply_table_pointer_routes_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI runtime input reply route guard child-owner split",
        &[
            "runtime_15_ui_runtime_input_reply_route_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs",
            "structure_convention/test_file_budget/ui_runtime_input_reply_routes/route_children.rs",
            "runtime_15_ui_runtime_input_reply_route_guard_child_owners_are_folder_backed",
        ],
    ),
];
