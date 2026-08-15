use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent, layout::UiPoint, surface::UiPointerEventKind,
};

use super::route_conversion::to_public_route;
use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_dispatch::WelcomeRecentPointerDispatch;
use super::welcome_recent_pointer_route_intent::WelcomeRecentPointerRouteIntent;

impl WelcomeRecentPointerBridge {
    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> Result<WelcomeRecentPointerDispatch, String> {
        self.refresh_layout_metrics();
        let dispatched_route = self.dispatch_event(
            UiPointerEvent::new(UiPointerEventKind::Scroll, point).with_scroll_delta(delta),
        )?;

        if dispatched_route.is_some() && delta.is_finite() {
            self.state.scroll_offset += delta;
            self.clamp_scroll_offset();
        }
        let route = self.project_route_at_point(dispatched_route, point);

        match route.as_ref() {
            Some(WelcomeRecentPointerRouteIntent::Action {
                item_index, action, ..
            }) => {
                self.state.hovered_item_index = Some(*item_index);
                self.state.hovered_action = Some(*action);
            }
            Some(WelcomeRecentPointerRouteIntent::Item(item_index)) => {
                self.state.hovered_item_index = Some(*item_index);
                self.state.hovered_action = None;
            }
            Some(WelcomeRecentPointerRouteIntent::ListSurface) | None => {
                self.state.hovered_item_index = None;
                self.state.hovered_action = None;
            }
        }

        Ok(WelcomeRecentPointerDispatch {
            route: route.map(to_public_route),
            state: self.state.clone(),
        })
    }
}
