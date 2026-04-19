use zircon_ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerDispatcher, UiPointerEvent},
    UiPointerEventKind, UiPointerRoute,
};

use crate::core::editor_event::{EditorEventRuntime, EditorViewportEvent};
use crate::ui::slint_host::event_bridge::SlintDispatchEffects;

use super::super::constants::VIEWPORT_SURFACE_NODE_ID;
use super::{dispatch_viewport_event, SharedViewportPointerBridge};

pub(crate) fn dispatch_viewport_pointer_event(
    runtime: &EditorEventRuntime,
    bridge: &mut SharedViewportPointerBridge,
    event: UiPointerEvent,
) -> Result<SlintDispatchEffects, String> {
    let dispatcher = viewport_pointer_dispatcher();
    let dispatch = bridge
        .surface
        .dispatch_pointer_event(&dispatcher, event)
        .map_err(|error| error.to_string())?;

    if dispatch.handled_by != Some(bridge.viewport_node_id)
        && dispatch.captured_by != Some(bridge.viewport_node_id)
    {
        return Ok(SlintDispatchEffects::default());
    }

    let Some(viewport_event) = map_pointer_route_to_viewport_event(&dispatch.route) else {
        return Ok(SlintDispatchEffects::default());
    };

    dispatch_viewport_event(runtime, viewport_event)
}

fn viewport_pointer_dispatcher() -> UiPointerDispatcher {
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
    dispatcher
}

fn map_pointer_route_to_viewport_event(route: &UiPointerRoute) -> Option<EditorViewportEvent> {
    match route.kind {
        UiPointerEventKind::Down => match route.button? {
            zircon_ui::UiPointerButton::Primary => Some(EditorViewportEvent::LeftPressed {
                x: route.point.x,
                y: route.point.y,
            }),
            zircon_ui::UiPointerButton::Secondary => Some(EditorViewportEvent::RightPressed {
                x: route.point.x,
                y: route.point.y,
            }),
            zircon_ui::UiPointerButton::Middle => Some(EditorViewportEvent::MiddlePressed {
                x: route.point.x,
                y: route.point.y,
            }),
        },
        UiPointerEventKind::Up => match route.button? {
            zircon_ui::UiPointerButton::Primary => Some(EditorViewportEvent::LeftReleased),
            zircon_ui::UiPointerButton::Secondary => Some(EditorViewportEvent::RightReleased),
            zircon_ui::UiPointerButton::Middle => Some(EditorViewportEvent::MiddleReleased),
        },
        UiPointerEventKind::Move => Some(EditorViewportEvent::PointerMoved {
            x: route.point.x,
            y: route.point.y,
        }),
        UiPointerEventKind::Scroll => Some(EditorViewportEvent::Scrolled {
            delta: route.scroll_delta,
        }),
    }
}
