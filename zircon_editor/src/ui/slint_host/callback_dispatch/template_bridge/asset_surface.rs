use std::collections::BTreeMap;

use crate::ui::EditorUiBinding;
use thiserror::Error;
use zircon_ui::binding::UiEventKind;

use crate::ui::slint_host::callback_dispatch::constants::BUILTIN_ASSET_SURFACE_DOCUMENT_ID;
use crate::ui::template_runtime::{EditorUiHostRuntimeError, SlintUiHostProjection};

use super::{binding_for_control, project_builtin_surface};

#[derive(Debug, Error)]
pub(crate) enum BuiltinAssetSurfaceTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
}

pub(crate) struct BuiltinAssetSurfaceTemplateBridge {
    bindings_by_id: BTreeMap<String, EditorUiBinding>,
    host_projection: SlintUiHostProjection,
}

impl BuiltinAssetSurfaceTemplateBridge {
    pub(crate) fn new() -> Result<Self, BuiltinAssetSurfaceTemplateBridgeError> {
        let (bindings_by_id, host_projection) =
            project_builtin_surface(BUILTIN_ASSET_SURFACE_DOCUMENT_ID)?;
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
