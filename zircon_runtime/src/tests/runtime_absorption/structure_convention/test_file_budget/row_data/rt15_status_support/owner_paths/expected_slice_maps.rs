use super::*;

#[path = "expected_slice_maps/base_and_top_level.rs"]
mod base_and_top_level;
#[path = "expected_slice_maps/review_guard_structure.rs"]
mod review_guard_structure;
#[path = "expected_slice_maps/route_metadata.rs"]
mod route_metadata;
#[path = "expected_slice_maps/status_support_maps.rs"]
mod status_support_maps;
#[path = "expected_slice_maps/structure_support.rs"]
mod structure_support;
#[path = "expected_slice_maps/warning_cleanup.rs"]
mod warning_cleanup;

pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_MAP_OWNER_PATH_GROUPS: &[&[(
    &str,
    &str,
    usize,
)]] = &[
    base_and_top_level::EXPECTED_SLICE_BASE_AND_TOP_LEVEL_OWNER_PATHS,
    route_metadata::EXPECTED_SLICE_ROUTE_METADATA_OWNER_PATHS,
    structure_support::EXPECTED_SLICE_STRUCTURE_SUPPORT_OWNER_PATHS,
    status_support_maps::EXPECTED_SLICE_STATUS_SUPPORT_MAPS_OWNER_PATHS,
    review_guard_structure::EXPECTED_SLICE_REVIEW_GUARD_STRUCTURE_OWNER_PATHS,
    warning_cleanup::EXPECTED_SLICE_WARNING_CLEANUP_OWNER_PATHS,
];
