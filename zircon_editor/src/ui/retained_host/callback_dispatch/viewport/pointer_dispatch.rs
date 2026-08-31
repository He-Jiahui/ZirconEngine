use zircon_runtime::ui::dispatch::UiPointerDispatcher;
use zircon_runtime_interface::ui::{
    dispatch::{UiInputModifiers, UiPointerDispatchEffect, UiPointerEvent},
    surface::{UiPointerButton, UiPointerEventKind, UiPointerRoute},
};

use crate::core::editor_event::EditorViewportEvent;
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;

use super::super::constants::VIEWPORT_SURFACE_NODE_ID;
use super::{dispatch_viewport_event, SharedViewportPointerBridge};

pub(crate) fn dispatch_viewport_pointer_event(
    runtime: &EditorHostEventController,
    bridge: &mut SharedViewportPointerBridge,
    event: UiPointerEvent,
    modifiers: UiInputModifiers,
) -> Result<UiHostEventEffects, String> {
    let dispatch = bridge
        .surface
        .dispatch_pointer_event(&bridge.dispatcher, event)
        .map_err(|error| error.to_string())?;

    if dispatch.handled_by != Some(bridge.viewport_node_id)
        && dispatch.captured_by != Some(bridge.viewport_node_id)
    {
        return Ok(UiHostEventEffects::default());
    }

    let Some(viewport_event) = map_pointer_route_to_viewport_event(&dispatch.route, modifiers)
    else {
        return Ok(UiHostEventEffects::default());
    };

    dispatch_viewport_event(runtime, viewport_event)
}

pub(super) fn viewport_pointer_dispatcher() -> UiPointerDispatcher {
    let mut dispatcher = UiPointerDispatcher::default();
    dispatcher.register(
        VIEWPORT_SURFACE_NODE_ID,
        UiPointerEventKind::Down,
        |_context| UiPointerDispatchEffect::capture(),
    );
    dispatcher.register(
        VIEWPORT_SURFACE_NODE_ID,
        UiPointerEventKind::Move,
        |_context| UiPointerDispatchEffect::handled(),
    );
    dispatcher.register(
        VIEWPORT_SURFACE_NODE_ID,
        UiPointerEventKind::Up,
        |_context| UiPointerDispatchEffect::handled(),
    );
    dispatcher.register(
        VIEWPORT_SURFACE_NODE_ID,
        UiPointerEventKind::Scroll,
        |_context| UiPointerDispatchEffect::handled(),
    );
    dispatcher.register(
        VIEWPORT_SURFACE_NODE_ID,
        UiPointerEventKind::Cancel,
        |_context| UiPointerDispatchEffect::handled(),
    );
    dispatcher
}

fn map_pointer_route_to_viewport_event(
    route: &UiPointerRoute,
    modifiers: UiInputModifiers,
) -> Option<EditorViewportEvent> {
    match route.kind {
        UiPointerEventKind::Down => match route.button? {
            UiPointerButton::Primary => Some(EditorViewportEvent::LeftPressed {
                x: route.point.x,
                y: route.point.y,
                selection_mutation: crate::scene::selection::SelectionMutation::from_modifier_flags(
                    modifiers.shift,
                    modifiers.control,
                ),
            }),
            UiPointerButton::Secondary => Some(EditorViewportEvent::RightPressed {
                x: route.point.x,
                y: route.point.y,
            }),
            UiPointerButton::Middle => Some(EditorViewportEvent::MiddlePressed {
                x: route.point.x,
                y: route.point.y,
            }),
        },
        UiPointerEventKind::Up => match route.button? {
            UiPointerButton::Primary => Some(EditorViewportEvent::LeftReleased),
            UiPointerButton::Secondary => Some(EditorViewportEvent::RightReleased),
            UiPointerButton::Middle => Some(EditorViewportEvent::MiddleReleased),
        },
        UiPointerEventKind::Move => Some(EditorViewportEvent::PointerMoved {
            x: route.point.x,
            y: route.point.y,
        }),
        UiPointerEventKind::Scroll => Some(EditorViewportEvent::Scrolled {
            delta: route.scroll_delta,
        }),
        UiPointerEventKind::Cancel => Some(EditorViewportEvent::CancelInteraction),
    }
}
