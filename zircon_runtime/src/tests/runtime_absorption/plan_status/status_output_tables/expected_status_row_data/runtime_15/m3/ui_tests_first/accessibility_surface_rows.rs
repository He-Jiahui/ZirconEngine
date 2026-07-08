type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
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
];
