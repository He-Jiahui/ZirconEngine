use super::*;

#[test]
fn runtime_15_p0_native_fixture_leaf_owner_guard_folder_backed_status_is_current() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = format!(
        "{}\n{}\n{}",
        read_runtime_src(REVIEW_GUARD_ROWS),
        read_runtime_src(REVIEW_GUARD_P0_ROWS),
        read_runtime_src(STRUCTURE_GUARD_ROWS)
    );
    let status_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_STATUS_MAP),
        read_runtime_src(REVIEW_GUARD_P0_STATUS_MAP)
    );
    let date_map = format!(
        "{}\n{}\n{}",
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
        read_runtime_src(REVIEW_GUARD_DATE_MAP),
        read_runtime_src(REVIEW_GUARD_P0_DATE_MAP)
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                SLICE,
                STATUS,
                FOLDER_BACKED_SLICE,
                FOLDER_BACKED_STATUS,
                STRUCTURE_GUARD_OWNER,
                PARENT,
                SDK_MACRO_LEAF,
                IMPORTER_LEAF,
                SDK_MACRO_REVIEW,
                IMPORTER_REVIEW,
                GUARD,
                FOLDER_BACKED_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
    assert_contains_all(
        "status-output slice status map",
        &status_map,
        &[SLICE, STATUS, FOLDER_BACKED_SLICE, FOLDER_BACKED_STATUS],
    );
    assert_contains_all(
        "status-output slice date map",
        &date_map,
        &[SLICE, DATE, FOLDER_BACKED_SLICE, FOLDER_BACKED_DATE],
    );

    let source_status_anchors = [
        P0_NATIVE_FIXTURE_SOURCE_STATUS_MAP_SLICE,
        P0_NATIVE_FIXTURE_SOURCE_STATUS_MAP_STATUS,
        P0_NATIVE_FIXTURE_ROOT_PATHS_CHILD,
        P0_NATIVE_FIXTURE_ROOT_CHILD_ROWS_CHILD,
        P0_NATIVE_FIXTURE_ROOT_INVENTORY_CHILD,
        P0_NATIVE_FIXTURE_ROOT_SOURCES_CHILD,
        P0_NATIVE_FIXTURE_STATUS_MIRRORS_CHILD,
        REVIEW_GUARD_P0_ROWS,
        REVIEW_GUARD_P0_STATUS_MAP,
        FOLDER_BACKED_GUARD,
        P0_NATIVE_FIXTURE_ROOT_INVENTORY_GUARD,
        FOLDER_BACKED_STATUS_GUARD,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &source_status_anchors);
    }
    assert_contains_all(
        "status-output slice status map",
        &status_map,
        &[
            P0_NATIVE_FIXTURE_SOURCE_STATUS_MAP_SLICE,
            P0_NATIVE_FIXTURE_SOURCE_STATUS_MAP_STATUS,
        ],
    );
    assert_contains_all(
        "status-output slice date map",
        &date_map,
        &[
            P0_NATIVE_FIXTURE_SOURCE_STATUS_MAP_SLICE,
            P0_NATIVE_FIXTURE_SOURCE_STATUS_MAP_DATE,
        ],
    );
}
