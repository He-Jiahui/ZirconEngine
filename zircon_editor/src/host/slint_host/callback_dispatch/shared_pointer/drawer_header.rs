use zircon_ui::UiPoint;

use crate::editor_event::EditorEventRuntime;
use crate::host::slint_host::{
    drawer_header_pointer::{
        WorkbenchDrawerHeaderPointerBridge, WorkbenchDrawerHeaderPointerDispatch,
        WorkbenchDrawerHeaderPointerRoute,
    },
    event_bridge::SlintDispatchEffects,
};

use super::super::{BuiltinWorkbenchTemplateBridge, dispatch_builtin_workbench_drawer_toggle};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedDrawerHeaderPointerClickDispatch {
    pub pointer: WorkbenchDrawerHeaderPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_drawer_header_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinWorkbenchTemplateBridge,
    pointer_bridge: &mut WorkbenchDrawerHeaderPointerBridge,
    surface_key: &str,
    item_index: usize,
    tab_x: f32,
    tab_width: f32,
    point: UiPoint,
) -> Result<SharedDrawerHeaderPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(surface_key, item_index, tab_x, tab_width, point)?;
    let effects = match pointer.route.as_ref() {
        Some(WorkbenchDrawerHeaderPointerRoute::Tab {
            slot, instance_id, ..
        }) => dispatch_builtin_workbench_drawer_toggle(runtime, template_bridge, slot, instance_id)
            .transpose()?,
        _ => None,
    };
    Ok(SharedDrawerHeaderPointerClickDispatch { pointer, effects })
}
