use super::*;

#[test]
fn runtime_15_status_support_plan_doc_expected_slice_maps_are_folder_backed() {
    let guard_parent = include_str!("../plan_doc_route_children.rs");
    let status_parent = read_runtime_src(STATUS_SUPPORT_PLAN_DOC_CHILD);
    let date_parent = read_runtime_src(DATE_SUPPORT_PLAN_DOC_CHILD);

    assert_contains_all(
        "status-support plan-doc route guard mounts folder-backed children",
        guard_parent,
        &[
            "#[path = \"plan_routes/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"plan_routes/child_paths.rs\"]",
            "mod child_paths;",
            "#[path = \"plan_routes/literal_ownership.rs\"]",
            "mod literal_ownership;",
            "#[path = \"plan_routes/route_input_ownership.rs\"]",
            "mod route_input_ownership;",
            "#[path = \"plan_routes/route_inputs.rs\"]",
            "mod route_inputs;",
            "#[path = \"plan_routes/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"plan_routes/source_reads.rs\"]",
            "mod source_reads;",
            "#[path = \"plan_routes/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );

    assert_contains_all(
        "status-support plan-doc expected-slice parents mount route children",
        &format!("{status_parent}\n{date_parent}"),
        &[
            "#[path = \"plan_doc_support_maps/expected_slice_support_maps.rs\"]",
            "mod expected_slice_support_maps;",
            "#[path = \"plan_doc_support_maps/runtime_index_anchor_maps.rs\"]",
            "mod runtime_index_anchor_maps;",
            "#[path = \"plan_doc_support_maps/priority_plan_doc_maps.rs\"]",
            "mod priority_plan_doc_maps;",
            "#[path = \"plan_doc_support_maps/status_row_data_support_maps.rs\"]",
            "mod status_row_data_support_maps;",
            "#[path = \"plan_doc_support_maps/render_shader_support_maps.rs\"]",
            "mod render_shader_support_maps;",
            "expected_slice_support_maps::expected_status_for_slice(slice)",
            "runtime_index_anchor_maps::expected_status_for_slice(slice)",
            "priority_plan_doc_maps::expected_status_for_slice(slice)",
            "status_row_data_support_maps::expected_status_for_slice(slice)",
            "render_shader_support_maps::expected_status_for_slice(slice)",
            "expected_slice_support_maps::expected_date_for_slice(slice)",
            "runtime_index_anchor_maps::expected_date_for_slice(slice)",
            "priority_plan_doc_maps::expected_date_for_slice(slice)",
            "status_row_data_support_maps::expected_date_for_slice(slice)",
            "render_shader_support_maps::expected_date_for_slice(slice)",
        ],
    );
}
