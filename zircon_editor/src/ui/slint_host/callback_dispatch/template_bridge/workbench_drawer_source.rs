use std::collections::BTreeMap;

use thiserror::Error;
use zircon_ui::tree::UiTreeError;
use zircon_ui::{AxisConstraint, StretchMode, UiFrame, UiSize, UiSurface};

use crate::ui::slint_host::callback_dispatch::constants::BUILTIN_WORKBENCH_DRAWER_SOURCE_DOCUMENT_ID;
use crate::ui::template_runtime::{EditorUiHostRuntime, EditorUiHostRuntimeError};
use crate::{
    ActivityDrawerMode, ActivityDrawerSlot, ActivityDrawerSnapshot, WorkbenchChromeMetrics,
    WorkbenchViewModel,
};

const LEFT_DRAWER_SHELL_CONTROL_ID: &str = "LeftDrawerShellRoot";
const LEFT_DRAWER_PANEL_CONTROL_ID: &str = "LeftDrawerPanelRoot";
const LEFT_DRAWER_HEADER_CONTROL_ID: &str = "LeftDrawerHeaderRoot";
const LEFT_DRAWER_CONTENT_CONTROL_ID: &str = "LeftDrawerContentRoot";
const RIGHT_DRAWER_SHELL_CONTROL_ID: &str = "RightDrawerShellRoot";
const RIGHT_DRAWER_PANEL_CONTROL_ID: &str = "RightDrawerPanelRoot";
const RIGHT_DRAWER_HEADER_CONTROL_ID: &str = "RightDrawerHeaderRoot";
const RIGHT_DRAWER_CONTENT_CONTROL_ID: &str = "RightDrawerContentRoot";
const BOTTOM_DRAWER_SHELL_CONTROL_ID: &str = "BottomDrawerShellRoot";
const BOTTOM_DRAWER_PANEL_CONTROL_ID: &str = "BottomDrawerPanelRoot";
const BOTTOM_DRAWER_OUTER_SEPARATOR_CONTROL_ID: &str = "BottomDrawerOuterSeparatorRoot";
const BOTTOM_DRAWER_HEADER_CONTROL_ID: &str = "BottomDrawerHeaderRoot";
const BOTTOM_DRAWER_CONTENT_CONTROL_ID: &str = "BottomDrawerContentRoot";

#[derive(Debug, Error)]
pub(crate) enum BuiltinWorkbenchDrawerSourceTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinWorkbenchDrawerSourceFrames {
    pub left_drawer_shell_frame: Option<UiFrame>,
    pub left_drawer_header_frame: Option<UiFrame>,
    pub left_drawer_content_frame: Option<UiFrame>,
    pub right_drawer_shell_frame: Option<UiFrame>,
    pub right_drawer_header_frame: Option<UiFrame>,
    pub right_drawer_content_frame: Option<UiFrame>,
    pub bottom_drawer_shell_frame: Option<UiFrame>,
    pub bottom_drawer_header_frame: Option<UiFrame>,
    pub bottom_drawer_content_frame: Option<UiFrame>,
}

impl BuiltinWorkbenchDrawerSourceFrames {
    pub(crate) fn control_frame(&self, control_id: &str) -> Option<UiFrame> {
        match control_id {
            LEFT_DRAWER_SHELL_CONTROL_ID => self.left_drawer_shell_frame,
            LEFT_DRAWER_HEADER_CONTROL_ID => self.left_drawer_header_frame,
            LEFT_DRAWER_CONTENT_CONTROL_ID => self.left_drawer_content_frame,
            RIGHT_DRAWER_SHELL_CONTROL_ID => self.right_drawer_shell_frame,
            RIGHT_DRAWER_HEADER_CONTROL_ID => self.right_drawer_header_frame,
            RIGHT_DRAWER_CONTENT_CONTROL_ID => self.right_drawer_content_frame,
            BOTTOM_DRAWER_SHELL_CONTROL_ID => self.bottom_drawer_shell_frame,
            BOTTOM_DRAWER_HEADER_CONTROL_ID => self.bottom_drawer_header_frame,
            BOTTOM_DRAWER_CONTENT_CONTROL_ID => self.bottom_drawer_content_frame,
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BuiltinWorkbenchDrawerRegionInput {
    visible: bool,
    extent: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BuiltinWorkbenchDrawerLayoutInputs {
    left: BuiltinWorkbenchDrawerRegionInput,
    right: BuiltinWorkbenchDrawerRegionInput,
    bottom: BuiltinWorkbenchDrawerRegionInput,
}

impl BuiltinWorkbenchDrawerLayoutInputs {
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
                &[
                    ActivityDrawerSlot::BottomLeft,
                    ActivityDrawerSlot::BottomRight,
                ],
                metrics.panel_header_height,
            ),
        }
    }
}

pub(crate) struct BuiltinWorkbenchDrawerSourceTemplateBridge {
    runtime: EditorUiHostRuntime,
    surface: UiSurface,
}

impl BuiltinWorkbenchDrawerSourceTemplateBridge {
    pub(crate) fn new(
        shell_size: UiSize,
    ) -> Result<Self, BuiltinWorkbenchDrawerSourceTemplateBridgeError> {
        let mut runtime = EditorUiHostRuntime::default();
        runtime.load_builtin_workbench_shell()?;
        let surface = build_builtin_workbench_drawer_source_surface(
            &runtime,
            shell_size,
            BuiltinWorkbenchDrawerLayoutInputs::default(),
            WorkbenchChromeMetrics::default(),
        )?;
        Ok(Self { runtime, surface })
    }

