use super::*;

const RUNTIME_ROW_DATA_CHILD_ROWS: &[(&str, &str, &str, &str, &str)] = &[
    (
        "foundation_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_FOUNDATION_ROWS_PATH,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/foundation_rows.rs",
        "Runtime 15 M3 foundation row-data guard child-owner split",
        "FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "lock_poison_scene_script_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_LOCK_POISON_SCENE_SCRIPT_ROWS_PATH,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/lock_poison_scene_script_rows.rs",
        "Runtime 15 M3 lock-poison status row-data guard folder-backed split",
        "LOCK_POISON_SCENE_SCRIPT_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "status_support_priority_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_ROWS_PATH,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/status_support_priority_rows.rs",
        "Runtime 15 M3 status-support row-data guard folder-backed split",
        "STATUS_SUPPORT_PRIORITY_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "asset_budget_rows",
        PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_ASSET_BUDGET_ROWS_PATH,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/asset_budget_rows.rs",
        "Runtime 15 M3 asset-budget row-data guard folder-backed split",
        "ASSET_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
];

#[test]
fn runtime_15_production_guard_runtime_row_data_children_are_child_owned() {
    let runtime_row_data = read_runtime_src(PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_ROWS_PATH);
    let production_guard_support = read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let top_level = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let status_rows = read_runtime_src(PRODUCTION_GUARD_SUPPORT_EXPECTED_SLICE_GUARDS_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    let mut status_paths = vec![
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data.rs",
    ];
    for (module_name, read_path, status_path, representative_row, export_name) in
        RUNTIME_ROW_DATA_CHILD_ROWS
    {
        let path_attr = format!("#[path = \"runtime_row_data/{module_name}.rs\"]");
        let export_const = format!("pub(super) const {export_name}");
        let child = read_runtime_src(read_path);
        assert_contains_all(
            "production guard runtime row-data parent delegates to child",
            &runtime_row_data,
            &[path_attr.as_str(), export_const.as_str()],
        );
        assert!(
            !runtime_row_data.contains(representative_row),
            "runtime_row_data.rs should route {representative_row} instead of owning it"
        );
        assert_contains_all(
            read_path,
            &child,
            &[
                "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES",
                *representative_row,
            ],
        );
        status_paths.push(*status_path);
    }

    assert_contains_all(
        "production guard support exports runtime row-data children",
        &production_guard_support,
        &[
            "RUNTIME_ROW_DATA_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_ROW_DATA_LOCK_POISON_SCENE_SCRIPT_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_EXPECTED_STATUS_OUTPUT_SLICES",
            "RUNTIME_ROW_DATA_ASSET_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "M3/Runtime15/top-level exports include runtime row-data children",
        &[runtime_15_m3.as_str(), runtime_15.as_str(), top_level.as_str()].join("\n"),
        &[
            "PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_LOCK_POISON_SCENE_SCRIPT_EXPECTED_STATUS_OUTPUT_SLICES",
            "PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_STATUS_SUPPORT_PRIORITY_EXPECTED_STATUS_OUTPUT_SLICES",
            "PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_ASSET_BUDGET_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );

    let mut status_anchors = vec![
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_ID,
        PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    status_anchors.extend(status_paths);
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("production guard expected-slice rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "status map",
        &status_map,
        &[
            PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map",
        &date_map,
        &[
            PRODUCTION_GUARD_RUNTIME_ROW_DATA_CHILD_SPLIT_STATUS_NAME,
            "2026-07-04",
        ],
    );
}
