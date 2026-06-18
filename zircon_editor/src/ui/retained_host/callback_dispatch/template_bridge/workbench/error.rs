use thiserror::Error;
use zircon_runtime_interface::ui::tree::UiTreeError;

use crate::ui::template_runtime::EditorUiHostRuntimeError;
use crate::ui::workbench::reference::EditorWorkbenchTemplateSurfaceError;

use super::super::virtual_rows::TemplateBridgeVirtualRowsError;

#[derive(Debug, Error)]
pub(crate) enum BuiltinHostWindowTemplateBridgeError {
    #[error(transparent)]
    HostRuntime(#[from] EditorUiHostRuntimeError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
    #[error(transparent)]
    ComponentizedWorkbench(#[from] EditorWorkbenchTemplateSurfaceError),
    #[error(transparent)]
    VirtualRows(#[from] TemplateBridgeVirtualRowsError),
}
