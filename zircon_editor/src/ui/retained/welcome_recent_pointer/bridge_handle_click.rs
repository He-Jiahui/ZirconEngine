use zircon_runtime_interface::ui::layout::UiPoint;

use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_dispatch::WelcomeRecentPointerDispatch;

impl WelcomeRecentPointerBridge {
    pub(crate) fn handle_click(&mut self, point: UiPoint) -> WelcomeRecentPointerDispatch {
        let previous_state = self.state;
        self.refresh_layout_metrics();
        let route = self.route_at_point(point);
        self.apply_hit_state(route);

        WelcomeRecentPointerDispatch {
            route: route.map(|hit| hit.to_public_route()),
            state: self.state,
            changed: self.state != previous_state,
        }
    }
}
