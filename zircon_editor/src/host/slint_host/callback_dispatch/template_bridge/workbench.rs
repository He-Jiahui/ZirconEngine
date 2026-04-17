use std::collections::BTreeMap;

use thiserror::Error;
use zircon_editor_ui::{DockCommand, EditorUiBinding, EditorUiBindingPayload};
use zircon_ui::{
    Anchor, AxisConstraint, BoxConstraints, Pivot, Position, StretchMode, UiBindingValue,
    UiEventKind, UiFrame, UiSize, UiSurface,
};

use crate::host::slint_host::callback_dispatch::constants::{
    BUILTIN_WORKBENCH_DOCUMENT_ID, DOCUMENT_TABS_CONTROL_ID, WORKBENCH_SHELL_CONTROL_ID,
};
use crate::host::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, SlintUiHostProjection, SlintUiProjection,
};
use crate::{
    ActivityDrawerMode, ActivityDrawerSlot, EditorChromeSnapshot, ShellRegionId,
    WorkbenchChromeMetrics,
};

use super::{binding_for_control, build_bindings_by_id, load_builtin_runtime_projection};

const WORKBENCH_BODY_CONTROL_ID: &str = "WorkbenchBody";
const HOST_PAGE_STRIP_CONTROL_ID: &str = "HostPageStripRoot";
const LEFT_DRAWER_SHELL_CONTROL_ID: &str = "LeftDrawerShellRoot";
const LEFT_DRAWER_HEADER_CONTROL_ID: &str = "LeftDrawerHeaderRoot";
const LEFT_DRAWER_CONTENT_CONTROL_ID: &str = "LeftDrawerContentRoot";
const RIGHT_DRAWER_SHELL_CONTROL_ID: &str = "RightDrawerShellRoot";
const RIGHT_DRAWER_HEADER_CONTROL_ID: &str = "RightDrawerHeaderRoot";
const RIGHT_DRAWER_CONTENT_CONTROL_ID: &str = "RightDrawerContentRoot";
const BOTTOM_DRAWER_SHELL_CONTROL_ID: &str = "BottomDrawerShellRoot";
const BOTTOM_DRAWER_HEADER_CONTROL_ID: &str = "BottomDrawerHeaderRoot";
const BOTTOM_DRAWER_CONTENT_CONTROL_ID: &str = "BottomDrawerContentRoot";

#[derive(Debug, Error)]
pub(crate) enum BuiltinWorkbenchTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
    #[error(transparent)]
    Layout(#[from] zircon_ui::UiTreeError),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinWorkbenchRootShellFrames {
    pub shell_frame: Option<UiFrame>,
    pub menu_bar_frame: Option<UiFrame>,
    pub activity_rail_frame: Option<UiFrame>,
    pub host_page_strip_frame: Option<UiFrame>,
    pub workbench_body_frame: Option<UiFrame>,
    pub left_drawer_shell_frame: Option<UiFrame>,
    pub left_drawer_header_frame: Option<UiFrame>,
    pub left_drawer_content_frame: Option<UiFrame>,
    pub right_drawer_shell_frame: Option<UiFrame>,
    pub right_drawer_header_frame: Option<UiFrame>,
    pub right_drawer_content_frame: Option<UiFrame>,
    pub bottom_drawer_shell_frame: Option<UiFrame>,
    pub bottom_drawer_header_frame: Option<UiFrame>,
    pub bottom_drawer_content_frame: Option<UiFrame>,
    pub document_host_frame: Option<UiFrame>,
    pub document_tabs_frame: Option<UiFrame>,
    pub pane_surface_frame: Option<UiFrame>,
    pub status_bar_frame: Option<UiFrame>,
}

