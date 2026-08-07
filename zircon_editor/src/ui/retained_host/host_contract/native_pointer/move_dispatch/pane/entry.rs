mod target;

use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use self::target::dispatch_pane_pointer_move_target;
use super::super::super::redraw_result::pointer_move_redraw;
use super::super::super::routing::route_pointer_move_to_pane;

pub(in super::super) fn dispatch_pane_pointer_move(
    ui: &UiHostWindow,
    presentation: &HostWindowPresentationData,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let pointer = route_pointer_move_to_pane(presentation, x, y)?;
    let before = ui.get_host_presentation_generation();
    dispatch_pane_pointer_move_target(ui, &pointer);
    let after = ui.get_host_presentation_generation();
    Some(pointer_move_redraw(
        &pointer,
        before.pane_interaction_state(),
        after.pane_interaction_state(),
    ))
}
