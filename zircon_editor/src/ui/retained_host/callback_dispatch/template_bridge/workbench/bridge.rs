use std::{collections::BTreeMap, sync::Arc};

use zircon_runtime::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    binding::{UiBindingValue, UiEventKind},
    layout::{UiFrame, UiSize},
};

use crate::ui::binding::{DockCommand, EditorUiBinding, EditorUiBindingPayload};
use crate::ui::retained_host::callback_dispatch::constants::{
    BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID, DOCUMENT_TABS_CONTROL_ID, UI_HOST_WINDOW_CONTROL_ID,
};
use crate::ui::template_runtime::{
    EditorUiHostRuntime, RetainedUiHostProjection, RetainedUiProjection,
};
use crate::ui::workbench::autolayout::WorkbenchChromeMetrics;
use crate::ui::workbench::model::WorkbenchViewModel;

#[cfg(test)]
use super::super::projection_support::load_builtin_runtime;
use super::super::projection_support::{
    binding_for_control, build_bindings_by_id, project_builtin_document_with_runtime,
};
use super::error::BuiltinHostWindowTemplateBridgeError;
use super::host_projection::{
    build_builtin_host_window_surface, project_builtin_host_window_projection,
    rebuild_builtin_host_window_surface,
};
use super::outer_shell_frames::BuiltinHostOuterShellFrames;
use super::root_shell_frames::BuiltinHostRootShellFrames;

const HOST_BODY_CONTROL_ID: &str = "WorkbenchBody";
pub(super) const HOST_PAGE_STRIP_CONTROL_ID: &str = "HostPageStripRoot";

pub(crate) struct BuiltinHostWindowTemplateBridge {
    runtime: Arc<EditorUiHostRuntime>,
    projection: RetainedUiProjection,
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_surface: UiSurface,
    host_projection: RetainedUiHostProjection,
    presentation_scale_factor: f32,
}

impl BuiltinHostWindowTemplateBridge {
    #[cfg(test)]
    pub(crate) fn new(shell_size: UiSize) -> Result<Self, BuiltinHostWindowTemplateBridgeError> {
        let runtime = Arc::new(load_builtin_runtime()?);
        Self::new_with_runtime(runtime, shell_size)
    }

    pub(crate) fn new_with_runtime(
        runtime: Arc<EditorUiHostRuntime>,
        shell_size: UiSize,
    ) -> Result<Self, BuiltinHostWindowTemplateBridgeError> {
        let projection =
            project_builtin_document_with_runtime(&runtime, BUILTIN_UI_HOST_WINDOW_DOCUMENT_ID)?;
        let bindings_by_id = build_bindings_by_id(&projection);
        let host_surface = build_builtin_host_window_surface(runtime.as_ref(), shell_size)?;
        let host_projection =
            project_builtin_host_window_projection(runtime.as_ref(), &projection, &host_surface)?;

        Ok(Self {
            runtime,
            projection,
            bindings_by_id,
            host_surface,
            host_projection,
            presentation_scale_factor: 1.0,
        })
    }

    #[cfg(test)]
    pub(crate) fn recompute_layout(
        &mut self,
        shell_size: UiSize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.presentation_scale_factor = 1.0;
        rebuild_builtin_host_window_surface(&mut self.host_surface, shell_size)?;
        self.host_projection = project_builtin_host_window_projection(
            self.runtime.as_ref(),
            &self.projection,
            &self.host_surface,
        )?;
        Ok(())
    }

    pub(crate) fn recompute_layout_with_workbench_model(
        &mut self,
        shell_size: UiSize,
        _model: &WorkbenchViewModel,
        _metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.recompute_layout_with_workbench_model_at_scale(shell_size, 1.0, _model, _metrics)
    }

    pub(crate) fn recompute_layout_with_workbench_model_at_scale(
        &mut self,
        physical_shell_size: UiSize,
        scale_factor: f32,
        _model: &WorkbenchViewModel,
        _metrics: &WorkbenchChromeMetrics,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        self.presentation_scale_factor = normalized_scale_factor(scale_factor);
        let logical_shell_size = UiSize::new(
            physical_shell_size.width / self.presentation_scale_factor,
            physical_shell_size.height / self.presentation_scale_factor,
        );
        rebuild_builtin_host_window_surface(&mut self.host_surface, logical_shell_size)?;
        self.host_projection = project_builtin_host_window_projection(
            self.runtime.as_ref(),
            &self.projection,
            &self.host_surface,
        )?;
        Ok(())
    }

    pub(crate) fn host_projection(&self) -> &RetainedUiHostProjection {
        &self.host_projection
    }

    pub(crate) fn presentation_scale_factor(&self) -> f32 {
        self.presentation_scale_factor
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
        if is_componentized_drawer_frame(control_id) {
            return None;
        }
        self.host_projection
            .node_by_control_id(control_id)
            .map(|node| scale_frame(node.frame, self.presentation_scale_factor))
    }

    pub(crate) fn outer_shell_frames(&self) -> BuiltinHostOuterShellFrames {
        BuiltinHostOuterShellFrames {
            shell_frame: self.control_frame(UI_HOST_WINDOW_CONTROL_ID),
            menu_bar_frame: self.control_frame("WorkbenchMenuBarRoot"),
            host_page_strip_frame: self.control_frame(HOST_PAGE_STRIP_CONTROL_ID),
        }
    }

    pub(crate) fn root_shell_frames(&self) -> BuiltinHostRootShellFrames {
        let outer_shell_frames = self.outer_shell_frames();
        BuiltinHostRootShellFrames {
            shell_frame: outer_shell_frames.shell_frame,
            menu_bar_frame: outer_shell_frames.menu_bar_frame,
            activity_rail_frame: self.control_frame("ActivityRailRoot"),
            host_page_strip_frame: outer_shell_frames.host_page_strip_frame,
            host_body_frame: self.control_frame(HOST_BODY_CONTROL_ID),
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

fn normalized_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

fn scale_frame(frame: UiFrame, scale_factor: f32) -> UiFrame {
    UiFrame::new(
        frame.x * scale_factor,
        frame.y * scale_factor,
        frame.width * scale_factor,
        frame.height * scale_factor,
    )
}

fn is_componentized_drawer_frame(control_id: &str) -> bool {
    matches!(
        control_id,
        "LeftDrawerShellRoot"
            | "LeftDrawerHeaderRoot"
            | "RightDrawerShellRoot"
            | "RightDrawerHeaderRoot"
            | "BottomDrawerShellRoot"
            | "BottomDrawerHeaderRoot"
    )
}
