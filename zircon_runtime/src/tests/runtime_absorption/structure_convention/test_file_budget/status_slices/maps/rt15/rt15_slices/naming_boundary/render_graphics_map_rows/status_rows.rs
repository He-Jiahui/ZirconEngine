use super::*;

#[test]
fn runtime_15_status_output_naming_boundary_render_graphics_map_rows_are_folder_backed() {
    let status_parent = read_runtime_src(STATUS_CHILD_PATHS[3]);
    let date_parent = read_runtime_src(DATE_CHILD_PATHS[3]);
    let status_sources = read_status_naming_boundary_sources();
    let date_sources = read_date_naming_boundary_sources();

    for (label, parent) in [
        ("status render-graphics map parent", status_parent.as_str()),
        ("date render-graphics map parent", date_parent.as_str()),
    ] {
        for moved in [
            "Runtime 15 M2 graphics render-framework receiver naming hard cutover",
            "Runtime 15 M2 render shader definition bare-flag naming hard cutover",
            "Runtime 15 M2 render feature fallback capability naming hard cutover",
            "Runtime 15 M2 Hybrid GI extract scene-source naming hard cutover",
            "Runtime 15 M2 material asset schema-v1 defaults naming hard cutover",
        ] {
            assert!(
                !parent.contains(moved),
                "{label} should delegate render-graphics row {moved}"
            );
        }
    }

    assert_contains_all(
        "status render-graphics children",
        &status_sources,
        &[
            MAP_ROWS_SLICE,
            MAP_ROWS_STATUS,
            "runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred",
            "runtime_15_scene_render_layer_schema_v1_mask_naming_hard_cutover_static_passed_cargo_deferred",
            "runtime_15_render_shader_definition_bare_flag_naming_hard_cutover_static_passed_cargo_deferred",
            "runtime_15_render_feature_fallback_capability_naming_hard_cutover_static_passed_cargo_deferred",
            "runtime_15_hybrid_gi_extract_scene_source_naming_hard_cutover_static_passed_cargo_deferred",
            "runtime_15_material_asset_schema_v1_defaults_naming_hard_cutover_static_passed_cargo_deferred",
        ],
    );
    assert_contains_all(
        "date render-graphics children",
        &date_sources,
        &[
            MAP_ROWS_SLICE,
            "Some(\"2026-07-07\")",
            "Some(\"2026-06-25\")",
            "Some(\"2026-06-27\")",
            "Some(\"2026-06-29\")",
        ],
    );
}

#[test]
fn runtime_15_naming_boundary_render_graphics_map_rows_status_rows_are_synced() {
    let row_data = read_status_support_expected_slice_rows();
    let status_map = read_status_structure_route_map_sources();
    let date_map = read_date_structure_route_map_sources();

    assert_contains_all(
        "render-graphics map row data",
        &row_data,
        &[
            MAP_ROWS_SLICE,
            MAP_ROWS_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/expected_slice_rows.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/render_framework_rows.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary/render_graphics/asset_font_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics/expected_slice_rows.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary/render_graphics/asset_font_rows.rs",
            MAP_ROWS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "render-graphics structure route maps",
        &format!("{status_map}\n{date_map}"),
        &[
            MAP_ROWS_SLICE,
            MAP_ROWS_STATUS,
            "2026-07-07",
            MAP_ROWS_GUARD,
        ],
    );
}
