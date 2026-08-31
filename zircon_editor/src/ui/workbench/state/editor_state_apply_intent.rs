use std::collections::BTreeSet;
use std::sync::Arc;

use zircon_runtime::scene::{NodeId, Scene};
use zircon_runtime_interface::math::Transform;

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{EditCommandError, MergeMode, SelectionSnapshot};
use crate::core::editing::intent::EditorIntent;
use crate::core::editing::selection::SceneSelection;
use crate::core::play::WorldDomain;

use super::no_project_open::no_project_open;
use super::{
    editor_state::EditorState, EditorStateOperationError, GizmoTransactionError,
    GizmoTransactionPhase,
};

impl EditorState {
    pub fn apply_intent(
        &mut self,
        intent: EditorIntent,
    ) -> Result<bool, EditorStateOperationError> {
        let mutates_edit_world = matches!(
            &intent,
            EditorIntent::CreateNode(_)
                | EditorIntent::DeleteNode(_)
                | EditorIntent::DeleteNodes(_)
                | EditorIntent::RenameNode(_, _)
                | EditorIntent::SetParent(_, _)
                | EditorIntent::SetParents(_, _)
                | EditorIntent::SetTransform(_, _)
                | EditorIntent::ApplyInspectorChanges
        );
        if self.is_playing() && mutates_edit_world {
            return Err(EditorStateOperationError::SceneEditingDisabledDuringPlay);
        }
        self.prepare_non_gizmo_scene_action()?;
        match intent {
            EditorIntent::CreateNode(kind) => {
                self.execute_scene_command("Create scene node", EditorCommand::create_node(kind))?;
                let id = self
                    .viewport_controller
                    .selection()
                    .active_primary()
                    .ok_or(EditorStateOperationError::CreatedNodeNotSelected)?;
                self.set_status_line(format!("Created node {id}"));
                Ok(true)
            }
            EditorIntent::DeleteNode(id) => {
                let command =
                    self.capture_scene_command(|scene| EditorCommand::delete_node(scene, id))?;
                self.execute_scene_command("Delete scene node", command)?;
                self.set_status_line(format!("Deleted node {id}"));
                Ok(true)
            }
            EditorIntent::DeleteNodes(node_ids) => {
                let mut seen = BTreeSet::new();
                let node_ids = node_ids
                    .into_iter()
                    .filter(|node_id| seen.insert(*node_id))
                    .collect::<Vec<_>>();
                if node_ids.is_empty() {
                    return Ok(false);
                }
                let commands = self.capture_scene_command(|scene| {
                    let root_ids = top_level_node_ids(scene, &node_ids);
                    let removed_camera_count = root_ids
                        .iter()
                        .flat_map(|node_id| scene.subtree_records(*node_id))
                        .filter(|record| record.camera.is_some())
                        .count();
                    if removed_camera_count >= scene.camera_count() {
                        return Err(
                            crate::core::editing::engine::EditCommandError::InvariantViolation {
                                invariant: "cannot delete the last remaining camera",
                            },
                        );
                    }
                    root_ids
                        .into_iter()
                        .map(|node_id| EditorCommand::delete_node(scene, node_id))
                        .collect::<Result<Vec<_>, _>>()
                })?;
                let deleted_count = commands.len();
                self.execute_scene_commands("Delete scene nodes", commands, MergeMode::Disable)?;
                self.set_status_line(format!("Deleted {deleted_count} scene node(s)"));
                Ok(true)
            }
            EditorIntent::SelectNode(id) => self.select_node_in_world(WorldDomain::Edit, id),
            EditorIntent::RenameNode(id, name) => {
                let command = self
                    .capture_scene_command(|scene| EditorCommand::rename_node(scene, id, name))?;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.execute_scene_command("Rename scene node", command)?;
                self.set_status_line(format!("Renamed node {id}"));
                Ok(true)
            }
            EditorIntent::SetParent(id, parent) => {
                let command = self
                    .capture_scene_command(|scene| EditorCommand::set_parent(scene, id, parent))?;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.execute_scene_command("Reparent scene node", command)?;
                let status_line = match parent {
                    Some(parent) => format!("Reparented node {id} under {parent}"),
                    None => format!("Detached node {id} to root"),
                };
                self.set_status_line(status_line);
                Ok(true)
            }
            EditorIntent::SetParents(node_ids, parent) => {
                let mut seen = BTreeSet::new();
                let node_ids = node_ids
                    .into_iter()
                    .filter(|node_id| seen.insert(*node_id))
                    .collect::<Vec<_>>();
                if node_ids.is_empty() {
                    return Ok(false);
                }
                let commands = self.capture_scene_command(|scene| {
                    top_level_node_ids(scene, &node_ids)
                        .iter()
                        .map(|node_id| EditorCommand::set_parent(scene, *node_id, parent))
                        .collect::<Result<Vec<_>, _>>()
                })?;
                let commands = commands.into_iter().flatten().collect::<Vec<_>>();
                if commands.is_empty() {
                    return Ok(false);
                }
                let changed_count = commands.len();
                self.execute_scene_commands("Reparent scene nodes", commands, MergeMode::Disable)?;
                let status_line = match parent {
                    Some(parent) => {
                        format!("Reparented {changed_count} scene node(s) under {parent}")
                    }
                    None => format!("Detached {changed_count} scene node(s) to root"),
                };
                self.set_status_line(status_line);
                Ok(true)
            }
            EditorIntent::SetTransform(id, transform) => {
                let command = self.capture_scene_command(|scene| {
                    EditorCommand::set_transform(scene, id, transform)
                })?;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.execute_scene_command("Transform scene node", command)?;
                self.set_status_line(format!("Updated transform for node {id}"));
                Ok(true)
            }
            EditorIntent::ApplyInspectorChanges => self.apply_inspector_changes(),
            EditorIntent::Undo => {
                let history_context = self.scene_history_context()?;
                self.bind_transaction_context()?;
                let camera_before = self.capture_active_scene_camera_authority()?;
                let changed = self.transactions().undo(history_context)?;
                if changed {
                    self.resync_active_scene_camera_after_mutation(camera_before)?;
                    self.sync_selection_from_transaction_context()?;
                    self.sync_selection_state();
                    self.set_status_line("Undo");
                } else {
                    self.set_status_line("Nothing to undo");
                }
                Ok(changed)
            }
            EditorIntent::Redo => {
                let history_context = self.scene_history_context()?;
                self.bind_transaction_context()?;
                let camera_before = self.capture_active_scene_camera_authority()?;
                let changed = self.transactions().redo(history_context)?;
                if changed {
                    self.resync_active_scene_camera_after_mutation(camera_before)?;
                    self.sync_selection_from_transaction_context()?;
                    self.sync_selection_state();
                    self.set_status_line("Redo");
                } else {
                    self.set_status_line("Nothing to redo");
                }
                Ok(changed)
            }
        }
    }

