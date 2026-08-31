mod asset_deletion_blocker;
mod body;
mod close_prompt;
mod primary_overlays;
mod text_focus;

use crate::ui::retained_host::host_contract::redraw::NativePointerDispatchResult;
use crate::ui::retained_host::host_contract::window::UiHostWindow;

use self::asset_deletion_blocker::dispatch_asset_deletion_blocker_step;
use self::body::dispatch_body_route_step;
use self::close_prompt::dispatch_close_prompt_step;
use self::primary_overlays::dispatch_primary_overlay_step;
use self::text_focus::clear_text_focus_step;
use super::super::super::super::NativePointerButtonState;
use super::super::input::ButtonDispatchInput;

pub(super) fn dispatch_button_steps(
    ui: &UiHostWindow,
    state: NativePointerButtonState,
    input: ButtonDispatchInput,
    x: f32,
    y: f32,
) -> NativePointerDispatchResult {
    if let Some(result) = dispatch_close_prompt_step(ui, state, &input, x, y) {
        return result;
    }
    if let Some(result) = dispatch_asset_deletion_blocker_step(ui, state, &input, x, y) {
        return result;
    }
    let cleared_text_input_frame = clear_text_focus_step(ui, state, &input);
    if let Some(result) =
        dispatch_primary_overlay_step(ui, state, &input, x, y, cleared_text_input_frame.clone())
    {
        return result;
    }
    dispatch_body_route_step(ui, state, input, x, y, cleared_text_input_frame)
}
