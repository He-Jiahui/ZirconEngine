use super::*;

#[test]
fn runtime_15_status_output_runtime_15_m4_row_data_is_child_owner() {
    let parent = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs",
    );
    let runtime_15 = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
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
        "status row aggregation exposes Runtime 15 M4 child group",
        &parent,
        &[
            "runtime_15::RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 status row parent mounts M4 row child",
        &runtime_15,
        &[
            "#[path = \"runtime_15/m3.rs\"]",
            "mod m3;",
            "#[path = \"runtime_15/m4.rs\"]",
            "mod m4;",
            "m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "m4::EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 status row child records M4 split status row",
        &runtime_15_m3,
        &[
            "Runtime 15 M3 status output Runtime 15 M4 row data split",
            "runtime_15_status_output_runtime_15_m4_row_data_split_static_passed_cargo_deferred",
            "runtime_15_status_output_runtime_15_m4_row_data_is_child_owner",
        ],
    );
    assert_contains_all(
        "Runtime 15 M4 status row child owns M4 row literals",
        &runtime_15_m4,
        &[
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICES",
            "Runtime 15 M4 core runtime service-list owner split",
            "Runtime 15 M4 RHI WGPU UI surface render/setup owner split",
            "Runtime 15 M4 material asset value/readiness helper owner split",
            "Runtime 15 M4 scene world project I/O mesh owner split",
            "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
            "runtime_15_scene_world_project_io_mesh_is_child_owner",
        ],
    );
    for moved_m4_row in [
        "Runtime 15 M4 core runtime service-list owner split",
        "Runtime 15 M4 RHI WGPU UI surface render/setup owner split",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
    ] {
        assert!(
            !runtime_15.contains(moved_m4_row),
            "expected_status_row_data/runtime_15.rs should delegate M4 row literals instead of keeping {moved_m4_row}"
        );
    }

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
                "Runtime 15 M3 status output Runtime 15 M4 row data split",
                "runtime_15_status_output_runtime_15_m4_row_data_split_static_passed_cargo_deferred",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
                "runtime_15_status_output_runtime_15_m4_row_data_is_child_owner",
            ],
        );
    }
}
