use zircon_runtime_interface::ui::dispatch::{
    UiDispatchReply, UiInputDispatchResult, UiInputEvent, UiMouseMotionInputEvent,
};

use super::{
    super::surface::UiSurface, route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
};

pub(super) fn dispatch_mouse_motion_input(
    surface: &UiSurface,
    motion: UiMouseMotionInputEvent,
) -> UiInputDispatchResult {
    let event = UiInputEvent::MouseMotion(motion);
    let mut result = UiInputDispatchResult::new(event.clone(), UiDispatchReply::unhandled());
    result
        .diagnostics
        .notes
        .push("raw_mouse_motion".to_string());
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}
