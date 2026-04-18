use std::collections::BTreeMap;

use thiserror::Error;
use zircon_editor_ui::{DockCommand, EditorUiBinding, EditorUiBindingPayload};
use zircon_ui::{
    Anchor, AxisConstraint, BoxConstraints, Pivot, Position, StretchMode, UiBindingValue,
    UiEventKind, UiFrame, UiSize, UiSurface,
};

use crate::host::slint_host::callback_dispatch::constants::{
    BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID, DOCUMENT_TABS_CONTROL_ID, UI_HOST_WINDOW_CONTROL_ID,
};
use crate::host::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, SlintUiHostProjection, SlintUiProjection,
};
use crate::{ShellRegionId, WorkbenchChromeMetrics, WorkbenchViewModel};

use super::{
    binding_for_control, build_bindings_by_id, load_builtin_runtime_projection,
    workbench_drawer_source::BuiltinWorkbenchDrawerSourceTemplateBridgeError,
    BuiltinWorkbenchDrawerSourceTemplateBridge,
};

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
    DrawerSource(#[from] BuiltinWorkbenchDrawerSourceTemplateBridgeError),
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

pub(crate) struct BuiltinWorkbenchTemplateBridge {
    runtime: EditorUiHostRuntime,
    projection: SlintUiProjection,
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_projection: SlintUiHostProjection,
    drawer_source_bridge: BuiltinWorkbenchDrawerSourceTemplateBridge,
}

impl BuiltinWorkbenchTemplateBridge {
    pub(crate) fn new(shell_size: UiSize) -> Result<Self, BuiltinWorkbenchTemplateBridgeError> {
        let (runtime, projection) =
            load_builtin_runtime_projection(BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID)?;
        let bindings_by_id = build_bindings_by_id(&projection);
        let host_projection =
            build_builtin_workbench_host_projection(&runtime, &projection, shell_size)?;
        let drawer_source_bridge = BuiltinWorkbenchDrawerSourceTemplateBridge::new(shell_size)?;

        Ok(Self {
            runtime,
            projection,
            bindings_by_id,
            host_projection,
            drawer_source_bridge,
        })
    }

    pub(crate) fn recompute_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinWorkbenchTemplateBridgeError> {
        self.host_projection =
            build_builtin_workbench_host_projection(&self.runtime, &self.projection, shell_size)?;
        self.drawer_source_bridge.recompute_layout(shell_size)?;
        Ok(())
    }

    pub(crate) fn recompute_layout_with_workbench_model(
        &mut self,
        shell_size: UiSize,
        model: &WorkbenchViewModel,
        metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinWorkbenchTemplateBridgeError> {
        self.host_projection =
            build_builtin_workbench_host_projection(&self.runtime, &self.projection, shell_size)?;
        self.drawer_source_bridge
            .recompute_layout_with_workbench_model(shell_size, model, metrics)?;
        Ok(())
    }

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
        self.drawer_source_bridge
            .control_frame(control_id)
            .or_else(|| {
                self.host_projection
                    .node_by_control_id(control_id)
                    .map(|node| node.frame)
            })
    }

    pub(crate) fn root_shell_frames(&self) -> BuiltinWorkbenchRootShellFrames {
        BuiltinWorkbenchRootShellFrames {
            shell_frame: self.control_frame(UI_HOST_WINDOW_CONTROL_ID),
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
            UI_HOST_WINDOW_CONTROL_ID,
            UiEventKind::Change,
            vec![UiBindingValue::string(page_id)],
        )
    }
}

fn build_builtin_workbench_host_projection(
    runtime: &EditorUiHostRuntime,
    projection: &SlintUiProjection,
    shell_size: UiSize,
) -> Result<SlintUiHostProjection, BuiltinWorkbenchTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID)?;
    surface.compute_layout(shell_size)?;
    apply_builtin_workbench_host_strip_layout(&mut surface);
    surface.compute_layout(shell_size)?;
    Ok(runtime.build_slint_host_projection_with_surface(projection, &surface)?)
}

fn apply_builtin_workbench_host_strip_layout(surface: &mut UiSurface) {
    let Some(shell_frame) = surface_control_frame(surface, UI_HOST_WINDOW_CONTROL_ID) else {
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
