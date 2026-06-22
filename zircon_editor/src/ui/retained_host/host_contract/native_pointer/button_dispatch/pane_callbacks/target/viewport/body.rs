use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;

use super::super::super::super::super::routing::PanePointerRoute;
use super::super::super::viewport::dispatch_viewport_button;

pub(super) fn dispatch_viewport_body_target_button(
    pane_host: &PaneSurfaceHostContext<'_>,
    pointer: &PanePointerRoute,
    kind: i32,
    button_id: i32,
    cleared_text_input_frame: Option<FrameRect>,
) -> NativePointerDispatchResult {
    dispatch_viewport_button(
        pane_host,
        pointer,
        kind,
        button_id,
        cleared_text_input_frame,
    )
}
