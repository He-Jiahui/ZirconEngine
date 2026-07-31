use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use zircon_runtime_interface::ui::dispatch::UiInputModifiers;

use super::super::super::super::routing::PanePointerRoute;

pub(in crate::ui::retained_host::host_contract) fn dispatch_viewport_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    kind: i32,
    button_id: i32,
    modifiers: UiInputModifiers,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    pane_host.invoke_viewport_pointer_event(
        kind,
        button_id,
        pointer.local_x,
        pointer.local_y,
        0.0,
        modifiers.shift,
        modifiers.control,
    );
    if let Some(frame) = cleared_text_input_frame {
        return NativePointerDispatchResult::region(frame);
    }
    NativePointerDispatchResult::idle()
}
