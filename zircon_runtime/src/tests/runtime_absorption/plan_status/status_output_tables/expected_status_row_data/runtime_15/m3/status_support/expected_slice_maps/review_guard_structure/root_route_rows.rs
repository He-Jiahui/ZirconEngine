type Slice = super::Slice;

#[path = "root_route_rows/route_metadata_rows.rs"]
mod route_metadata_rows;
#[path = "root_route_rows/route_mount_rows.rs"]
mod route_mount_rows;
#[path = "root_route_rows/status_mirror_rows.rs"]
mod status_mirror_rows;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    route_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    route_mount_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    route_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    route_mount_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    status_mirror_rows::EXPECTED_STATUS_OUTPUT_SLICES[0],
    status_mirror_rows::EXPECTED_STATUS_OUTPUT_SLICES[1],
    route_mount_rows::EXPECTED_STATUS_OUTPUT_SLICES[2],
    route_metadata_rows::EXPECTED_STATUS_OUTPUT_SLICES[3],
];
