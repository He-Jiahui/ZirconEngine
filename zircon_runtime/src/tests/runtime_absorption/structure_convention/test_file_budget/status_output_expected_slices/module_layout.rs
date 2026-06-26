use super::*;

#[test]
fn runtime_15_status_output_expected_slice_guard_child_owner_split() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices.rs",
    );
    let module_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/module_layout.rs",
    );
    let maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
    );
    let legacy_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_maps.rs",
    );
    let legacy_group_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_group_maps.rs",
    );
    let status_scan = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs",
    );
    let status_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let date_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "expected-slice guard parent mounts child guard owners",
        &parent,
        &[
            "#[path = \"status_output_expected_slices/module_layout.rs\"]",
            "mod module_layout;",
            "#[path = \"status_output_expected_slices/maps.rs\"]",
            "mod maps;",
            "#[path = \"status_output_expected_slices/legacy_maps.rs\"]",
            "mod legacy_maps;",
            "#[path = \"status_output_expected_slices/legacy_group_maps.rs\"]",
            "mod legacy_group_maps;",
        ],
    );
    for moved_guard in [
        "fn runtime_15_status_output_expected_slice_maps_are_child_owners",
        "fn runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
        "fn runtime_15_status_output_expected_slice_legacy_group_maps_are_child_owners",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "status_output_expected_slices.rs should mount child guard owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "expected-slice maps child owns Runtime 15 split guard",
        &maps,
        &[
            "fn runtime_15_status_output_expected_slice_maps_are_child_owners",
            "Runtime 15 status expected-slice child owns Runtime 15 status literals",
        ],
    );
    assert_contains_all(
        "expected-slice legacy child owns legacy split guard",
        &legacy_maps,
        &[
            "fn runtime_15_status_output_expected_slice_legacy_maps_are_child_owners",
            "legacy status/date children own pre-Runtime-15 branches",
        ],
    );
    assert_contains_all(
        "expected-slice legacy group child owns grouped legacy split guard",
        &legacy_group_maps,
        &[
            "fn runtime_15_status_output_expected_slice_legacy_group_maps_are_child_owners",
            "legacy expected-slice child groups own representative runtime ranges",
        ],
    );
    assert_contains_all(
        "root-layout status scan includes expected-slice guard children",
        &status_scan,
        &[
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/module_layout.rs",
            "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_group_maps.rs",
        ],
    );

    for (path, source) in [
        (
            "structure_convention/test_file_budget/status_output_expected_slices.rs",
            parent.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/module_layout.rs",
            module_layout.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
            maps.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/legacy_maps.rs",
            legacy_maps.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/legacy_group_maps.rs",
            legacy_group_maps.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Runtime 15 status rows record expected-slice guard child-owner split",
        &status_rows,
        &[
            "Runtime 15 M3 status output expected-slice guard child-owner split",
            "runtime_15_status_output_expected_slice_guard_child_owner_split_static_passed_cargo_deferred",
            "structure_convention/test_file_budget/status_output_expected_slices.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/module_layout.rs",
            "structure_convention/test_file_budget/status_output_expected_slices/legacy_group_maps.rs",
            "runtime_15_status_output_expected_slice_guard_child_owner_split",
        ],
    );
    assert_contains_all(
        "Runtime 15 status/date maps record expected-slice guard child-owner split",
        &format!("{status_runtime_15}\n{date_runtime_15}"),
        &[
            "Runtime 15 M3 status output expected-slice guard child-owner split",
            "runtime_15_status_output_expected_slice_guard_child_owner_split_static_passed_cargo_deferred",
            "Some(\"2026-06-24\")",
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
                "structure_convention/test_file_budget/status_output_expected_slices.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/module_layout.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/legacy_group_maps.rs",
                "runtime_15_status_output_expected_slice_guard_child_owner_split",
            ],
        );
    }
}
