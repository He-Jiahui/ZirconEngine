use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::{
    document_tab_pointer::{
        HostDocumentTabPointerBridge, HostDocumentTabPointerDispatch, HostDocumentTabPointerRoute,
    },
    event_bridge::UiHostEventEffects,
};
use crate::ui::workbench::layout::LayoutCommand;

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
    runtime: &EditorHostEventController,
    template_bridge: &BuiltinHostWindowTemplateBridge,
    pointer_bridge: &HostDocumentTabPointerBridge,
    surface_key: &str,
    item_index: usize,
) -> Result<SharedDocumentTabPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_activate_click(surface_key, item_index)?;
    let effects = match pointer.route {
        Some(route @ HostDocumentTabPointerRoute::ActivateTab { .. }) => {
            let instance_id = pointer_bridge
                .target_for_route(route)
                .ok_or_else(|| "Document tab activation receipt target is stale".to_string())?;
            match dispatch_builtin_host_document_tab_activation(
                runtime,
                template_bridge,
                instance_id.0.as_str(),
            ) {
                Some(result) => Some(result?),
                None => Some(dispatch_layout_command(
                    runtime,
                    LayoutCommand::FocusView {
                        instance_id: instance_id.clone(),
                    },
                )?),
            }
        }
        _ => None,
    };
    Ok(SharedDocumentTabPointerClickDispatch { pointer, effects })
}

pub(crate) fn dispatch_shared_document_tab_close_pointer_click(
    runtime: &EditorHostEventController,
    template_bridge: &BuiltinHostWindowTemplateBridge,
    pointer_bridge: &HostDocumentTabPointerBridge,
    surface_key: &str,
    item_index: usize,
) -> Result<SharedDocumentTabPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_close_click(surface_key, item_index)?;
    let effects = match pointer.route {
        Some(route @ HostDocumentTabPointerRoute::CloseTab { .. }) => {
            let instance_id = pointer_bridge
                .target_for_route(route)
                .ok_or_else(|| "Document tab close receipt target is stale".to_string())?;
            match dispatch_builtin_host_document_tab_close(
                runtime,
                template_bridge,
                instance_id.0.as_str(),
            ) {
                Some(result) => Some(result?),
                None => Some(dispatch_layout_command(
                    runtime,
                    LayoutCommand::CloseView {
                        instance_id: instance_id.clone(),
                    },
                )?),
            }
        }
        _ => None,
    };
    Ok(SharedDocumentTabPointerClickDispatch { pointer, effects })
}
