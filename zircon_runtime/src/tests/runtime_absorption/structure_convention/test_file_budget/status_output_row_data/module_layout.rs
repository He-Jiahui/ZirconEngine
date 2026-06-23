use super::*;

#[test]
fn runtime_15_status_output_row_data_guard_child_owner_split() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs",
    );
    let evidence_anchors = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs",
    );
    let runtime_15_row_data = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
    );
    let runtime_15_m4_row_data = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data.rs",
    );
    let runtime_15_m3_row_data = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data.rs",
    );
    let runtime_15_m3_child_groups = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
    );
    let expected_status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let expected_date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "status-output row-data guard parent mounts child owners",
        &parent,
        &[
            "#[path = \"status_output_row_data/module_layout.rs\"]",
            "mod module_layout;",
            "#[path = \"status_output_row_data/evidence_anchors.rs\"]",
            "mod evidence_anchors;",
            "#[path = \"status_output_row_data/runtime_15_row_data.rs\"]",
            "mod runtime_15_row_data;",
            "#[path = \"status_output_row_data/runtime_15_m4_row_data.rs\"]",
            "mod runtime_15_m4_row_data;",
            "#[path = \"status_output_row_data/runtime_15_m3_row_data.rs\"]",
            "mod runtime_15_m3_row_data;",
            "#[path = \"status_output_row_data/runtime_15_m3_child_groups.rs\"]",
            "mod runtime_15_m3_child_groups;",
        ],
    );
    for moved_guard in [
        "fn runtime_15_expected_status_output_rows_accept_variable_evidence_anchors",
        "fn runtime_15_status_output_runtime_15_row_data_is_child_owner",
        "fn runtime_15_status_output_runtime_15_m4_row_data_is_child_owner",
        "fn runtime_15_status_output_runtime_15_m3_row_data_is_child_owner",
        "fn runtime_15_status_output_m3_row_data_child_owner_split",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "status_output_row_data.rs should mount child guard owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "evidence anchor child owns variable evidence guard",
        &evidence_anchors,
        &[
            "fn runtime_15_expected_status_output_rows_accept_variable_evidence_anchors",
            "Runtime 15 M3 keeps multi-anchor evidence rows as slices",
        ],
    );
    assert_contains_all(
        "Runtime 15 row-data child owns Runtime 15 parent split guard",
        &runtime_15_row_data,
        &[
            "fn runtime_15_status_output_runtime_15_row_data_is_child_owner",
            "status row data parent keeps only group aggregation",
        ],
    );
    assert_contains_all(
        "Runtime 15 M4 row-data child owns M4 split guard",
        &runtime_15_m4_row_data,
        &[
            "fn runtime_15_status_output_runtime_15_m4_row_data_is_child_owner",
            "Runtime 15 M4 status row child owns M4 row literals",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 row-data child owns M3 split guard",
        &runtime_15_m3_row_data,
        &[
            "fn runtime_15_status_output_runtime_15_m3_row_data_is_child_owner",
            "Runtime 15 M3 status support child owns M3 row split literals",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 child-groups guard owns M3 child split guard",
        &runtime_15_m3_child_groups,
        &[
            "fn runtime_15_status_output_m3_row_data_child_owner_split",
            "top-level status rows include every Runtime 15 M3 child group",
        ],
    );

    for (path, source) in [
        (
            "structure_convention/test_file_budget/status_output_row_data.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/evidence_anchors.rs",
            evidence_anchors.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
            runtime_15_row_data.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data.rs",
            runtime_15_m4_row_data.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_row_data.rs",
            runtime_15_m3_row_data.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs",
            runtime_15_m3_child_groups.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Runtime 15 status rows record row-data guard child-owner split",
        &status_rows,
        &[
            "Runtime 15 M3 status output row-data guard child-owner split",
            "runtime_15_status_output_row_data_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_row_data.rs",
            "structure_convention/test_file_budget/status_output_row_data/module_layout.rs",
            "runtime_15_status_output_row_data_guard_child_owner_split",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected status map records row-data guard child-owner split",
        &expected_status_map,
        &[
            "Runtime 15 M3 status output row-data guard child-owner split",
            "runtime_15_status_output_row_data_guard_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected date map records row-data guard child-owner split",
        &expected_date_map,
        &[
            "Runtime 15 M3 status output row-data guard child-owner split",
            "2026-06-24",
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
                "Runtime 15 M3 status output row-data guard child-owner split",
                "runtime_15_status_output_row_data_guard_child_owner_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_output_row_data.rs",
                "structure_convention/test_file_budget/status_output_row_data/module_layout.rs",
                "runtime_15_status_output_row_data_guard_child_owner_split",
            ],
        );
    }
}
