use super::*;

#[test]
fn runtime_15_status_output_expected_slice_legacy_maps_are_child_owners() {
    let status_parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status.rs",
    );
    let status_pre_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
    );
    let status_pre_runtime_15_runtime_01_05 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
    );
    let status_pre_runtime_15_runtime_06_10 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs",
    );
    let status_pre_runtime_15_runtime_11_14 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs",
    );
    let date_parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date.rs",
    );
    let date_pre_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
    );
    let date_pre_runtime_15_runtime_01_05 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs",
    );
    let date_pre_runtime_15_runtime_06_10 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs",
    );
    let date_pre_runtime_15_runtime_11_14 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
    );
    let status_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let date_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let legacy_parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_maps.rs",
    );
    let legacy_guard_body = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/expected_slice_guards.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let framework_plan =
        read_repo("docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "expected-slice legacy parent mounts guard body child",
        &legacy_parent,
        &[
            "#[path = \"legacy_maps/guard_body.rs\"]",
            "mod guard_body;",
        ],
    );
    for moved_anchor in [
        "fn runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
        "legacy status/date children own pre-Runtime-15 branches",
        "Runtime 14 Cargo 验证窗口探测",
    ] {
        assert!(
            !legacy_parent.contains(moved_anchor),
            "status_output_expected_slices/legacy_maps.rs should mount guard_body instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "expected-slice legacy guard body owns moved legacy guard",
        &legacy_guard_body,
        &[
            "fn runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
            "legacy status/date children own pre-Runtime-15 branches",
            "Runtime 14 Cargo 验证窗口探测",
        ],
    );

    assert_contains_all(
        "expected-slice parents are routing-only after legacy split",
        &format!("{status_parent}\n{date_parent}"),
        &[
            "#[path = \"status/pre_runtime_15.rs\"]",
            "#[path = \"date/pre_runtime_15.rs\"]",
            "pre_runtime_15::expected_status_for_slice(slice)",
            "pre_runtime_15::expected_date_for_slice(slice)",
        ],
    );
    for moved_literal in [
        "Runtime 14 Cargo 验证窗口探测",
        "Runtime 10 F18 asset manager resolution return shape",
        "Runtime 05 status-output expected anchor split",
        "Runtime 08 ECS root leaf owner guard",
    ] {
        assert!(
            !status_parent.contains(moved_literal),
            "status expected-slice parent should not keep pre-Runtime-15 literal {moved_literal}"
        );
        assert!(
            !date_parent.contains(moved_literal),
            "date expected-slice parent should not keep pre-Runtime-15 literal {moved_literal}"
        );
    }
    assert_contains_all(
        "legacy status/date children own pre-Runtime-15 branches",
        &format!(
            "{status_pre_runtime_15_runtime_01_05}\n{status_pre_runtime_15_runtime_06_10}\n{status_pre_runtime_15_runtime_11_14}\n{date_pre_runtime_15_runtime_01_05}\n{date_pre_runtime_15_runtime_06_10}\n{date_pre_runtime_15_runtime_11_14}"
        ),
        &[
            "Runtime 14 Cargo 验证窗口探测",
            "Runtime 10 F18 asset manager resolution return shape",
            "Runtime 05 status-output expected anchor split",
            "Runtime 08 ECS root leaf owner guard",
        ],
    );
    assert_contains_all(
        "legacy status/date parents route pre-Runtime-15 child groups",
        &format!("{status_pre_runtime_15}\n{date_pre_runtime_15}"),
        &[
            "runtime_01_05::expected_status_for_slice(slice)",
            "runtime_06_10::expected_status_for_slice(slice)",
            "runtime_11_14::expected_status_for_slice(slice)",
            "runtime_01_05::expected_date_for_slice(slice)",
            "runtime_06_10::expected_date_for_slice(slice)",
            "runtime_11_14::expected_date_for_slice(slice)",
            "mirror_docs_static_passed_cargo_pending",
            "2026-06-14",
        ],
    );
    assert_contains_all(
        "Runtime 15 status/date maps record legacy child-owner split",
        &format!("{status_runtime_15}\n{date_runtime_15}"),
        &[
            "Runtime 15 M3 status output expected-slice legacy child-owner split",
            "runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred",
            "Some(\"2026-06-24\")",
        ],
    );
    assert_contains_all(
        "Runtime 15 status rows record legacy child-owner split",
        &status_rows,
        &[
            "Runtime 15 M3 status output expected-slice legacy child-owner split",
            "runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred",
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
            "runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
        ],
    );

    for (path, source) in [
        (
            "plan_status/status_output_tables/expected_slices/status.rs",
            status_parent.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
            status_pre_runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_01_05.rs",
            status_pre_runtime_15_runtime_01_05.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_06_10.rs",
            status_pre_runtime_15_runtime_06_10.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/status/pre_runtime_15/runtime_11_14.rs",
            status_pre_runtime_15_runtime_11_14.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date.rs",
            date_parent.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
            date_pre_runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_01_05.rs",
            date_pre_runtime_15_runtime_01_05.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_06_10.rs",
            date_pre_runtime_15_runtime_06_10.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/pre_runtime_15/runtime_11_14.rs",
            date_pre_runtime_15_runtime_11_14.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices.rs",
            include_str!("../../status_output_expected_slices.rs"),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/legacy_maps.rs",
            legacy_parent.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body.rs",
            legacy_guard_body.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
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
                "Runtime 15 M3 status output expected-slice legacy child-owner split",
                "runtime_15_status_output_expected_slice_legacy_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status/pre_runtime_15.rs",
                "plan_status/status_output_tables/expected_slices/date/pre_runtime_15.rs",
                "runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
            ],
        );
    }

    assert_contains_all(
        "Runtime 15 status rows record legacy guard body child split",
        &status_rows,
        &[
            "Runtime 15 M3 status-output expected-slice legacy guard body child split",
            "runtime_15_status_output_expected_slice_legacy_guard_body_child_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices/legacy_maps.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body.rs",
            "runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
            "Cargo gate deferred active Render Plan08 lane",
        ],
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("framework plan", framework_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status-output expected-slice legacy guard body child split",
                "runtime_15_status_output_expected_slice_legacy_guard_body_child_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_output_expected_slices/legacy_maps.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body.rs",
                "runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
                "Cargo gate deferred active Render Plan08 lane",
            ],
        );
    }
}
