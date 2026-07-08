#[path = "m3_structure_support/asset_budget_maps.rs"]
mod asset_budget_maps;
#[path = "m3_structure_support/dead_code_guard_maps.rs"]
mod dead_code_guard_maps;
#[path = "m3_structure_support/lock_poison_module_maps.rs"]
mod lock_poison_module_maps;
#[path = "m3_structure_support/naming_guard_maps.rs"]
mod naming_guard_maps;
#[path = "m3_structure_support/plugin_export_gameplay_maps.rs"]
mod plugin_export_gameplay_maps;
#[path = "m3_structure_support/review_guard_maps.rs"]
mod review_guard_maps;
#[path = "m3_structure_support/row_data_owner_maps.rs"]
mod row_data_owner_maps;
#[path = "m3_structure_support/runtime07_script_maps.rs"]
mod runtime07_script_maps;
#[path = "m3_structure_support/scene_ecs_maps.rs"]
mod scene_ecs_maps;
#[path = "m3_structure_support/status_support_maps.rs"]
mod status_support_maps;
#[path = "m3_structure_support/structure_route_maps.rs"]
mod structure_route_maps;
#[path = "m3_structure_support/ui_maps.rs"]
mod ui_maps;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    structure_route_maps::expected_status_for_slice(slice)
        .or_else(|| review_guard_maps::expected_status_for_slice(slice))
        .or_else(|| naming_guard_maps::expected_status_for_slice(slice))
        .or_else(|| status_support_maps::expected_status_for_slice(slice))
        .or_else(|| row_data_owner_maps::expected_status_for_slice(slice))
        .or_else(|| dead_code_guard_maps::expected_status_for_slice(slice))
        .or_else(|| lock_poison_module_maps::expected_status_for_slice(slice))
        .or_else(|| ui_maps::expected_status_for_slice(slice))
        .or_else(|| asset_budget_maps::expected_status_for_slice(slice))
        .or_else(|| runtime07_script_maps::expected_status_for_slice(slice))
        .or_else(|| plugin_export_gameplay_maps::expected_status_for_slice(slice))
        .or_else(|| scene_ecs_maps::expected_status_for_slice(slice))
}
