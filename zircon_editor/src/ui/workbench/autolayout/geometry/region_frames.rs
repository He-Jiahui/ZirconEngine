use std::collections::BTreeMap;

use super::super::constraints::aggregate_row_constraints;
use super::super::region_state::RegionState;
use super::super::WorkbenchChromeMetrics;
use super::super::{solve_axis_constraints, ShellFrame, ShellRegionId, ShellSizePx};
use super::resolved_region_frames::ResolvedRegionFrames;

const COMPACT_BOTTOM_AVAILABLE_HEIGHT: f32 = 900.0;
const COMPACT_BOTTOM_MAX_HEIGHT: f32 = 148.0;
const COMPACT_BOTTOM_MAX_AVAILABLE_FRACTION: f32 = 0.23;
const COMPACT_BOTTOM_MIN_HEIGHT: f32 = 120.0;
const ULTRA_COMPACT_BOTTOM_AVAILABLE_HEIGHT: f32 = 420.0;
const ULTRA_COMPACT_BOTTOM_MAX_HEIGHT: f32 = 96.0;
const ULTRA_COMPACT_BOTTOM_MAX_AVAILABLE_FRACTION: f32 = 0.20;
const ULTRA_COMPACT_BOTTOM_MIN_HEIGHT: f32 = 80.0;
const COMPACT_SIDE_AVAILABLE_WIDTH: f32 = 1100.0;
const COMPACT_LEFT_SIDE_MAX_WIDTH: f32 = 340.0;
const COMPACT_RIGHT_SIDE_MAX_WIDTH: f32 = 220.0;
const COMPACT_SIDE_MIN_WIDTH: f32 = 196.0;
const ULTRA_COMPACT_SIDE_AVAILABLE_WIDTH: f32 = 760.0;
const ULTRA_COMPACT_LEFT_SIDE_MAX_WIDTH: f32 = 220.0;
const ULTRA_COMPACT_RIGHT_SIDE_MAX_WIDTH: f32 = 160.0;

pub(super) fn build_region_frames(
    size: ShellSizePx,
    left: RegionState,
    document: RegionState,
    right: RegionState,
    bottom: RegionState,
    metrics: &WorkbenchChromeMetrics,
) -> ResolvedRegionFrames {
    let center_y = metrics.top_bar_height
        + metrics.separator_thickness
        + metrics.host_bar_height
        + metrics.separator_thickness;
    let fixed_vertical = metrics.top_bar_height
        + metrics.separator_thickness
        + metrics.host_bar_height
        + metrics.separator_thickness
        + metrics.separator_thickness
        + metrics.status_bar_height
        + if bottom.visible {
            metrics.separator_thickness
        } else {
            0.0
        };
    let center_and_bottom_available = (size.height - fixed_vertical).max(0.0);

    let row_height_constraint =
        aggregate_row_constraints(&[left.constraints, document.constraints, right.constraints]);
    let mut center_height = center_and_bottom_available;
    let mut bottom_height = 0.0;
    if bottom.visible {
        let band_heights = solve_axis_constraints(
            center_and_bottom_available,
            &[row_height_constraint.height, bottom.constraints.height],
        );
        center_height = band_heights[0].resolved;
        bottom_height = band_heights[1].resolved;
        if let Some(compact_limit) = compact_bottom_height_limit(center_and_bottom_available) {
            bottom_height = bottom_height.min(compact_limit);
            center_height = (center_and_bottom_available - bottom_height).max(0.0);
        }
    }

    let visible_row_count = [left.visible, true, right.visible]
        .into_iter()
        .filter(|visible| *visible)
        .count();
    let row_separator_count = visible_row_count.saturating_sub(1) as f32;
    let available_row_width =
        (size.width - row_separator_count * metrics.separator_thickness).max(0.0);

    let mut horizontal_constraints = Vec::new();
    let mut horizontal_regions = Vec::new();
    if left.visible {
        horizontal_regions.push(ShellRegionId::Left);
        horizontal_constraints.push(left.constraints.width);
    }
    horizontal_regions.push(ShellRegionId::Document);
    horizontal_constraints.push(document.constraints.width);
    if right.visible {
        horizontal_regions.push(ShellRegionId::Right);
        horizontal_constraints.push(right.constraints.width);
    }
    let solved_widths = compact_side_widths(
        size.width,
        horizontal_regions
            .iter()
            .copied()
            .zip(
                solve_axis_constraints(available_row_width, &horizontal_constraints)
                    .iter()
                    .map(|solved| solved.resolved),
            )
            .collect(),
    );

    let center_band_frame = ShellFrame::new(0.0, center_y, size.width, center_height);
    let mut region_frames = BTreeMap::new();
    let mut x = 0.0;
    for (region, width) in solved_widths {
        let frame = ShellFrame::new(x, center_y, width, center_height);
        region_frames.insert(region, frame);
        x += width + metrics.separator_thickness;
    }

    let left_frame = region_frames
        .get(&ShellRegionId::Left)
        .copied()
        .unwrap_or_default();
    let document_frame = region_frames
        .get(&ShellRegionId::Document)
        .copied()
        .unwrap_or_default();
    let right_frame = region_frames
        .get(&ShellRegionId::Right)
        .copied()
        .unwrap_or_default();

    let bottom_y = center_y
        + center_height
        + if bottom.visible {
            metrics.separator_thickness
        } else {
            0.0
        };
    let bottom_frame = if bottom.visible {
        ShellFrame::new(0.0, bottom_y, size.width, bottom_height)
    } else {
        ShellFrame::default()
    };
    region_frames.insert(ShellRegionId::Bottom, bottom_frame);

    ResolvedRegionFrames {
        center_band_frame,
        region_frames,
        left_frame,
        document_frame,
        right_frame,
        bottom_frame,
    }
}

