use zircon_runtime_interface::ui::{
    dispatch::{
        UiAnalogInputEvent, UiDispatchDisposition, UiDispatchReply, UiInputDispatchResult,
        UiInputEvent, UiNavigationInputEvent,
    },
    tree::UiTreeError,
};

use super::{
    super::surface::UiSurface,
    analog_navigation::{analog_navigation_decision, AnalogNavigationDecision},
    owner_route::owner_routed_result,
    route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
};
use crate::ui::dispatch::UiNavigationDispatcher;

pub(super) fn dispatch_analog_input(
    surface: &mut UiSurface,
    navigation_dispatcher: &UiNavigationDispatcher,
    analog: UiAnalogInputEvent,
    dispatch_navigation_input: impl FnOnce(
        &mut UiSurface,
        &UiNavigationDispatcher,
        UiNavigationInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError>,
) -> Result<UiInputDispatchResult, UiTreeError> {
    let changed = surface
        .input
        .update_analog_control(analog.control.as_str(), analog.value);
    let navigation_analog = analog_with_retained_control_value(surface, &analog);
    let analog_navigation = analog_navigation_decision(&mut surface.input, &navigation_analog);
    if let AnalogNavigationDecision::Navigate(navigation_kind) = analog_navigation {
        let mut navigation_result = dispatch_navigation_input(
            surface,
            navigation_dispatcher,
            UiNavigationInputEvent {
                metadata: analog.metadata.clone(),
                kind: navigation_kind,
            },
        )?;
        navigation_result.event = UiInputEvent::Analog(analog);
        if navigation_result.reply.disposition != UiDispatchDisposition::Unhandled {
            navigation_result.diagnostics.handled_phase = Some("analog.navigation".to_string());
        }
        navigation_result
            .diagnostics
            .notes
            .push(format!("analog_navigation={navigation_kind:?}"));
        return Ok(navigation_result);
    }
    let focused = surface.focus.focused;
    let mut result = owner_routed_result(
        surface,
        UiInputEvent::Analog(analog),
        focused,
        "analog.focused",
    );
    if !changed {
        result.reply = UiDispatchReply::unhandled();
        result.diagnostics.routed = false;
        result
            .diagnostics
            .notes
            .push("analog_repeat_suppressed".to_string());
    }
    if let AnalogNavigationDecision::Suppressed(navigation_kind) = analog_navigation {
        result.diagnostics.notes.push(format!(
            "analog_navigation_repeat_suppressed={navigation_kind:?}"
        ));
    }
    Ok(with_analog_route_policy(surface, result))
}

fn with_analog_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}

fn analog_with_retained_control_value(
    surface: &UiSurface,
    analog: &UiAnalogInputEvent,
) -> UiAnalogInputEvent {
    let mut analog = analog.clone();
    if let Some(state) = surface.input.analog_controls.get(analog.control.as_str()) {
        analog.value = state.value;
    }
    analog
}
