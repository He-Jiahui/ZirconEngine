use std::collections::{BTreeMap, HashMap};

use crate::ui::workbench::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::view::ViewDescriptor;

use super::super::region::{build_document_region_state, build_tool_region_state};
use super::super::right_drawer_should_collapse_for_logical_width;
use super::super::{
    LogicalRegionPreferredExtents, ResolutionContext, ResolutionScaleMode, ShellFrame,
    ShellRegionId, ShellSizePx,
};
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
    scale_factor: f32,
    metrics: &WorkbenchChromeMetrics,
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
) -> WorkbenchShellGeometry {
    compute_workbench_shell_geometry_with_scale_mode(
        model,
        _chrome,
        layout,
        descriptors,
        shell_size,
        scale_factor,
        ResolutionScaleMode::ConstantPhysical,
        metrics,
        transient_region_preferred,
    )
}

/// Solves shell geometry with an explicitly declared root scaling policy.
pub fn compute_workbench_shell_geometry_with_scale_mode(
    model: &WorkbenchViewModel,
    _chrome: &EditorChromeSnapshot,
    layout: &WorkbenchLayout,
    descriptors: &[ViewDescriptor],
    shell_size: ShellSizePx,
    system_scale_factor: f32,
    scale_mode: ResolutionScaleMode,
    metrics: &WorkbenchChromeMetrics,
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
) -> WorkbenchShellGeometry {
    compute_workbench_shell_geometry_with_region_defaults_and_scale_mode(
        model,
        _chrome,
        layout,
        descriptors,
        shell_size,
        system_scale_factor,
        scale_mode,
        metrics,
        transient_region_preferred,
        None,
    )
}

/// Solves shell geometry with independent user-drag and token-default inputs.
///
/// Both maps use physical host extents at the public boundary. Tool drawers resolve
/// active drag capture first, persisted runtime extent second, and token defaults last.
pub fn compute_workbench_shell_geometry_with_region_defaults(
    model: &WorkbenchViewModel,
    _chrome: &EditorChromeSnapshot,
    layout: &WorkbenchLayout,
    descriptors: &[ViewDescriptor],
    shell_size: ShellSizePx,
    scale_factor: f32,
    metrics: &WorkbenchChromeMetrics,
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
    token_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
) -> WorkbenchShellGeometry {
    compute_workbench_shell_geometry_with_region_defaults_and_scale_mode(
        model,
        _chrome,
        layout,
        descriptors,
        shell_size,
        scale_factor,
        ResolutionScaleMode::ConstantPhysical,
        metrics,
        transient_region_preferred,
        token_region_preferred,
    )
}

/// Solves shell geometry with independent user-drag, token-default, and root-scale inputs.
pub fn compute_workbench_shell_geometry_with_region_defaults_and_scale_mode(
    model: &WorkbenchViewModel,
    _chrome: &EditorChromeSnapshot,
    layout: &WorkbenchLayout,
    descriptors: &[ViewDescriptor],
    shell_size: ShellSizePx,
    system_scale_factor: f32,
    scale_mode: ResolutionScaleMode,
    metrics: &WorkbenchChromeMetrics,
    transient_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
    token_region_preferred: Option<&BTreeMap<ShellRegionId, f32>>,
) -> WorkbenchShellGeometry {
    let descriptor_map: HashMap<&str, &ViewDescriptor> = descriptors
        .iter()
        .map(|descriptor| (descriptor.descriptor_id.0.as_str(), descriptor))
        .collect();
    let resolution = ResolutionContext::from_physical_size_with_scale_mode(
        shell_size,
        system_scale_factor,
        scale_mode,
    );
    let logical_size = resolution.logical_size();
    let size = ShellSizePx::new(logical_size.width.max(1.0), logical_size.height.max(1.0));
    // Resize capture records host-space extents. Borrow both maps and convert only values that the
    // logical solver actually requests, avoiding two temporary maps per geometry solve.
    let transient_region_preferred =
        LogicalRegionPreferredExtents::new(transient_region_preferred, resolution);
    let token_region_preferred =
        LogicalRegionPreferredExtents::new(token_region_preferred, resolution);
    let collapse_right_drawer =
        right_drawer_should_collapse_for_logical_width(resolution.logical_width());

    let left = build_tool_region_state(
        model,
        layout,
        &descriptor_map,
        ShellRegionId::Left,
        &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
        transient_region_preferred,
        token_region_preferred,
        metrics,
        false,
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
        token_region_preferred,
        metrics,
        collapse_right_drawer,
    );
    let bottom = build_tool_region_state(
        model,
        layout,
        &descriptor_map,
        ShellRegionId::Bottom,
        &[ActivityDrawerSlot::Bottom],
        transient_region_preferred,
        token_region_preferred,
        metrics,
        false,
    );
    let document =
        build_document_region_state(model, layout, &descriptor_map, transient_region_preferred);

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
    let window_min_width =
        compute_window_min_width(left, document, right, metrics, resolution.logical_width());
    let window_min_height =
        compute_window_min_height(left, document, right, bottom, metrics, size.height);

    WorkbenchShellGeometry {
        window_min_width,
        window_min_height,
        center_band_frame: resolved_frames.center_band_frame,
        status_bar_frame: resolved_frames.status_bar_frame,
        region_frames: resolved_frames.region_frames,
        splitter_frames,
        floating_window_frames,
        viewport_content_frame,
    }
    .scaled_to_physical(resolution)
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn geometry_indexes_descriptor_rows_by_borrowed_id() {
        let source = include_str!("compute.rs");
        let implementation = source.split("#[cfg(test)]").next().expect("implementation");
        assert!(!implementation.contains("descriptor.descriptor_id.clone()"));
        assert!(implementation.contains("HashMap<&str, &ViewDescriptor>"));
    }
}
