use thiserror::Error;
use zircon_runtime::ui::tree::UiTreeError;

use crate::ui::template_runtime::EditorUiHostRuntimeError;

#[derive(Debug, Error)]
pub(crate) enum BuiltinHostDrawerSourceTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
}
