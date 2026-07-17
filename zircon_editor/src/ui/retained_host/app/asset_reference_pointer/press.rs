use super::super::*;

impl RetainedEditorHost {
    pub(super) fn dispatch_asset_reference_pointer_press(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.active_scene_drag_payload = None;
        self.active_object_drag_payload = None;
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) = self.prepare_asset_reference_pointer_target(
            surface_mode,
            list_kind,
            width,
            height,
            true,
        ) else {
            return;
        };
        let point = UiPoint::new(x, y);
        let Some(dispatch) = self.dispatch_prepared_asset_reference_pointer(
            surface_mode,
            list_kind,
            &target,
            true,
            |bridge| bridge.handle_press(point),
        ) else {
            return;
        };

        match dispatch {
            Ok(dispatch) => {
                self.write_asset_reference_pointer_state(surface_mode, list_kind, dispatch.state);
                if let Some(AssetPointerReferenceRoute::Item { asset_uuid, .. }) = dispatch.route {
                    self.active_asset_drag_payload =
                        asset_drag_payload::asset_drag_payload_from_reference(
                            surface_mode,
                            list_kind,
                            asset_uuid.as_str(),
                            target.snapshot.as_ref(),
                        );
                    if let Some(summary) = self
                        .active_asset_drag_payload
                        .as_ref()
                        .and_then(UiDragPayload::source_summary)
                    {
                        self.set_status_line(format!("Asset reference drag source: {summary}"));
                    }
                } else {
                    self.active_asset_drag_payload = None;
                }
            }
            Err(error) => {
                self.active_asset_drag_payload = None;
                self.set_status_line(error);
            }
        }
    }
}
