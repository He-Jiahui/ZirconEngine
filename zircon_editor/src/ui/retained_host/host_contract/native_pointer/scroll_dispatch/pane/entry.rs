use crate::ui::retained_host::host_contract::data::HostPresentationGeneration;
use crate::ui::retained_host::host_contract::globals::PaneSurfaceHostContext;
use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use super::super::super::routing::route_pointer_scroll_to_pane;
use super::asset::dispatch_asset_pane_scroll;
use super::native::dispatch_native_pane_scroll;
use super::passive::is_passive_pane_scroll_target;
use super::viewport_target::dispatch_viewport_pane_scroll;

pub(in super::super) fn dispatch_pane_pointer_scroll(
    ui: &UiHostWindow,
    generation: &HostPresentationGeneration,
    x: f32,
    y: f32,
    delta: f32,
) -> Option<NativePointerDispatchResult> {
    let presentation = generation.structure();
    let pointer =
        route_pointer_scroll_to_pane(presentation, generation.pane_interaction_state(), x, y)?;
    let pane_host = ui.global::<PaneSurfaceHostContext>();
    let before = ui.get_host_interaction_generation();

    if dispatch_native_pane_scroll(&pane_host, &pointer, delta)
        || dispatch_asset_pane_scroll(&pane_host, &pointer, delta)
    {
        if before == ui.get_host_interaction_generation() {
            return Some(NativePointerDispatchResult::idle());
        }
        return Some(NativePointerDispatchResult::region(pointer.frame.clone()));
    }
    if let Some(result) = dispatch_viewport_pane_scroll(&pane_host, &pointer, delta) {
        return Some(result);
    }
    if is_passive_pane_scroll_target(&pointer) {
        return Some(NativePointerDispatchResult::idle());
    }
    Some(NativePointerDispatchResult::idle())
}
