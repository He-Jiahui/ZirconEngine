type Slice = super::Slice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    (
        "Runtime 15 M3 D13 importer top-row closed status sync",
        &[
            "d13_importer_top_row_closed_status_static_passed_cargo_deferred",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "tests/runtime_absorption/code_review_findings/plugin_importer_dx/d13_importer_sdk.rs",
            "review_d13_importer_manifest_parity_guard_lives_in_sdk_builder",
            "| D13 | importer 系列",
            "| M3 / closed |",
            "d13_importer_runtime_export_macro_convergence_static_passed_cargo_deferred",
            "d13_importer_runtime_manifest_builder_convergence_static_passed_cargo_deferred",
            "d13_importer_manifest_parity_guard_static_passed_cargo_deferred",
            "cargo deferred",
        ],
    ),
    (
        "Runtime 15 M3 D-S8/D3 native fixture top-row closed status sync",
        &[
            "ds8_d3_native_fixture_top_row_closed_status_static_passed_cargo_deferred",
            "docs/plans/engine-code-review-findings-2026-06.md",
            "tests/runtime_absorption/code_review_findings/p0_robustness.rs",
            "review_ds8_d3_native_fixture_uses_sdk_macro_and_single_manifest",
            "| D-S8 | native 插件 ABI v3 样板已由 plugin SDK macro 承接",
            "| D3 | native manifest 双写已由 plugin.toml 单源闭合",
            "| Plugins 13 M2 + Plugins 12 / closed |",
            "| Plugins 13 M1 + Plugins 12 / closed |",
            "native_dynamic_fixture_validation_plugin_review_passed_unused_import_warning_fixed",
            "native_dynamic_fixture_importer_manifest_self_description_static_passed_cargo_deferred",
            "cargo deferred",
        ],
    ),
];
