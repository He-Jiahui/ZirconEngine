use super::*;

#[test]
fn runtime_15_status_support_expected_slice_status_mirrors_are_registered() {
    let status_rows = read_status_support_expected_slice_rows();

    for (label, source) in [
        ("status-output expected-slice rows", status_rows.as_str()),
        (
            "Runtime 15 plan",
            &read_repo(
                "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            ),
        ),
        (
            "Runtime index",
            &read_repo("docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "review findings",
            &read_repo("docs/plans/engine-code-review-findings-2026-06.md"),
        ),
        (
            "structure convention",
            &read_repo("docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "module convention doc",
            &read_repo("docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "session note",
            &read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md"),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status-support expected-slice map child split",
                "runtime_15_status_support_expected_slice_map_child_split_static_passed_cargo_blocked_render_environment_exports",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps.rs",
                "runtime_15_status_support_expected_slice_maps_are_child_owned",
                "Cargo gate blocked by render environment exports",
            ],
        );
    }
}
