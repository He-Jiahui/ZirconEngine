type Slice = super::ExpectedStatusOutputSlice;

#[path = "expected_slice_maps/base_maps.rs"]
mod base_maps;
#[path = "expected_slice_maps/review_guard_structure.rs"]
mod review_guard_structure;
#[path = "expected_slice_maps/route_metadata.rs"]
mod route_metadata;
#[path = "expected_slice_maps/status_support_maps.rs"]
mod status_support_maps;
#[path = "expected_slice_maps/structure_support.rs"]
mod structure_support;
#[path = "expected_slice_maps/top_level_support.rs"]
mod top_level_support;
#[path = "expected_slice_maps/warning_cleanup.rs"]
mod warning_cleanup;

pub(super) const BASE_MAPS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    base_maps::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const TOP_LEVEL_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    top_level_support::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const ROUTE_METADATA_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    route_metadata::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STRUCTURE_SUPPORT_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    structure_support::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const STATUS_SUPPORT_MAPS_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    status_support_maps::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const REVIEW_GUARD_STRUCTURE_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    review_guard_structure::EXPECTED_STATUS_OUTPUT_SLICES;
pub(super) const WARNING_CLEANUP_EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] =
    warning_cleanup::EXPECTED_STATUS_OUTPUT_SLICES;
