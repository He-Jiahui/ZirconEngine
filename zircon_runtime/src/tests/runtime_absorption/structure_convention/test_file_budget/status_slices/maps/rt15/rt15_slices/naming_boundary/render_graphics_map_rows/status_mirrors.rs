use super::*;

#[test]
fn runtime_15_naming_boundary_render_graphics_map_rows_guard_status_mirrors_are_synced() {
    let row_data = read_status_support_expected_slice_rows();
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/expected_slice_guard_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/expected_slice_guard_maps.rs",
    );

    assert_contains_all(
        "render-graphics map rows guard row data",
        &row_data,
        &[
            MAP_ROWS_GUARD_SLICE,
            MAP_ROWS_GUARD_STATUS,
            MAP_ROWS_GUARD_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "render-graphics map rows guard status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[MAP_ROWS_GUARD_SLICE, MAP_ROWS_GUARD_STATUS, "2026-07-07"],
    );
}

#[test]
fn runtime_15_naming_boundary_render_graphics_map_rows_docs_are_synced() {
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let frameworks_index =
        read_repo("docs/plans/zircon_runtime/frameworks/02/2026-07-09-index-output-records.md");
    for (label, source) in [
        (
            "Runtime 15 plan",
            read_repo(
                "docs/plans/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
            ),
        ),
        (
            "Runtime index",
            read_repo(
                "docs/plans/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
            ),
        ),
        ("Frameworks 02", frameworks_02.clone()),
        ("Frameworks index", frameworks_index.clone()),
        (
            "review findings",
            read_repo(
                "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
            ),
        ),
        (
            "structure convention",
            read_repo(
                "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
            ),
        ),
        (
            "module convention doc",
            read_repo("docs/zircon_runtime/structure/module-convention.md"),
        ),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                MAP_ROWS_SLICE,
                MAP_ROWS_STATUS,
                "naming_boundary/render_graphics.rs",
                "naming_boundary/render_graphics/expected_slice_rows.rs",
                MAP_ROWS_GUARD,
                MAP_ROWS_GUARD_SLICE,
                MAP_ROWS_GUARD_STATUS,
                MAP_ROWS_GUARD_FRAMEWORKS_STATUS,
                "naming_boundary/render_graphics_map_rows.rs",
                "naming_boundary/render_graphics_map_rows/status_mirrors.rs",
                MAP_ROWS_GUARD_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "Frameworks render-graphics map row mirrors",
        &format!("{frameworks_02}\n{frameworks_index}"),
        &[MAP_ROWS_FRAMEWORKS_STATUS, MAP_ROWS_GUARD_FRAMEWORKS_STATUS],
    );
}
