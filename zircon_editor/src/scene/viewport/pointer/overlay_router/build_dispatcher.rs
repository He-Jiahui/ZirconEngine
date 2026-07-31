use std::sync::{Arc, Mutex};

use zircon_runtime::ui::dispatch::UiPointerDispatcher;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchEffect, surface::UiPointerEventKind,
};

use crate::scene::viewport::pointer::{
    constants::VIEWPORT_NODE_ID, precision::SharedResolutionState,
    runtime_picking_adapter::resolve_runtime_route_and_debug_feed,
};

pub(in crate::scene::viewport::pointer) fn build_dispatcher(
    shared: Arc<Mutex<SharedResolutionState>>,
) -> UiPointerDispatcher {
    let mut dispatcher = UiPointerDispatcher::default();
    for kind in [
        UiPointerEventKind::Move,
        UiPointerEventKind::Down,
        UiPointerEventKind::Up,
        UiPointerEventKind::Scroll,
    ] {
        let shared_state = Arc::clone(&shared);
        dispatcher.register(VIEWPORT_NODE_ID, kind, move |context| {
            let Ok(mut shared) = shared_state.lock() else {
                return UiPointerDispatchEffect::Unhandled;
            };
            let (route, debug_feed) = resolve_runtime_route_and_debug_feed(
                &shared.candidates,
                &context.route.stacked,
                context.route.point,
            );
            shared.last_route = route;
            shared.last_debug_feed = Some(debug_feed);
            if shared.last_route.is_some() {
                UiPointerDispatchEffect::handled()
            } else {
                UiPointerDispatchEffect::Unhandled
            }
        });
    }
    dispatcher
}
