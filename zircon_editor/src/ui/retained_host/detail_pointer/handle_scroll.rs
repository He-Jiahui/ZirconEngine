use zircon_runtime_interface::ui::layout::UiPoint;

use super::scroll_surface_pointer_bridge::ScrollSurfacePointerBridge;
use super::scroll_surface_pointer_dispatch::ScrollSurfacePointerDispatch;
use super::scroll_surface_pointer_route::ScrollSurfacePointerRoute;
use super::viewport_frame::viewport_frame;

impl ScrollSurfacePointerBridge {
    pub(crate) fn handle_scroll(
        &mut self,
        point: UiPoint,
        delta: f32,
    ) -> ScrollSurfacePointerDispatch {
        let point_is_inside = point.x.is_finite()
            && point.y.is_finite()
            && viewport_frame(&self.layout).contains_point(point);
        let previous_offset = self.state.scroll_offset;
        if point_is_inside && delta.is_finite() {
            self.state.scroll_offset += delta;
            self.clamp_scroll_offset();
        }

        ScrollSurfacePointerDispatch {
            route: point_is_inside.then_some(ScrollSurfacePointerRoute::Viewport),
            state: self.state,
            changed: self.state.scroll_offset != previous_offset,
        }
    }
}
