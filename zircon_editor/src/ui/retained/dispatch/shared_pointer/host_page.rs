use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::{
    event_bridge::UiHostEventEffects,
    host_page_pointer::{HostPagePointerBridge, HostPagePointerDispatch, HostPagePointerRoute},
};
use crate::ui::workbench::layout::LayoutCommand;

use super::super::{
    dispatch_builtin_host_document_tab_close, dispatch_builtin_host_page_activation,
    dispatch_layout_command, BuiltinHostWindowTemplateBridge,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedHostPagePointerClickDispatch {
    pub pointer: HostPagePointerDispatch,
    pub effects: Option<UiHostEventEffects>,
}

pub(crate) fn dispatch_shared_host_page_pointer_click(
    runtime: &EditorHostEventController,
    template_bridge: &BuiltinHostWindowTemplateBridge,
    pointer_bridge: &HostPagePointerBridge,
    item_index: usize,
    close: bool,
) -> Result<SharedHostPagePointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(item_index, close)?;
    let effects = match pointer.route {
        Some(route @ HostPagePointerRoute::Activate { .. }) => {
            let page_id = pointer_bridge
                .activation_target_for_route(route)
                .ok_or_else(|| "Host page activation receipt target is stale".to_string())?;
            match dispatch_builtin_host_page_activation(
                runtime,
                template_bridge,
                page_id.0.as_str(),
            ) {
                Some(result) => Some(result?),
                None => Some(dispatch_layout_command(
                    runtime,
                    LayoutCommand::ActivateMainPage {
                        page_id: page_id.clone(),
                    },
                )?),
            }
        }
        Some(route @ HostPagePointerRoute::Close { .. }) => {
            let instance_id = pointer_bridge
                .close_target_for_route(route)
                .ok_or_else(|| "Host page close receipt target is stale".to_string())?;
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
    Ok(SharedHostPagePointerClickDispatch { pointer, effects })
}
