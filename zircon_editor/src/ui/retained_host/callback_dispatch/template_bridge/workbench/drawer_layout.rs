use std::collections::BTreeMap;

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::{
    layout::{AxisConstraint, StretchMode, UiFrame, UiSize},
    tree::UiVisibility,
};

use crate::ui::workbench::autolayout::{compact_bottom_height_limit, WorkbenchChromeMetrics};
use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::ActivityDrawerSnapshot;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

pub(super) const LEFT_DRAWER_SHELL_CONTROL_ID: &str = "LeftDrawerShellRoot";
pub(super) const LEFT_DRAWER_HEADER_CONTROL_ID: &str = "LeftDrawerHeaderRoot";
pub(super) const LEFT_DRAWER_CONTENT_CONTROL_ID: &str = "LeftDrawerContentRoot";
pub(super) const RIGHT_DRAWER_SHELL_CONTROL_ID: &str = "RightDrawerShellRoot";
pub(super) const RIGHT_DRAWER_HEADER_CONTROL_ID: &str = "RightDrawerHeaderRoot";
pub(super) const RIGHT_DRAWER_CONTENT_CONTROL_ID: &str = "RightDrawerContentRoot";
pub(super) const BOTTOM_DRAWER_SHELL_CONTROL_ID: &str = "BottomDrawerShellRoot";
pub(super) const BOTTOM_DRAWER_HEADER_CONTROL_ID: &str = "BottomDrawerHeaderRoot";
pub(super) const BOTTOM_DRAWER_CONTENT_CONTROL_ID: &str = "BottomDrawerContentRoot";

const AUTHORED_DRAWER_HEADER_HEIGHT: f32 = 42.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct WorkbenchDrawerRegionInput {
    visible: bool,
    extent: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct WorkbenchDrawerLayoutInputs {
    left: WorkbenchDrawerRegionInput,
    right: WorkbenchDrawerRegionInput,
    bottom: WorkbenchDrawerRegionInput,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WorkbenchDrawerLayoutAnchors {
    body_height: f32,
}

impl WorkbenchDrawerLayoutAnchors {
    fn from_body_frame(body_frame: Option<UiFrame>) -> Option<Self> {
        let body_frame = body_frame.filter(frame_is_visible)?;
        let body_height = body_frame.height.max(0.0);

        Some(Self { body_height })
    }
}

impl WorkbenchDrawerLayoutInputs {
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
                AUTHORED_DRAWER_HEADER_HEIGHT,
            ),
        }
    }
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn recompute_layout_with_workbench_model(
        &mut self,
        shell_size: UiSize,
        model: &WorkbenchViewModel,
        metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.recompute_layout(shell_size)?;
        let anchors = self.componentized_drawer_layout_anchors(shell_size);
        apply_workbench_drawer_layout_to_surface(
            &mut self.template_surface.surface,
            shell_size,
            model,
            metrics,
            anchors,
        )?;
        self.template_surface
            .recompute_layout(self.runtime.as_ref(), shell_size)?;
        Ok(())
    }

    fn componentized_drawer_layout_anchors(
        &self,
        shell_size: UiSize,
    ) -> Option<WorkbenchDrawerLayoutAnchors> {
        let frames = self.template_surface.frames;
        let body_y = frames.top_toolbar.y + frames.top_toolbar.height;
        let body_bottom = frames.status_bar.y.max(body_y);
        let body_frame = UiFrame::new(
            0.0,
            body_y,
            shell_size.width.max(0.0),
            (body_bottom - body_y).max(0.0),
        );
        WorkbenchDrawerLayoutAnchors::from_body_frame(Some(body_frame))
    }
}

fn apply_workbench_drawer_layout_to_surface(
    surface: &mut UiSurface,
    shell_size: UiSize,
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    anchors: Option<WorkbenchDrawerLayoutAnchors>,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    apply_workbench_drawer_layout(
        surface,
        shell_size,
        WorkbenchDrawerLayoutInputs::from_workbench_model(model, metrics),
        *metrics,
        anchors,
    )
}

fn apply_workbench_drawer_layout(
    surface: &mut UiSurface,
    shell_size: UiSize,
    drawer_inputs: WorkbenchDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
    anchors: Option<WorkbenchDrawerLayoutAnchors>,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    let rail_width = metrics.rail_width.max(0.0);
    let bottom = compacted_bottom_region_input(drawer_inputs.bottom, shell_size, metrics, anchors);

    mark_roots_layout_dirty(surface)?;

    apply_fixed_control_width(
        surface,
        LEFT_DRAWER_SHELL_CONTROL_ID,
        resolved_side_panel_extent(drawer_inputs.left, rail_width),
    )?;
    apply_fixed_control_width(
        surface,
        RIGHT_DRAWER_SHELL_CONTROL_ID,
        resolved_side_panel_extent(drawer_inputs.right, rail_width),
    )?;
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_SHELL_CONTROL_ID,
        resolved_drawer_shell_extent(bottom),
    )?;
    Ok(())
}

fn mark_roots_layout_dirty(
    surface: &mut UiSurface,
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
    for root_id in surface.tree.roots.clone() {
        surface.tree.mark_layout_dirty(root_id)?;
    }
    Ok(())
}

fn compacted_bottom_region_input(
    region: WorkbenchDrawerRegionInput,
    shell_size: UiSize,
    metrics: WorkbenchChromeMetrics,
    anchors: Option<WorkbenchDrawerLayoutAnchors>,
) -> WorkbenchDrawerRegionInput {
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

    WorkbenchDrawerRegionInput { extent, ..region }
}

fn resolved_drawer_shell_extent(region: WorkbenchDrawerRegionInput) -> f32 {
    if region.visible {
        region.extent.max(0.0)
    } else {
        0.0
    }
}

fn resolved_side_panel_extent(region: WorkbenchDrawerRegionInput, rail_width: f32) -> f32 {
    if region.visible {
        (region.extent - rail_width).max(0.0)
    } else {
        0.0
    }
}

fn drawer_region_input(
    drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerSnapshot>,
    slots: &[ActivityDrawerSlot],
    collapsed_extent: f32,
) -> WorkbenchDrawerRegionInput {
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

    WorkbenchDrawerRegionInput {
        visible,
        extent: if visible { extent.max(0.0) } else { 0.0 },
    }
}

fn surface_control_node_id(
    surface: &UiSurface,
    control_id: &str,
) -> Option<zircon_runtime_interface::ui::event_ui::UiNodeId> {
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
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
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
) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
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
