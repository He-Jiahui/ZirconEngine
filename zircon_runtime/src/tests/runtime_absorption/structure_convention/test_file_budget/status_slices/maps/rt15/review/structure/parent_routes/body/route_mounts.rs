use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_maps_are_folder_backed() {
    let parents = format!(
        "{}\n{}",
        read_runtime_src(STATUS_PARENT),
        read_runtime_src(DATE_PARENT)
    );
    assert_contains_all(
        "structure-support expected-slice parents mount route children",
        &parents,
        &[
            "#[path = \"m3_structure_support/structure_route_maps.rs\"]",
            "mod structure_route_maps;",
            "#[path = \"m3_structure_support/row_data_owner_maps.rs\"]",
            "mod row_data_owner_maps;",
            "#[path = \"m3_structure_support/dead_code_guard_maps.rs\"]",
            "mod dead_code_guard_maps;",
            "#[path = \"m3_structure_support/lock_poison_module_maps.rs\"]",
            "mod lock_poison_module_maps;",
            "#[path = \"m3_structure_support/ui_maps.rs\"]",
            "mod ui_maps;",
            "#[path = \"m3_structure_support/asset_budget_maps.rs\"]",
            "mod asset_budget_maps;",
            "#[path = \"m3_structure_support/runtime07_script_maps.rs\"]",
            "mod runtime07_script_maps;",
            "#[path = \"m3_structure_support/plugin_export_gameplay_maps.rs\"]",
            "mod plugin_export_gameplay_maps;",
            "#[path = \"m3_structure_support/scene_ecs_maps.rs\"]",
            "mod scene_ecs_maps;",
            "structure_route_maps::expected_status_for_slice(slice)",
            "row_data_owner_maps::expected_status_for_slice(slice)",
            "dead_code_guard_maps::expected_status_for_slice(slice)",
            "lock_poison_module_maps::expected_status_for_slice(slice)",
            "ui_maps::expected_status_for_slice(slice)",
            "asset_budget_maps::expected_status_for_slice(slice)",
            "runtime07_script_maps::expected_status_for_slice(slice)",
            "plugin_export_gameplay_maps::expected_status_for_slice(slice)",
            "scene_ecs_maps::expected_status_for_slice(slice)",
            "structure_route_maps::expected_date_for_slice(slice)",
            "row_data_owner_maps::expected_date_for_slice(slice)",
            "dead_code_guard_maps::expected_date_for_slice(slice)",
            "lock_poison_module_maps::expected_date_for_slice(slice)",
            "ui_maps::expected_date_for_slice(slice)",
            "asset_budget_maps::expected_date_for_slice(slice)",
            "runtime07_script_maps::expected_date_for_slice(slice)",
            "plugin_export_gameplay_maps::expected_date_for_slice(slice)",
            "scene_ecs_maps::expected_date_for_slice(slice)",
        ],
    );
}
