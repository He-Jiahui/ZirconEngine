use zircon_ui::UiPoint;

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::{
    document_tab_pointer::{
        WorkbenchDocumentTabPointerBridge, WorkbenchDocumentTabPointerDispatch,
        WorkbenchDocumentTabPointerRoute,
    },
    event_bridge::SlintDispatchEffects,
};
use crate::{LayoutCommand, ViewInstanceId};

use super::super::{
    BuiltinWorkbenchTemplateBridge, dispatch_builtin_workbench_document_tab_activation,
    dispatch_builtin_workbench_document_tab_close, dispatch_layout_command,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedDocumentTabPointerClickDispatch {
    pub pointer: WorkbenchDocumentTabPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_document_tab_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinWorkbenchTemplateBridge,
    pointer_bridge: &mut WorkbenchDocumentTabPointerBridge,
    surface_key: &str,
    item_index: usize,
    tab_x: f32,
    tab_width: f32,
    point: UiPoint,
) -> Result<SharedDocumentTabPointerClickDispatch, String> {
    let pointer =
        pointer_bridge.handle_activate_click(surface_key, item_index, tab_x, tab_width, point)?;
    let effects = match pointer.route.as_ref() {
        Some(WorkbenchDocumentTabPointerRoute::ActivateTab { instance_id, .. }) => {
            match dispatch_builtin_workbench_document_tab_activation(
                runtime,
                template_bridge,
                instance_id,
            ) {
                Some(result) => Some(result?),
                None => Some(dispatch_layout_command(
                    runtime,
                    LayoutCommand::FocusView {
                        instance_id: ViewInstanceId::new(instance_id),
                    },
                )?),
            }
        }
        _ => None,
    };
    Ok(SharedDocumentTabPointerClickDispatch { pointer, effects })
}

pub(crate) fn dispatch_shared_document_tab_close_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinWorkbenchTemplateBridge,
    pointer_bridge: &mut WorkbenchDocumentTabPointerBridge,
    surface_key: &str,
    item_index: usize,
    tab_x: f32,
    tab_width: f32,
    point: UiPoint,
) -> Result<SharedDocumentTabPointerClickDispatch, String> {
    let pointer =
        pointer_bridge.handle_close_click(surface_key, item_index, tab_x, tab_width, point)?;
    let effects = match pointer.route.as_ref() {
        Some(WorkbenchDocumentTabPointerRoute::CloseTab { instance_id, .. }) => {
            match dispatch_builtin_workbench_document_tab_close(
                runtime,
                template_bridge,
                instance_id,
            ) {
                Some(result) => Some(result?),
                None => Some(dispatch_layout_command(
                    runtime,
                    LayoutCommand::CloseView {
                        instance_id: ViewInstanceId::new(instance_id),
                    },
                )?),
            }
        }
        _ => None,
    };
    Ok(SharedDocumentTabPointerClickDispatch { pointer, effects })
}
