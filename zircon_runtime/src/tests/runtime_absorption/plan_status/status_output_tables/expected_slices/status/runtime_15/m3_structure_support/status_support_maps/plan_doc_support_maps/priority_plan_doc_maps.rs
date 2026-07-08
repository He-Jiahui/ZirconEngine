#[path = "priority_plan_doc_maps/expected_slice_map_rows.rs"]
mod expected_slice_map_rows;
#[path = "priority_plan_doc_maps/guard_child_owner_maps.rs"]
mod guard_child_owner_maps;
#[path = "priority_plan_doc_maps/integrity_guard_maps.rs"]
mod integrity_guard_maps;
#[path = "priority_plan_doc_maps/inventory_sync_maps.rs"]
mod inventory_sync_maps;
#[path = "priority_plan_doc_maps/row_data_guard_maps.rs"]
mod row_data_guard_maps;
#[path = "priority_plan_doc_maps/status_mirror_maps.rs"]
mod status_mirror_maps;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    integrity_guard_maps::expected_status_for_slice(slice)
        .or_else(|| guard_child_owner_maps::expected_status_for_slice(slice))
        .or_else(|| inventory_sync_maps::expected_status_for_slice(slice))
        .or_else(|| row_data_guard_maps::expected_status_for_slice(slice))
        .or_else(|| status_mirror_maps::expected_status_for_slice(slice))
        .or_else(|| expected_slice_map_rows::expected_status_for_slice(slice))
}
