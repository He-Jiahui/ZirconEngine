use std::collections::BTreeMap;

use super::super::constraints::aggregate_row_constraints;
use super::super::region_state::RegionState;
use super::super::{
    compact_side_defaults, minimum_document_width_fraction, WorkbenchChromeMetrics,
};
use super::super::{
    solve_axis_constraints, AxisConstraint, ShellFrame, ShellRegionId, ShellSizePx,
};
use super::resolved_region_frames::ResolvedRegionFrames;
use super::side_width_allocation::balanced_side_widths_for_budget;
use super::vertical_bands::{resolve_vertical_flex_bands, VerticalFlexBandRequest};

pub(super) fn build_region_frames(
    size: ShellSizePx,
    left: RegionState,
    document: RegionState,
    right: RegionState,
    bottom: RegionState,
    metrics: &WorkbenchChromeMetrics,
) -> ResolvedRegionFrames {
    let row_height_constraint =
        aggregate_row_constraints(&[left.constraints, document.constraints, right.constraints]);
    let vertical = resolve_vertical_flex_bands(
        size,
        VerticalFlexBandRequest::new(
            row_height_constraint.height,
            bottom.visible.then_some(bottom.constraints.height),
            *metrics,
        ),
    );
    let center_y = vertical.center_band_frame.y;
    let center_height = vertical.center_band_frame.height;

    let visible_row_count = [left.visible, true, right.visible]
        .into_iter()
        .filter(|visible| *visible)
        .count();
    let row_separator_count = visible_row_count.saturating_sub(1) as f32;
    let available_row_width =
        (size.width - row_separator_count * metrics.separator_thickness).max(0.0);

    let solved_widths = solve_visible_row_widths(
        size.width,
        available_row_width,
        left.visible.then_some(left.constraints.width),
        document.constraints.width,
        right.visible.then_some(right.constraints.width),
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

    let bottom_frame = vertical.bottom_frame;
    region_frames.insert(ShellRegionId::Bottom, bottom_frame);

    ResolvedRegionFrames {
        center_band_frame,
        status_bar_frame: vertical.status_bar_frame,
        region_frames,
        left_frame,
        document_frame,
        right_frame,
        bottom_frame,
    }
}

fn solve_visible_row_widths(
    shell_width: f32,
    available_row_width: f32,
    left: Option<AxisConstraint>,
    document: AxisConstraint,
    right: Option<AxisConstraint>,
) -> Vec<(ShellRegionId, f32)> {
    match (left, right) {
        (Some(left), Some(right)) => solve_row_widths(
            shell_width,
            available_row_width,
            &[
                ShellRegionId::Left,
                ShellRegionId::Document,
                ShellRegionId::Right,
            ],
            &[left, document, right],
        ),
        (Some(left), None) => solve_row_widths(
            shell_width,
            available_row_width,
            &[ShellRegionId::Left, ShellRegionId::Document],
            &[left, document],
        ),
        (None, Some(right)) => solve_row_widths(
            shell_width,
            available_row_width,
            &[ShellRegionId::Document, ShellRegionId::Right],
            &[document, right],
        ),
        (None, None) => solve_row_widths(
            shell_width,
            available_row_width,
            &[ShellRegionId::Document],
            &[document],
        ),
    }
}

fn solve_row_widths(
    shell_width: f32,
    available_row_width: f32,
    regions: &[ShellRegionId],
    constraints: &[AxisConstraint],
) -> Vec<(ShellRegionId, f32)> {
    let solved = solve_axis_constraints(available_row_width, constraints);
    compact_side_widths(
        shell_width,
        available_row_width,
        regions
            .iter()
            .copied()
            .zip(solved.iter().map(|solved| solved.resolved))
            .collect(),
    )
}

#[cfg(test)]
#[path = "region_frames/allocation_tests.rs"]
mod allocation_tests;

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
    shell_width: f32,
    available_row_width: f32,
    mut widths: Vec<(ShellRegionId, f32)>,
) -> Vec<(ShellRegionId, f32)> {
    let mut released_width = 0.0;
    for (region, width) in &mut widths {
        let Some(limit) = compact_side_width_limit(*region, shell_width) else {
            continue;
        };
        if matches!(region, ShellRegionId::Left | ShellRegionId::Right) && *width > limit {
            released_width += *width - limit;
            *width = limit;
        }
    }
    let left_width = widths
        .iter()
        .find_map(|(region, width)| (*region == ShellRegionId::Left).then_some(*width))
        .unwrap_or(0.0);
    let right_width = widths
        .iter()
        .find_map(|(region, width)| (*region == ShellRegionId::Right).then_some(*width))
        .unwrap_or(0.0);
    let side_budget =
        (available_row_width - shell_width.max(0.0) * minimum_document_width_fraction()).max(0.0);
    let balanced = balanced_side_widths_for_budget(left_width, right_width, side_budget);
    for (region, width) in &mut widths {
        let next_width = match region {
            ShellRegionId::Left => balanced.left,
            ShellRegionId::Right => balanced.right,
            ShellRegionId::Bottom | ShellRegionId::Document => continue,
        };
        if *width > next_width {
            released_width += *width - next_width;
            *width = next_width;
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
