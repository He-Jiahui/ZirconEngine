use std::collections::{BTreeMap, HashMap};

use crate::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::snapshot::EditorChromeSnapshot;
use crate::view::ViewDescriptor;
use crate::workbench::model::WorkbenchViewModel;

use super::super::region::{build_document_region_state, build_tool_region_state};
use super::super::{ShellFrame, ShellRegionId, ShellSizePx};
use super::super::{WorkbenchChromeMetrics, WorkbenchShellGeometry};
use super::floating_window_frames::build_floating_window_frames;
use super::region_frames::build_region_frames;
use super::splitter_frames::build_splitter_frames;
use super::viewport_content_frame::build_viewport_content_frame;
use super::window_minimums::{compute_window_min_height, compute_window_min_width};

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
    let resolved_frames = build_region_frames(size, left, document, right, bottom, metrics);
    let splitter_frames = build_splitter_frames(
        left,
        right,
        bottom,
        resolved_frames.left_frame,
        resolved_frames.right_frame,
        resolved_frames.bottom_frame,
        resolved_frames.center_band_frame,
        size.width,
        metrics,
    );
    let floating_window_frames = build_floating_window_frames(
        layout,
        resolved_frames.document_frame,
        resolved_frames.center_band_frame,
    );
    let viewport_content_frame =
        build_viewport_content_frame(model, resolved_frames.document_frame, metrics);
    let window_min_width = compute_window_min_width(left, document, right, metrics);
    let window_min_height = compute_window_min_height(left, document, right, bottom, metrics);

    WorkbenchShellGeometry {
        window_min_width,
        window_min_height,
        center_band_frame: resolved_frames.center_band_frame,
        status_bar_frame,
        region_frames: resolved_frames.region_frames,
        splitter_frames,
        floating_window_frames,
        viewport_content_frame,
    }
}
