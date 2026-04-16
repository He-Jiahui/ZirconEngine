use std::collections::{BTreeMap, HashMap};

use crate::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::snapshot::{EditorChromeSnapshot, ViewContentKind};
use crate::view::ViewDescriptor;
use crate::workbench::model::WorkbenchViewModel;

use super::active_tab::active_document_tab;
use super::constraints::aggregate_row_constraints;
use super::floating_window::{clamp_floating_window_frame, default_floating_window_frame};
use super::region::{build_document_region_state, build_tool_region_state};
use super::{solve_axis_constraints, ShellFrame, ShellRegionId, ShellSizePx};
use super::{WorkbenchChromeMetrics, WorkbenchShellGeometry};

const EPSILON: f32 = 0.001;

pub fn compute_workbench_shell_geometry(
    model: &WorkbenchViewModel,
    _chrome: &EditorChromeSnapshot,
    layout: &WorkbenchLayout,
    descriptors: &[ViewDescriptor],
    shell_size: ShellSizePx,
    metrics: &WorkbenchChromeMetrics,
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
) -> WorkbenchShellGeometry {
    let descriptor_map: HashMap<_, _> = descriptors
        .iter()
        .map(|descriptor| (descriptor.descriptor_id.clone(), descriptor))
        .collect();
    let size = ShellSizePx::new(shell_size.width.max(1.0), shell_size.height.max(1.0));

    let left = build_tool_region_state(
        model,
        layout,
        &descriptor_map,
        ShellRegionId::Left,
        &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
        transient_region_preferred,
        metrics,
    );
    let right = build_tool_region_state(
        model,
        layout,
        &descriptor_map,
        ShellRegionId::Right,
        &[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ],
        transient_region_preferred,
        metrics,
    );
    let bottom = build_tool_region_state(
        model,
        layout,
        &descriptor_map,
        ShellRegionId::Bottom,
        &[
            ActivityDrawerSlot::BottomLeft,
            ActivityDrawerSlot::BottomRight,
        ],
        transient_region_preferred,
        metrics,
    );
    let document =
        build_document_region_state(model, layout, &descriptor_map, transient_region_preferred);

    let status_bar_frame = ShellFrame::new(
        0.0,
        size.height - metrics.status_bar_height,
        size.width,
        metrics.status_bar_height,
    );
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
    let solved_widths = solve_axis_constraints(available_row_width, &horizontal_constraints);

    let center_band_frame = ShellFrame::new(0.0, center_y, size.width, center_height);
    let mut region_frames = BTreeMap::new();
    let mut x = 0.0;
    for (region, solved) in horizontal_regions.into_iter().zip(solved_widths.iter()) {
        let frame = ShellFrame::new(x, center_y, solved.resolved, center_height);
        region_frames.insert(region, frame);
        x += solved.resolved + metrics.separator_thickness;
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

    let viewport_toolbar_height = active_document_tab(model)
        .map(|tab| {
            matches!(
                tab.content_kind,
                ViewContentKind::Scene | ViewContentKind::Game
            )
        })
        .unwrap_or(false)
        .then_some(metrics.viewport_toolbar_height)
        .unwrap_or(0.0);
    let viewport_content_frame = ShellFrame::new(
        document_frame.x,
        document_frame.y
            + metrics.document_header_height
            + metrics.separator_thickness
            + viewport_toolbar_height,
        document_frame.width,
        (document_frame.height
            - metrics.document_header_height
            - metrics.separator_thickness
            - viewport_toolbar_height)
            .max(0.0),
    );

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

    let floating_window_frames = layout
        .floating_windows
        .iter()
        .enumerate()
        .map(|(index, window)| {
            let requested_frame = if window.frame.width > EPSILON && window.frame.height > EPSILON {
                window.frame
            } else {
                default_floating_window_frame(index, document_frame, center_band_frame)
            };
            (
                window.window_id.clone(),
                clamp_floating_window_frame(requested_frame, center_band_frame),
            )
        })
        .collect();

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
                size.width,
                metrics.splitter_hit_size,
            ),
        );
    }

    let window_min_width = {
        let mut widths = Vec::new();
        if left.visible {
            widths.push(left.constraints);
        }
        widths.push(document.constraints);
        if right.visible {
            widths.push(right.constraints);
        }
        let separators = widths.len().saturating_sub(1) as f32 * metrics.separator_thickness;
        aggregate_row_constraints(&widths).width.resolved().min + separators
    };
    let window_min_height = {
        let mut min_height = metrics.top_bar_height
            + metrics.separator_thickness
            + metrics.host_bar_height
            + metrics.separator_thickness
            + metrics.status_bar_height
            + metrics.separator_thickness;
        let center_min = row_height_constraint.height.resolved().min;
        if bottom.visible {
            min_height +=
                center_min + bottom.constraints.height.resolved().min + metrics.separator_thickness;
        } else {
            min_height += center_min;
        }
        min_height
    };

    WorkbenchShellGeometry {
        window_min_width,
        window_min_height,
        center_band_frame,
        status_bar_frame,
        region_frames,
        splitter_frames,
        floating_window_frames,
        viewport_content_frame,
    }
}
