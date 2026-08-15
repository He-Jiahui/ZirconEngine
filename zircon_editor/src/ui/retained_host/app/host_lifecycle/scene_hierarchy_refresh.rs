use super::super::*;
use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{EditorViewInvalidationMask, SceneInspectionMessage};
use crate::ui::retained_host::ui::{
    patch_host_contract_workbench_window_nodes_at_mount_and_scale,
    to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale,
};
use crate::ui::workbench::snapshot::{SceneEntries, SceneInspectionHierarchyFragment};
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration};

const HIERARCHY_VIEW_INSTANCE_ID: &str = "scene.hierarchy";

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn ensure_hierarchy_world_watch(
        &mut self,
    ) -> Result<(), String> {
        let gateway_generation = self.runtime.edit_world_gateway_generation();
        if self
            .hierarchy_world_watch
            .is_some_and(|watch| watch.belongs_to_gateway_generation(gateway_generation))
        {
            return Ok(());
        }
        self.hierarchy_world_watch = None;
        let (token, issued_generation) = self
            .runtime
            .watch_edit_world_for_view(
                WatchRegistration::new(WatchKey::WorldStructure),
                ViewInstanceId::new(HIERARCHY_VIEW_INSTANCE_ID),
                EditorViewInvalidationMask::TREE_STRUCTURE,
            )
            .map_err(|error| format!("Hierarchy world watch registration failed: {error}"))?;
        self.hierarchy_world_watch = Some(HierarchyWorldWatch::new(token, issued_generation));
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn pump_edit_world_invalidations(&mut self) {
        match self.runtime.pump_edit_world_invalidations() {
            Ok(_) => {
                self.runtime.drain_pending_view_refreshes();
            }
            Err(error) => {
                self.set_status_line(format!("Hierarchy world invalidation pump failed: {error}"))
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn consume_scene_hierarchy_fragment(&mut self) {
        let Some(message) = self.runtime.take_retained_scene_inspection_message() else {
            return;
        };
        let Some(fragment) = self
            .runtime
            .scene_inspection_hierarchy_fragment(message.clone())
        else {
            self.resync_scene_hierarchy_from_message(message);
            return;
        };
        self.apply_scene_hierarchy_fragment(fragment);
    }

    fn apply_scene_hierarchy_fragment(&mut self, fragment: SceneInspectionHierarchyFragment) {
        let message = fragment.message().clone();
        if !self.hierarchy_filter_query().trim().is_empty() {
            self.resync_scene_hierarchy_from_message(message);
            return;
        }
        let reflow_entries = fragment.reflow_entries().cloned();
        let apply = match self
            .workbench_window_bridge
            .apply_scene_hierarchy_fragment(&fragment)
        {
            Ok(apply) => apply,
            Err(error) => {
                self.set_status_line(format!("Hierarchy fragment apply failed: {error}"));
                self.resync_scene_hierarchy_from_message(message);
                return;
            }
        };
        if apply.applied() {
            if !self.publish_sparse_hierarchy_host_nodes(apply.changed_control_ids()) {
                self.resync_scene_hierarchy_from_message(message);
            }
            return;
        }
        if apply.selection_resync_required() {
            let (selection_revision, selected_entities) =
                self.runtime.scene_inspection_selection_snapshot();
            match self
                .workbench_window_bridge
                .resync_scene_hierarchy_selection(selection_revision, &selected_entities)
            {
                Ok(selection_apply) if selection_apply.applied() => {
                    // A newer authoritative snapshot has no compatible base for this delta.
                    if selection_revision != message.selection().revision() {
                        self.resync_scene_hierarchy_from_message(message);
                        return;
                    }
                    match self
                        .workbench_window_bridge
                        .apply_scene_hierarchy_fragment(&fragment)
                    {
                        Ok(fragment_apply) if fragment_apply.applied() => {
                            let mut changed_control_ids =
                                selection_apply.changed_control_ids().to_vec();
                            changed_control_ids
                                .extend(fragment_apply.changed_control_ids().iter().cloned());
                            changed_control_ids.sort_unstable();
                            changed_control_ids.dedup();
                            if !self.publish_sparse_hierarchy_host_nodes(&changed_control_ids) {
                                self.resync_scene_hierarchy_from_message(message);
                            }
                        }
                        Ok(_) => self.resync_scene_hierarchy_from_message(message),
                        Err(error) => {
                            self.set_status_line(format!(
                                "Hierarchy fragment retry after selection resync failed: {error}"
                            ));
                            self.resync_scene_hierarchy_from_message(message);
                        }
                    }
                }
                Ok(_) => self.resync_scene_hierarchy_from_message(message),
                Err(error) => {
                    self.set_status_line(format!("Hierarchy selection resync failed: {error}"));
                    self.resync_scene_hierarchy_from_message(message);
                }
            }
            return;
        }
        if let Some(entries) = reflow_entries {
            self.commit_scene_hierarchy_reflow(&message, &entries);
        } else {
            self.resync_scene_hierarchy_from_message(message);
        }
    }

    fn resync_scene_hierarchy_from_message(&mut self, message: SceneInspectionMessage) {
        let Some(fragment) = self.runtime.scene_inspection_hierarchy_reflow(message) else {
            self.set_status_line("Hierarchy authoritative reflow is unavailable".to_string());
            return;
        };
        let message = fragment.message().clone();
        let Some(entries) = fragment.reflow_entries() else {
            self.set_status_line(
                "Hierarchy authoritative reflow returned a sparse patch".to_string(),
            );
            return;
        };
        self.commit_scene_hierarchy_reflow(&message, entries);
    }

    fn commit_scene_hierarchy_reflow(
        &mut self,
        message: &SceneInspectionMessage,
        entries: &SceneEntries,
    ) {
        let filtered_entries = self.filtered_hierarchy_entries(entries);
        let entries = filtered_entries.as_ref().unwrap_or(entries);
        if let Err(error) = self
            .workbench_window_bridge
            .resync_scene_hierarchy_at_selection(entries, message.selection().revision())
        {
            self.set_status_line(format!("Hierarchy authoritative reflow failed: {error}"));
            self.mark_layout_dirty();
            return;
        }

        let mut presentation = self.ui.get_host_presentation();
        presentation.workbench_window_nodes =
            to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale(
                Some(self.workbench_window_bridge.host_projection()),
                Some(&presentation.workbench_window_nodes),
                self.workbench_window_bridge.layout_frames().mount_frame,
                self.workbench_window_bridge.presentation_scale_factor(),
            );
        self.ui.set_host_presentation(presentation);
        self.sync_hierarchy_pointer_layout(entries);
        self.invalidate_host(HostInvalidationMask::RENDER);
    }

    fn publish_sparse_hierarchy_host_nodes(&mut self, control_ids: &[String]) -> bool {
        let Some(projection_nodes) = self
            .workbench_window_bridge
            .host_projection_nodes_for_controls(control_ids)
        else {
            return false;
        };
        let mut presentation = self.ui.get_host_presentation();
        let Some(nodes) = patch_host_contract_workbench_window_nodes_at_mount_and_scale(
            &self.workbench_window_bridge.host_projection().document_id,
            &projection_nodes,
            &presentation.workbench_window_nodes,
            self.workbench_window_bridge.layout_frames().mount_frame,
            self.workbench_window_bridge.presentation_scale_factor(),
        ) else {
            return false;
        };
        presentation.workbench_window_nodes = nodes;
        self.ui.set_host_presentation(presentation);
        self.invalidate_host(HostInvalidationMask::RENDER);
        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn fragment_failures_request_an_authoritative_reflow() {
        let source = include_str!("scene_hierarchy_refresh.rs");
        for failure in [
            "Hierarchy fragment apply failed: {error}",
            "Hierarchy selection resync failed: {error}",
        ] {
            let failure = source
                .find(failure)
                .unwrap_or_else(|| panic!("missing hierarchy failure branch: {failure}"));
            let reflow = source[failure..]
                .find("self.resync_scene_hierarchy_from_message(message);")
                .unwrap_or_else(|| panic!("{failure} must request an authoritative reflow"));
            assert!(
                reflow < 256,
                "failure recovery must stay in the same match arm"
            );
        }
    }

    #[test]
    fn selection_gap_retries_the_same_sparse_fragment_before_publishing() {
        let production = include_str!("scene_hierarchy_refresh.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("hierarchy refresh production section");
        let selection_recovery = production
            .split("if apply.selection_resync_required()")
            .nth(1)
            .expect("selection-gap recovery branch");
        let recovery_end = selection_recovery
            .find("if let Some(entries) = reflow_entries")
            .expect("selection-gap recovery must finish before ordinary reflow handling");
        let selection_recovery = &selection_recovery[..recovery_end];
        let selection_resync = selection_recovery
            .find(".resync_scene_hierarchy_selection(")
            .expect("selection-gap recovery must repair the selection overlay");
        let fragment_retry = selection_recovery[selection_resync..]
            .find(".apply_scene_hierarchy_fragment(&fragment)")
            .expect("selection-gap recovery must retry the interrupted sparse fragment");
        let sparse_publish = selection_recovery[selection_resync..]
            .find("self.publish_sparse_hierarchy_host_nodes")
            .expect("selection-gap recovery must publish the repaired sparse projection");
        let revision_mismatch = selection_recovery[selection_resync..]
            .find("selection_revision != message.selection().revision()")
            .expect("selection-gap recovery must reject an advanced snapshot");
        let mismatch_recovery = &selection_recovery[selection_resync + revision_mismatch..];
        let mismatch_reflow = mismatch_recovery
            .find("self.resync_scene_hierarchy_from_message(message);")
            .expect("advanced snapshot must use authoritative reflow");
        let mismatch_return = mismatch_recovery[mismatch_reflow..]
            .find("return;")
            .expect("advanced snapshot reflow must terminate the branch");
        let merged_controls = selection_recovery[selection_resync..]
            .find(".extend(fragment_apply.changed_control_ids().iter().cloned())")
            .expect("selection and fragment control ids must merge before publication");

        assert!(revision_mismatch < fragment_retry);
        assert!(mismatch_reflow < mismatch_return);
        assert!(merged_controls < sparse_publish);
        assert_eq!(
            selection_recovery
                .matches("self.publish_sparse_hierarchy_host_nodes")
                .count(),
            1,
            "selection-gap recovery must publish only the merged sparse patch"
        );
        assert!(
            fragment_retry < sparse_publish,
            "the row patch must commit before its sparse host publication"
        );
    }

    #[test]
    fn authoritative_reflow_commits_host_and_pointer_state_after_bridge_success() {
        let source = include_str!("scene_hierarchy_refresh.rs");
        let reflow = source
            .split("fn commit_scene_hierarchy_reflow")
            .nth(1)
            .expect("authoritative reflow function");
        let bridge = reflow
            .find(".resync_scene_hierarchy_at_selection(")
            .expect("bridge resync must be attempted first");
        let presentation = reflow
            .find("self.ui.set_host_presentation(presentation);")
            .expect("host presentation must be committed");
        let pointer = reflow
            .find("self.sync_hierarchy_pointer_layout(entries);")
            .expect("pointer routing must be committed");
        assert!(bridge < presentation);
        assert!(presentation < pointer);
    }

    #[test]
    fn failed_authoritative_reflow_marks_the_layout_for_the_next_recovery_attempt() {
        let source = include_str!("scene_hierarchy_refresh.rs");
        let reflow = source
            .split("fn commit_scene_hierarchy_reflow")
            .nth(1)
            .expect("authoritative reflow function");
        let failure = reflow
            .find("Hierarchy authoritative reflow failed: {error}")
            .expect("authoritative reflow failure branch");
        let mark_dirty = reflow[failure..]
            .find("self.mark_layout_dirty();")
            .expect("failed authoritative reflow must schedule recovery");
        let return_statement = reflow[failure..]
            .find("return;")
            .expect("authoritative reflow failure return");
        assert!(mark_dirty < return_statement);
    }

    #[test]
    fn active_hierarchy_filters_force_one_filtered_authoritative_projection() {
        let source = include_str!("scene_hierarchy_refresh.rs");
        let apply = source
            .split("fn apply_scene_hierarchy_fragment")
            .nth(1)
            .expect("fragment apply function");
        let filter_gate = apply
            .find("self.hierarchy_filter_query().trim().is_empty()")
            .expect("active hierarchy filters must gate sparse patches");
        let sparse_publish = apply
            .find("self.publish_sparse_hierarchy_host_nodes")
            .expect("sparse publication path");
        assert!(filter_gate < sparse_publish);

        let reflow = source
            .split("fn commit_scene_hierarchy_reflow")
            .nth(1)
            .expect("authoritative reflow function");
        let filtered_entries = reflow
            .find("self.filtered_hierarchy_entries(entries)")
            .expect("authoritative projection must apply the active filter once");
        let bridge = reflow
            .find(".resync_scene_hierarchy_at_selection(")
            .expect("filtered bridge resync");
        let pointer = reflow
            .find("self.sync_hierarchy_pointer_layout(entries);")
            .expect("filtered pointer commit");
        assert!(filtered_entries < bridge);
        assert!(bridge < pointer);
    }
}
