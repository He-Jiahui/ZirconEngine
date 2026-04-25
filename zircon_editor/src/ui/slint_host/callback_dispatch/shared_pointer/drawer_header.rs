use zircon_runtime::ui::layout::UiPoint;

use crate::core::editor_event::EditorEventRuntime;
use crate::ui::slint_host::{
    drawer_header_pointer::{
        HostDrawerHeaderPointerBridge, HostDrawerHeaderPointerDispatch,
        HostDrawerHeaderPointerRoute,
    },
    event_bridge::SlintDispatchEffects,
};

use super::super::{dispatch_builtin_host_drawer_toggle, BuiltinHostWindowTemplateBridge};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SharedDrawerHeaderPointerClickDispatch {
    pub pointer: HostDrawerHeaderPointerDispatch,
    pub effects: Option<SlintDispatchEffects>,
}

pub(crate) fn dispatch_shared_drawer_header_pointer_click(
    runtime: &EditorEventRuntime,
    template_bridge: &BuiltinHostWindowTemplateBridge,
    pointer_bridge: &mut HostDrawerHeaderPointerBridge,
    surface_key: &str,
    item_index: usize,
    tab_x: f32,
    tab_width: f32,
    point: UiPoint,
) -> Result<SharedDrawerHeaderPointerClickDispatch, String> {
    let pointer = pointer_bridge.handle_click(surface_key, item_index, tab_x, tab_width, point)?;
    let effects = match pointer.route.as_ref() {
        Some(HostDrawerHeaderPointerRoute::Tab {
            slot, instance_id, ..
        }) => dispatch_builtin_host_drawer_toggle(runtime, template_bridge, slot, instance_id)
            .transpose()?,
        _ => None,
    };
    Ok(SharedDrawerHeaderPointerClickDispatch { pointer, effects })
}
