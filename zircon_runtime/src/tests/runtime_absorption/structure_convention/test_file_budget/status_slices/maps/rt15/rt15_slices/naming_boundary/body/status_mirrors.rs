use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_expected_slice_status_mirrors_are_registered() {
    let row_data = read_top_level_support_row_sources();
    assert_contains_all(
        "naming-boundary expected-slice row data",
        &row_data,
        &[
            SLICE,
            STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/core_bootstrap.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics.rs",
            GUARD,
        ],
    );

    let status_map = read_status_structure_route_map_sources();
    assert_contains_all("naming-boundary status map", &status_map, &[SLICE, STATUS]);
    let date_map = read_date_structure_route_map_sources();
    assert_contains_all(
        "naming-boundary date map",
        &date_map,
        &[SLICE, "2026-07-05"],
    );

    for (label, source) in [
        (
            "Runtime 15 plan",
            read_repo(
                "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
            ),
        ),
        (
            "Runtime index",
            read_repo("docs/plans/zircon_runtime/runtime/index.md"),
        ),
        (
            "review findings",
            read_repo("docs/plans/engine-code-review-findings-2026-06.md"),
        ),
        (
            "structure convention",
            read_repo("docs/plans/engine-code-structure-convention.md"),
        ),
        (
            "module convention doc",
            read_repo("docs/zircon_runtime/structure/module-convention.md"),
        ),
        (
            "session note",
            read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md"),
        ),
    ] {
        assert_contains_all(label, &source, &[SLICE, STATUS, GUARD]);
    }
}
