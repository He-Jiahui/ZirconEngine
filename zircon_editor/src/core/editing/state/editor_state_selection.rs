use zircon_math::Transform;
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle};

use crate::command::{EditorCommand, NodeEditState};

use super::editor_state::EditorState;
use super::no_project_open::no_project_open;
use super::parse_parent_field::parse_parent_field;

impl EditorState {
    pub fn delete_selected(&mut self) -> Result<bool, String> {
        let selected = self.viewport_controller.selected_node();
        let Some(node_id) = selected else {
            self.status_line = "Nothing selected".to_string();
            return Ok(false);
        };
        self.apply_intent(crate::EditorIntent::DeleteNode(node_id))
    }

    pub fn apply_inspector_changes(&mut self) -> Result<bool, String> {
        let selected = self.viewport_controller.selected_node().and_then(|node_id| {
            self.world
                .try_with_world(|scene| scene.find_node(node_id).cloned().map(|node| (node_id, node)))
                .flatten()
        });
        let Some((node_id, current)) = selected else {
            return Err("Nothing selected".to_string());
        };

        let parent = parse_parent_field(&self.parent_field)?;
        let parsed = [
            self.transform_fields[0].parse::<f32>(),
            self.transform_fields[1].parse::<f32>(),
            self.transform_fields[2].parse::<f32>(),
        ];
        let [Ok(x), Ok(y), Ok(z)] = parsed else {
            return Err("Transform fields must be valid numbers".to_string());
        };
        let transform = Transform {
            translation: zircon_math::Vec3::new(x, y, z),
            ..current.transform
        };
        let selected = self.viewport_controller.selected_node();
        let command = self
            .world
            .try_with_world_mut(|scene| {
                EditorCommand::update_node(
                    scene,
                    selected,
                    node_id,
                    self.name_field.clone(),
                    parent,
                    transform,
                )
            })
            .ok_or_else(no_project_open)??;
        let Some(command) = command else {
            return Ok(false);
        };
        self.history.push(command);
        self.sync_selection_state();
        self.status_line = format!("Applied inspector changes to node {node_id}");
        Ok(true)
    }

    pub fn import_mesh_asset(
        &mut self,
        model: ResourceHandle<ModelMarker>,
        material: ResourceHandle<MaterialMarker>,
        display_path: impl Into<String>,
    ) -> Result<bool, String> {
        let selected = self.viewport_controller.selected_node();
        let command = self
            .world
            .try_with_world_mut(|scene| {
                EditorCommand::import_mesh(scene, selected, model, material)
            })
            .ok_or_else(no_project_open)??;
        let id = command.target_node();
        self.mesh_import_path = display_path.into();
        self.viewport_controller
            .set_selected_node(command.selection_after());
        self.history.push(command);
        self.sync_selection_state();
        self.status_line = format!("Imported mesh node {id}");
        Ok(true)
    }

    pub(crate) fn sync_selection_state(&mut self) {
        let selected_state = self.viewport_controller.selected_node().and_then(|selected| {
            self.world
                .try_with_world(|scene| {
                    scene.find_node(selected).map(|node| NodeEditState {
                        name: node.name.clone(),
                        parent: node.parent,
                        transform: node.transform,
                    })
                })
                .flatten()
        });
        if let Some(node) = selected_state {
            let translation = node.transform.translation;
            self.name_field = node.name;
            self.parent_field = node
                .parent
                .map(|value| value.to_string())
                .unwrap_or_default();
            self.transform_fields = [
                format!("{:.2}", translation.x),
                format!("{:.2}", translation.y),
                format!("{:.2}", translation.z),
            ];
            self.viewport_controller.set_orbit_target(translation);
            return;
        }

        self.name_field.clear();
        self.parent_field.clear();
        self.transform_fields = [String::new(), String::new(), String::new()];
    }
}