    pub(crate) fn execute_scene_command(
        &mut self,
        label: &str,
        command: EditorCommand,
    ) -> Result<(), EditorStateOperationError> {
        self.execute_scene_commands(label, [command], MergeMode::Disable)
    }

    pub(crate) fn execute_scene_commands(
        &mut self,
        label: &str,
        commands: impl IntoIterator<Item = EditorCommand>,
        merge_mode: MergeMode,
    ) -> Result<(), EditorStateOperationError> {
        if self.interactive_transform.is_some() || self.viewport_controller.is_handle_drag_active()
        {
            return Err(EditorStateOperationError::SceneActionBlockedByActiveGizmo);
        }
        self.execute_prepared_scene_commands(label, commands, merge_mode)
    }

    pub(crate) fn execute_gizmo_scene_command(
        &mut self,
        label: &str,
        command: EditorCommand,
    ) -> Result<(), GizmoTransactionError> {
        let history_context = self
            .active_scene_history_context()
            .ok_or(GizmoTransactionError::SceneDocumentNotActive)?;
        self.bind_interactive_transform_context()?;
        let editor_context = Arc::clone(&self.context);
        let mut scope = editor_context
            .transactions()
            .begin(label, history_context)
            .map_err(|source| GizmoTransactionError::EditCommand {
                phase: GizmoTransactionPhase::CommandExecution,
                source,
            })?;
        scope.set_merge_mode(MergeMode::Disable);
        scope
            .push(command)
            .map_err(|source| GizmoTransactionError::EditCommand {
                phase: GizmoTransactionPhase::CommandExecution,
                source,
            })?;
        scope
            .commit_after_apply(|selection_after| {
                self.sync_selection_from_transaction_snapshot(selection_after)
            })
            .map_err(|source| GizmoTransactionError::EditCommand {
                phase: GizmoTransactionPhase::CommandExecution,
                source,
            })?;
        self.sync_selection_state();
        Ok(())
    }

    fn execute_prepared_scene_commands(
        &mut self,
        label: &str,
        commands: impl IntoIterator<Item = EditorCommand>,
        merge_mode: MergeMode,
    ) -> Result<(), EditorStateOperationError> {
        if self.is_playing() {
            return Err(EditorStateOperationError::SceneEditingDisabledDuringPlay);
        }
        let history_context = self.scene_history_context()?;
        self.bind_transaction_context()?;
        let camera_before = self.capture_active_scene_camera_authority()?;
        let editor_context = Arc::clone(&self.context);
        let mut scope = editor_context
            .transactions()
            .begin(label, history_context)?;
        scope.set_merge_mode(merge_mode);
        for command in commands {
            scope.push(command)?;
        }
        scope.commit_after_apply(|selection_after| {
            self.sync_selection_from_transaction_snapshot(selection_after)
        })?;
        self.resync_active_scene_camera_after_mutation(camera_before)?;
        self.sync_selection_state();
        Ok(())
    }

