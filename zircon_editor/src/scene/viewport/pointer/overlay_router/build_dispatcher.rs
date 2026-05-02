use std::sync::{Arc, Mutex};

use zircon_runtime::ui::dispatch::UiPointerDispatcher;
use zircon_runtime_interface::ui::{
    dispatch::UiPointerDispatchEffect, surface::UiPointerEventKind,
};

use crate::scene::viewport::pointer::{
    constants::VIEWPORT_NODE_ID, precision::SharedResolutionState,
};

use super::resolve_best_route::resolve_best_route;

pub(in crate::scene::viewport::pointer) fn build_dispatcher(
    shared: Arc<Mutex<SharedResolutionState>>,
) -> UiPointerDispatcher {
    let mut dispatcher = UiPointerDispatcher::default();
    for kind in [UiPointerEventKind::Move, UiPointerEventKind::Down] {
        let shared_state = Arc::clone(&shared);
        dispatcher.register(VIEWPORT_NODE_ID, kind, move |context| {
            let Ok(mut shared) = shared_state.lock() else {
                return UiPointerDispatchEffect::Unhandled;
            };
            shared.last_route = resolve_best_route(
                &shared.candidates,
                &context.route.stacked,
                context.route.point,
            );
            if shared.last_route.is_some() {
                UiPointerDispatchEffect::handled()
            } else {
                UiPointerDispatchEffect::Unhandled
            }
        });
    }
    dispatcher
}
