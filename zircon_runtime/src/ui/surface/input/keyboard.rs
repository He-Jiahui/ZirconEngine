use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchDisposition, UiDispatchPhase, UiDispatchReply, UiInputDispatchResult,
        UiInputEvent, UiKeyboardInputEvent, UiNavigationInputEvent,
    },
    focus::UiFocusedInputKind,
    tree::UiTreeError,
};

use crate::{ui::dispatch::UiNavigationDispatcher, ui::tree::UiRuntimeTreeRoutingExt};

use super::{
    super::surface::UiSurface,
    editable_text::dispatch_keyboard_text_edit,
    is_valid_input_owner,
    keyboard_action::{keyboard_requests_default_activation, keyboard_requests_popup_dismissal},
    keyboard_navigation::keyboard_navigation_kind,
    owner_route::owner_routed_result,
    route_policy::annotate_route_policy,
    route_steps::annotate_result_route_steps,
};

pub(super) fn dispatch_keyboard_input(
    surface: &mut UiSurface,
    navigation_dispatcher: &UiNavigationDispatcher,
    keyboard: UiKeyboardInputEvent,
    dispatch_navigation_input: impl FnOnce(
        &mut UiSurface,
        &UiNavigationDispatcher,
        UiNavigationInputEvent,
    ) -> Result<UiInputDispatchResult, UiTreeError>,
) -> Result<UiInputDispatchResult, UiTreeError> {
    let target = surface.focus.focused;
    let mut result = UiInputDispatchResult::new(
        UiInputEvent::Keyboard(keyboard.clone()),
        UiDispatchReply::unhandled(),
    );
    result.diagnostics.routed = target.is_some();
    result.diagnostics.route_target = target;
    if let Some(target) = target {
        if !is_valid_input_owner(surface, target) {
            let result = owner_routed_result(
                surface,
                UiInputEvent::Keyboard(keyboard),
                Some(target),
                "keyboard.focused",
            );
            return Ok(with_keyboard_route_policy(surface, result));
        }
        let route = surface.tree.bubble_route(target)?;
        result
            .diagnostics
            .notes
            .push(format!("focused_route_len={}", route.len()));
        if let Some(mut text_result) =
            dispatch_keyboard_text_edit(surface, keyboard.clone(), target)
        {
            text_result
                .diagnostics
                .notes
                .insert(0, format!("focused_route_len={}", route.len()));
            return Ok(with_keyboard_route_policy(surface, text_result));
        }
        if let Some(kind) = keyboard_navigation_kind(&keyboard) {
            let focused_route_len = route.len();
            let mut navigation_result = dispatch_navigation_input(
                surface,
                navigation_dispatcher,
                UiNavigationInputEvent {
                    metadata: keyboard.metadata.clone(),
                    kind,
                },
            )?;
            navigation_result.event = UiInputEvent::Keyboard(keyboard);
            if navigation_result.reply.disposition != UiDispatchDisposition::Unhandled {
                navigation_result.diagnostics.handled_phase =
                    Some("keyboard.navigation".to_string());
            }
            navigation_result
                .diagnostics
                .notes
                .insert(0, format!("focused_route_len={focused_route_len}"));
            navigation_result
                .diagnostics
                .notes
                .push(format!("keyboard_navigation={kind:?}"));
            return Ok(navigation_result);
        }
        if keyboard_requests_popup_dismissal(&keyboard) {
            let report = surface.apply_default_popup_dismissal_action(target)?;
            if report.handled {
                result.reply = UiDispatchReply::handled()
                    .from_handler(target)
                    .in_phase(UiDispatchPhase::Target);
                result.diagnostics.handled_phase = Some("keyboard.popup_dismiss".to_string());
                result.component_events = report.component_events;
                result.binding_reports = report.binding_reports;
            }
        }
        if keyboard_requests_default_activation(&keyboard) {
            let report = surface.apply_default_keyboard_component_action(target)?;
            if report.handled {
                result.reply = UiDispatchReply::handled()
                    .from_handler(target)
                    .in_phase(UiDispatchPhase::Target);
                result.diagnostics.handled_phase = Some("keyboard.widget".to_string());
                result.component_events = report.component_events;
                result.binding_reports = report.binding_reports;
            }
        }
        surface.record_focused_input(
            UiFocusedInputKind::Keyboard,
            target,
            route,
            Some(target),
            true,
        );
    }
    Ok(with_keyboard_route_policy(surface, result))
}

fn with_keyboard_route_policy(
    surface: &UiSurface,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let event = result.event.clone();
    annotate_route_policy(surface, &event, &mut result);
    annotate_result_route_steps(&mut result);
    result
}