    fn capture_active_scene_camera_authority(
        &self,
    ) -> Result<(NodeId, Option<Transform>), EditorStateOperationError> {
        self.world
            .with_world(|scene| {
                let active_camera = scene.active_camera();
                (active_camera, scene.world_transform(active_camera))
            })?
            .ok_or_else(no_project_open)
    }

    fn resync_active_scene_camera_after_mutation(
        &mut self,
        (active_camera_before, active_camera_transform_before): (NodeId, Option<Transform>),
    ) -> Result<(), EditorStateOperationError> {
        let world = &self.world;
        let viewport = &mut self.viewport_controller;
        world
            .with_world(|scene| {
                viewport.resync_active_camera_after_scene_mutation(
                    scene,
                    active_camera_before,
                    active_camera_transform_before,
                );
            })?
            .ok_or_else(no_project_open)
    }

    pub(crate) fn capture_scene_command<R>(
        &self,
        capture: impl FnOnce(
            &zircon_runtime::scene::Scene,
        ) -> Result<R, crate::core::editing::engine::EditCommandError>,
    ) -> Result<R, EditorStateOperationError> {
        let command = self
            .world
            .with_world(capture)?
            .ok_or_else(no_project_open)??;
        Ok(command)
    }

    pub(crate) fn bind_transaction_context(&self) -> Result<(), EditorStateOperationError> {
        let selection = self.viewport_controller.selection();
        let world_domain = selection.active_domain();
        self.bind_transaction_context_for(world_domain)
    }

    pub(crate) fn bind_transaction_context_for(
        &self,
        world_domain: crate::core::play::WorldDomain,
    ) -> Result<(), EditorStateOperationError> {
        let selection = self.viewport_controller.selection();
        let selection = SceneSelection::new(
            selection.items(world_domain).iter().copied().collect(),
            selection.primary(world_domain),
        );
        self.transactions()
            .with_context_mut::<CoreEditContext, _>(|context| {
                context.bind_selection(world_domain, selection)
            })?
            .ok_or(EditorStateOperationError::TransactionContextMissing)??;
        Ok(())
    }

    fn bind_interactive_transform_context(&self) -> Result<(), GizmoTransactionError> {
        let selection = self.viewport_controller.selection();
        let world_domain = selection.active_domain();
        let selection = SceneSelection::new(
            selection.active_items().iter().copied().collect(),
            selection.active_primary(),
        );
        self.transactions()
            .with_context_mut::<CoreEditContext, _>(|context| {
                context.bind_selection(world_domain, selection)
            })
            .map_err(|source| GizmoTransactionError::EditCommand {
                phase: GizmoTransactionPhase::ContextBinding,
                source,
            })?
            .ok_or(GizmoTransactionError::TransactionContextMissing)?
            .map_err(|source| GizmoTransactionError::EditCommand {
                phase: GizmoTransactionPhase::ContextBinding,
                source,
            })
    }

    pub(crate) fn ensure_transaction_context_selection_is_current(
        &self,
    ) -> Result<(), EditorStateOperationError> {
        let selection = self.viewport_controller.selection();
        let selection = SceneSelection::new(
            selection.active_items().iter().copied().collect(),
            selection.active_primary(),
        );
        let current = self
            .transactions()
            .with_context::<CoreEditContext, _>(CoreEditContext::scene_selection)?
            .ok_or(EditorStateOperationError::TransactionContextMissing)?;
        if matches!(current, Ok(current) if current == selection) {
            return Ok(());
        }
        self.bind_transaction_context()
    }

    fn sync_selection_from_transaction_context(&mut self) -> Result<(), EditorStateOperationError> {
        let selection = self
            .transactions()
            .with_context::<CoreEditContext, _>(CoreEditContext::selection_snapshot)?
            .ok_or(EditorStateOperationError::TransactionContextMissing)?;
        self.sync_selection_from_transaction_snapshot(&selection)?;
        Ok(())
    }

    pub(in crate::ui::workbench) fn sync_selection_from_transaction_snapshot(
        &mut self,
        selection: &SelectionSnapshot,
    ) -> Result<(), EditCommandError> {
        #[cfg(test)]
        if std::mem::take(&mut self.fail_next_transaction_selection_sync) {
            return Err(EditCommandError::InvariantViolation {
                invariant: "forced transaction selection synchronization failure",
            });
        }
        let selection = selection.scene_selection()?;
        self.viewport_controller
            .selection_mut()
            .replace_active(selection.items().to_vec(), selection.primary());
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn fail_next_transaction_selection_sync_for_test(&mut self) {
        self.fail_next_transaction_selection_sync = true;
    }
}

fn top_level_node_ids(scene: &Scene, node_ids: &[NodeId]) -> Vec<NodeId> {
    let selected = node_ids.iter().copied().collect::<BTreeSet<_>>();
    node_ids
        .iter()
        .filter(|node_id| {
            let mut parent = scene.parent_of(**node_id);
            while let Some(parent_id) = parent {
                if selected.contains(&parent_id) {
                    return false;
                }
                parent = scene.parent_of(parent_id);
            }
            true
        })
        .copied()
        .collect()
}
