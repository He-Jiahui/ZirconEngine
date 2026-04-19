use zircon_runtime::ui::{
    dispatch::UiPointerEvent, layout::UiFrame, layout::UiPoint, surface::UiPointerEventKind,
};

use super::active_viewport_toolbar_control::ActiveViewportToolbarControl;
use super::route_for_control::route_for_control;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_dispatch::ViewportToolbarPointerDispatch;

impl ViewportToolbarPointerBridge {
    pub(crate) fn handle_click(
        &mut self,
        surface_key: &str,
        control_id: &str,
        control_x: f32,
        control_y: f32,
        control_width: f32,
        control_height: f32,
        point: UiPoint,
    ) -> Result<ViewportToolbarPointerDispatch, String> {
        let surface_frame = self
            .surface_layout(surface_key)
            .map(|surface| surface.frame)
            .ok_or_else(|| format!("Unknown viewport toolbar surface {surface_key}"))?;
        route_for_control(surface_key, control_id)?;
        self.active_controls.insert(
            surface_key.to_string(),
            ActiveViewportToolbarControl {
                action_key: control_id.to_string(),
                frame: UiFrame::new(
                    surface_frame.x + control_x,
                    surface_frame.y + control_y,
                    control_width.max(1.0),
                    control_height.max(1.0),
                ),
            },
        );
        self.rebuild_surface();

        let point = UiPoint::new(surface_frame.x + point.x, surface_frame.y + point.y);
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(ViewportToolbarPointerDispatch {
            route: route.map(|target| target.route),
        })
    }
}
