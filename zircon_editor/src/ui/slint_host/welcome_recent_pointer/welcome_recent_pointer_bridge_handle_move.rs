use zircon_ui::{dispatch::UiPointerEvent, UiPoint, UiPointerEventKind};

use super::route_conversion::to_public_route;
use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_dispatch::WelcomeRecentPointerDispatch;
use super::welcome_recent_pointer_target::WelcomeRecentPointerTarget;

impl WelcomeRecentPointerBridge {
    pub(crate) fn handle_move(
        &mut self,
        point: UiPoint,
    ) -> Result<WelcomeRecentPointerDispatch, String> {
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Move, point))?;
        match route.as_ref() {
            Some(WelcomeRecentPointerTarget::Action {
                item_index, action, ..
            }) => {
                self.state.hovered_item_index = Some(*item_index);
                self.state.hovered_action = Some(*action);
            }
            Some(WelcomeRecentPointerTarget::Item(item_index)) => {
                self.state.hovered_item_index = Some(*item_index);
                self.state.hovered_action = None;
            }
            Some(WelcomeRecentPointerTarget::ListSurface) | None => {
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
