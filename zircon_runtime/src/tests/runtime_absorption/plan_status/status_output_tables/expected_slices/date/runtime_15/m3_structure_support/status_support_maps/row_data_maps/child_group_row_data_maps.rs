#[path = "child_group_row_data_maps/expected_slice_map_rows.rs"]
mod expected_slice_map_rows;
#[path = "child_group_row_data_maps/moved_row_maps.rs"]
mod moved_row_maps;
#[path = "child_group_row_data_maps/row_data_maps.rs"]
mod row_data_maps;
#[path = "child_group_row_data_maps/status_doc_maps.rs"]
mod status_doc_maps;
#[path = "child_group_row_data_maps/status_row_doc_maps.rs"]
mod status_row_doc_maps;

pub(super) fn expected_date_for_slice(slice: &str) -> Option<&'static str> {
    status_doc_maps::expected_date_for_slice(slice)
        .or_else(|| row_data_maps::expected_date_for_slice(slice))
        .or_else(|| status_row_doc_maps::expected_date_for_slice(slice))
        .or_else(|| moved_row_maps::expected_date_for_slice(slice))
        .or_else(|| expected_slice_map_rows::expected_date_for_slice(slice))
}
