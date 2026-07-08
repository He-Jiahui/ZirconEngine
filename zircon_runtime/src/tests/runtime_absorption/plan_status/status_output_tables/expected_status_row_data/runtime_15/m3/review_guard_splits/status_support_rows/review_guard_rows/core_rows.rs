use super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 review guard status row-data child-owner split",
        &[
            "runtime_15_review_guard_status_row_data_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits.rs",
            "runtime_15_status_output_m3_review_guard_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 review guard row-data topic child-owner split",
        &[
            "runtime_15_review_guard_row_data_topic_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/status_support_rows.rs",
            "runtime_15_status_output_m3_review_guard_row_data_is_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 review-guard direct-assertion row-data child-owner split",
        &[
            "runtime_15_review_guard_direct_assertion_row_data_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/direct_assertion_rows.rs",
            "runtime_15_status_output_review_guard_direct_assertion_rows_are_child_owner",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 review-guard typed-error row-data child split",
        &[
            "runtime_15_review_guard_typed_error_row_data_child_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/native_plugin_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/runtime_surface_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/typed_error_rows/asset_shader_rows.rs",
            "runtime_15_review_guard_typed_error_rows_are_child_owned",
            "Cargo gate deferred",
        ],
    ),
];
