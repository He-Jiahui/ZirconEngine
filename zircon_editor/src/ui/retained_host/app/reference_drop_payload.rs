use super::*;

pub(super) struct StaleReferenceDragSource;

impl RetainedEditorHost {
    pub(super) fn take_active_reference_drag_payload_for_drop(
        &mut self,
        action_id: &str,
    ) -> Result<Option<UiDragPayload>, StaleReferenceDragSource> {
        let Some(preferred_kinds) = preferred_reference_drop_kinds(action_id) else {
            return Ok(None);
        };

        for kind in preferred_kinds {
            if let Some(payload) = self.take_active_reference_drag_payload_kind(*kind)? {
                self.clear_active_reference_drag_payloads();
                return Ok(Some(payload));
            }
        }
        Ok(None)
    }

    fn take_active_reference_drag_payload_kind(
        &mut self,
        kind: UiDragPayloadKind,
    ) -> Result<Option<UiDragPayload>, StaleReferenceDragSource> {
        match kind {
            UiDragPayloadKind::Asset => Ok(self.active_asset_drag_payload.take()),
            UiDragPayloadKind::SceneInstance => {
                if self.active_scene_drag_payload.is_none() {
                    self.retire_hierarchy_drag();
                    return Ok(None);
                }
                if self.active_hierarchy_drag_identity.is_some()
                    && !self.hierarchy_drag_identity_is_current()
                {
                    self.retire_hierarchy_drag();
                    return Err(StaleReferenceDragSource);
                }
                self.active_hierarchy_drag_identity = None;
                self.active_hierarchy_drag_node_ids.clear();
                self.hierarchy_pointer_bridge.cancel_reparent_drag();
                Ok(self.active_scene_drag_payload.take())
            }
            UiDragPayloadKind::Object => Ok(self.active_object_drag_payload.take()),
        }
    }

    fn clear_active_reference_drag_payloads(&mut self) {
        self.active_asset_drag_payload = None;
        self.retire_hierarchy_drag();
        self.active_object_drag_payload = None;
    }
}

fn preferred_reference_drop_kinds(action_id: &str) -> Option<&'static [UiDragPayloadKind]> {
    let mut asset = false;
    let mut instance = false;
    let mut object = false;
    for (suffix_start, _) in action_id.match_indices("FieldDropped") {
        let prefix = &action_id[..suffix_start];
        asset |= prefix.ends_with("Asset");
        instance |= prefix.ends_with("Instance");
        object |= prefix.ends_with("Object");
    }

    if asset {
        Some(&[
            UiDragPayloadKind::Asset,
            UiDragPayloadKind::SceneInstance,
            UiDragPayloadKind::Object,
        ])
    } else if instance {
        Some(&[
            UiDragPayloadKind::SceneInstance,
            UiDragPayloadKind::Asset,
            UiDragPayloadKind::Object,
        ])
    } else if object {
        Some(&[
            UiDragPayloadKind::Object,
            UiDragPayloadKind::SceneInstance,
            UiDragPayloadKind::Asset,
        ])
    } else {
        None
    }
}

#[cfg(test)]
#[path = "tests/reference_drop_payload_optimization_tests.rs"]
mod optimization_tests;
