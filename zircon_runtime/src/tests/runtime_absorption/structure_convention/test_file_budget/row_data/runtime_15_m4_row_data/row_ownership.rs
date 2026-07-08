use super::*;

#[test]
fn runtime_15_status_output_runtime_15_m4_row_data_is_child_owner() {
    let parent = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m4 = read_runtime_src(RUNTIME_15_M4_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3_status_support = read_runtime_src(RUNTIME_15_M3_STATUS_SUPPORT_ROW_DATA_PATH);

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
        &runtime_15_m3_status_support,
        &[
            ROW_DATA_SPLIT_STATUS_NAME,
            ROW_DATA_SPLIT_STATUS_ID,
            ROW_DATA_SPLIT_GUARD_NAME,
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
}
