use super::*;

pub(super) fn assert_runtime_15_production_guard_support_row_data_children_are_child_owned() {
    let parent = read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH);
    let status_rows = [
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_EXPECTED_SLICE_GUARDS_ROWS_PATH),
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_ROWS_PATH),
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_FOUNDATION_M2_ROWS_PATH),
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_CHILD_GROUP_STATUS_DOC_ROWS_PATH),
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_CHILD_GROUP_STATUS_ROW_DOC_ROWS_PATH),
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_DOCS_CHILD_GROUP_MOVED_ROW_ROWS_PATH),
    ]
    .join("\n");
    let status_map = [
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PLAN_DOC_EXPECTED_SLICE_SUPPORT_PATH),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_PLAN_DOC_EXPECTED_SLICE_SUPPORT_PATH),
    ]
    .join("\n");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    for (module_name, path, representative_row) in PRODUCTION_GUARD_SUPPORT_CHILD_ROWS {
        let path_attr = format!("#[path = \"production_guard_support/{module_name}.rs\"]");
        let module_mount = format!("mod {module_name};");
        assert_contains_all(
            "production guard support parent mounts child row module",
            &parent,
            &[path_attr.as_str(), module_mount.as_str()],
        );

        let child = read_runtime_src(path);
        assert_contains_all(path, &child, &[*representative_row]);
        if !matches!(
            *module_name,
            "core_and_evidence"
                | "module_layout"
                | "review_guard"
                | "runtime_row_data"
                | "status_docs"
        ) {
            assert_contains_all(
                path,
                &child,
                &["pub(super) const EXPECTED_STATUS_OUTPUT_SLICES"],
            );
        }
    }

    let status_anchors = [
        PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_STATUS_NAME,
        PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/core_and_evidence.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/module_layout.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status_docs.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status/foundation_m2_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status/child_group_status_doc_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status/child_group_status_row_doc_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/status/child_group_moved_row_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/expected_slice_guards.rs",
        PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
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
            PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_STATUS_NAME,
            PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "date map",
        &date_map,
        &[
            PRODUCTION_GUARD_SUPPORT_CHILD_SPLIT_STATUS_NAME,
            "2026-07-04",
        ],
    );
}
