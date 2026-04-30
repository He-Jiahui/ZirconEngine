mod compute;
mod floating_window_frames;
mod region_frames;
mod resolved_region_frames;
mod splitter_frames;
mod viewport_content_frame;
mod window_minimums;

pub use compute::compute_workbench_shell_geometry;
pub(crate) use region_frames::{compact_bottom_height_limit, compact_side_width_limit};
