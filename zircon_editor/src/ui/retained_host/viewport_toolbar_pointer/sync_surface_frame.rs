use std::sync::Arc;

use zircon_runtime_interface::ui::{layout::UiFrame, surface::UiSurfaceFrame};

use super::rebuild_surface::{
    viewport_toolbar_control_node_id, ViewportToolbarNodeFrameChange, ViewportToolbarSurfaceDelta,
};
use super::route_for_control::control_route_for_id;
use super::viewport_toolbar_pointer_bridge::ViewportToolbarPointerBridge;
use super::viewport_toolbar_pointer_control::ViewportToolbarPointerControl;

impl ViewportToolbarPointerBridge {
    pub(crate) fn sync_surface_frame(
        &mut self,
        surface_key: &str,
        surface_frame: &Arc<UiSurfaceFrame>,
    ) -> Result<bool, String> {
        let (surface_index, surface_origin) = self
            .layout
            .surfaces
            .iter()
            .enumerate()
            .find(|(_, surface)| surface.key == surface_key)
            .map(|(index, surface)| (index, surface.frame))
            .ok_or_else(|| format!("Unknown viewport toolbar surface {surface_key}"))?;
        if self.applied_surface_frames.get(surface_key).is_some_and(
            |(applied_frame, applied_origin)| {
                *applied_origin == surface_origin
                    && std::ptr::eq(applied_frame.as_ptr(), Arc::as_ptr(&surface_frame.hit_grid))
            },
        ) {
            return Ok(false);
        }
        let existing = self
            .controls_by_surface
            .get(surface_key)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let existing_capacity = existing.len();
        let mut controls = Vec::with_capacity(existing_capacity);
        let mut changes = Vec::new();
        let mut topology_changed = false;
        for entry in surface_frame.hit_grid.entries.iter() {
            let Some(control_id) = entry.control_id.as_deref() else {
                continue;
            };
            if control_route_for_id(control_id).is_none() {
                continue;
            }
            let frame = UiFrame::new(
                surface_origin.x + entry.frame.x,
                surface_origin.y + entry.frame.y,
                entry.frame.width.max(1.0),
                entry.frame.height.max(1.0),
            );
            let control_index = controls.len();
            match existing.get(control_index) {
                Some(current) if current.action_key == control_id => {
                    if current.frame != frame {
                        changes.push(ViewportToolbarNodeFrameChange {
                            node_id: viewport_toolbar_control_node_id(surface_index, control_index),
                            frame,
                        });
                    }
                }
                Some(_) | None => topology_changed = true,
            }
            controls.push(ViewportToolbarPointerControl {
                action_key: control_id.to_string(),
                frame,
            });
        }
        topology_changed |= existing.len() != controls.len();

        self.applied_surface_frames.insert(
            surface_key.to_string(),
            (Arc::downgrade(&surface_frame.hit_grid), surface_origin),
        );

        if !topology_changed && changes.is_empty() {
            self.apply_surface_delta(ViewportToolbarSurfaceDelta::NoChange);
            return Ok(false);
        }

        self.controls_by_surface
            .insert(surface_key.to_string(), controls);
        self.apply_surface_delta(if topology_changed {
            ViewportToolbarSurfaceDelta::Topology
        } else {
            ViewportToolbarSurfaceDelta::Geometry(changes)
        });
        Ok(true)
    }
}

#[cfg(test)]
#[path = "tests/sync_surface_frame.rs"]
mod tests;
