#[path = "runtime_index_anchor_maps/cargo_attempt_maps.rs"]
mod cargo_attempt_maps;
#[path = "runtime_index_anchor_maps/index_baseline_maps.rs"]
mod index_baseline_maps;
#[path = "runtime_index_anchor_maps/plan_status_guard_maps.rs"]
mod plan_status_guard_maps;
#[path = "runtime_index_anchor_maps/runtime_status_anchor_maps.rs"]
mod runtime_status_anchor_maps;
#[path = "runtime_index_anchor_maps/status_support_map_rows.rs"]
mod status_support_map_rows;
#[path = "runtime_index_anchor_maps/support_inventory_maps.rs"]
mod support_inventory_maps;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    index_baseline_maps::expected_date_for_slice(slice)
        .or_else(|| runtime_status_anchor_maps::expected_date_for_slice(slice))
        .or_else(|| cargo_attempt_maps::expected_date_for_slice(slice))
        .or_else(|| plan_status_guard_maps::expected_date_for_slice(slice))
        .or_else(|| support_inventory_maps::expected_date_for_slice(slice))
        .or_else(|| status_support_map_rows::expected_date_for_slice(slice))
}
