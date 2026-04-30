use super::*;

impl SlintEditorHost {
    pub(super) fn take_active_reference_drag_payload_for_drop(
        &mut self,
        action_id: &str,
    ) -> Option<UiDragPayload> {
        let preferred_kinds = if action_id.contains("AssetFieldDropped") {
            &[
                UiDragPayloadKind::Asset,
                UiDragPayloadKind::SceneInstance,
                UiDragPayloadKind::Object,
            ][..]
        } else if action_id.contains("InstanceFieldDropped") {
            &[
                UiDragPayloadKind::SceneInstance,
                UiDragPayloadKind::Asset,
                UiDragPayloadKind::Object,
            ][..]
        } else if action_id.contains("ObjectFieldDropped") {
            &[
                UiDragPayloadKind::Object,
                UiDragPayloadKind::SceneInstance,
                UiDragPayloadKind::Asset,
            ][..]
        } else {
            return None;
        };

        let payload = preferred_kinds
            .iter()
            .find_map(|kind| self.take_active_reference_drag_payload_kind(*kind));
        if payload.is_some() {
            self.clear_active_reference_drag_payloads();
        }
        payload
    }

    fn take_active_reference_drag_payload_kind(
        &mut self,
        kind: UiDragPayloadKind,
    ) -> Option<UiDragPayload> {
        match kind {
            UiDragPayloadKind::Asset => self.active_asset_drag_payload.take(),
            UiDragPayloadKind::SceneInstance => self.active_scene_drag_payload.take(),
            UiDragPayloadKind::Object => self.active_object_drag_payload.take(),
        }
    }

    fn clear_active_reference_drag_payloads(&mut self) {
        self.active_asset_drag_payload = None;
        self.active_scene_drag_payload = None;
        self.active_object_drag_payload = None;
    }
}
