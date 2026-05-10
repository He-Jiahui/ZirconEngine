use thiserror::Error;

use crate::ui::template_runtime::EditorUiHostRuntimeError;

#[derive(Debug, Error)]
pub(crate) enum BuiltinPaneSurfaceTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
}
