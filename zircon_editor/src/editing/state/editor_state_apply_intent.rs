use crate::command::EditorCommand;
use crate::intent::EditorIntent;

use super::editor_state::EditorState;
use super::no_project_open::no_project_open;

impl EditorState {
    pub fn apply_intent(&mut self, intent: EditorIntent) -> Result<bool, String> {
        match intent {
            EditorIntent::CreateNode(kind) => {
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::create_node(scene, kind))
                    .ok_or_else(no_project_open)??;
                let id = command.target_node();
                self.history.push(command);
                self.sync_selection_state();
                self.status_line = format!("Created node {id}");
                Ok(true)
            }
            EditorIntent::DeleteNode(id) => {
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::delete_node(scene, id))
                    .ok_or_else(no_project_open)??;
                self.history.push(command);
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
                self.world
                    .try_with_world_mut(|scene| scene.set_selected(Some(id)))
                    .ok_or_else(no_project_open)?;
                self.sync_selection_state();
                self.status_line = format!("Selected node {id}");
                Ok(true)
            }
            EditorIntent::RenameNode(id, name) => {
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::rename_node(scene, id, name))
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
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::set_parent(scene, id, parent))
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
                let command = self
                    .world
                    .try_with_world_mut(|scene| EditorCommand::set_transform(scene, id, transform))
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
                let history = &mut self.history;
                self.world
                    .try_with_world(|scene| history.begin_drag(scene))
                    .ok_or_else(no_project_open)?;
                self.status_line = "Translate gizmo drag".to_string();
                Ok(false)
            }
            EditorIntent::DragGizmo => {
                self.status_line = "Dragging translate gizmo".to_string();
                Ok(false)
            }
            EditorIntent::EndGizmoDrag => {
                let history = &mut self.history;
                let command = self
                    .world
                    .try_with_world(|scene| history.end_drag(scene))
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
                let changed = self
                    .world
                    .try_with_world_mut(|scene| history.undo(scene))
                    .ok_or_else(no_project_open)??;
                if changed {
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
                let changed = self
                    .world
                    .try_with_world_mut(|scene| history.redo(scene))
                    .ok_or_else(no_project_open)??;
                if changed {
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
}
