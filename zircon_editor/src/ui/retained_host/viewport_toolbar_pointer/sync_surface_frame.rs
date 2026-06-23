use zircon_runtime_interface::ui::{layout::UiFrame, surface::UiSurfaceFrame};

use super::route_for_control::route_for_control;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_control::ViewportToolbarPointerControl;

impl ViewportToolbarPointerBridge {
    pub(crate) fn sync_surface_frame(
        &mut self,
        surface_key: &str,
        surface_frame: &UiSurfaceFrame,
    ) -> Result<bool, String> {
        let surface_origin = self
            .surface_layout(surface_key)
            .map(|surface| surface.frame)
            .ok_or_else(|| format!("Unknown viewport toolbar surface {surface_key}"))?;
        let controls = surface_frame
            .arranged_tree
            .nodes
            .iter()
            .filter_map(|node| {
                let control_id = node.control_id.as_deref()?;
                route_for_control(surface_key, control_id).ok()?;
                Some(ViewportToolbarPointerControl {
                    action_key: control_id.to_string(),
                    frame: UiFrame::new(
                        surface_origin.x + node.frame.x,
                        surface_origin.y + node.frame.y,
                        node.frame.width.max(1.0),
                        node.frame.height.max(1.0),
                    ),
                })
            })
            .collect::<Vec<_>>();

        if self
            .controls_by_surface
            .get(surface_key)
            .is_some_and(|existing| *existing == controls)
        {
            return Ok(false);
        }

        self.controls_by_surface
            .insert(surface_key.to_string(), controls);
        self.rebuild_surface();
        Ok(true)
    }
}
