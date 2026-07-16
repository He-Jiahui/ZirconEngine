use crate::core::editing::command::EditorCommand;
use crate::core::editing::history::HistorySelectionSnapshot;
use crate::core::editing::intent::EditorIntent;

use super::editor_state::EditorState;
use super::no_project_open::no_project_open;

impl EditorState {
    pub fn apply_intent(&mut self, intent: EditorIntent) -> Result<bool, String> {
        match intent {
            EditorIntent::CreateNode(kind) => {
                let selection_before = self.active_history_selection_snapshot();
                let selected = self.viewport_controller.selection().active_primary();
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::create_node(scene, selected, kind))
                    .ok_or_else(no_project_open)??;
                let id = command.target_node();
                self.viewport_controller
                    .selection_mut()
                    .select_only_active(id);
                let selection_after = self.active_history_selection_snapshot();
                self.history
                    .push_with_selection(command, selection_before, selection_after);
                self.sync_selection_state();
                self.status_line = format!("Created node {id}");
                Ok(true)
            }
            EditorIntent::DeleteNode(id) => {
                let selection_before = self
                    .viewport_controller
                    .selection()
                    .active_items()
                    .iter()
                    .copied()
                    .collect::<Vec<_>>();
                let primary_before = self.viewport_controller.selection().active_primary();
                let history_selection_before =
                    HistorySelectionSnapshot::new(selection_before.clone(), primary_before);
                let command = self
                    .world
                    .try_with_world_mut(|scene| {
                        EditorCommand::delete_node(scene, primary_before, id)
                    })
                    .ok_or_else(no_project_open)??;
                self.reconcile_selection_after_delete(
                    selection_before,
                    primary_before,
                    command.selection_after(),
                )?;
                let selection_after = self.active_history_selection_snapshot();
                self.history.push_with_selection(
                    command,
                    history_selection_before,
                    selection_after,
                );
                self.sync_selection_state();
                self.status_line = format!("Deleted node {id}");
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
                let selected = self.viewport_controller.selection().active_primary();
                let command = self
                    .world
                    .try_with_world_mut(|scene| {
                        EditorCommand::rename_node(scene, selected, id, name)
                    })
                    .ok_or_else(no_project_open)??;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.history.push(command);
                self.sync_selection_state();
                self.status_line = format!("Renamed node {id}");
                Ok(true)
            }
            EditorIntent::SetParent(id, parent) => {
                let selected = self.viewport_controller.selection().active_primary();
                let command = self
                    .world
                    .try_with_world_mut(|scene| {
                        EditorCommand::set_parent(scene, selected, id, parent)
                    })
                    .ok_or_else(no_project_open)??;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.history.push(command);
                self.sync_selection_state();
                self.status_line = match parent {
                    Some(parent) => format!("Reparented node {id} under {parent}"),
                    None => format!("Detached node {id} to root"),
                };
                Ok(true)
            }
            EditorIntent::SetTransform(id, transform) => {
                let selected = self.viewport_controller.selection().active_primary();
                let command = self
                    .world
                    .try_with_world_mut(|scene| {
                        EditorCommand::set_transform(scene, selected, id, transform)
                    })
                    .ok_or_else(no_project_open)??;
                let Some(command) = command else {
                    return Ok(false);
                };
                self.history.push(command);
                self.sync_selection_state();
                self.status_line = format!("Updated transform for node {id}");
                Ok(true)
            }
            EditorIntent::ApplyInspectorChanges => self.apply_inspector_changes(),
            EditorIntent::BeginGizmoDrag => {
                let selected = self.viewport_controller.selection().active_primary();
                let history = &mut self.history;
                self.world
                    .try_with_world(|scene| history.begin_drag(scene, selected))
                    .ok_or_else(no_project_open)?;
                self.status_line = "Translate gizmo drag".to_string();
                Ok(false)
            }
            EditorIntent::DragGizmo => {
                self.status_line = "Dragging translate gizmo".to_string();
                Ok(false)
            }
            EditorIntent::EndGizmoDrag => {
                let selected = self.viewport_controller.selection().active_primary();
                let history = &mut self.history;
                let command = self
                    .world
                    .try_with_world(|scene| history.end_drag(scene, selected))
                    .ok_or_else(no_project_open)??;
                if let Some(command) = command {
                    self.history.push(command);
                    self.sync_selection_state();
                }
                self.status_line = "Gizmo drag finished".to_string();
                Ok(false)
            }
            EditorIntent::Undo => {
                let history = &mut self.history;
                let outcome = self
                    .world
                    .try_with_world_mut(|scene| history.undo(scene))
                    .ok_or_else(no_project_open)??;
                if let Some(outcome) = outcome {
                    if let Some(selection) = outcome.selection {
                        let (items, primary) = selection.into_parts();
                        self.viewport_controller
                            .selection_mut()
                            .replace_active(items, primary);
                    }
                    self.sync_selection_state();
                    self.status_line = "Undo".to_string();
                    Ok(true)
                } else {
                    self.status_line = "Nothing to undo".to_string();
                    Ok(false)
                }
            }
            EditorIntent::Redo => {
                let history = &mut self.history;
                let outcome = self
                    .world
                    .try_with_world_mut(|scene| history.redo(scene))
                    .ok_or_else(no_project_open)??;
                if let Some(outcome) = outcome {
                    if let Some(selection) = outcome.selection {
                        let (items, primary) = selection.into_parts();
                        self.viewport_controller
                            .selection_mut()
                            .replace_active(items, primary);
                    }
                    self.sync_selection_state();
                    self.status_line = "Redo".to_string();
                    Ok(true)
                } else {
                    self.status_line = "Nothing to redo".to_string();
                    Ok(false)
                }
            }
        }
    }

    fn reconcile_selection_after_delete(
        &mut self,
        selection_before: Vec<u64>,
        primary_before: Option<u64>,
        fallback: Option<u64>,
    ) -> Result<(), String> {
        let surviving = self
            .world
            .try_with_world(|scene| {
                selection_before
                    .into_iter()
                    .filter(|entity| scene.contains_entity(*entity))
                    .collect::<Vec<_>>()
            })
            .ok_or_else(no_project_open)?;
        if surviving.is_empty() {
            match fallback {
                Some(entity) => self
                    .viewport_controller
                    .selection_mut()
                    .select_only_active(entity),
                None => self.viewport_controller.selection_mut().clear_active(),
            };
            return Ok(());
        }

        let primary = primary_before.filter(|entity| surviving.contains(entity));
        self.viewport_controller
            .selection_mut()
            .replace_active(surviving, primary);
        Ok(())
    }

    fn active_history_selection_snapshot(&self) -> HistorySelectionSnapshot {
        let selection = self.viewport_controller.selection();
        HistorySelectionSnapshot::new(
            selection.active_items().iter().copied().collect(),
            selection.active_primary(),
        )
    }
}
