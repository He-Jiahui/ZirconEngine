use zircon_runtime_interface::ui::{
    dispatch::{
        UiAnalogInputEvent, UiDispatchDisposition, UiDispatchReply, UiInputDiagnosticsMode,
        UiInputDispatchResult, UiInputEvent, UiNavigationInputEvent,
    },
    tree::UiTreeError,
};

use super::{
    super::surface::UiSurface,
    analog_navigation::{analog_navigation_decision, AnalogNavigationDecision},
    owner_route::owner_routed_result_with_diagnostics_mode,
    route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
};
use crate::ui::dispatch::UiNavigationDispatcher;

#[cfg(test)]
#[path = "analog/owned_event_route_policy_tests.rs"]
mod owned_event_route_policy_tests;

pub(super) fn dispatch_analog_input(
    surface: &mut UiSurface,
    navigation_dispatcher: &UiNavigationDispatcher,
    analog: UiAnalogInputEvent,
    diagnostics_mode: UiInputDiagnosticsMode,
    dispatch_navigation_input: impl FnOnce(
        &mut UiSurface,
        &UiNavigationDispatcher,
        UiNavigationInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError>,
) -> Result<UiInputDispatchResult, UiTreeError> {
    let changed = surface
        .input
        .update_analog_control(analog.control.as_str(), analog.value);
    let retained_analog_value = surface
        .input
        .analog_controls
        .get(analog.control.as_str())
        .map(|state| state.value)
        .unwrap_or(analog.value);
    let analog_navigation =
        analog_navigation_decision(&mut surface.input, &analog, retained_analog_value);
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
        if diagnostics_mode.captures_full_trace() {
            if navigation_result.reply.disposition != UiDispatchDisposition::Unhandled {
                navigation_result.diagnostics.handled_phase = Some("analog.navigation".to_string());
            }
            navigation_result
                .diagnostics
                .notes
                .push(format!("analog_navigation={navigation_kind:?}"));
        }
        return Ok(navigation_result);
    }
    let focused = surface.focus.focused;
    let mut result = owner_routed_result_with_diagnostics_mode(
        surface,
        UiInputEvent::Analog(analog),
        focused,
        "analog.focused",
        diagnostics_mode,
    );
    if !changed {
        result.reply = UiDispatchReply::unhandled();
        result.diagnostics.routed = false;
        if diagnostics_mode.captures_full_trace() {
            result
                .diagnostics
                .notes
                .push("analog_repeat_suppressed".to_string());
        }
    }
    if diagnostics_mode.captures_full_trace() {
        if let AnalogNavigationDecision::Suppressed(navigation_kind) = analog_navigation {
            result.diagnostics.notes.push(format!(
                "analog_navigation_repeat_suppressed={navigation_kind:?}"
            ));
        }
    }
    Ok(with_analog_route_policy(surface, result, diagnostics_mode))
}

fn with_analog_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
    diagnostics_mode: UiInputDiagnosticsMode,
) -> UiInputDispatchResult {
    if diagnostics_mode.captures_full_trace() {
        let event = take_owned_input_event(&mut result.event);
        annotate_route_policy(surface, &event, &mut result);
        result.event = event;
        annotate_result_route_steps(&mut result);
    }
    result
}

fn take_owned_input_event(event: &mut UiInputEvent) -> UiInputEvent {
    std::mem::replace(
        event,
        UiInputEvent::Analog(UiAnalogInputEvent {
            metadata: Default::default(),
            control: String::new(),
            value: 0.0,
        }),
    )
}
