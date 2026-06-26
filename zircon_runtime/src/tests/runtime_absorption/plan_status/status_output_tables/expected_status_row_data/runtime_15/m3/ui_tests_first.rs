use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 15 M3 UI architecture test folder split",
        &[
            "runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/ui_architecture.rs",
            "tests/runtime_absorption/ui_architecture/architecture_boundaries.rs",
            "tests/runtime_absorption/ui_architecture/legacy_renames.rs",
            "runtime_15_ui_architecture_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI v2 asset test folder split",
        &[
            "runtime_15_ui_v2_asset_tests_folder_split_static_passed_cargo_lock_blocked",
            "ui/tests/v2_asset.rs",
            "ui/tests/v2_asset/style_runtime.rs",
            "runtime_15_ui_v2_asset_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core test folder split",
        &[
            "runtime_15_ui_shared_core_tests_folder_split_static_passed_cargo_lock_blocked",
            "ui/tests/shared_core.rs",
            "ui/tests/shared_core/layout_surface.rs",
            "runtime_15_ui_shared_core_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core guard child-owner split",
        &[
            "runtime_15_ui_shared_core_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/ui_shared_core.rs",
            "structure_convention/test_file_budget/ui_shared_core/layout_surface.rs",
            "runtime_15_ui_shared_core_guard_child_owners_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core input visibility child folder split",
        &[
            "runtime_15_ui_shared_core_input_visibility_child_folder_split_static_passed_cargo_deferred",
            "ui/tests/shared_core/input_visibility.rs",
            "ui/tests/shared_core/input_visibility/hit_visibility.rs",
            "ui/tests/shared_core/input_visibility/pointer_routes.rs",
            "runtime_15_ui_shared_core_input_visibility_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core scroll mutation child folder split",
        &[
            "runtime_15_ui_shared_core_scroll_mutation_child_folder_split_static_passed_cargo_deferred",
            "ui/tests/shared_core/scroll_mutation.rs",
            "ui/tests/shared_core/scroll_mutation/property_mutation.rs",
            "ui/tests/shared_core/scroll_mutation/virtual_scroll.rs",
            "runtime_15_ui_shared_core_scroll_mutation_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI shared core layout surface child folder split",
        &[
            "runtime_15_ui_shared_core_layout_surface_child_folder_split_static_passed_cargo_deferred",
            "ui/tests/shared_core/layout_surface.rs",
            "ui/tests/shared_core/layout_surface/layout_measurement.rs",
            "ui/tests/shared_core/layout_surface/render_extract.rs",
            "runtime_15_ui_shared_core_layout_surface_children_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI accessibility test folder split",
        &[
            "runtime_15_ui_accessibility_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/accessibility.rs",
            "ui/tests/accessibility/extraction.rs",
            "ui/tests/accessibility/value_actions.rs",
            "runtime_15_ui_accessibility_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI accessibility widget actions test folder split",
        &[
            "runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/accessibility_widget_actions.rs",
            "ui/tests/accessibility_widget_actions/popup_actions.rs",
            "runtime_15_ui_accessibility_widget_actions_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI layout slots test folder split",
        &[
            "runtime_15_ui_layout_slots_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/layout_slots.rs",
            "ui/tests/layout_slots/linear_free.rs",
            "runtime_15_ui_layout_slots_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI surface-frame authority test folder split",
        &[
            "runtime_15_ui_surface_frame_authority_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/surface_frame_authority.rs",
            "ui/tests/surface_frame_authority/arranged_authority.rs",
            "ui/tests/surface_frame_authority/taffy_wrap_grid.rs",
            "runtime_15_ui_surface_frame_authority_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI surface dirty domains test folder split",
        &[
            "runtime_15_ui_surface_dirty_domains_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/surface_dirty_domains.rs",
            "ui/tests/surface_dirty_domains/rebuild_domains.rs",
            "ui/tests/surface_dirty_domains/incremental_layout.rs",
            "runtime_15_ui_surface_dirty_domains_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 UI material layout test folder split",
        &[
            "runtime_15_ui_material_layout_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/material_layout.rs",
            "ui/tests/material_layout/button_icon_metrics.rs",
            "ui/tests/material_layout/field_values.rs",
            "runtime_15_ui_material_layout_tests_are_folder_backed",
        ],
    ),
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
