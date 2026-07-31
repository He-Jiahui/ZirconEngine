mod compute;
mod floating_window_frames;
mod region_frames;
mod resolved_region_frames;
mod side_width_allocation;
mod splitter_frames;
mod vertical_bands;
mod viewport_content_frame;
mod window_minimums;

pub use compute::compute_workbench_shell_geometry;
pub(crate) use region_frames::compact_side_width_limit;
pub(crate) use side_width_allocation::balanced_side_widths_for_budget;
pub(crate) use vertical_bands::compact_bottom_height_limit;
