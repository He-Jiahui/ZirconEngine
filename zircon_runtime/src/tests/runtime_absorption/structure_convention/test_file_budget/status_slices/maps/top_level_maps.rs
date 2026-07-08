use super::*;

#[path = "top_level_maps/assertions.rs"]
mod assertions;
#[path = "top_level_maps/sources.rs"]
mod sources;
#[path = "top_level_maps/support_layout.rs"]
mod support_layout;

#[test]
fn runtime_15_status_output_expected_slice_maps_are_child_owners() {
    let sources = sources::read_top_level_map_sources();

    assertions::assert_expected_slice_maps_are_child_owners(&sources);
}
