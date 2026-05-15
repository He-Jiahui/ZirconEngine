use std::collections::BTreeMap;

use zircon_runtime_interface::ui::binding::UiEventKind;

use crate::ui::binding::EditorUiBinding;
use crate::ui::retained_host::callback_dispatch::constants::BUILTIN_WELCOME_SURFACE_DOCUMENT_ID;
use crate::ui::template_runtime::{EditorUiHostRuntime, RetainedUiHostProjection};

#[cfg(test)]
use super::super::project_builtin_surface;
use super::super::{
    binding_for_control, project_builtin_surface_with_runtime,
    projection_support::load_builtin_runtime_for_documents,
};
use super::error::BuiltinWelcomeSurfaceTemplateBridgeError;

pub(crate) struct BuiltinWelcomeSurfaceTemplateBridge {
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_projection: RetainedUiHostProjection,
}

impl BuiltinWelcomeSurfaceTemplateBridge {
    #[cfg(test)]
    pub(crate) fn new() -> Result<Self, BuiltinWelcomeSurfaceTemplateBridgeError> {
        let (bindings_by_id, host_projection) =
            project_builtin_surface(BUILTIN_WELCOME_SURFACE_DOCUMENT_ID)?;
        Ok(Self {
            bindings_by_id,
            host_projection,
        })
    }

    pub(crate) fn new_with_runtime(
        runtime: &EditorUiHostRuntime,
    ) -> Result<Self, BuiltinWelcomeSurfaceTemplateBridgeError> {
        let (bindings_by_id, host_projection) =
            project_builtin_surface_with_runtime(runtime, BUILTIN_WELCOME_SURFACE_DOCUMENT_ID)?;
        Ok(Self {
            bindings_by_id,
            host_projection,
        })
    }

    pub(crate) fn new_minimal() -> Result<Self, BuiltinWelcomeSurfaceTemplateBridgeError> {
        let runtime = load_builtin_runtime_for_documents(&[BUILTIN_WELCOME_SURFACE_DOCUMENT_ID])?;
        Self::new_with_runtime(&runtime)
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
}
