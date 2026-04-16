use std::collections::BTreeMap;

use thiserror::Error;
use zircon_editor_ui::{DockCommand, EditorUiBinding, EditorUiBindingPayload};
use zircon_ui::{UiBindingValue, UiEventKind, UiSize};

use crate::host::slint_host::callback_dispatch::constants::{
    BUILTIN_WORKBENCH_DOCUMENT_ID, DOCUMENT_TABS_CONTROL_ID, WORKBENCH_SHELL_CONTROL_ID,
};
use crate::host::template_runtime::{
    EditorUiHostRuntime, EditorUiHostRuntimeError, SlintUiHostProjection, SlintUiProjection,
};

use super::{binding_for_control, build_bindings_by_id, load_builtin_runtime_projection};

#[derive(Debug, Error)]
pub(crate) enum BuiltinWorkbenchTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
    #[error(transparent)]
    Layout(#[from] zircon_ui::UiTreeError),
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
        let host_projection =
            build_builtin_workbench_host_projection(&runtime, &projection, shell_size)?;

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
        self.host_projection =
            build_builtin_workbench_host_projection(&self.runtime, &self.projection, shell_size)?;
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
) -> Result<SlintUiHostProjection, BuiltinWorkbenchTemplateBridgeError> {
    let mut surface = runtime.build_shared_surface(BUILTIN_WORKBENCH_DOCUMENT_ID)?;
    surface.compute_layout(shell_size)?;
    Ok(runtime.build_slint_host_projection_with_surface(projection, &surface)?)
}
