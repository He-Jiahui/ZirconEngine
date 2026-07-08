#[path = "m3_m4_expected_slice_maps/expected_slice_guard_maps.rs"]
mod expected_slice_guard_maps;
#[path = "m3_m4_expected_slice_maps/m3_row_data_maps.rs"]
mod m3_row_data_maps;
#[path = "m3_m4_expected_slice_maps/m4_row_data_maps.rs"]
mod m4_row_data_maps;
#[path = "m3_m4_expected_slice_maps/status_support_guard_maps.rs"]
mod status_support_guard_maps;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = m4_row_data_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = expected_slice_guard_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = status_support_guard_maps::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = m3_row_data_maps::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}
