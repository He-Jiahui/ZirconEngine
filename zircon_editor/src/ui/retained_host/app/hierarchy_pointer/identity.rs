use crate::core::editing::engine::HistoryContextId;
use crate::core::editor_message::DocumentId;
use crate::core::play::WorldDomain;
use crate::ui::workbench::layout::MainPageId;
use zircon_runtime_interface::runtime_api::GatewaySessionIdentity;

use super::super::RetainedEditorHost;
use super::HierarchyTerminalReason;

/// Press-time authority for a hierarchy node reference and its reparent gesture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::app) struct HierarchyDragIdentity {
    domain: WorldDomain,
    gateway: GatewaySessionIdentity,
    history_context: Option<HistoryContextId>,
    scene_document: Option<(DocumentId, u64)>,
    source_window: Option<MainPageId>,
}

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn capture_hierarchy_drag_identity(
        &self,
    ) -> Option<HierarchyDragIdentity> {
        let domain = self.runtime.active_hierarchy_world_domain();
        Some(HierarchyDragIdentity {
            domain,
            gateway: self.runtime.world_gateway_identity(domain)?,
            history_context: self.runtime.active_scene_history_context(),
            scene_document: self.editor_manager.active_scene_revision_for_session(),
            source_window: self.callback_source_window.clone(),
        })
    }

    pub(in crate::ui::retained_host::app) fn hierarchy_drag_identity_is_current(&self) -> bool {
        self.active_hierarchy_drag_identity
            .as_ref()
            .is_some_and(|pressed| self.capture_hierarchy_drag_identity().as_ref() == Some(pressed))
    }

    fn hierarchy_drag_authority_is_current(&self) -> bool {
        let (Some(pressed), Some(current)) = (
            self.active_hierarchy_drag_identity.as_ref(),
            self.capture_hierarchy_drag_identity(),
        ) else {
            return false;
        };
        pressed.domain == current.domain
            && pressed.gateway == current.gateway
            && pressed.history_context == current.history_context
            && pressed.scene_document == current.scene_document
    }

    pub(in crate::ui::retained_host::app) fn retire_hierarchy_drag_if_stale(&mut self) {
        if self.active_hierarchy_drag_identity.is_some()
            && !self.hierarchy_drag_authority_is_current()
        {
            self.retire_hierarchy_drag();
        }
    }

    pub(in crate::ui::retained_host::app) fn retire_hierarchy_drag(&mut self) {
        self.retire_hierarchy_drag_with_reason(HierarchyTerminalReason::Superseded);
    }

    pub(in crate::ui::retained_host::app) fn retire_hierarchy_drag_with_reason(
        &mut self,
        reason: HierarchyTerminalReason,
    ) {
        self.hierarchy_input_owner.finish(reason);
        self.hierarchy_pointer_bridge.cancel_reparent_drag();
        self.active_scene_drag_payload = None;
        self.active_hierarchy_drag_node_ids.clear();
        self.active_hierarchy_drag_identity = None;
    }
}