pub(crate) fn compact_bottom_height_limit(available_height: f32) -> Option<f32> {
    if available_height <= ULTRA_COMPACT_BOTTOM_AVAILABLE_HEIGHT {
        return Some(
            (available_height * ULTRA_COMPACT_BOTTOM_MAX_AVAILABLE_FRACTION)
                .min(ULTRA_COMPACT_BOTTOM_MAX_HEIGHT)
                .max(ULTRA_COMPACT_BOTTOM_MIN_HEIGHT),
        );
    }

    (available_height <= COMPACT_BOTTOM_AVAILABLE_HEIGHT).then(|| {
        (available_height * COMPACT_BOTTOM_MAX_AVAILABLE_FRACTION)
            .min(COMPACT_BOTTOM_MAX_HEIGHT)
            .max(COMPACT_BOTTOM_MIN_HEIGHT)
    })
}

pub(crate) fn compact_side_width_limit(region: ShellRegionId, available_width: f32) -> Option<f32> {
    if available_width <= ULTRA_COMPACT_SIDE_AVAILABLE_WIDTH {
        return Some(match region {
            ShellRegionId::Left => ULTRA_COMPACT_LEFT_SIDE_MAX_WIDTH,
            ShellRegionId::Right => ULTRA_COMPACT_RIGHT_SIDE_MAX_WIDTH,
            ShellRegionId::Bottom | ShellRegionId::Document => available_width,
        });
    }

    (available_width <= COMPACT_SIDE_AVAILABLE_WIDTH).then(|| match region {
        ShellRegionId::Left => COMPACT_LEFT_SIDE_MAX_WIDTH.max(COMPACT_SIDE_MIN_WIDTH),
        ShellRegionId::Right => COMPACT_RIGHT_SIDE_MAX_WIDTH.max(COMPACT_SIDE_MIN_WIDTH),
        ShellRegionId::Bottom | ShellRegionId::Document => available_width,
    })
}

fn compact_side_widths(
    available_width: f32,
    mut widths: Vec<(ShellRegionId, f32)>,
) -> Vec<(ShellRegionId, f32)> {
    let mut released_width = 0.0;
    for (region, width) in &mut widths {
        let Some(limit) = compact_side_width_limit(*region, available_width) else {
            continue;
        };
        if matches!(region, ShellRegionId::Left | ShellRegionId::Right) && *width > limit {
            released_width += *width - limit;
            *width = limit;
        }
    }
    if released_width > 0.0 {
        if let Some((_, document_width)) = widths
            .iter_mut()
            .find(|(region, _)| *region == ShellRegionId::Document)
        {
            *document_width += released_width;
        }
    }
    widths
}
