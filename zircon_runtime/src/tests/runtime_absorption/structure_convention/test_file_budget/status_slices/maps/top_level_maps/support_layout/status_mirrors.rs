use super::*;

const TOP_LEVEL_SUPPORT_GUARD_SLICE: &str =
    "Runtime 15 M3 top-level expected-slice support-layout guard folder-backed split";
const TOP_LEVEL_SUPPORT_GUARD_STATUS: &str =
    "runtime_15_top_level_expected_slice_support_layout_guard_folder_backed_static_passed_cargo_deferred";
const TOP_LEVEL_SUPPORT_GUARD_NAME: &str =
    "runtime_15_top_level_expected_slice_support_layout_guard_is_folder_backed";

#[test]
fn runtime_15_top_level_expected_slice_support_layout_status_mirrors_are_current() {
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );
    let top_level_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (label, source) in [
        ("status-output M3 row data", status_rows.as_str()),
        ("top-level expected-slice row data", top_level_rows.as_str()),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02", frameworks_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status output expected-slice top-level map support child-owner split",
                "runtime_15_status_output_expected_slice_top_level_map_support_child_owner_split_static_passed_cargo_deferred",
                "Runtime 15 M3 top-level expected-slice assertion helper child split",
                "runtime_15_top_level_expected_slice_assertion_helper_child_split_static_passed_cargo_deferred",
                "Runtime 15 M3 top-level expected-slice support-layout guard body child split",
                "runtime_15_top_level_expected_slice_support_layout_guard_body_child_split_static_passed_cargo_deferred",
                TOP_LEVEL_SUPPORT_GUARD_SLICE,
                TOP_LEVEL_SUPPORT_GUARD_STATUS,
                TOP_LEVEL_SUPPORT_GUARD_NAME,
                "structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout.rs",
                "structure_convention/test_file_budget/status_slices/maps/top_level_maps/support_layout/split_layout.rs",
                "Cargo gate deferred",
            ],
        );
    }
}
