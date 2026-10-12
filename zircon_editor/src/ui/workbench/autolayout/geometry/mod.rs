#[path = "compute.rs"]
mod compute;
#[path = "floating_window_frames.rs"]
mod floating_window_frames;
#[path = "region_frames.rs"]
mod region_frames;
#[path = "resolved_region_frames.rs"]
mod resolved_region_frames;
#[path = "side_width_allocation.rs"]
mod side_width_allocation;
#[path = "splitter_frames.rs"]
mod splitter_frames;
#[path = "vertical_bands.rs"]
mod vertical_bands;
#[path = "viewport_content_frame.rs"]
mod viewport_content_frame;
#[path = "window_minimums.rs"]
mod window_minimums;

pub use compute::{
    compute_workbench_shell_geometry, compute_workbench_shell_geometry_with_region_defaults,
    compute_workbench_shell_geometry_with_region_defaults_and_scale_mode,
    compute_workbench_shell_geometry_with_scale_mode,
};
pub(crate) use region_frames::compact_side_width_limit;
pub(crate) use side_width_allocation::balanced_side_widths_for_budget;
pub(crate) use vertical_bands::compact_bottom_height_limit;
