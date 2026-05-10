use std::collections::BTreeMap;

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::tree::{UiRuntimeTreeAccessExt, UiRuntimeTreeLayoutExt};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{AxisConstraint, StretchMode, UiFrame, UiSize},
    tree::UiVisibility,
};

use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_HOST_DRAWER_SOURCE_DOCUMENT_ID;
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::autolayout::{compact_bottom_height_limit, WorkbenchChromeMetrics};
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ActivityDrawerSnapshot;

use super::control_ids::{
    BOTTOM_DRAWER_OUTER_SEPARATOR_CONTROL_ID, BOTTOM_DRAWER_PANEL_CONTROL_ID,
    BOTTOM_DRAWER_SHELL_CONTROL_ID, LEFT_DRAWER_PANEL_CONTROL_ID, LEFT_DRAWER_SHELL_CONTROL_ID,
    RIGHT_DRAWER_PANEL_CONTROL_ID, RIGHT_DRAWER_SHELL_CONTROL_ID,
    WORKBENCH_DRAWER_STATUS_BAR_CONTROL_ID, WORKBENCH_DRAWER_TOP_BAR_CONTROL_ID,
};
use super::error::BuiltinHostDrawerSourceTemplateBridgeError;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BuiltinHostDrawerRegionInput {
    visible: bool,
    extent: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct BuiltinHostDrawerLayoutInputs {
    left: BuiltinHostDrawerRegionInput,
    right: BuiltinHostDrawerRegionInput,
    bottom: BuiltinHostDrawerRegionInput,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::callback_dispatch::template_bridge) struct BuiltinHostDrawerLayoutAnchors
{
    top_chrome_height: f32,
    body_height: f32,
    status_bar_height: f32,
}

impl BuiltinHostDrawerLayoutAnchors {
    pub(in crate::ui::retained_host::callback_dispatch::template_bridge) fn from_root_frames(
        shell_size: UiSize,
        body_frame: Option<UiFrame>,
        status_bar_frame: Option<UiFrame>,
    ) -> Option<Self> {
        let body_frame = body_frame.filter(frame_is_visible)?;
        let top_chrome_height = body_frame.y.max(0.0);
        let body_height = body_frame.height.max(0.0);
        let status_bar_height = status_bar_frame
            .filter(frame_is_visible)
            .map(|frame| frame.height.max(0.0))
            .unwrap_or_else(|| (shell_size.height - top_chrome_height - body_height).max(0.0));

        Some(Self {
            top_chrome_height,
            body_height,
            status_bar_height,
        })
    }
}

impl BuiltinHostDrawerLayoutInputs {
    fn from_workbench_model(model: &WorkbenchViewModel, metrics: &WorkbenchChromeMetrics) -> Self {
        Self {
            left: drawer_region_input(
                &model.drawer_ring.drawers,
                &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
                metrics.rail_width,
            ),
            right: drawer_region_input(
                &model.drawer_ring.drawers,
                &[
                    ActivityDrawerSlot::RightTop,
                    ActivityDrawerSlot::RightBottom,
                ],
                metrics.rail_width,
            ),
            bottom: drawer_region_input(
                &model.drawer_ring.drawers,
                &[ActivityDrawerSlot::Bottom],
                metrics.panel_header_height,
            ),
        }
    }
}

pub(super) fn default_drawer_layout_inputs() -> BuiltinHostDrawerLayoutInputs {
    BuiltinHostDrawerLayoutInputs::default()
}

pub(super) fn drawer_layout_inputs_from_workbench_model(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
) -> BuiltinHostDrawerLayoutInputs {
    BuiltinHostDrawerLayoutInputs::from_workbench_model(model, metrics)
}

pub(super) fn build_builtin_host_drawer_source_surface(
    runtime: &EditorUiHostRuntime,
    shell_size: UiSize,
    drawer_inputs: BuiltinHostDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
) -> Result<UiSurface, BuiltinHostDrawerSourceTemplateBridgeError> {
    build_builtin_host_drawer_source_surface_with_anchors(
        runtime,
        shell_size,
        drawer_inputs,
        metrics,
        None,
    )
}

pub(super) fn build_builtin_host_drawer_source_surface_with_anchors(
    runtime: &EditorUiHostRuntime,
    shell_size: UiSize,
    drawer_inputs: BuiltinHostDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
    anchors: Option<BuiltinHostDrawerLayoutAnchors>,
) -> Result<UiSurface, BuiltinHostDrawerSourceTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_HOST_DRAWER_SOURCE_DOCUMENT_ID)?;
    apply_builtin_host_drawer_source_layout(
        &mut surface,
        shell_size,
        drawer_inputs,
        metrics,
        anchors,
    )?;
    surface.compute_layout(shell_size)?;
    Ok(surface)
}

