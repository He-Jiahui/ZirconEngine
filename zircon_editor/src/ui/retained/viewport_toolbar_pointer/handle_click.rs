use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    layout::{UiFrame, UiPoint},
    surface::UiPointerEventKind,
};

use super::route_for_control::validate_control_id;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_control::ViewportToolbarPointerControl;
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
        validate_control_id(surface_key, control_id)?;
        let control = ViewportToolbarPointerControl {
            action_key: control_id.to_string(),
            frame: UiFrame::new(
                surface_frame.x + control_x,
                surface_frame.y + control_y,
                control_width.max(1.0),
                control_height.max(1.0),
            ),
        };
        if self.sync_clicked_control(surface_key, control) {
            self.applied_surface_frames.remove(surface_key);
            self.rebuild_surface();
        }

        let point = UiPoint::new(surface_frame.x + point.x, surface_frame.y + point.y);
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(ViewportToolbarPointerDispatch { route })
    }

    fn sync_clicked_control(
        &mut self,
        surface_key: &str,
        control: ViewportToolbarPointerControl,
    ) -> bool {
        if let Some(controls) = self.controls_by_surface.get_mut(surface_key) {
            if let Some(control_index) = controls
                .iter()
                .position(|existing| existing.action_key == control.action_key)
            {
                if controls[control_index] == control {
                    return false;
                }
                controls[control_index] = control;
                return true;
            }
            controls.push(control);
            return true;
        }

        self.controls_by_surface
            .insert(surface_key.to_string(), vec![control]);
        true
    }

    pub(crate) fn handle_click_at_point(
        &mut self,
        surface_key: &str,
        point: UiPoint,
    ) -> Result<ViewportToolbarPointerDispatch, String> {
        let surface_frame = self
            .surface_layout(surface_key)
            .map(|surface| surface.frame)
            .ok_or_else(|| format!("Unknown viewport toolbar surface {surface_key}"))?;

        let point = UiPoint::new(surface_frame.x + point.x, surface_frame.y + point.y);
        let route = self.dispatch_event(UiPointerEvent::new(UiPointerEventKind::Down, point))?;
        Ok(ViewportToolbarPointerDispatch { route })
    }
}
