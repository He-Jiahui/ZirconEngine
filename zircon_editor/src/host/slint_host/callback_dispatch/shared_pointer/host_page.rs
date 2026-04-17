use zircon_ui::UiPoint;

use crate::LayoutCommand;
use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::{
    event_bridge::SlintDispatchEffects,
    host_page_pointer::{
        WorkbenchHostPagePointerBridge, WorkbenchHostPagePointerDispatch,
        WorkbenchHostPagePointerRoute,
    },
};

use super::super::{
    BuiltinWorkbenchTemplateBridge, dispatch_builtin_workbench_host_page_activation,
    dispatch_layout_command,
};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedHostPagePointerClickDispatch {
    pub pointer: WorkbenchHostPagePointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_host_page_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinWorkbenchTemplateBridge,
    pointer_bridge: &mut WorkbenchHostPagePointerBridge,
    item_index: usize,
    tab_x: f32,
    tab_width: f32,
    point: UiPoint,
) -> Result<SharedHostPagePointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(item_index, tab_x, tab_width, point)?;
    let effects = match pointer.route.as_ref() {
        Some(WorkbenchHostPagePointerRoute::Tab { page_id, .. }) => {
            match dispatch_builtin_workbench_host_page_activation(runtime, template_bridge, page_id)
            {
                Some(result) => Some(result?),
                None => Some(dispatch_layout_command(
                    runtime,
                    LayoutCommand::ActivateMainPage {
                        page_id: crate::MainPageId::new(page_id),
                    },
                )?),
            }
        }
        _ => None,
    };
    Ok(SharedHostPagePointerClickDispatch { pointer, effects })
}
