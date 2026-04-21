use std::collections::BTreeMap;

use zircon_runtime::ui::binding::UiEventKind;

use crate::ui::binding::EditorUiBinding;
use crate::ui::slint_host::callback_dispatch::constants::BUILTIN_WELCOME_SURFACE_DOCUMENT_ID;
use crate::ui::template_runtime::SlintUiHostProjection;

use super::super::{binding_for_control, project_builtin_surface};
use super::error::BuiltinWelcomeSurfaceTemplateBridgeError;

pub(crate) struct BuiltinWelcomeSurfaceTemplateBridge {
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_projection: SlintUiHostProjection,
}

impl BuiltinWelcomeSurfaceTemplateBridge {
    pub(crate) fn new() -> Result<Self, BuiltinWelcomeSurfaceTemplateBridgeError> {
        let (bindings_by_id, host_projection) =
            project_builtin_surface(BUILTIN_WELCOME_SURFACE_DOCUMENT_ID)?;
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
