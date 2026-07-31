use std::collections::BTreeSet;

use zircon_runtime::scene::{NodeId, Scene};

use crate::core::editing::command::EditorCommand;
use crate::core::editing::context::CoreEditContext;
use crate::core::editing::engine::{HistoryContextId, MergeMode};
use crate::core::editing::intent::EditorIntent;
use crate::core::editing::selection::SceneSelection;

use super::editor_state::EditorState;
use super::no_project_open::no_project_open;

impl EditorState {
    pub fn apply_intent(&mut self, intent: EditorIntent) -> Result<bool, String> {
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
                | EditorIntent::Undo
                | EditorIntent::Redo
        );
        if self.is_playing() && mutates_edit_world {
            return Err("scene editing is disabled during play mode".to_string());
        }
        self.prepare_non_gizmo_scene_action()?;
        match intent {
            EditorIntent::CreateNode(kind) => {
                self.execute_scene_command("Create scene node", EditorCommand::create_node(kind))?;
                let id = self
                    .viewport_controller
                    .selection()
                    .active_primary()
                    .ok_or_else(|| "created scene node did not become selected".to_string())?;
                self.status_line = format!("Created node {id}");
                Ok(true)
            }
            EditorIntent::DeleteNode(id) => {
                let command =
                    self.capture_scene_command(|scene| EditorCommand::delete_node(scene, id))?;
                self.execute_scene_command("Delete scene node", command)?;
                self.status_line = format!("Deleted node {id}");
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
                self.status_line = format!("Deleted {deleted_count} scene node(s)");
                Ok(true)
            }
            EditorIntent::SelectNode(id) => {
                if self
                    .world
                    .try_with_world(|scene| scene.find_node(id).is_none())
                    .ok_or_else(no_project_open)?
                {
                    return Err(format!("Cannot select missing node {id}"));
                }
                self.viewport_controller
                    .selection_mut()
                    .select_only_active(id);
                self.sync_selection_state();
                self.status_line = format!("Selected node {id}");
                Ok(true)
            }
            EditorIntent::RenameNode(id, name) => {
                let command = self
                    .capture_scene_command(|scene| EditorCommand::rename_node(scene, id, name))?;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.execute_scene_command("Rename scene node", command)?;
                self.status_line = format!("Renamed node {id}");
                Ok(true)
            }
            EditorIntent::SetParent(id, parent) => {
                let command = self
                    .capture_scene_command(|scene| EditorCommand::set_parent(scene, id, parent))?;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.execute_scene_command("Reparent scene node", command)?;
                self.status_line = match parent {
                    Some(parent) => format!("Reparented node {id} under {parent}"),
                    None => format!("Detached node {id} to root"),
                };
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
                self.status_line = match parent {
                    Some(parent) => {
                        format!("Reparented {changed_count} scene node(s) under {parent}")
                    }
                    None => format!("Detached {changed_count} scene node(s) to root"),
                };
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
                self.status_line = format!("Updated transform for node {id}");
                Ok(true)
            }
            EditorIntent::ApplyInspectorChanges => self.apply_inspector_changes(),
            EditorIntent::Undo => {
                self.bind_transaction_context()?;
                let changed = self
                    .transactions()
                    .undo(HistoryContextId::Global)
                    .map_err(|error| error.to_string())?;
                if changed {
                    self.sync_selection_from_transaction_context()?;
                    self.sync_selection_state();
                    self.status_line = "Undo".to_string();
                } else {
                    self.status_line = "Nothing to undo".to_string();
                }
                Ok(changed)
            }
            EditorIntent::Redo => {
                self.bind_transaction_context()?;
                let changed = self
                    .transactions()
                    .redo(HistoryContextId::Global)
                    .map_err(|error| error.to_string())?;
                if changed {
                    self.sync_selection_from_transaction_context()?;
                    self.sync_selection_state();
                    self.status_line = "Redo".to_string();
                } else {
                    self.status_line = "Nothing to redo".to_string();
                }
                Ok(changed)
            }
        }
    }

    pub(crate) fn execute_scene_command(
        &mut self,
        label: &str,
        command: EditorCommand,
    ) -> Result<(), String> {
        self.execute_scene_commands(label, [command], MergeMode::Disable)
    }

    pub(crate) fn execute_scene_commands(
        &mut self,
        label: &str,
        commands: impl IntoIterator<Item = EditorCommand>,
        merge_mode: MergeMode,
    ) -> Result<(), String> {
        if self.gizmo_transaction.is_some() || self.viewport_controller.is_handle_drag_active() {
            return Err(
                "scene mutation requires the active gizmo preview to be canceled first".to_string(),
            );
        }
        self.execute_prepared_scene_commands(label, commands, merge_mode)
    }

    pub(in crate::ui::workbench) fn execute_gizmo_scene_command(
        &mut self,
        label: &str,
        command: EditorCommand,
    ) -> Result<(), String> {
        self.execute_prepared_scene_commands(label, [command], MergeMode::Disable)
    }

    fn execute_prepared_scene_commands(
        &mut self,
        label: &str,
        commands: impl IntoIterator<Item = EditorCommand>,
        merge_mode: MergeMode,
    ) -> Result<(), String> {
        if self.is_playing() {
            return Err("scene editing is disabled during play mode".to_string());
        }
        self.bind_transaction_context()?;
        let mut scope = self
            .transactions()
            .begin(label, HistoryContextId::Global)
            .map_err(|error| error.to_string())?;
        scope.set_merge_mode(merge_mode);
        for command in commands {
            scope.push(command).map_err(|error| error.to_string())?;
        }
        scope.commit().map_err(|error| error.to_string())?;
        self.sync_selection_from_transaction_context()?;
        self.sync_selection_state();
        Ok(())
    }

    pub(crate) fn capture_scene_command<R>(
        &self,
        capture: impl FnOnce(
            &zircon_runtime::scene::Scene,
        ) -> Result<R, crate::core::editing::engine::EditCommandError>,
    ) -> Result<R, String> {
        self.world
            .try_with_world(capture)
            .ok_or_else(no_project_open)
            .and_then(|result| result.map_err(|error| error.to_string()))
    }

    pub(crate) fn bind_transaction_context(&self) -> Result<(), String> {
        let selection = self.viewport_controller.selection();
        let selection = SceneSelection::new(
            selection.active_items().iter().copied().collect(),
            selection.active_primary(),
        );
        self.transactions()
            .with_context_mut::<CoreEditContext, _>(|context| {
                context.bind_authoring_selection(selection)
            })
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "editor transaction context is not CoreEditContext".to_string())?
            .map_err(|error| error.to_string())
    }

    fn sync_selection_from_transaction_context(&mut self) -> Result<(), String> {
        let selection = self
            .transactions()
            .with_context::<CoreEditContext, _>(CoreEditContext::scene_selection)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "editor transaction context is not CoreEditContext".to_string())?
            .map_err(|error| error.to_string())?;
        self.viewport_controller
            .selection_mut()
            .replace_active(selection.items().to_vec(), selection.primary());
        Ok(())
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
