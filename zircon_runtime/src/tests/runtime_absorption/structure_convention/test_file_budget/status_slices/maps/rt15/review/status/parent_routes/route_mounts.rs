use super::*;

#[test]
fn runtime_15_status_support_expected_slice_parent_maps_are_folder_backed() {
    let guard_parent = include_str!("../parent_route_children.rs");
    let status_parent = read_runtime_src(STATUS_SUPPORT_CHILD);
    let date_parent = read_runtime_src(DATE_SUPPORT_CHILD);

    assert_contains_all(
        "status-support parent route guard mounts folder-backed children",
        guard_parent,
        &[
            "#[path = \"parent_routes/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"parent_routes/literal_ownership.rs\"]",
            "mod literal_ownership;",
            "#[path = \"parent_routes/route_input_ownership.rs\"]",
            "mod route_input_ownership;",
            "#[path = \"parent_routes/route_inputs.rs\"]",
            "mod route_inputs;",
            "#[path = \"parent_routes/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"parent_routes/source_reads.rs\"]",
            "mod source_reads;",
            "#[path = \"parent_routes/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );

    assert_contains_all(
        "status-support expected-slice parent maps mount route children",
        &format!("{status_parent}\n{date_parent}"),
        &[
            "#[path = \"status_support_maps/runtime_row_data_maps.rs\"]",
            "mod runtime_row_data_maps;",
            "#[path = \"status_support_maps/foundation_row_data_maps.rs\"]",
            "mod foundation_row_data_maps;",
            "#[path = \"status_support_maps/m2_row_data_maps.rs\"]",
            "mod m2_row_data_maps;",
            "#[path = \"status_support_maps/hub_editor_maps.rs\"]",
            "mod hub_editor_maps;",
            "#[path = \"status_support_maps/render_shader_maps.rs\"]",
            "mod render_shader_maps;",
            "#[path = \"status_support_maps/m3_m4_expected_slice_maps.rs\"]",
            "mod m3_m4_expected_slice_maps;",
            "#[path = \"status_support_maps/root_layout_ui_maps.rs\"]",
            "mod root_layout_ui_maps;",
            "#[path = \"status_support_maps/evidence_maps.rs\"]",
            "mod evidence_maps;",
            "runtime_row_data_maps::expected_status_for_slice(slice)",
            "foundation_row_data_maps::expected_status_for_slice(slice)",
            "m2_row_data_maps::expected_status_for_slice(slice)",
            "hub_editor_maps::expected_status_for_slice(slice)",
            "render_shader_maps::expected_status_for_slice(slice)",
            "m3_m4_expected_slice_maps::expected_status_for_slice(slice)",
            "root_layout_ui_maps::expected_status_for_slice(slice)",
            "evidence_maps::expected_status_for_slice(slice)",
            "runtime_row_data_maps::expected_date_for_slice(slice)",
            "foundation_row_data_maps::expected_date_for_slice(slice)",
            "m2_row_data_maps::expected_date_for_slice(slice)",
            "hub_editor_maps::expected_date_for_slice(slice)",
            "render_shader_maps::expected_date_for_slice(slice)",
            "m3_m4_expected_slice_maps::expected_date_for_slice(slice)",
            "root_layout_ui_maps::expected_date_for_slice(slice)",
            "evidence_maps::expected_date_for_slice(slice)",
        ],
    );
}