impl BuiltinWorkbenchRootShellFrames {
    pub(crate) fn drawer_shell_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_drawer_shell_frame,
            ShellRegionId::Right => self.right_drawer_shell_frame,
            ShellRegionId::Bottom => self.bottom_drawer_shell_frame,
            ShellRegionId::Document => None,
        }
    }

    pub(crate) fn drawer_header_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_drawer_header_frame,
            ShellRegionId::Right => self.right_drawer_header_frame,
            ShellRegionId::Bottom => self.bottom_drawer_header_frame,
            ShellRegionId::Document => None,
        }
    }

    pub(crate) fn drawer_content_frame(&self, region: ShellRegionId) -> Option<UiFrame> {
        match region {
            ShellRegionId::Left => self.left_drawer_content_frame,
            ShellRegionId::Right => self.right_drawer_content_frame,
            ShellRegionId::Bottom => self.bottom_drawer_content_frame,
            ShellRegionId::Document => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinWorkbenchDrawerRegionInput {
    pub visible: bool,
    pub extent: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct BuiltinWorkbenchDrawerLayoutInputs {
    pub left: BuiltinWorkbenchDrawerRegionInput,
    pub right: BuiltinWorkbenchDrawerRegionInput,
    pub bottom: BuiltinWorkbenchDrawerRegionInput,
}

impl BuiltinWorkbenchDrawerLayoutInputs {
    pub(crate) fn from_chrome_snapshot(
        chrome: &EditorChromeSnapshot,
        metrics: &WorkbenchChromeMetrics,
    ) -> Self {
        Self {
            left: drawer_region_input(
                chrome,
                &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
                metrics.rail_width,
            ),
            right: drawer_region_input(
                chrome,
                &[
                    ActivityDrawerSlot::RightTop,
                    ActivityDrawerSlot::RightBottom,
                ],
                metrics.rail_width,
            ),
            bottom: drawer_region_input(
                chrome,
                &[
                    ActivityDrawerSlot::BottomLeft,
                    ActivityDrawerSlot::BottomRight,
                ],
                metrics.panel_header_height,
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BuiltinWorkbenchDrawerResolvedFrames {
    shell: UiFrame,
    header: UiFrame,
    content: UiFrame,
}

pub(crate) struct BuiltinWorkbenchTemplateBridge {
    runtime: EditorUiHostRuntime,
    projection: SlintUiProjection,
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_projection: SlintUiHostProjection,
}

impl BuiltinWorkbenchTemplateBridge {
    pub(crate) fn new(shell_size: UiSize) -> Result<Self, BuiltinWorkbenchTemplateBridgeError> {
        let (runtime, projection) = load_builtin_runtime_projection(BUILTIN_WORKBENCH_DOCUMENT_ID)?;
        let bindings_by_id = build_bindings_by_id(&projection);
        let host_projection = build_builtin_workbench_host_projection(
            &runtime,
            &projection,
            shell_size,
            BuiltinWorkbenchDrawerLayoutInputs::default(),
        )?;

        Ok(Self {
            runtime,
            projection,
            bindings_by_id,
            host_projection,
        })
    }

    pub(crate) fn recompute_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinWorkbenchTemplateBridgeError> {
        self.recompute_layout_with_drawer_inputs(
            shell_size,
            BuiltinWorkbenchDrawerLayoutInputs::default(),
        )
    }

    pub(crate) fn recompute_layout_with_chrome(
        &mut self,
        shell_size: UiSize,
        chrome: &EditorChromeSnapshot,
        metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinWorkbenchTemplateBridgeError> {
        self.recompute_layout_with_drawer_inputs(
            shell_size,
            BuiltinWorkbenchDrawerLayoutInputs::from_chrome_snapshot(chrome, metrics),
        )
    }

    pub(crate) fn recompute_layout_with_drawer_inputs(
        &mut self,
        shell_size: UiSize,
        drawer_inputs: BuiltinWorkbenchDrawerLayoutInputs,
    ) -> Result<(), BuiltinWorkbenchTemplateBridgeError> {
        self.host_projection = build_builtin_workbench_host_projection(
            &self.runtime,
            &self.projection,
            shell_size,
            drawer_inputs,
        )?;
        Ok(())
    }

    #[cfg(test)]
    #[cfg(test)]
    pub(crate) fn host_projection(&self) -> &SlintUiHostProjection {
        &self.host_projection
    }

    pub(crate) fn binding_for_control(
        &self,
        control_id: &str,
        event_kind: UiEventKind,
    ) -> Option<&EditorUiBinding> {
        binding_for_control(
            &self.bindings_by_id,
            &self.host_projection,
            control_id,
            event_kind,
        )
    }

    pub(crate) fn control_frame(&self, control_id: &str) -> Option<UiFrame> {
        self.host_projection
            .node_by_control_id(control_id)
            .map(|node| node.frame)
    }

    pub(crate) fn root_shell_frames(&self) -> BuiltinWorkbenchRootShellFrames {
        BuiltinWorkbenchRootShellFrames {
            shell_frame: self.control_frame(WORKBENCH_SHELL_CONTROL_ID),
            menu_bar_frame: self.control_frame("WorkbenchMenuBarRoot"),
            activity_rail_frame: self.control_frame("ActivityRailRoot"),
            host_page_strip_frame: self.control_frame(HOST_PAGE_STRIP_CONTROL_ID),
            workbench_body_frame: self.control_frame(WORKBENCH_BODY_CONTROL_ID),
            left_drawer_shell_frame: self.control_frame(LEFT_DRAWER_SHELL_CONTROL_ID),
            left_drawer_header_frame: self.control_frame(LEFT_DRAWER_HEADER_CONTROL_ID),
            left_drawer_content_frame: self.control_frame(LEFT_DRAWER_CONTENT_CONTROL_ID),
            right_drawer_shell_frame: self.control_frame(RIGHT_DRAWER_SHELL_CONTROL_ID),
            right_drawer_header_frame: self.control_frame(RIGHT_DRAWER_HEADER_CONTROL_ID),
            right_drawer_content_frame: self.control_frame(RIGHT_DRAWER_CONTENT_CONTROL_ID),
            bottom_drawer_shell_frame: self.control_frame(BOTTOM_DRAWER_SHELL_CONTROL_ID),
            bottom_drawer_header_frame: self.control_frame(BOTTOM_DRAWER_HEADER_CONTROL_ID),
            bottom_drawer_content_frame: self.control_frame(BOTTOM_DRAWER_CONTENT_CONTROL_ID),
            document_host_frame: self.control_frame("DocumentHostRoot"),
            document_tabs_frame: self.control_frame(DOCUMENT_TABS_CONTROL_ID),
            pane_surface_frame: self.control_frame("PaneSurfaceRoot"),
            status_bar_frame: self.control_frame("StatusBarRoot"),
        }
    }

    pub(crate) fn activity_binding_for_target(
        &self,
        slot: &str,
        instance_id: &str,
    ) -> Option<&EditorUiBinding> {
        self.bindings_by_id.values().find(|binding| {
            matches!(
                binding.payload(),
                EditorUiBindingPayload::DockCommand(DockCommand::ActivateDrawerTab {
                    slot: binding_slot,
                    instance_id: binding_instance_id,
                }) if binding_slot == slot && binding_instance_id == instance_id
            )
        })
    }

    fn binding_for_control_with_arguments(
        &self,
        control_id: &str,
        event_kind: UiEventKind,
        arguments: Vec<UiBindingValue>,
    ) -> Option<EditorUiBinding> {
        self.binding_for_control(control_id, event_kind)?
            .with_arguments(arguments)
            .ok()
    }

    pub(crate) fn document_tab_activation_binding(
        &self,
        instance_id: &str,
    ) -> Option<EditorUiBinding> {
        self.binding_for_control_with_arguments(
            DOCUMENT_TABS_CONTROL_ID,
            UiEventKind::Change,
            vec![UiBindingValue::string(instance_id)],
        )
    }

    pub(crate) fn document_tab_close_binding(&self, instance_id: &str) -> Option<EditorUiBinding> {
        self.binding_for_control_with_arguments(
            DOCUMENT_TABS_CONTROL_ID,
            UiEventKind::Submit,
            vec![UiBindingValue::string(instance_id)],
        )
    }

    pub(crate) fn host_page_activation_binding(&self, page_id: &str) -> Option<EditorUiBinding> {
        self.binding_for_control_with_arguments(
            WORKBENCH_SHELL_CONTROL_ID,
            UiEventKind::Change,
            vec![UiBindingValue::string(page_id)],
        )
    }
}

fn build_builtin_workbench_host_projection(
    runtime: &EditorUiHostRuntime,
    projection: &SlintUiProjection,
    shell_size: UiSize,
    drawer_inputs: BuiltinWorkbenchDrawerLayoutInputs,
) -> Result<SlintUiHostProjection, BuiltinWorkbenchTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_WORKBENCH_DOCUMENT_ID)?;
    surface.compute_layout(shell_size)?;
    apply_builtin_workbench_host_strip_layout(&mut surface);
    apply_builtin_workbench_drawer_layout(&mut surface, drawer_inputs);
    surface.compute_layout(shell_size)?;
    Ok(runtime.build_slint_host_projection_with_surface(projection, &surface)?)
}

fn apply_builtin_workbench_host_strip_layout(surface: &mut UiSurface) {
    let Some(shell_frame) = surface_control_frame(surface, WORKBENCH_SHELL_CONTROL_ID) else {
        return;
    };

    let metrics = WorkbenchChromeMetrics::default();
    apply_fixed_control_frame(
        surface,
        HOST_PAGE_STRIP_CONTROL_ID,
        UiFrame::new(
            shell_frame.x,
            shell_frame.y + metrics.top_bar_height + metrics.separator_thickness,
            shell_frame.width.max(0.0),
            metrics.host_bar_height.max(0.0),
        ),
    );
}

fn drawer_region_input(
    chrome: &EditorChromeSnapshot,
    slots: &[ActivityDrawerSlot],
    collapsed_extent: f32,
) -> BuiltinWorkbenchDrawerRegionInput {
    let mut visible = false;
    let mut extent = 0.0_f32;

    for slot in slots {
        let Some(drawer) = chrome.workbench.drawers.get(slot) else {
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

fn apply_builtin_workbench_drawer_layout(
    surface: &mut UiSurface,
    drawer_inputs: BuiltinWorkbenchDrawerLayoutInputs,
) {
    let Some(body_frame) = surface_control_frame(surface, WORKBENCH_BODY_CONTROL_ID) else {
        return;
    };

    let metrics = WorkbenchChromeMetrics::default();
    let resolved = BuiltinWorkbenchResolvedDrawerFrames {
        left: resolve_drawer_frames(
            ShellRegionId::Left,
            body_frame,
            drawer_inputs.left,
            drawer_inputs.bottom,
            metrics,
        ),
        right: resolve_drawer_frames(
            ShellRegionId::Right,
            body_frame,
            drawer_inputs.right,
            drawer_inputs.bottom,
            metrics,
        ),
        bottom: resolve_drawer_frames(
            ShellRegionId::Bottom,
            body_frame,
            drawer_inputs.bottom,
            drawer_inputs.bottom,
            metrics,
        ),
    };

    apply_drawer_frames(
        surface,
        LEFT_DRAWER_SHELL_CONTROL_ID,
        LEFT_DRAWER_HEADER_CONTROL_ID,
        LEFT_DRAWER_CONTENT_CONTROL_ID,
        resolved.left,
    );
    apply_drawer_frames(
        surface,
        RIGHT_DRAWER_SHELL_CONTROL_ID,
        RIGHT_DRAWER_HEADER_CONTROL_ID,
        RIGHT_DRAWER_CONTENT_CONTROL_ID,
        resolved.right,
    );
    apply_drawer_frames(
        surface,
        BOTTOM_DRAWER_SHELL_CONTROL_ID,
        BOTTOM_DRAWER_HEADER_CONTROL_ID,
        BOTTOM_DRAWER_CONTENT_CONTROL_ID,
        resolved.bottom,
    );
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BuiltinWorkbenchResolvedDrawerFrames {
    left: BuiltinWorkbenchDrawerResolvedFrames,
    right: BuiltinWorkbenchDrawerResolvedFrames,
    bottom: BuiltinWorkbenchDrawerResolvedFrames,
}

fn resolve_drawer_frames(
    region: ShellRegionId,
    body_frame: UiFrame,
    region_input: BuiltinWorkbenchDrawerRegionInput,
    bottom_input: BuiltinWorkbenchDrawerRegionInput,
    metrics: WorkbenchChromeMetrics,
) -> BuiltinWorkbenchDrawerResolvedFrames {
    if !region_input.visible || region_input.extent <= f32::EPSILON {
        return Default::default();
    }

    let separator = metrics.separator_thickness.max(0.0);
    let header_height = metrics.panel_header_height.max(0.0);
    let bottom_visible = bottom_input.visible && bottom_input.extent > f32::EPSILON;
    let bottom_extent = if bottom_visible {
        bottom_input.extent.max(0.0)
    } else {
        0.0
    };
    let center_height =
        (body_frame.height - bottom_extent - if bottom_visible { separator } else { 0.0 }).max(0.0);

    let shell = match region {
        ShellRegionId::Left => UiFrame::new(
            body_frame.x,
            body_frame.y,
            region_input.extent.max(0.0),
            center_height,
        ),
        ShellRegionId::Right => UiFrame::new(
            body_frame.x + body_frame.width - region_input.extent.max(0.0),
            body_frame.y,
            region_input.extent.max(0.0),
            center_height,
        ),
        ShellRegionId::Bottom => UiFrame::new(
            body_frame.x,
            body_frame.y + body_frame.height - region_input.extent.max(0.0),
            body_frame.width.max(0.0),
            region_input.extent.max(0.0),
        ),
        ShellRegionId::Document => UiFrame::default(),
    };

    if shell.width <= f32::EPSILON || shell.height <= f32::EPSILON {
        return Default::default();
    }

    let (panel_x, panel_width) = match region {
        ShellRegionId::Left => {
            let panel_x = metrics.rail_width.max(0.0) + separator;
            (panel_x, (shell.width - panel_x).max(0.0))
        }
        ShellRegionId::Right => (
            0.0,
            (shell.width - metrics.rail_width.max(0.0) - separator).max(0.0),
        ),
        ShellRegionId::Bottom => (0.0, shell.width.max(0.0)),
        ShellRegionId::Document => (0.0, 0.0),
    };
    let header = if panel_width > f32::EPSILON {
        UiFrame::new(panel_x, 0.0, panel_width, header_height)
    } else {
        UiFrame::default()
    };
    let content = if panel_width > f32::EPSILON && shell.height > header_height + separator {
        UiFrame::new(
            panel_x,
            header_height + separator,
            panel_width,
            (shell.height - header_height - separator).max(0.0),
        )
    } else {
        UiFrame::default()
    };

    BuiltinWorkbenchDrawerResolvedFrames {
        shell,
        header,
        content,
    }
}

fn apply_drawer_frames(
    surface: &mut UiSurface,
    shell_control_id: &str,
    header_control_id: &str,
    content_control_id: &str,
    frames: BuiltinWorkbenchDrawerResolvedFrames,
) {
    apply_fixed_control_frame(surface, shell_control_id, frames.shell);
    apply_fixed_control_frame(surface, header_control_id, frames.header);
    apply_fixed_control_frame(surface, content_control_id, frames.content);
}

fn surface_control_frame(surface: &UiSurface, control_id: &str) -> Option<UiFrame> {
    let node_id = surface_control_node_id(surface, control_id)?;
    surface
        .tree
        .node(node_id)
        .map(|node| node.layout_cache.frame)
}

fn surface_control_node_id(surface: &UiSurface, control_id: &str) -> Option<zircon_ui::UiNodeId> {
    surface.tree.nodes.values().find_map(|node| {
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref())
            .filter(|candidate| *candidate == control_id)
            .map(|_| node.node_id)
    })
}

fn apply_fixed_control_frame(surface: &mut UiSurface, control_id: &str, frame: UiFrame) {
    let Some(node_id) = surface_control_node_id(surface, control_id) else {
        return;
    };
    let Some(node) = surface.tree.node_mut(node_id) else {
        return;
    };

    node.anchor = Anchor::default();
    node.pivot = Pivot::default();
    node.position = Position::new(frame.x, frame.y);
    node.constraints = fixed_box_constraints(frame.width, frame.height);
    node.state_flags.visible = frame.width > f32::EPSILON && frame.height > f32::EPSILON;
}

fn fixed_box_constraints(width: f32, height: f32) -> BoxConstraints {
    BoxConstraints {
        width: fixed_axis(width),
        height: fixed_axis(height),
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
