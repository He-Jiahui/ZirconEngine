use super::super::super::{RetainedEditorHost, UiDragPayload, UiPoint};
use super::super::drag_source::scene_drag_payload_from_route;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn hierarchy_pointer_event(
        &mut self,
        kind: i32,
        button: i32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        if button == 1 && kind == 2 {
            self.active_scene_drag_payload = None;
            return;
        }
        if kind != 0 || button != 1 {
            return;
        }
        self.active_asset_drag_payload = None;
        self.active_object_drag_payload = None;

        let scene_entries = self.prepare_hierarchy_pointer_target(width, height, true);

        match self
            .hierarchy_pointer_bridge
            .handle_move(UiPoint::new(x, y))
        {
            Ok(dispatch) => {
                self.hierarchy_pointer_state = dispatch.state;
                self.apply_hierarchy_pointer_state_to_ui();
                self.active_scene_drag_payload =
                    scene_drag_payload_from_route(dispatch.route, &scene_entries);
                if let Some(summary) = self
                    .active_scene_drag_payload
                    .as_ref()
                    .and_then(UiDragPayload::source_summary)
                {
                    self.set_status_line(format!("Scene drag source: {summary}"));
                }
            }
            Err(error) => {
                self.active_scene_drag_payload = None;
                self.set_status_line(error);
            }
        }
    }
}