    #[cfg(test)]
    pub(crate) fn recompute_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinWorkbenchDrawerSourceTemplateBridgeError> {
        self.surface = build_builtin_workbench_drawer_source_surface(
            &self.runtime,
            shell_size,
            BuiltinWorkbenchDrawerLayoutInputs::default(),
            WorkbenchChromeMetrics::default(),
        )?;
        Ok(())
    }

    pub(crate) fn recompute_layout_with_workbench_model(
        &mut self,
        shell_size: UiSize,
        model: &WorkbenchViewModel,
        metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinWorkbenchDrawerSourceTemplateBridgeError> {
        self.surface = build_builtin_workbench_drawer_source_surface(
            &self.runtime,
            shell_size,
            BuiltinWorkbenchDrawerLayoutInputs::from_workbench_model(model, metrics),
            *metrics,
        )?;
        Ok(())
    }

    pub(crate) fn control_frame(&self, control_id: &str) -> Option<UiFrame> {
        self.source_frames().control_frame(control_id)
    }

    pub(crate) fn source_frames(&self) -> BuiltinWorkbenchDrawerSourceFrames {
        BuiltinWorkbenchDrawerSourceFrames {
            left_drawer_shell_frame: surface_control_frame(
                &self.surface,
                LEFT_DRAWER_SHELL_CONTROL_ID,
            ),
            left_drawer_header_frame: surface_control_frame(
                &self.surface,
                LEFT_DRAWER_HEADER_CONTROL_ID,
            ),
            left_drawer_content_frame: surface_control_frame(
                &self.surface,
                LEFT_DRAWER_CONTENT_CONTROL_ID,
            ),
            right_drawer_shell_frame: surface_control_frame(
                &self.surface,
                RIGHT_DRAWER_SHELL_CONTROL_ID,
            ),
            right_drawer_header_frame: surface_control_frame(
                &self.surface,
                RIGHT_DRAWER_HEADER_CONTROL_ID,
            ),
            right_drawer_content_frame: surface_control_frame(
                &self.surface,
                RIGHT_DRAWER_CONTENT_CONTROL_ID,
            ),
            bottom_drawer_shell_frame: surface_control_frame(
                &self.surface,
                BOTTOM_DRAWER_SHELL_CONTROL_ID,
            ),
            bottom_drawer_header_frame: surface_control_frame(
                &self.surface,
                BOTTOM_DRAWER_HEADER_CONTROL_ID,
            ),
            bottom_drawer_content_frame: surface_control_frame(
                &self.surface,
                BOTTOM_DRAWER_CONTENT_CONTROL_ID,
            ),
        }
    }
}

fn build_builtin_workbench_drawer_source_surface(
    runtime: &EditorUiHostRuntime,
    shell_size: UiSize,
    drawer_inputs: BuiltinWorkbenchDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
) -> Result<UiSurface, BuiltinWorkbenchDrawerSourceTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_WORKBENCH_DRAWER_SOURCE_DOCUMENT_ID)?;
    apply_builtin_workbench_drawer_source_layout(&mut surface, drawer_inputs, metrics);
    surface.compute_layout(shell_size)?;
    Ok(surface)
}

fn apply_builtin_workbench_drawer_source_layout(
    surface: &mut UiSurface,
    drawer_inputs: BuiltinWorkbenchDrawerLayoutInputs,
    metrics: WorkbenchChromeMetrics,
) {
    let rail_width = metrics.rail_width.max(0.0);
    let header_height = metrics.panel_header_height.max(0.0);

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
        resolved_drawer_separator_extent(drawer_inputs.bottom),
    );
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_SHELL_CONTROL_ID,
        resolved_drawer_shell_extent(drawer_inputs.bottom),
    );
    apply_fixed_control_height(
        surface,
        BOTTOM_DRAWER_PANEL_CONTROL_ID,
        resolved_bottom_panel_extent(drawer_inputs.bottom, header_height),
    );
}

fn resolved_drawer_shell_extent(region: BuiltinWorkbenchDrawerRegionInput) -> f32 {
    if region.visible {
        region.extent.max(0.0)
    } else {
        0.0
    }
}

fn resolved_side_panel_extent(region: BuiltinWorkbenchDrawerRegionInput, rail_width: f32) -> f32 {
    if region.visible {
        (region.extent - rail_width).max(0.0)
    } else {
        0.0
    }
}

fn resolved_bottom_panel_extent(
    region: BuiltinWorkbenchDrawerRegionInput,
    header_height: f32,
) -> f32 {
    if region.visible {
        (region.extent - header_height).max(0.0)
    } else {
        0.0
    }
}

fn resolved_drawer_separator_extent(region: BuiltinWorkbenchDrawerRegionInput) -> f32 {
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
) -> BuiltinWorkbenchDrawerRegionInput {
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

    BuiltinWorkbenchDrawerRegionInput {
        visible,
        extent: if visible { extent.max(0.0) } else { 0.0 },
    }
}

fn surface_control_frame(surface: &UiSurface, control_id: &str) -> Option<UiFrame> {
    let node_id = surface_control_node_id(surface, control_id)?;
    surface
        .tree
        .node(node_id)
        .map(|node| node.layout_cache.frame)
}

fn surface_control_node_id(
    surface: &UiSurface,
    control_id: &str,
) -> Option<zircon_ui::event_ui::UiNodeId> {
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
    node.state_flags.visible = width > f32::EPSILON;
}

fn apply_fixed_control_height(surface: &mut UiSurface, control_id: &str, height: f32) {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return;
    };
    let Some(node) = surface.tree.node_mut(node_id) else {
        return;
    };

    node.constraints.height = fixed_axis(height);
    node.state_flags.visible = height > f32::EPSILON;
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
