use zircon_runtime_interface::ui::layout::UiPoint;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::{
    document_tab_pointer::{
        HostDocumentTabPointerBridge, HostDocumentTabPointerDispatch, HostDocumentTabPointerRoute,
    },
    event_bridge::UiHostEventEffects,
};
use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::{
    dispatch_builtin_host_document_tab_activation, dispatch_builtin_host_document_tab_close,
    dispatch_layout_command, BuiltinHostWindowTemplateBridge,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedDocumentTabPointerClickDispatch {
    pub pointer: HostDocumentTabPointerDispatch,
    pub effects: Option<UiHostEventEffects>,
}

pub(crate) fn dispatch_shared_document_tab_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinHostWindowTemplateBridge,
    pointer_bridge: &mut HostDocumentTabPointerBridge,
    surface_key: &str,
    item_index: usize,
    tab_x: f32,
    tab_width: f32,
    point: UiPoint,
) -> Result<SharedDocumentTabPointerClickDispatch, String> {
    let pointer =
        pointer_bridge.handle_activate_click(surface_key, item_index, tab_x, tab_width, point)?;
    let effects = match pointer.route.as_ref() {
        Some(HostDocumentTabPointerRoute::ActivateTab { instance_id, .. }) => {
            match dispatch_builtin_host_document_tab_activation(
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
    template_bridge: &BuiltinHostWindowTemplateBridge,
    pointer_bridge: &mut HostDocumentTabPointerBridge,
    surface_key: &str,
    item_index: usize,
    tab_x: f32,
    tab_width: f32,
    point: UiPoint,
) -> Result<SharedDocumentTabPointerClickDispatch, String> {
    let pointer =
        pointer_bridge.handle_close_click(surface_key, item_index, tab_x, tab_width, point)?;
    let effects = match pointer.route.as_ref() {
        Some(HostDocumentTabPointerRoute::CloseTab { instance_id, .. }) => {
            match dispatch_builtin_host_document_tab_close(runtime, template_bridge, instance_id) {
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
