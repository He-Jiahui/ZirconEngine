use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 09 navigation legacy reply rename",
        [
            "runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending",
            "`routed_reply`",
            "`ui_legacy_hits=153`",
            "standalone rustc 6/6",
        ],
    ),
    (
        "Runtime 09 pointer legacy reply rename",
        [
            "runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending",
            "`routed_result`",
            "`ui_legacy_hits=104`",
            "standalone rustc 10/10",
        ],
    ),
    (
        "Runtime 09 pointer capture fallback rename",
        [
            "runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending",
            "`has_pointer_capture_or_unindexed_fallback_for_owner`",
            "`ui_legacy_hits=102`",
            "standalone rustc 11/11",
        ],
    ),
    (
        "Runtime 09 table row label fallback rename",
        [
            "runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending",
            "`split_row_label_table_text`",
            "`ui_legacy_hits=100`",
            "standalone rustc 12/12",
        ],
    ),
    (
        "Runtime 09 template component-name fallback rename",
        [
            "runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending",
            "`component_name_interaction_fallback`",
            "`ui_legacy_hits=95`",
            "standalone rustc 13/13",
        ],
    ),
    (
        "Runtime 09 property visibility flag rename",
        [
            "runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending",
            "`state_visible_flag`",
            "`ui_legacy_hits=92`",
            "standalone rustc 14/14",
        ],
    ),
    (
        "Runtime 09 responsive MUI visibility flag rename",
        [
            "runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending",
            "`state_visible_flag`",
            "`ui_legacy_hits=84`",
            "standalone rustc 15/15",
        ],
    ),
    (
        "Runtime 09 accessibility open-state fallback rename",
        [
            "runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending",
            "`fallback_properties`",
            "`ui_legacy_hits=76`",
            "standalone rustc 16/16",
        ],
    ),
    (
        "Runtime 09 layout engine backend name cutover",
        [
            "runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending",
            "UiLayoutEngineBackend::Zircon",
            "`zircon_selected_count`",
            "standalone rustc 17/17",
        ],
    ),
    (
        "Runtime 09 surface default interaction fallback rename",
        [
            "runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending",
            "`default_open_boolean_value`",
            "`ui_legacy_hits=54`",
            "standalone rustc 18/18",
        ],
    ),
];
