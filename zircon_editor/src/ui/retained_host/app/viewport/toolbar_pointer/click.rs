use std::sync::Arc;

use super::super::super::RetainedEditorHost;
use crate::ui::retained_host::{
    callback_dispatch, viewport_toolbar_pointer::build_viewport_toolbar_pointer_layout_with_size,
    HostWindowPresentationData,
};
use zircon_runtime_interface::ui::{layout::UiPoint, layout::UiSize, surface::UiSurfaceFrame};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn viewport_toolbar_pointer_clicked(
        &mut self,
        surface_key: &str,
        point_x: f32,
        point_y: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let surface_size = if width > f32::EPSILON && height > f32::EPSILON {
            UiSize::new(width, height)
        } else {
            self.viewport_toolbar_surface_size(surface_key)
        };
        self.viewport_toolbar_pointer_bridge
            .sync(build_viewport_toolbar_pointer_layout_with_size(
                [surface_key],
                surface_size,
            ));

        let generation = self.ui.get_host_presentation_generation();
        let Some(surface_frame) =
            viewport_toolbar_surface_frame_for_surface(generation.structure(), surface_key)
        else {
            self.set_status_line(format!(
                "Missing viewport toolbar surface frame for {surface_key}"
            ));
            return;
        };

        match callback_dispatch::dispatch_shared_viewport_toolbar_pointer_click_at_point(
            &self.runtime,
            &self.viewport_toolbar_bridge,
            &mut self.viewport_toolbar_pointer_bridge,
            surface_key,
            &surface_frame,
            UiPoint::new(point_x, point_y),
        ) {
            Ok(dispatch) => {
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }
}

fn viewport_toolbar_surface_frame_for_surface(
    presentation: &HostWindowPresentationData,
    surface_key: &str,
) -> Option<Arc<UiSurfaceFrame>> {
    let scene = &presentation.host_scene_data;
    let dock_toolbar_frames: [(&str, Option<&Arc<UiSurfaceFrame>>); 4] = [
        (
            scene.document_dock.surface_key.as_str(),
            scene
                .document_dock
                .pane
                .viewport
                .toolbar_surface_frame
                .as_ref(),
        ),
        (
            scene.left_dock.surface_key.as_str(),
            scene.left_dock.pane.viewport.toolbar_surface_frame.as_ref(),
        ),
        (
            scene.right_dock.surface_key.as_str(),
            scene
                .right_dock
                .pane
                .viewport
                .toolbar_surface_frame
                .as_ref(),
        ),
        (
            scene.bottom_dock.surface_key.as_str(),
            scene
                .bottom_dock
                .pane
                .viewport
                .toolbar_surface_frame
                .as_ref(),
        ),
    ];
    for (candidate_key, frame) in dock_toolbar_frames {
        if candidate_key == surface_key {
            return frame.cloned();
        }
    }

    for row in 0..scene.floating_layer.floating_windows.row_count() {
        let Some(window) = scene.floating_layer.floating_windows.row_data(row) else {
            continue;
        };
        if window.window_id.as_str() == surface_key {
            return window.active_pane.viewport.toolbar_surface_frame.clone();
        }
    }

    None
}
