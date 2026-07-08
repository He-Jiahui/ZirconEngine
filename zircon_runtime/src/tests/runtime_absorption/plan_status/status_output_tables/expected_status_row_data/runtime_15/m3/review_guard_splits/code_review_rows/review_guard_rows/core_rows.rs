use super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 code review findings test folder split",
        &[
            "runtime_15_code_review_findings_tests_folder_split_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings.rs",
            "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence.rs",
            "tests/runtime_absorption/code_review_findings/p0_robustness.rs",
            "review_priority_recommendation_tracks_current_remaining_work",
            "ds7_static_plugin_manifest_generation_parity_review_synced_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d6_runtime_plugin_id.rs",
            "review_d6_runtime_plugin_id_accepts_external_string_keys",
            "d6_runtime_plugin_id_open_string_newtype_review_static_passed_cargo_deferred",
            "review_d13_importer_runtime_exports_use_sdk_macro",
            "review_d13_importer_runtime_manifests_use_sdk_builder",
            "tests/runtime_absorption/code_review_findings/render_structure.rs",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
            "runtime_15_code_review_findings_tests_are_folder_backed",
        ],
    ),
    (
        "Runtime 15 M3 code-review standalone harness current-path sync",
        &[
            "runtime_15_code_review_standalone_harness_current_path_sync_static_passed_cargo_deferred",
            "tests/runtime_absorption/code_review_findings.rs",
            "#[path = \"code_review_findings/f12_dead_code.rs\"]",
            "#[path = \"code_review_findings/typed_error_convergence/mod.rs\"]",
            "tests/runtime_absorption/code_review_findings/f8_api_convergence/descriptor_builder/test_fixtures.rs",
            "review_f8_runtime_plugin_descriptor_test_fixtures_use_builder",
            "builder call count 59",
            "tests/runtime_absorption/code_review_findings/f12_dead_code.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs",
            "review_f12_runtime_production_dead_code_suppression_is_globally_gated",
            "runtime_15_code_review_findings_tests_are_folder_backed",
            "Cargo gate deferred",
        ],
    ),
];
