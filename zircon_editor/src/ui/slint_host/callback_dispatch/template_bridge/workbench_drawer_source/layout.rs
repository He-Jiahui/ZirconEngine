use std::collections::BTreeMap;

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{AxisConstraint, StretchMode, UiSize},
    tree::UiVisibility,
};

use crate::ui::slint_host::callback_dispatch::constants::BUILTIN_HOST_DRAWER_SOURCE_DOCUMENT_ID;
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::autolayout::{compact_bottom_height_limit, WorkbenchChromeMetrics};
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ActivityDrawerSnapshot;

use super::control_ids::{
    BOTTOM_DRAWER_OUTER_SEPARATOR_CONTROL_ID, BOTTOM_DRAWER_PANEL_CONTROL_ID,
    BOTTOM_DRAWER_SHELL_CONTROL_ID, LEFT_DRAWER_PANEL_CONTROL_ID, LEFT_DRAWER_SHELL_CONTROL_ID,
    RIGHT_DRAWER_PANEL_CONTROL_ID, RIGHT_DRAWER_SHELL_CONTROL_ID,
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
    let mut surface = runtime.build_shared_surface(BUILTIN_HOST_DRAWER_SOURCE_DOCUMENT_ID)?;
    apply_builtin_host_drawer_source_layout(&mut surface, shell_size, drawer_inputs, metrics);
    surface.compute_layout(shell_size)?;
    Ok(surface)
}

fn apply_builtin_host_drawer_source_layout(
    surface: &mut UiSurface,
    shell_size: UiSize,
    drawer_inputs: BuiltinHostDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
) {
    let rail_width = metrics.rail_width.max(0.0);
    let header_height = metrics.panel_header_height.max(0.0);
    let bottom = compacted_bottom_region_input(drawer_inputs.bottom, shell_size, metrics);

    apply_fixed_control_width(
        surface,
        LEFT_DRAWER_SHELL_CONTROL_ID,
        resolved_drawer_shell_extent(drawer_inputs.left),
    );
    apply_fixed_control_width(
        surface,
        LEFT_DRAWER_PANEL_CONTROL_ID,
        resolved_side_panel_extent(drawer_inputs.left, rail_width),
    );
    apply_fixed_control_width(
        surface,
        RIGHT_DRAWER_SHELL_CONTROL_ID,
        resolved_drawer_shell_extent(drawer_inputs.right),
    );
    apply_fixed_control_width(
        surface,
        RIGHT_DRAWER_PANEL_CONTROL_ID,
        resolved_side_panel_extent(drawer_inputs.right, rail_width),
    );
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_OUTER_SEPARATOR_CONTROL_ID,
        resolved_drawer_separator_extent(bottom),
    );
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_SHELL_CONTROL_ID,
        resolved_drawer_shell_extent(bottom),
    );
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_PANEL_CONTROL_ID,
        resolved_bottom_panel_extent(bottom, header_height),
    );
}

fn compacted_bottom_region_input(
    region: BuiltinHostDrawerRegionInput,
    shell_size: UiSize,
    metrics: WorkbenchChromeMetrics,
) -> BuiltinHostDrawerRegionInput {
    if !region.visible {
        return region;
    }

    let separator = metrics.separator_thickness.max(0.0);
    let available_height = (shell_size.height
        - metrics.top_bar_height.max(0.0)
        - separator
        - metrics.host_bar_height.max(0.0)
        - separator
        - metrics.status_bar_height.max(0.0)
        - separator)
        .max(0.0);
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

fn apply_fixed_control_width(surface: &mut UiSurface, control_id: &str, width: f32) {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return;
    };
    let Some(node) = surface.tree.node_mut(node_id) else {
        return;
    };

    node.constraints.width = fixed_axis(width);
    node.visibility = fixed_extent_visibility(width);
}

fn apply_fixed_control_height(surface: &mut UiSurface, control_id: &str, height: f32) {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return;
    };
    let Some(node) = surface.tree.node_mut(node_id) else {
        return;
    };

    node.constraints.height = fixed_axis(height);
    node.visibility = fixed_extent_visibility(height);
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
