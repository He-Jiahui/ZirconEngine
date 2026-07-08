use super::*;

#[test]
fn runtime_15_review_guard_expected_slice_status_mirrors_are_registered() {
    let status_rows = read_status_support_expected_slice_rows();
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("status-output expected-slice rows", status_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 review-guard expected-slice maps folder-backed split",
                "runtime_15_review_guard_expected_slice_maps_folder_backed_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/foundation_review_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/code_review_guard_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/typed_error_structure_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/plugin_importer_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review/top_row_review_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/foundation_review_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/code_review_guard_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/typed_error_structure_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/plugin_importer_maps.rs",
                "plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review/top_row_review_maps.rs",
                "runtime_15_review_guard_expected_slice_maps_are_folder_backed",
                "Cargo gate deferred",
            ],
        );
    }
}
