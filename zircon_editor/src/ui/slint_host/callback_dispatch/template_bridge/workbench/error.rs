use thiserror::Error;
use zircon_runtime::ui::tree::UiTreeError;

use crate::ui::template_runtime::EditorUiHostRuntimeError;

use super::super::workbench_drawer_source::BuiltinWorkbenchDrawerSourceTemplateBridgeError;

#[derive(Debug, Error)]
pub(crate) enum BuiltinWorkbenchTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
    #[error(transparent)]
    DrawerSource(#[from] BuiltinWorkbenchDrawerSourceTemplateBridgeError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
}
