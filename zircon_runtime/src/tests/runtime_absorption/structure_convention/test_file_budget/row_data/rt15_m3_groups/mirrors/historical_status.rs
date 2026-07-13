use super::*;

#[test]
fn runtime_15_m3_child_groups_row_data_historical_status_is_current() {
    let status_map = [
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_CHILD_GROUP_ROW_DATA_PATH),
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_ROOT_RUNTIME_PATH),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_CHILD_GROUP_ROW_DATA_PATH),
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_ROOT_RUNTIME_PATH),
    ]
    .join("\n");
    let production_guard_support = [
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH),
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_CHILD_GROUP_ROW_DATA_ROWS_PATH),
    ]
    .join("\n");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let historical_status_anchors = [
        HISTORICAL_CHILD_OWNER_STATUS_NAME,
        HISTORICAL_CHILD_OWNER_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
        HISTORICAL_CHILD_OWNER_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 production support row data",
            production_guard_support.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &historical_status_anchors);
    }
    assert_contains_all(
        "Runtime 15 expected status map owns historical child-groups row status",
        &status_map,
        &[
            HISTORICAL_CHILD_OWNER_STATUS_NAME,
            HISTORICAL_CHILD_OWNER_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 expected date map owns historical child-groups row date",
        &date_map,
        &[HISTORICAL_CHILD_OWNER_STATUS_NAME, "2026-06-24"],
    );
}
