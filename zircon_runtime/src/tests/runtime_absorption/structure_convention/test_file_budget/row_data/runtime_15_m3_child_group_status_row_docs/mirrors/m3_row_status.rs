use super::*;

#[test]
fn runtime_15_m3_child_group_status_row_doc_m3_row_statuses_are_current() {
    let lock_poison_status = read_runtime_src(LOCK_POISON_STATUS_ROWS_PATH);
    let module_convention_status = read_runtime_src(MODULE_CONVENTION_STATUS_ROWS_PATH);
    let review_status_sync = read_runtime_src(REVIEW_STATUS_SYNC_ROWS_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source, anchors) in [
        (
            "status-output Runtime 15 M3 lock-poison row data",
            lock_poison_status.as_str(),
            &[
                "Runtime 15 M3 lock-poison status row-data child-owner split",
                "runtime_15_lock_poison_status_row_data_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/lock_poison_status.rs",
            ][..],
        ),
        (
            "status-output Runtime 15 M3 module-convention row data",
            module_convention_status.as_str(),
            &[
                "Runtime 15 M3 module-convention status row-data child-owner split",
                "runtime_15_module_convention_status_row_data_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/module_convention_status.rs",
            ][..],
        ),
        (
            "status-output Runtime 15 M3 review status-sync row data",
            review_status_sync.as_str(),
            &[
                "Runtime 15 M3 review top-row status row-data child-owner split",
                "runtime_15_review_top_row_status_row_data_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_status_sync.rs",
            ][..],
        ),
    ] {
        assert_contains_all(label, source, anchors);
        assert_contains_all(
            label,
            source,
            &[
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
                "runtime_15_status_output_m3_row_data_child_owner_split",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 lock-poison status row-data child-owner split",
                "Runtime 15 M3 module-convention status row-data child-owner split",
                "Runtime 15 M3 review top-row status row-data child-owner split",
                "runtime_15_status_output_m3_row_data_child_owner_split",
            ],
        );
    }
}
