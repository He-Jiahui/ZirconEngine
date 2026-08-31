use super::super::*;
use zircon_runtime_interface::ui::{dispatch::UiPointerEvent, surface::UiPointerEventKind};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn inspector_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        match self.route_workbench_inspector_scroll(x, y, delta) {
            Ok(true) => return,
            Ok(false) => {}
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        }
        let size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.inspector_scroll_surface.size(),
            ViewContentKind::Inspector,
        );
        if self.inspector_scroll_surface.set_size(size) {
            self.sync_inspector_pointer_layout();
        }
        if self
            .inspector_scroll_surface
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            self.apply_inspector_pointer_state_to_ui();
        }
    }

    fn route_workbench_inspector_scroll(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
    ) -> Result<bool, String> {
        let Some(inspector_frame) = self
            .workbench_window_bridge
            .layout_frames()
            .right_region_frame
        else {
            return Ok(false);
        };
        let event = UiPointerEvent::new(
            UiPointerEventKind::Scroll,
            UiPoint::new(inspector_frame.x + x, inspector_frame.y + y),
        )
        .with_scroll_delta(delta);
        let route = self
            .workbench_window_bridge
            .route_pointer_event(event)
            .map_err(|error| error.to_string())?;
        let changed = self
            .workbench_window_bridge
            .refresh_component_property_rows_after_scroll(&route)
            .map_err(|error| error.to_string())?;
        let refreshed = self
            .workbench_window_bridge
            .refresh_pointer_feedback(changed)
            .map_err(|error| error.to_string())?;
        if !refreshed {
            return Ok(false);
        }
        let mut effects = UiHostEventEffects::default();
        effects.request_paint_only();
        self.apply_dispatch_effects(effects);
        Ok(true)
    }
}