pub(super) fn rebuild_builtin_host_drawer_source_surface_with_anchors(
    surface: &mut UiSurface,
    shell_size: UiSize,
    drawer_inputs: BuiltinHostDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
    anchors: Option<BuiltinHostDrawerLayoutAnchors>,
) -> Result<(), BuiltinHostDrawerSourceTemplateBridgeError> {
    apply_builtin_host_drawer_source_layout(surface, shell_size, drawer_inputs, metrics, anchors)?;
    surface.rebuild_dirty(shell_size)?;
    Ok(())
}

pub(super) fn rebuild_builtin_host_drawer_source_surface(
    surface: &mut UiSurface,
    shell_size: UiSize,
    drawer_inputs: BuiltinHostDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
) -> Result<(), BuiltinHostDrawerSourceTemplateBridgeError> {
    rebuild_builtin_host_drawer_source_surface_with_anchors(
        surface,
        shell_size,
        drawer_inputs,
        metrics,
        None,
    )
}

fn apply_builtin_host_drawer_source_layout(
    surface: &mut UiSurface,
    shell_size: UiSize,
    drawer_inputs: BuiltinHostDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
    anchors: Option<BuiltinHostDrawerLayoutAnchors>,
) -> Result<(), BuiltinHostDrawerSourceTemplateBridgeError> {
    let rail_width = metrics.rail_width.max(0.0);
    let header_height = metrics.panel_header_height.max(0.0);
    let bottom = compacted_bottom_region_input(drawer_inputs.bottom, shell_size, metrics, anchors);

    mark_roots_layout_dirty(surface)?;

    if let Some(anchors) = anchors {
        apply_fixed_control_height(
            surface,
            WORKBENCH_DRAWER_TOP_BAR_CONTROL_ID,
            anchors.top_chrome_height,
        )?;
        apply_fixed_control_height(
            surface,
            WORKBENCH_DRAWER_STATUS_BAR_CONTROL_ID,
            anchors.status_bar_height,
        )?;
    }

    apply_fixed_control_width(
        surface,
        LEFT_DRAWER_SHELL_CONTROL_ID,
        resolved_drawer_shell_extent(drawer_inputs.left),
    )?;
    apply_fixed_control_width(
        surface,
        LEFT_DRAWER_PANEL_CONTROL_ID,
        resolved_side_panel_extent(drawer_inputs.left, rail_width),
    )?;
    apply_fixed_control_width(
        surface,
        RIGHT_DRAWER_SHELL_CONTROL_ID,
        resolved_drawer_shell_extent(drawer_inputs.right),
    )?;
    apply_fixed_control_width(
        surface,
        RIGHT_DRAWER_PANEL_CONTROL_ID,
        resolved_side_panel_extent(drawer_inputs.right, rail_width),
    )?;
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_OUTER_SEPARATOR_CONTROL_ID,
        resolved_drawer_separator_extent(bottom),
    )?;
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_SHELL_CONTROL_ID,
        resolved_drawer_shell_extent(bottom),
    )?;
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_PANEL_CONTROL_ID,
        resolved_bottom_panel_extent(bottom, header_height),
    )?;
    Ok(())
}

fn mark_roots_layout_dirty(
    surface: &mut UiSurface,
) -> Result<(), BuiltinHostDrawerSourceTemplateBridgeError> {
    for root_id in surface.tree.roots.clone() {
        surface.tree.mark_layout_dirty(root_id)?;
    }
    Ok(())
}

