#[path = "structure_route_maps/core_route_rows.rs"]
mod core_route_rows;
#[path = "structure_route_maps/guard_rows.rs"]
mod guard_rows;
#[path = "structure_route_maps/naming_boundary_rows.rs"]
mod naming_boundary_rows;
#[path = "structure_route_maps/review_guard_rows.rs"]
mod review_guard_rows;
#[path = "structure_route_maps/structure_support_rows.rs"]
mod structure_support_rows;

pub(super) fn expected_status_for_slice(slice: &str) -> Option<&'static str> {
    if let Some(status) = structure_support_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = review_guard_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = naming_boundary_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = core_route_rows::expected_status_for_slice(slice) {
        return Some(status);
    }
    if let Some(status) = guard_rows::expected_status_for_slice(slice) {
        return Some(status);
    }

    None
}
