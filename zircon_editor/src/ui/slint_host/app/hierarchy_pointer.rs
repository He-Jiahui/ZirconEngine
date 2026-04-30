use super::*;

impl SlintEditorHost {
    pub(super) fn hierarchy_pointer_event(
        &mut self,
        kind: i32,
        button: i32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        if button == 1 && kind == 2 {
            self.active_scene_drag_payload = None;
            return;
        }
        if kind != 0 || button != 1 {
            return;
        }
        self.active_asset_drag_payload = None;
        self.active_object_drag_payload = None;

        self.hierarchy_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = self.runtime.editor_snapshot().scene_entries;
        self.sync_hierarchy_pointer_layout(&scene_entries);
        self.focus_callback_source_window();

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

    pub(super) fn hierarchy_pointer_clicked(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.hierarchy_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = self.runtime.editor_snapshot().scene_entries;
        self.sync_hierarchy_pointer_layout(&scene_entries);
        self.focus_callback_source_window();
        match callback_dispatch::dispatch_shared_hierarchy_pointer_click(
            &self.runtime,
            &mut self.hierarchy_pointer_bridge,
            UiPoint::new(x, y),
        ) {
            Ok(dispatch) => {
                self.hierarchy_pointer_state = dispatch.pointer.state;
                self.apply_hierarchy_pointer_state_to_ui();
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn hierarchy_pointer_moved(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.hierarchy_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = self.runtime.editor_snapshot().scene_entries;
        self.sync_hierarchy_pointer_layout(&scene_entries);
        match self
            .hierarchy_pointer_bridge
            .handle_move(UiPoint::new(x, y))
        {
            Ok(dispatch) => {
                self.hierarchy_pointer_state = dispatch.state;
                self.apply_hierarchy_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }

    pub(super) fn hierarchy_pointer_scrolled(
        &mut self,
        x: f32,
        y: f32,
        delta: f32,
        width: f32,
        height: f32,
    ) {
        self.hierarchy_pointer_size = self.resolve_callback_surface_size_for_kind(
            width,
            height,
            self.hierarchy_pointer_size,
            ViewContentKind::Hierarchy,
        );
        let scene_entries = self.runtime.editor_snapshot().scene_entries;
        self.sync_hierarchy_pointer_layout(&scene_entries);
        self.focus_callback_source_window();
        match self
            .hierarchy_pointer_bridge
            .handle_scroll(UiPoint::new(x, y), delta)
        {
            Ok(dispatch) => {
                self.hierarchy_pointer_state = dispatch.state;
                self.apply_hierarchy_pointer_state_to_ui();
            }
            Err(error) => self.set_status_line(error),
        }
    }
}

fn scene_drag_payload_from_route(
    route: Option<crate::ui::slint_host::hierarchy_pointer::HierarchyPointerRoute>,
    scene_entries: &[SceneEntry],
) -> Option<UiDragPayload> {
    let crate::ui::slint_host::hierarchy_pointer::HierarchyPointerRoute::Node { node_id, .. } =
        route?
    else {
        return None;
    };
    scene_entries
        .iter()
        .find(|entry| entry.id.to_string() == node_id)
        .map(scene_drag_payload_from_entry)
}

fn scene_drag_payload_from_entry(entry: &SceneEntry) -> UiDragPayload {
    let reference = format!("scene://node/{}", entry.id);
    UiDragPayload::new(UiDragPayloadKind::SceneInstance, reference.clone()).with_source(
        UiDragSourceMetadata {
            source_surface: "hierarchy".to_string(),
            source_control_id: "HierarchyListPanel".to_string(),
            locator: Some(reference),
            display_name: Some(entry.name.clone()),
            asset_kind: Some("Scene Instance".to_string()),
            ..UiDragSourceMetadata::default()
        },
    )
}