fn compacted_bottom_region_input(
    region: BuiltinHostDrawerRegionInput,
    shell_size: UiSize,
    metrics: WorkbenchChromeMetrics,
    anchors: Option<BuiltinHostDrawerLayoutAnchors>,
) -> BuiltinHostDrawerRegionInput {
    if !region.visible {
        return region;
    }

    let separator = metrics.separator_thickness.max(0.0);
    let available_height = anchors
        .map(|anchors| (anchors.body_height - separator).max(0.0))
        .unwrap_or_else(|| {
            (shell_size.height
                - metrics.top_bar_height.max(0.0)
                - separator
                - metrics.host_bar_height.max(0.0)
                - separator
                - metrics.status_bar_height.max(0.0)
                - separator)
                .max(0.0)
        });
    let extent = compact_bottom_height_limit(available_height)
        .map(|limit| region.extent.min(limit))
        .unwrap_or(region.extent);

    BuiltinHostDrawerRegionInput { extent, ..region }
}

fn resolved_drawer_shell_extent(region: BuiltinHostDrawerRegionInput) -> f32 {
    if region.visible {
        region.extent.max(0.0)
    } else {
        0.0
    }
}

fn resolved_side_panel_extent(region: BuiltinHostDrawerRegionInput, rail_width: f32) -> f32 {
    if region.visible {
        (region.extent - rail_width).max(0.0)
    } else {
        0.0
    }
}

fn resolved_bottom_panel_extent(region: BuiltinHostDrawerRegionInput, header_height: f32) -> f32 {
    if region.visible {
        (region.extent - header_height).max(0.0)
    } else {
        0.0
    }
}

fn resolved_drawer_separator_extent(region: BuiltinHostDrawerRegionInput) -> f32 {
    if region.visible {
        1.0
    } else {
        0.0
    }
}

fn drawer_region_input(
    drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerSnapshot>,
    slots: &[ActivityDrawerSlot],
    collapsed_extent: f32,
) -> BuiltinHostDrawerRegionInput {
    let mut visible = false;
    let mut extent = 0.0_f32;

    for slot in slots {
        let Some(drawer) = drawers.get(slot) else {
            continue;
        };
        if !drawer.visible || drawer.tabs.is_empty() {
            continue;
        }

        visible = true;
        let next_extent = match drawer.mode {
            ActivityDrawerMode::Collapsed => collapsed_extent,
            ActivityDrawerMode::Pinned | ActivityDrawerMode::AutoHide => {
                drawer.extent.max(collapsed_extent)
            }
        };
        extent = extent.max(next_extent);
    }

    BuiltinHostDrawerRegionInput {
        visible,
        extent: if visible { extent.max(0.0) } else { 0.0 },
    }
}

fn surface_control_node_id(surface: &UiSurface, control_id: &str) -> Option<UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}

fn frame_is_visible(frame: &UiFrame) -> bool {
    frame.width > f32::EPSILON && frame.height > f32::EPSILON
}

fn apply_fixed_control_width(
    surface: &mut UiSurface,
    control_id: &str,
    width: f32,
) -> Result<(), BuiltinHostDrawerSourceTemplateBridgeError> {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return Ok(());
    };
    let changed = {
        let Some(node) = surface.tree.node_mut(node_id) else {
            return Ok(());
        };
        let next_width = fixed_axis(width);
        let next_visibility = fixed_extent_visibility(width);
        let changed = node.constraints.width != next_width || node.visibility != next_visibility;
        node.constraints.width = next_width;
        node.visibility = next_visibility;
        changed
    };

    if changed {
        surface.tree.mark_layout_dirty(node_id)?;
    }
    Ok(())
}

fn apply_fixed_control_height(
    surface: &mut UiSurface,
    control_id: &str,
    height: f32,
) -> Result<(), BuiltinHostDrawerSourceTemplateBridgeError> {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return Ok(());
    };
    let changed = {
        let Some(node) = surface.tree.node_mut(node_id) else {
            return Ok(());
        };
        let next_height = fixed_axis(height);
        let next_visibility = fixed_extent_visibility(height);
        let changed = node.constraints.height != next_height || node.visibility != next_visibility;
        node.constraints.height = next_height;
        node.visibility = next_visibility;
        changed
    };

    if changed {
        surface.tree.mark_layout_dirty(node_id)?;
    }
    Ok(())
}

fn fixed_extent_visibility(size: f32) -> UiVisibility {
    if size > f32::EPSILON {
        UiVisibility::Visible
    } else {
        UiVisibility::Collapsed
    }
}

fn fixed_axis(size: f32) -> AxisConstraint {
    AxisConstraint {
        min: size.max(0.0),
        max: size.max(0.0),
        preferred: size.max(0.0),
        priority: 100,
        weight: 1.0,
        stretch_mode: StretchMode::Fixed,
    }
}
