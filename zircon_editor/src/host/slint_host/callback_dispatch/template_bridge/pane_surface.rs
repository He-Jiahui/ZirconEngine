use std::collections::BTreeMap;

use thiserror::Error;
use zircon_editor_ui::EditorUiBinding;
use zircon_ui::UiEventKind;

use crate::host::slint_host::callback_dispatch::constants::BUILTIN_PANE_SURFACE_DOCUMENT_ID;
use crate::host::template_runtime::{EditorUiHostRuntimeError, SlintUiHostProjection};

use super::{binding_for_control, project_builtin_surface};

#[derive(Debug, Error)]
pub(crate) enum BuiltinPaneSurfaceTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
}

pub(crate) struct BuiltinPaneSurfaceTemplateBridge {
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_projection: SlintUiHostProjection,
}

impl BuiltinPaneSurfaceTemplateBridge {
    pub(crate) fn new() -> Result<Self, BuiltinPaneSurfaceTemplateBridgeError> {
        let (bindings_by_id, host_projection) =
            project_builtin_surface(BUILTIN_PANE_SURFACE_DOCUMENT_ID)?;
        Ok(Self {
            bindings_by_id,
            host_projection,
        })
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
