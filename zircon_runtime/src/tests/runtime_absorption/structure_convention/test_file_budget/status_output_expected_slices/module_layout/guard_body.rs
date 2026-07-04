use super::*;

#[test]
fn runtime_15_status_output_expected_slice_guard_child_owner_split() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices.rs",
    );
    let module_layout = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/module_layout.rs",
    );
    let module_layout_guard_body = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body.rs",
    );
    let maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
    );
    let top_level_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
    );
    let top_level_map_assertions = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs",
    );
    let legacy_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_maps.rs",
    );
    let legacy_maps_guard_body = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body.rs",
    );
    let legacy_group_maps = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/status_output_expected_slices/legacy_group_maps.rs",
    );
    let status_scan = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/root_layout/status_scan.rs",
    );
    let status_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let date_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/expected_slice_guards.rs",
    );
    let status_support_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "expected-slice module-layout parent mounts guard body child",
        &module_layout,
        &[
            "#[path = \"module_layout/guard_body.rs\"]",
            "mod guard_body;",
        ],
    );
    for moved_anchor in [
        "fn runtime_15_status_output_expected_slice_guard_child_owner_split",
        "let parent = read_runtime_src(",
        "expected-slice guard parent mounts child guard owners",
    ] {
        assert!(
            !module_layout.contains(moved_anchor),
            "status_output_expected_slices/module_layout.rs should mount guard_body instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "expected-slice module-layout guard body owns moved test",
        &module_layout_guard_body,
        &[
            "fn runtime_15_status_output_expected_slice_guard_child_owner_split",
            "expected-slice guard parent mounts child guard owners",
            "Runtime 15 status rows record expected-slice guard child-owner split",
        ],
    );

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
        &format!("{top_level_maps}\n{top_level_map_assertions}"),
        &[
            "fn runtime_15_status_output_expected_slice_maps_are_child_owners",
            "Runtime 15 status expected-slice topic owners preserve representative literals",
        ],
    );
    assert_contains_all(
        "expected-slice legacy parent mounts guard body child",
        &legacy_maps,
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
            !legacy_maps.contains(moved_anchor),
            "status_output_expected_slices/legacy_maps.rs should mount guard_body instead of keeping {moved_anchor}"
        );
    }
    assert_contains_all(
        "expected-slice legacy guard body owns legacy split guard",
        &legacy_maps_guard_body,
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
            "structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body.rs",
            module_layout_guard_body.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps.rs",
            maps.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps.rs",
            top_level_maps.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/maps/top_level_maps/assertions.rs",
            top_level_map_assertions.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/legacy_maps.rs",
            legacy_maps.as_str(),
        ),
        (
            "structure_convention/test_file_budget/status_output_expected_slices/legacy_maps/guard_body.rs",
            legacy_maps_guard_body.as_str(),
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
        (
            "status-output M3 status-support row data",
            status_support_rows.as_str(),
        ),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
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
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 expected-slice module-layout guard body child split",
                "runtime_15_expected_slice_module_layout_guard_body_child_split_static_passed_cargo_deferred",
                "structure_convention/test_file_budget/status_output_expected_slices/module_layout.rs",
                "structure_convention/test_file_budget/status_output_expected_slices/module_layout/guard_body.rs",
                "runtime_15_status_output_expected_slice_guard_child_owner_split",
                "Cargo gate deferred active Render Plan08 lane",
            ],
        );
    }
}
