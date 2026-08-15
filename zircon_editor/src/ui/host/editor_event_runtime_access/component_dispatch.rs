use crate::core::editor_event::EditorEventSource;
use crate::ui::host::EditorHostEventController;
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentEventEnvelope,
};

impl EditorHostEventController {
    pub(crate) fn dispatch_ui_component_adapter_event(
        &self,
        envelope: &UiComponentEventEnvelope,
    ) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
        if envelope.target.domain == "component_drawer" {
            return self.dispatch_component_drawer_adapter_event(envelope);
        }
        if envelope.target.domain
            == crate::ui::template_runtime::component_adapter::command::COMMAND_DOMAIN
        {
            return self.dispatch_command_component_adapter_event(envelope);
        }
        let result = {
            let mut inner = self.shell().lock();
            let manager = inner.manager.clone();
            crate::ui::template_runtime::component_adapter::registry::EditorUiComponentAdapterRegistry::apply_envelope(
                    &mut inner.state,
                    manager.as_ref(),
                    envelope,
                )?
        };
        if result.refresh_projection {
            self.refresh_workbench(
                crate::core::editor_message::EditorViewInvalidationMask::PRESENTATION_DATA,
            );
        }
        Ok(result)
    }

    fn dispatch_component_drawer_adapter_event(
        &self,
        envelope: &UiComponentEventEnvelope,
    ) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
        let component_type = envelope.target.subject.as_deref().ok_or_else(|| {
            UiComponentAdapterError::MissingSource {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
                source_name: "subject".to_string(),
            }
        })?;
        let customization = self.inspector_customization(component_type);
        let operation_path =
            crate::ui::template_runtime::component_adapter::component_drawer::validate_component_drawer_envelope(
                envelope,
                customization.as_deref(),
            )?;
        let (source, invocation) = crate::ui::template_runtime::component_adapter::component_drawer::component_drawer_operation_invocation(operation_path.clone());
        self.invoke_operation(source, invocation).map_err(|error| {
            UiComponentAdapterError::HostMutation {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
                reason: error,
            }
        })?;
        Ok(
            crate::ui::template_runtime::component_adapter::component_drawer::component_drawer_operation_result(
                &operation_path,
            ),
        )
    }

    fn dispatch_command_component_adapter_event(
        &self,
        envelope: &UiComponentEventEnvelope,
    ) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
        let binding =
            crate::ui::template_runtime::component_adapter::command::editor_command_binding_for_envelope(
                envelope,
            )?;
        let command_id = match binding.payload() {
            crate::ui::binding::EditorUiBindingPayload::EditorCommand { command_id } => {
                command_id.clone()
            }
            _ => {
                return Err(UiComponentAdapterError::HostMutation {
                    domain: envelope.target.domain.clone(),
                    path: envelope.target.path.clone(),
                    reason: "command adapter returned a non-command binding".to_string(),
                });
            }
        };

        self.dispatch_binding(binding, EditorEventSource::RetainedHost)
            .map_err(|error| UiComponentAdapterError::HostMutation {
                domain: envelope.target.domain.clone(),
                path: envelope.target.path.clone(),
                reason: error,
            })?;
        Ok(
            crate::ui::template_runtime::component_adapter::command::command_adapter_result(
                &command_id,
            ),
        )
    }

    #[cfg(test)]
    pub(crate) fn ui_component_data_sources(
        &self,
    ) -> Vec<zircon_runtime_interface::ui::component::UiComponentDataSourceDescriptor> {
        crate::ui::template_runtime::component_adapter::registry::EditorUiComponentAdapterRegistry::data_sources()
    }
}
