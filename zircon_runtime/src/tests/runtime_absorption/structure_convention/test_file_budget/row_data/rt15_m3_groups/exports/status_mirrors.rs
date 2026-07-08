use super::*;

#[test]
fn runtime_15_m3_child_groups_exports_status_mirrors_are_current() {
    let status_rows = [
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_ROWS_PATH),
        read_runtime_src(
            PRODUCTION_GUARD_SUPPORT_CORE_AND_EVIDENCE_CHILD_GROUP_INVENTORY_ROOT_ROWS_PATH,
        ),
    ]
    .join("\n");
    let status_map = [
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_STATUS_MAP_CHILD_GROUP_ROW_DATA_PATH),
    ]
    .join("\n");
    let date_map = [
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH),
        read_runtime_src(STATUS_SUPPORT_DATE_MAP_CHILD_GROUP_ROW_DATA_PATH),
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

    let status_anchors = [
        EXPORTS_CHILD_SPLIT_STATUS_NAME,
        EXPORTS_CHILD_SPLIT_STATUS_ID,
        "structure_convention/test_file_budget/row_data/rt15_m3_groups/exports.rs",
        "structure_convention/test_file_budget/row_data/rt15_m3_groups/exports/top_level.rs",
        "structure_convention/test_file_budget/row_data/rt15_m3_groups/exports/runtime_15_parent.rs",
        "structure_convention/test_file_budget/row_data/rt15_m3_groups/exports/runtime_15_m3_parent.rs",
        "structure_convention/test_file_budget/row_data/rt15_m3_groups/exports/status_mirrors.rs",
        EXPORTS_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("production guard core/evidence rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 status map records child-groups exports split",
        &status_map,
        &[
            EXPORTS_CHILD_SPLIT_STATUS_NAME,
            EXPORTS_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 date map records child-groups exports split",
        &date_map,
        &[EXPORTS_CHILD_SPLIT_STATUS_NAME, "2026-07-04"],
    );
}
