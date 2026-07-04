use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners() {
    let status_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
    );
    let status_children = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m4_surface_cleanup.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
        ),
    ];
    let status_m3_structure_support_children = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/review_guard_maps.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/naming_guard_maps.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs",
        ),
    ];
    let date_runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
    );
    let date_children = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m4_surface_cleanup.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
        ),
    ];
    let date_m3_structure_support_children = [
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/review_guard_maps.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/naming_guard_maps.rs",
        ),
        read_runtime_src(
            "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs",
        ),
    ];
    let status_rows = read_runtime_src(
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
        "Runtime 15 status expected-slice parent mounts topic owners",
        &status_runtime_15,
        &[
            "mod foundation;",
            "mod naming_boundary;",
            "mod m4_surface_cleanup;",
            "mod m3_structure_support;",
            "foundation::expected_status_for_slice(slice)",
            "m3_structure_support::expected_status_for_slice(slice)",
        ],
    );
    assert_contains_all(
        "Runtime 15 date expected-slice parent mounts topic owners",
        &date_runtime_15,
        &[
            "mod foundation;",
            "mod naming_boundary;",
            "mod m4_surface_cleanup;",
            "mod m3_structure_support;",
            "foundation::expected_date_for_slice(slice)",
            "m3_structure_support::expected_date_for_slice(slice)",
        ],
    );
    for moved_literal in [
        "Runtime 15 M3 lock poison policy guard folder split",
        "Runtime 15 M2 core runtime state module naming hard cutover",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "Runtime 15 M3 status output expected-slice guard child-owner split",
    ] {
        assert!(
            !status_runtime_15.contains(moved_literal),
            "Runtime 15 status expected-slice parent should delegate {moved_literal}"
        );
        assert!(
            !date_runtime_15.contains(moved_literal),
            "Runtime 15 date expected-slice parent should delegate {moved_literal}"
        );
    }

    let status_child_sources = status_children
        .iter()
        .chain(status_m3_structure_support_children.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    let date_child_sources = date_children
        .iter()
        .chain(date_m3_structure_support_children.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    assert_contains_all(
        "Runtime 15 status expected-slice children own topic literals",
        &status_child_sources,
        &[
            "Runtime 15 M3 core runtime lock poison guard child-owner split",
            "Runtime 15 M2 core runtime state module naming hard cutover",
            "Runtime 15 M4 scene world project I/O mesh owner split",
            "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 date expected-slice children own topic literals",
        &date_child_sources,
        &[
            "Runtime 15 M3 core runtime lock poison guard child-owner split",
            "Runtime 15 M2 core runtime state module naming hard cutover",
            "Runtime 15 M4 scene world project I/O mesh owner split",
            "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
            "Some(\"2026-06-25\")",
        ],
    );

    for (path, source) in [
        (
            "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
            status_runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
            date_runtime_15.as_str(),
        ),
        (
            "status/runtime_15/foundation.rs",
            status_children[0].as_str(),
        ),
        (
            "status/runtime_15/naming_boundary.rs",
            status_children[1].as_str(),
        ),
        (
            "status/runtime_15/m4_surface_cleanup.rs",
            status_children[2].as_str(),
        ),
        (
            "status/runtime_15/m3_structure_support.rs",
            status_children[3].as_str(),
        ),
        (
            "status/runtime_15/m3_structure_support/review_guard_maps.rs",
            status_m3_structure_support_children[0].as_str(),
        ),
        (
            "status/runtime_15/m3_structure_support/naming_guard_maps.rs",
            status_m3_structure_support_children[1].as_str(),
        ),
        (
            "status/runtime_15/m3_structure_support/status_support_maps.rs",
            status_m3_structure_support_children[2].as_str(),
        ),
        ("date/runtime_15/foundation.rs", date_children[0].as_str()),
        (
            "date/runtime_15/naming_boundary.rs",
            date_children[1].as_str(),
        ),
        (
            "date/runtime_15/m4_surface_cleanup.rs",
            date_children[2].as_str(),
        ),
        (
            "date/runtime_15/m3_structure_support.rs",
            date_children[3].as_str(),
        ),
        (
            "date/runtime_15/m3_structure_support/review_guard_maps.rs",
            date_m3_structure_support_children[0].as_str(),
        ),
        (
            "date/runtime_15/m3_structure_support/naming_guard_maps.rs",
            date_m3_structure_support_children[1].as_str(),
        ),
        (
            "date/runtime_15/m3_structure_support/status_support_maps.rs",
            date_m3_structure_support_children[2].as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the Runtime 15 focused expected-slice budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("status-output M3 row data", status_rows.as_str()),
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
                "Runtime 15 M3 status output Runtime 15 expected-slice child-owner split",
                "runtime_15_status_output_runtime_15_expected_slice_child_owner_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
                "plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
                "runtime_15_status_output_runtime_15_expected_slice_maps_are_child_owners",
            ],
        );
    }
}
