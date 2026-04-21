use std::collections::BTreeMap;

use zircon_runtime::ui::{
    binding::{UiBindingValue, UiEventKind},
    layout::{UiFrame, UiSize},
};

use crate::ui::binding::{DockCommand, EditorUiBinding, EditorUiBindingPayload};
use crate::ui::slint_host::callback_dispatch::constants::{
    BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID, DOCUMENT_TABS_CONTROL_ID, UI_HOST_WINDOW_CONTROL_ID,
};
use crate::ui::template_runtime::{EditorUiHostRuntime, SlintUiHostProjection, SlintUiProjection};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::super::projection_support::{
    binding_for_control, build_bindings_by_id, load_builtin_runtime_projection,
};
use super::super::workbench_drawer_source::BuiltinWorkbenchDrawerSourceTemplateBridge;
use super::error::BuiltinWorkbenchTemplateBridgeError;
use super::host_projection::build_builtin_workbench_host_projection;
use super::root_shell_frames::BuiltinWorkbenchRootShellFrames;

const WORKBENCH_BODY_CONTROL_ID: &str = "WorkbenchBody";
pub(super) const HOST_PAGE_STRIP_CONTROL_ID: &str = "HostPageStripRoot";
const LEFT_DRAWER_SHELL_CONTROL_ID: &str = "LeftDrawerShellRoot";
const LEFT_DRAWER_HEADER_CONTROL_ID: &str = "LeftDrawerHeaderRoot";
const LEFT_DRAWER_CONTENT_CONTROL_ID: &str = "LeftDrawerContentRoot";
const RIGHT_DRAWER_SHELL_CONTROL_ID: &str = "RightDrawerShellRoot";
const RIGHT_DRAWER_HEADER_CONTROL_ID: &str = "RightDrawerHeaderRoot";
const RIGHT_DRAWER_CONTENT_CONTROL_ID: &str = "RightDrawerContentRoot";
const BOTTOM_DRAWER_SHELL_CONTROL_ID: &str = "BottomDrawerShellRoot";
const BOTTOM_DRAWER_HEADER_CONTROL_ID: &str = "BottomDrawerHeaderRoot";
const BOTTOM_DRAWER_CONTENT_CONTROL_ID: &str = "BottomDrawerContentRoot";

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

    #[cfg(test)]
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
