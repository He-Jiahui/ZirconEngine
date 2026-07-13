use super::*;

#[test]
fn runtime_15_expected_slice_module_layout_guard_body_status_is_synced() {
    let status_rows = read_runtime_src(&format!("tests/runtime_absorption/{STATUS_ROW_PATH}"));
    let status_map = read_runtime_src(&format!("tests/runtime_absorption/{STATUS_MAP_PATH}"));
    let date_map = read_runtime_src(&format!("tests/runtime_absorption/{DATE_MAP_PATH}"));
    let status_support_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );
    let status_support_anchor_mirror_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/anchor_mirror.rs",
    );
    let status_support_row_sources =
        format!("{status_support_rows}\n{status_support_anchor_mirror_rows}\n{status_rows}");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let framework_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "module-layout guard-body folder-backed row data",
        &status_rows,
        &[
            MODULE_LAYOUT_GUARD_BODY_SLICE,
            MODULE_LAYOUT_GUARD_BODY_STATUS,
            MODULE_LAYOUT_GUARD_BODY_PARENT,
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[0],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[1],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[2],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[3],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[4],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[5],
            MODULE_LAYOUT_GUARD_BODY_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "module-layout guard-body status map",
        &status_map,
        &[
            MODULE_LAYOUT_GUARD_BODY_SLICE,
            MODULE_LAYOUT_GUARD_BODY_STATUS,
        ],
    );
    assert_contains_all(
        "module-layout guard-body date map",
        &date_map,
        &[MODULE_LAYOUT_GUARD_BODY_SLICE, "Some(\"2026-07-07\")"],
    );

    assert_contains_all(
        "status-output M3 status-support row data",
        &status_support_row_sources,
        &[
            MODULE_LAYOUT_GUARD_BODY_SLICE,
            MODULE_LAYOUT_GUARD_BODY_STATUS,
            MODULE_LAYOUT_GUARD_BODY_PARENT,
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[0],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[1],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[2],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[3],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[4],
            MODULE_LAYOUT_GUARD_BODY_CHILDREN[5],
            MODULE_LAYOUT_GUARD_BODY_GUARD,
            "Cargo gate deferred",
        ],
    );

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
                "Runtime 15 M3 status output expected-slice guard child-owner split",
                "runtime_15_status_output_expected_slice_guard_child_owner_split_static_passed_cargo_deferred",
                EXPECTED_SLICES_PARENT,
                MODULE_LAYOUT_PARENT,
                "structure_convention/test_file_budget/status_slices/legacy_group_maps.rs",
                EXPECTED_SLICE_GUARD,
            ],
        );
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 expected-slice module-layout guard body child split",
                "runtime_15_expected_slice_module_layout_guard_body_child_split_static_passed_cargo_deferred",
                MODULE_LAYOUT_PARENT,
                MODULE_LAYOUT_GUARD_BODY_PARENT,
                EXPECTED_SLICE_GUARD,
                "Cargo gate deferred active Render Plan08 lane",
            ],
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("framework plan", framework_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                MODULE_LAYOUT_GUARD_BODY_SLICE,
                MODULE_LAYOUT_GUARD_BODY_STATUS,
                MODULE_LAYOUT_GUARD_BODY_FRAMEWORKS_STATUS,
                MODULE_LAYOUT_GUARD_BODY_PARENT,
                MODULE_LAYOUT_GUARD_BODY_CHILDREN[0],
                MODULE_LAYOUT_GUARD_BODY_CHILDREN[1],
                MODULE_LAYOUT_GUARD_BODY_CHILDREN[2],
                MODULE_LAYOUT_GUARD_BODY_CHILDREN[3],
                MODULE_LAYOUT_GUARD_BODY_CHILDREN[4],
                MODULE_LAYOUT_GUARD_BODY_CHILDREN[5],
                MODULE_LAYOUT_GUARD_BODY_GUARD,
                "Cargo gate deferred",
            ],
        );
    }
}
