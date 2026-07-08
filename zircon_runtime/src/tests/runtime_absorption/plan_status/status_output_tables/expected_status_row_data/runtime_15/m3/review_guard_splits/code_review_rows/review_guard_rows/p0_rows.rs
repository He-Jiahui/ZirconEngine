use super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 P0 robustness review guard child-owner split",
        &[
            "runtime_15_p0_robustness_review_guard_child_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/p0_robustness.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_host_callbacks.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/lock_poison.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/render_submit.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/priority_recommendation.rs",
            "review_f1_native_host_callbacks_catch_unwind_before_crossing_ffi",
            "review_f2_scene_eventbus_locks_recover_after_poison",
            "review_f4_render_submit_capability_gaps_return_typed_errors",
            "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
            "review_d13_native_fixture_importer_is_manifest_described",
            "review_priority_recommendation_tracks_current_remaining_work",
            "runtime_15_p0_robustness_review_guards_are_child_owners",
            "Cargo gate deferred",
        ],
    ),
    (
        "Runtime 15 M3 P0 native fixture review guard leaf-owner split",
        &[
            "runtime_15_p0_native_fixture_review_guard_leaf_owner_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/sdk_macro_manifest.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness/native_fixture/importer_manifest.rs",
            "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
            "review_d13_native_fixture_importer_is_manifest_described",
            "runtime_15_p0_native_fixture_review_guards_are_leaf_owners",
            "runtime_15_p0_robustness_review_guards_are_child_owners",
            "runtime_15_code_review_findings_tests_are_folder_backed",
            "Cargo gate deferred",
        ],
    ),
];
