mod hierarchy_row_patch;

use std::collections::BTreeMap;

use super::super::*;
use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::{EditorViewInvalidationMask, SceneInspectionMessage};
use crate::core::play::WorldDomain;
use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::ui::{
    build_host_contract_workbench_window_node_patch_at_mount_and_scale,
    to_host_contract_workbench_window_nodes_with_previous_at_mount_and_scale,
};
use crate::ui::retained_host::{HostPresentationPatch, SceneNodeData};
use crate::ui::workbench::snapshot::{SceneEntries, SceneInspectionHierarchyFragment};
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration};

use hierarchy_row_patch::{
    build_presented_hierarchy_pane_patch, replace_presented_hierarchy_rows,
    PresentedHierarchyRowPatch,
};

const HIERARCHY_VIEW_INSTANCE_ID: &str = "scene.hierarchy";

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_active_play_inspector(&mut self) {
        match self.runtime.active_hierarchy_world_domain() {
            WorldDomain::Edit => {
                if self.runtime.clear_play_inspector() {
                    self.mark_layout_dirty();
                }
            }
            WorldDomain::Play(instance) => match self
                .runtime
                .refresh_play_inspector_if_due(instance, std::time::Instant::now())
            {
                Ok(true) => self.mark_layout_dirty(),
                Ok(false) => {}
                Err(error) => self.set_status_line(error),
            },
        }
    }

    pub(in crate::ui::retained_host::app) fn ensure_hierarchy_world_watch(
        &mut self,
        domain: WorldDomain,
    ) -> Result<bool, String> {
        let gateway_identity = self.runtime.world_gateway_identity(domain).ok_or_else(|| {
            format!("Hierarchy world gateway is unavailable for domain {domain:?}")
        })?;
        if self
            .hierarchy_world_watch
            .as_ref()
            .is_some_and(|watch| watch.belongs_to(domain, &gateway_identity))
        {
            return Ok(false);
        }
        if let Some(previous) = self.hierarchy_world_watch.take() {
            if self
                .runtime
                .world_gateway_identity(previous.domain())
                .is_some()
            {
                self.runtime
                    .unwatch_world_for_view(previous.domain(), previous.token())
                    .map_err(|error| {
                        format!("Hierarchy previous world watch retirement failed: {error}")
                    })?;
            }
        }
        let token = self
            .runtime
            .watch_world_for_view(
                domain,
                WatchRegistration::new(WatchKey::WorldStructure),
                ViewInstanceId::new(HIERARCHY_VIEW_INSTANCE_ID),
                EditorViewInvalidationMask::TREE_STRUCTURE,
            )
            .map_err(|error| format!("Hierarchy world watch registration failed: {error}"))?;
        self.hierarchy_world_watch = Some(HierarchyWorldWatch::new(domain, token));
        Ok(true)
    }

    pub(in crate::ui::retained_host::app) fn sync_active_hierarchy_world(&mut self) {
        self.retire_hierarchy_drag_if_stale();
        let domain = self.runtime.active_hierarchy_world_domain();
        let domain_changed = match self.ensure_hierarchy_world_watch(domain) {
            Ok(changed) => changed,
            Err(error) => {
                self.set_status_line(error);
                return;
            }
        };
        let report = match self.runtime.pump_world_invalidations(domain) {
            Ok(report) => report,
            Err(error) => {
                self.set_status_line(format!("Hierarchy world invalidation pump failed: {error}"));
                return;
            }
        };
        if report.advanced_world_replacement_epoch().is_some() {
            self.retire_hierarchy_drag();
        }
        let mut force_reflow = domain_changed;
        if let (WorldDomain::Play(instance), Some(replacement_epoch)) =
            (domain, report.advanced_world_replacement_epoch())
        {
            let Some(identity) = report.drain_identity().cloned() else {
                self.set_status_line(
                    "Play world replacement report is missing its gateway identity".to_string(),
                );
                return;
            };
            if let Err(error) = self.play_viewport_pick.cancel() {
                self.set_status_line(format!(
                    "Play viewport pick retirement during world replacement failed: {error}"
                ));
            }
            let retirement = match self.runtime.retire_replaced_play_world(
                instance,
                &identity,
                replacement_epoch,
            ) {
                Ok(retirement) => retirement,
                Err(error) => {
                    self.set_status_line(format!(
                        "Play world replacement retirement failed: {error}"
                    ));
                    return;
                }
            };
            if !self
                .runtime
                .acknowledge_world_replacement(domain, replacement_epoch)
            {
                self.set_status_line(
                    "Play world replacement retirement acknowledgement was stale".to_string(),
                );
                return;
            }
            force_reflow = true;
            if retirement.history_discarded() {
                zircon_runtime::profile_counter!(
                    "editor",
                    "play.world_replacement.history_discard_count",
                    1
                );
            }
            if retirement.selection_cleared()
                || retirement.hierarchy_cleared()
                || retirement.inspector_cleared()
            {
                self.mark_layout_dirty();
            }
        }
        self.runtime.drain_pending_view_refreshes();
        let selection_revision = self.runtime.scene_inspection_selection_snapshot().0;
        let watch = self
            .hierarchy_world_watch
            .as_mut()
            .expect("hierarchy world watch was ensured before pumping");
        if report.dirty_views() > 0 || watch.selection_revision_changed(selection_revision) {
            watch.mark_projection_pending();
        }

        match domain {
            WorldDomain::Edit => {
                if domain_changed || report.advanced_world_replacement_epoch().is_some() {
                    self.runtime.publish_scene_inspection_resync();
                }
                self.consume_scene_hierarchy_fragment();
                if let Some(watch) = self.hierarchy_world_watch.as_mut() {
                    watch.complete_projection(selection_revision);
                }
                if let Some(replacement_epoch) = report.advanced_world_replacement_epoch() {
                    if !self
                        .runtime
                        .acknowledge_world_replacement(domain, replacement_epoch)
                    {
                        self.set_status_line(
                            "Edit world replacement acknowledgement was stale".to_string(),
                        );
                    }
                }
            }
            WorldDomain::Play(instance) => {
                // Authoring publications stay drainable while Play is active, but never become
                // runtime hierarchy data.
                let _ = self.runtime.take_retained_scene_inspection_message();
                if !self
                    .hierarchy_world_watch
                    .as_ref()
                    .is_some_and(HierarchyWorldWatch::projection_pending)
                {
                    return;
                }
                match self
                    .runtime
                    .query_play_hierarchy_fragment(instance, force_reflow)
                {
                    Ok(fragment) => {
                        if let Some(watch) = self.hierarchy_world_watch.as_mut() {
                            watch.complete_projection(selection_revision);
                        }
                        if let Some(fragment) = fragment {
                            self.apply_scene_hierarchy_fragment(fragment);
                        }
                    }
                    Err(error) => {
                        self.set_status_line(format!("Play hierarchy projection failed: {error}"))
                    }
                }
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
            let row_patches = apply
                .logical_row_patches()
                .iter()
                .map(|patch| {
                    (
                        patch.row_index(),
                        PresentedHierarchyRowPatch::new(
                            patch.replacement().map(|replacement| SceneNodeData {
                                id: replacement.entity().to_string().into(),
                                name: replacement.display_name().into(),
                                depth: i32::try_from(replacement.depth()).unwrap_or(i32::MAX),
                                selected: patch.selected(),
                            }),
                            patch.selected(),
                        ),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            if !self.publish_sparse_hierarchy_host_nodes(apply.changed_control_ids(), &row_patches)
            {
                self.workbench_window_bridge
                    .invalidate_scene_hierarchy_projection();
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
                            let row_patches = selection_apply
                                .logical_row_patches()
                                .iter()
                                .chain(fragment_apply.logical_row_patches())
                                .map(|patch| {
                                    (
                                        patch.row_index(),
                                        PresentedHierarchyRowPatch::new(
                                            patch.replacement().map(|replacement| SceneNodeData {
                                                id: replacement.entity().to_string().into(),
                                                name: replacement.display_name().into(),
                                                depth: i32::try_from(replacement.depth())
                                                    .unwrap_or(i32::MAX),
                                                selected: patch.selected(),
                                            }),
                                            patch.selected(),
                                        ),
                                    )
                                })
                                .collect::<BTreeMap<_, _>>();
                            if !self.publish_sparse_hierarchy_host_nodes(
                                &changed_control_ids,
                                &row_patches,
                            ) {
                                self.workbench_window_bridge
                                    .invalidate_scene_hierarchy_projection();
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
            self.mark_layout_dirty();
            return;
        };
        let message = fragment.message().clone();
        let Some(entries) = fragment.reflow_entries() else {
            self.set_status_line(
                "Hierarchy authoritative reflow returned a sparse patch".to_string(),
            );
            self.mark_layout_dirty();
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
        let hierarchy_rows = ModelRc::with_metadata(
            entries
                .iter()
                .map(|row| SceneNodeData {
                    id: row.entity.to_string().into(),
                    name: row.display_name.as_str().into(),
                    depth: i32::try_from(row.depth).unwrap_or(i32::MAX),
                    selected: entries.is_selected(row.entity),
                })
                .collect(),
            entries.inspection_generation(),
        );
        replace_presented_hierarchy_rows(&mut presentation, &hierarchy_rows);
        self.ui.set_host_presentation(presentation);
        self.sync_hierarchy_pointer_layout(entries.hierarchy_rows_arc());
        self.invalidate_host(HostInvalidationMask::RENDER);
    }

    fn publish_sparse_hierarchy_host_nodes(
        &mut self,
        control_ids: &[String],
        row_patches: &BTreeMap<usize, PresentedHierarchyRowPatch>,
    ) -> bool {
        let workbench_patch_source = if control_ids.is_empty() {
            None
        } else {
            let Some(projection_nodes) = self
                .workbench_window_bridge
                .host_projection_nodes_for_controls(control_ids)
            else {
                return false;
            };
            Some((
                self.workbench_window_bridge
                    .host_projection()
                    .document_id
                    .clone(),
                projection_nodes,
                self.workbench_window_bridge.layout_frames().mount_frame,
                self.workbench_window_bridge.presentation_scale_factor(),
            ))
        };
        let committed = self
            .ui
            .patch_host_presentation(|presentation| {
                let pane_patch = build_presented_hierarchy_pane_patch(presentation, row_patches)?;
                let mut patch = HostPresentationPatch::new(pane_patch);
                if let Some((document_id, projection_nodes, mount_frame, scale_factor)) =
                    workbench_patch_source.as_ref()
                {
                    let workbench_patch =
                        build_host_contract_workbench_window_node_patch_at_mount_and_scale(
                            document_id,
                            projection_nodes,
                            &presentation.workbench_window_nodes,
                            *mount_frame,
                            *scale_factor,
                        )?;
                    patch = patch
                        .with_workbench_nodes(workbench_patch.nodes, workbench_patch.changed_rows);
                }
                Some((patch, ()))
            })
            .is_some();
        if !committed {
            return false;
        }
        self.invalidate_host(HostInvalidationMask::RENDER);
        true
    }
}

#[cfg(test)]
#[path = "tests/scene_hierarchy_refresh.rs"]
mod tests;
