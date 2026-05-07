use zircon_runtime_interface::ui::layout::UiPoint;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::{
    event_bridge::UiHostEventEffects,
    host_page_pointer::{HostPagePointerBridge, HostPagePointerDispatch, HostPagePointerRoute},
};
use crate::ui::workbench::layout::LayoutCommand;

use super::super::{
    dispatch_builtin_host_page_activation, dispatch_layout_command, BuiltinHostWindowTemplateBridge,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedHostPagePointerClickDispatch {
    pub pointer: HostPagePointerDispatch,
    pub effects: Option<UiHostEventEffects>,
}

pub(crate) fn dispatch_shared_host_page_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinHostWindowTemplateBridge,
    pointer_bridge: &mut HostPagePointerBridge,
    item_index: usize,
    tab_x: f32,
    tab_width: f32,
    point: UiPoint,
) -> Result<SharedHostPagePointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(item_index, tab_x, tab_width, point)?;
    let effects = match pointer.route.as_ref() {
        Some(HostPagePointerRoute::Tab { page_id, .. }) => {
            match dispatch_builtin_host_page_activation(runtime, template_bridge, page_id) {
                Some(result) => Some(result?),
                None => Some(dispatch_layout_command(
                    runtime,
                    LayoutCommand::ActivateMainPage {
                        page_id: crate::ui::workbench::layout::MainPageId::new(page_id),
                    },
                )?),
            }
        }
        _ => None,
    };
    Ok(SharedHostPagePointerClickDispatch { pointer, effects })
}
