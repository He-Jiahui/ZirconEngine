use zircon_runtime_interface::ui::dispatch::{UiAccessibilityInputEvent, UiInputDispatchResult};

use super::super::surface::UiSurface;
use super::{route_policy::annotate_route_policy, route_steps::annotate_result_route_steps};

pub(super) fn dispatch_accessibility_input(
    surface: &mut UiSurface,
    accessibility: UiAccessibilityInputEvent,
) -> UiInputDispatchResult {
    let result = crate::ui::accessibility::dispatch_accessibility_action(surface, accessibility);
    with_accessibility_route_policy(surface, result)
}

fn with_accessibility_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}
