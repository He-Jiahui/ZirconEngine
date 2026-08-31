mod target;

use crate::ui::retained_host::host_contract::data::HostPresentationGeneration;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use self::target::dispatch_pane_pointer_move_target;
use super::super::super::redraw_result::pointer_move_redraw;
use super::super::super::routing::route_pointer_move_to_pane;

pub(in super::super) fn dispatch_pane_pointer_move(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    x: f32,
    y: f32,
) -> Option<NativePointerDispatchResult> {
    let pointer = route_pointer_move_to_pane(
        generation.structure(),
        generation.pane_interaction_state(),
        x,
        y,
    )?;
    let before = generation.pane_interaction_state();
    dispatch_pane_pointer_move_target(ui, &pointer, before);
    let after = ui.get_pane_interaction_generation();
    Some(pointer_move_redraw(&pointer, before, after.as_ref()))
}
