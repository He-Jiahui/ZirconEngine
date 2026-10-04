use super::super::super::{callback_dispatch, RetainedEditorHost, UiDragPayload, UiPoint};
use super::super::drag_source::{
    hierarchy_drag_source_from_route, hierarchy_reparent_target_from_route,
};
use super::super::input_owner::ReleaseOwnership;
use super::super::HierarchyTerminalReason;

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
            match self
                .hierarchy_input_owner
                .release_ownership(&self.callback_source_window)
            {
                ReleaseOwnership::DifferentPointer => return,
                ReleaseOwnership::ForeignSamePointer => {
                    self.retire_hierarchy_drag_with_reason(HierarchyTerminalReason::ForeignWindow);
                    return;
                }
                ReleaseOwnership::Owner | ReleaseOwnership::NoOwner => {}
            }
            if !self.hierarchy_drag_identity_is_current() {
                self.retire_hierarchy_drag();
                return;
            }
            let drag_armed = self.active_scene_drag_payload.take().is_some();
            let node_ids = std::mem::take(&mut self.active_hierarchy_drag_node_ids);
            if !drag_armed || node_ids.is_empty() {
                self.retire_hierarchy_drag();
                return;
            }
            let scene_entries = self.prepare_hierarchy_pointer_target(width, height, true);
            let dispatch = self
                .hierarchy_pointer_bridge
                .handle_move(UiPoint::new(x, y));
            self.hierarchy_pointer_state = dispatch.state;
            self.apply_hierarchy_pointer_state_to_ui();
            if !self.hierarchy_pointer_bridge.finish_reparent_drag() {
                self.retire_hierarchy_drag();
                return;
            }
            let Some(parent) = hierarchy_reparent_target_from_route(dispatch.route, &scene_entries)
            else {
                self.retire_hierarchy_drag();
                return;
            };
            if !self.hierarchy_drag_identity_is_current() {
                self.retire_hierarchy_drag();
                return;
            }
            self.hierarchy_input_owner
                .finish(HierarchyTerminalReason::Released);
            self.active_hierarchy_drag_identity = None;
            match callback_dispatch::dispatch_hierarchy_reparent(&self.runtime, node_ids, parent) {
                Ok(effects) => self.apply_dispatch_effects(effects),
                Err(error) => self.set_status_line(error),
            }
            return;
        }
        if kind != 0 || button != 1 {
            return;
        }
        if !self
            .hierarchy_input_owner
            .admits_primary_press(&self.callback_source_window)
        {
            return;
        }
        self.retire_hierarchy_drag();
        self.active_asset_drag_payload = None;
        self.active_object_drag_payload = None;

        let scene_entries = self.prepare_hierarchy_pointer_target(width, height, true);
        let authoritative_scene_entries = self.runtime.editor_snapshot().scene_entries;

        let dispatch = self
            .hierarchy_pointer_bridge
            .handle_move(UiPoint::new(x, y));
        self.hierarchy_pointer_state = dispatch.state;
        self.apply_hierarchy_pointer_state_to_ui();
        if let (Some(source), Some(identity)) = (
            hierarchy_drag_source_from_route(
                dispatch.route,
                &scene_entries,
                &authoritative_scene_entries,
            ),
            self.capture_hierarchy_drag_identity(),
        ) {
            if !self
                .hierarchy_input_owner
                .arm(self.callback_source_window.clone())
            {
                self.retire_hierarchy_drag();
                return;
            }
            self.active_hierarchy_drag_node_ids = source.node_ids;
            self.active_scene_drag_payload = Some(source.payload);
            self.active_hierarchy_drag_identity = Some(identity);
            self.hierarchy_pointer_bridge
                .begin_reparent_drag(UiPoint::new(x, y));
        }
        if let Some(summary) = self
            .active_scene_drag_payload
            .as_ref()
            .and_then(UiDragPayload::source_summary)
        {
            self.set_status_line(format!("Scene drag source: {summary}"));
        }
    }
}
