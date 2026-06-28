use super::*;

#[test]
fn runtime_15_status_output_runtime_15_row_data_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs",
    );
    let runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
    let runtime_15_foundation = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
    );
    let runtime_15_m2 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
    );
    let runtime_15_m3 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
    );
    let runtime_15_m4 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "status row data parent keeps only group aggregation",
        &parent,
        &[
            "#[path = \"expected_status_row_data/runtime_15.rs\"]",
            "mod runtime_15;",
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICE_GROUPS",
            "runtime_15::RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_owner in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "expected_status_row_data.rs should delegate Runtime 15 row literals instead of keeping {moved_owner}"
        );
    }

    assert_contains_all(
        "Runtime 15 status row child owns Runtime 15 row groups",
        &runtime_15,
        &[
            "#[path = \"runtime_15/foundation.rs\"]",
            "mod foundation;",
            "#[path = \"runtime_15/m2.rs\"]",
            "mod m2;",
            "#[path = \"runtime_15/m3.rs\"]",
            "mod m3;",
            "#[path = \"runtime_15/m4.rs\"]",
            "mod m4;",
            "pub(super) const RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "foundation::FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "m2::EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "Runtime 15 F12 offscreen target texture owner cleanup",
        ],
    );
    for moved_row in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 M3 graphics dead-code guard module split",
        "Runtime 15 M3 status output Runtime 15 row data split",
        "Runtime 15 M3 status output expected-slice maps split",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
        "runtime_15_scene_world_project_io_mesh_is_child_owner",
    ] {
        assert!(
            !runtime_15.contains(moved_row),
            "expected_status_row_data/runtime_15.rs should delegate moved row literals instead of keeping {moved_row}"
        );
    }

    assert_contains_all(
        "Runtime 15 foundation row-data child owns foundation row literals",
        &runtime_15_foundation,
        &[
            "pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "Runtime 15 F9 runtime prelude required type coverage",
            "Runtime 15 F5 UI input surrounding-text error source",
        ],
    );

    for (path, source) in [
        (
            "plan_status/status_output_tables/expected_status_row_data.rs",
            parent.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
            runtime_15_foundation.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
            runtime_15_m2.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
            runtime_15_m3.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
            runtime_15_m4.as_str(),
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
        (
            "status-output Runtime 15 M3 row data",
            runtime_15_m3.as_str(),
        ),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 status output Runtime 15 row data split",
                "runtime_15_status_output_runtime_15_row_data_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
                "runtime_15_status_output_runtime_15_row_data_is_child_owner",
            ],
        );
    }
}

#[test]
fn runtime_15_status_output_runtime_15_m2_row_data_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs",
    );
    let runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
    let runtime_15_foundation = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
    );
    let runtime_15_m2 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
    );
    let expected_status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
    );
    let expected_date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "top-level status rows include Runtime 15 M2 row-data group",
        &parent,
        &["runtime_15::RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES"],
    );
    assert_contains_all(
        "Runtime 15 root delegates M2 rows",
        &runtime_15,
        &[
            "#[path = \"runtime_15/m2.rs\"]",
            "mod m2;",
            "pub(super) const RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "m2::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_row in [
        "Runtime 15 M2 core runtime state module naming hard cutover",
        "Runtime 15 M2 editor Workbench archived fixture naming hard cutover",
    ] {
        assert!(
            !runtime_15.contains(moved_row),
            "expected_status_row_data/runtime_15.rs should delegate M2 row literal {moved_row}"
        );
        assert!(
            !runtime_15_foundation.contains(moved_row),
            "expected_status_row_data/runtime_15/foundation.rs should not keep M2 row literal {moved_row}"
        );
    }
    assert_contains_all(
        "Runtime 15 M2 child owns M2 rows",
        &runtime_15_m2,
        &[
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES",
            "Runtime 15 M3 status output Runtime 15 M2 row data split",
            "Runtime 15 M2 core runtime state module naming hard cutover",
            "Runtime 15 M2 editor Workbench archived fixture naming hard cutover",
        ],
    );

    for (path, source) in [
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
            runtime_15_foundation.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
            runtime_15_m2.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Runtime 15 expected status map records M2 row-data split",
        &expected_status_map,
        &[
            "Runtime 15 M3 status output Runtime 15 M2 row data split",
            "runtime_15_status_output_runtime_15_m2_row_data_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected date map records M2 row-data split",
        &expected_date_map,
        &[
            "Runtime 15 M3 status output Runtime 15 M2 row data split",
            "2026-06-28",
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
                "Runtime 15 M3 status output Runtime 15 M2 row data split",
                "runtime_15_status_output_runtime_15_m2_row_data_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
                "runtime_15_status_output_runtime_15_m2_row_data_is_child_owner",
            ],
        );
    }
}

#[test]
fn runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner() {
    let runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
    );
    let runtime_15_foundation = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
    );
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
    );
    let expected_status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support.rs",
    );
    let expected_date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "Runtime 15 root delegates foundation rows",
        &runtime_15,
        &[
            "#[path = \"runtime_15/foundation.rs\"]",
            "mod foundation;",
            "pub(super) const RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "foundation::FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for foundation_row in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 F5 UI input surrounding-text error source",
    ] {
        assert!(
            !runtime_15.contains(foundation_row),
            "expected_status_row_data/runtime_15.rs should delegate foundation row literal {foundation_row}"
        );
    }
    assert_contains_all(
        "Runtime 15 foundation child owns foundation rows",
        &runtime_15_foundation,
        &[
            "pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "Runtime 15 F9 runtime prelude required type coverage",
            "Runtime 15 F5 UI input surrounding-text error source",
        ],
    );

    for (path, source) in [
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
            runtime_15_foundation.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    assert_contains_all(
        "Runtime 15 expected status map records foundation row-data split",
        &expected_status_map,
        &[
            "Runtime 15 M3 status output Runtime 15 foundation row data split",
            "runtime_15_status_output_runtime_15_foundation_row_data_split_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected date map records foundation row-data split",
        &expected_date_map,
        &[
            "Runtime 15 M3 status output Runtime 15 foundation row data split",
            "2026-06-27",
        ],
    );

    for (label, source) in [
        ("Runtime 15 status rows", status_rows.as_str()),
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
                "Runtime 15 M3 status output Runtime 15 foundation row data split",
                "runtime_15_status_output_runtime_15_foundation_row_data_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
                "runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner",
            ],
        );
    }
}
