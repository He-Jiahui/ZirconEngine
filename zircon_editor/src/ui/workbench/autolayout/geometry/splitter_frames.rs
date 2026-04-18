use std::collections::BTreeMap;

use super::super::region_state::RegionState;
use super::super::{ShellFrame, ShellRegionId, WorkbenchChromeMetrics};

const EPSILON: f32 = 0.001;

pub(super) fn build_splitter_frames(
    left: RegionState,
    right: RegionState,
    bottom: RegionState,
    left_frame: ShellFrame,
    right_frame: ShellFrame,
    bottom_frame: ShellFrame,
    center_band_frame: ShellFrame,
    shell_width: f32,
    metrics: &WorkbenchChromeMetrics,
) -> BTreeMap<ShellRegionId, ShellFrame> {
    let mut splitter_frames = BTreeMap::new();

    if left.expanded && left_frame.width > metrics.rail_width + EPSILON {
        splitter_frames.insert(
            ShellRegionId::Left,
            ShellFrame::new(
                left_frame.right() - metrics.splitter_hit_size / 2.0,
                center_band_frame.y,
                metrics.splitter_hit_size,
                center_band_frame.height,
            ),
        );
    }

    if right.expanded && right_frame.width > metrics.rail_width + EPSILON {
        splitter_frames.insert(
            ShellRegionId::Right,
            ShellFrame::new(
                right_frame.x - metrics.separator_thickness - metrics.splitter_hit_size / 2.0,
                center_band_frame.y,
                metrics.splitter_hit_size,
                center_band_frame.height,
            ),
        );
    }

    if bottom.expanded && bottom_frame.height > EPSILON {
        splitter_frames.insert(
            ShellRegionId::Bottom,
            ShellFrame::new(
                0.0,
                bottom_frame.y - metrics.separator_thickness - metrics.splitter_hit_size / 2.0,
                shell_width,
                metrics.splitter_hit_size,
            ),
        );
    }

    splitter_frames
}
