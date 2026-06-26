use std::collections::BTreeMap;

use super::super::constraints::aggregate_row_constraints;
use super::super::region_state::RegionState;
use super::super::{compact_bottom_defaults, compact_side_defaults, WorkbenchChromeMetrics};
use super::super::{solve_axis_constraints, ShellFrame, ShellRegionId, ShellSizePx};
use super::resolved_region_frames::ResolvedRegionFrames;

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
    let defaults = compact_bottom_defaults();
    if available_height <= defaults.ultra_available_height {
        return Some(round_to_layout_pixel(
            (available_height * defaults.ultra_max_available_fraction)
                .min(defaults.ultra_max_height)
                .max(defaults.ultra_min_height),
        ));
    }

    (available_height <= defaults.available_height).then(|| {
        round_to_layout_pixel(
            (available_height * defaults.max_available_fraction)
                .min(defaults.max_height)
                .max(defaults.min_height),
        )
    })
}

fn round_to_layout_pixel(value: f32) -> f32 {
    value.round()
}

pub(crate) fn compact_side_width_limit(region: ShellRegionId, available_width: f32) -> Option<f32> {
    let defaults = compact_side_defaults();
    if available_width <= defaults.ultra_available_width {
        return Some(match region {
            ShellRegionId::Left => defaults.ultra_left_max_width,
            ShellRegionId::Right => defaults.ultra_right_max_width,
            ShellRegionId::Bottom | ShellRegionId::Document => available_width,
        });
    }

    (available_width <= defaults.available_width).then(|| match region {
        ShellRegionId::Left => defaults.left_max_width.max(defaults.side_min_width),
        ShellRegionId::Right => defaults.right_max_width.max(defaults.side_min_width),
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
