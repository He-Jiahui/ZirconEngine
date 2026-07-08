use super::*;

#[path = "expected_slice_status_support_route_guard_rows/route_children.rs"]
mod route_children;
#[path = "expected_slice_status_support_route_guard_rows/status_current.rs"]
mod status_current;

#[test]
fn runtime_15_status_support_route_guard_rows_are_child_owned() {
    route_children::assert_route_guard_rows_are_child_owned();
    status_current::assert_route_guard_rows_status_is_current(
        &route_children::status_support_route_guard_rows_child_source_blob(),
    );
}
