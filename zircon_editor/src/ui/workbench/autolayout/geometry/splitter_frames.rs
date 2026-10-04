use std::collections::BTreeMap;

use super::super::region_state::RegionState;
use super::super::{ShellFrame, ShellRegionId, WorkbenchChromeMetrics};

const EPSILON: f32 = 0.001;

pub(super) fn build_splitter_frames(
    left: RegionState,
    right: RegionState,
    bottom: RegionState,
    left_frame: ShellFrame,
    document_frame: ShellFrame,
    right_frame: ShellFrame,
    bottom_frame: ShellFrame,
    center_band_frame: ShellFrame,
    shell_size: zircon_runtime_interface::ui::layout::UiSize,
    metrics: &WorkbenchChromeMetrics,
) -> BTreeMap<ShellRegionId, ShellFrame> {
    let mut splitter_frames = BTreeMap::new();
    let shell_width = finite_non_negative(shell_size.width);
    let shell_height = finite_non_negative(shell_size.height);
    let hit_size = finite_non_negative(metrics.splitter_hit_size);

    if left.expanded
        && document_frame.width > EPSILON
        && center_band_frame.height > EPSILON
        && left_frame.width > metrics.rail_width + EPSILON
    {
        let (x, width) = centered_hit_target(left_frame.right(), hit_size, shell_width);
        splitter_frames.insert(
            ShellRegionId::Left,
            ShellFrame::new(x, center_band_frame.y, width, center_band_frame.height),
        );
    }

    if right.expanded
        && document_frame.width > EPSILON
        && center_band_frame.height > EPSILON
        && right_frame.width > metrics.rail_width + EPSILON
    {
        let boundary = right_frame.x - finite_non_negative(metrics.separator_thickness);
        let (x, width) = centered_hit_target(boundary, hit_size, shell_width);
        splitter_frames.insert(
            ShellRegionId::Right,
            ShellFrame::new(x, center_band_frame.y, width, center_band_frame.height),
        );
    }

    if bottom.expanded && center_band_frame.height > EPSILON && bottom_frame.height > EPSILON {
        let boundary = bottom_frame.y - finite_non_negative(metrics.separator_thickness);
        let (y, height) = centered_hit_target(boundary, hit_size, shell_height);
        splitter_frames.insert(
            ShellRegionId::Bottom,
            ShellFrame::new(0.0, y, shell_width, height),
        );
    }

    splitter_frames
}

fn centered_hit_target(boundary: f32, requested_size: f32, host_size: f32) -> (f32, f32) {
    let size = requested_size.min(host_size);
    let position = (finite_non_negative(boundary) - size / 2.0).clamp(0.0, host_size - size);
    (position, size)
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
