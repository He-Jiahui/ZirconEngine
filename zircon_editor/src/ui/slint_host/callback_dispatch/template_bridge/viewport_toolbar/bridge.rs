use std::collections::BTreeMap;

use zircon_runtime::ui::{
    binding::UiEventKind,
    layout::{UiFrame, UiSize},
};

use crate::ui::binding::EditorUiBinding;
use crate::ui::slint_host::callback_dispatch::constants::BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID;
use crate::ui::template_runtime::{EditorUiHostRuntime, SlintUiHostProjection, SlintUiProjection};

use super::super::projection_support::{
    binding_for_control, build_bindings_by_id, load_builtin_runtime_projection,
};
use super::action_control::projection_control_for_action;
use super::error::BuiltinViewportToolbarTemplateBridgeError;
use super::host_projection::build_builtin_viewport_toolbar_host_projection;

pub(crate) struct BuiltinViewportToolbarTemplateBridge {
    runtime: EditorUiHostRuntime,
    projection: SlintUiProjection,
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_projection: SlintUiHostProjection,
}

impl BuiltinViewportToolbarTemplateBridge {
    pub(crate) fn new() -> Result<Self, BuiltinViewportToolbarTemplateBridgeError> {
        let (runtime, projection) =
            load_builtin_runtime_projection(BUILTIN_VIEWPORT_TOOLBAR_DOCUMENT_ID)?;
        let bindings_by_id = build_bindings_by_id(&projection);
        let host_projection = build_builtin_viewport_toolbar_host_projection(
            &runtime,
            &projection,
            UiSize::new(1280.0, 28.0),
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
        surface_size: UiSize,
    ) -> Result<(), BuiltinViewportToolbarTemplateBridgeError> {
        self.host_projection = build_builtin_viewport_toolbar_host_projection(
            &self.runtime,
            &self.projection,
            surface_size,
        )?;
        Ok(())
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

    pub(crate) fn control_frame_for_action(&self, control_id: &str) -> Option<UiFrame> {
        let projection_control_id = projection_control_for_action(control_id)?;
        self.host_projection
            .node_by_control_id(projection_control_id)
            .map(|node| node.frame)
    }
}
