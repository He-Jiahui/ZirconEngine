use zircon_runtime::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use zircon_runtime_interface::math::Transform;
use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use crate::core::editing::command::{EditorCommand, NodeEditState};
use crate::core::editing::intent::EditorIntent;

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
        self.apply_intent(EditorIntent::DeleteNode(node_id))
    }

    pub fn apply_inspector_changes(&mut self) -> Result<bool, String> {
        let selected = self
            .viewport_controller
            .selected_node()
            .and_then(|node_id| {
                self.world
                    .try_with_world(|scene| {
                        scene
                            .find_node(node_id)
                            .cloned()
                            .map(|node| (node_id, node))
                    })
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
            translation: zircon_runtime_interface::math::Vec3::new(x, y, z),
            ..current.transform
        };
        let dynamic_updates = self.prepare_dynamic_component_updates(node_id)?;
        let selected = self.viewport_controller.selected_node();
        let mut commands = Vec::new();
        if let Some(command) = self
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
            .ok_or_else(no_project_open)??
        {
            commands.push(command);
        }

        for (property_path, value) in dynamic_updates {
            if let Some(command) = self
                .world
                .try_with_world_mut(|scene| {
                    EditorCommand::set_scene_property(
                        scene,
                        selected,
                        node_id,
                        property_path,
                        value,
                    )
                })
                .ok_or_else(no_project_open)??
            {
                commands.push(command);
            }
        }

        let Some(command) = EditorCommand::batch(commands) else {
            return Ok(false);
        };
        self.history.push(command);
        self.sync_selection_state();
        self.status_line = format!("Applied inspector changes to node {node_id}");
        Ok(true)
    }

    fn prepare_dynamic_component_updates(
        &self,
        node_id: zircon_runtime::scene::NodeId,
    ) -> Result<Vec<(ComponentPropertyPath, ScenePropertyValue)>, String> {
        let updates = self.inspector_dynamic_fields.clone();
        self.world
            .try_with_world(|scene| {
                updates
                    .into_iter()
                    .map(|(field_id, value)| {
                        let property_path = ComponentPropertyPath::parse(&field_id)
                            .map_err(|error| error.to_string())?;
                        let current = scene.property(node_id, &property_path)?;
                        let value = scene_property_value_from_text(&value, &current)?;
                        Ok((property_path, value))
                    })
                    .collect()
            })
            .ok_or_else(no_project_open)?
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
        let selected_state = self
            .viewport_controller
            .selected_node()
            .and_then(|selected| {
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
            self.inspector_dynamic_fields.clear();
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
        self.inspector_dynamic_fields.clear();
    }
}

fn scene_property_value_from_text(
    value: &str,
    current: &ScenePropertyValue,
) -> Result<ScenePropertyValue, String> {
    match current {
        ScenePropertyValue::Bool(_) => parse_bool(value).map(ScenePropertyValue::Bool),
        ScenePropertyValue::Integer(_) => value
            .trim()
            .parse::<i64>()
            .map(ScenePropertyValue::Integer)
            .map_err(|_| format!("Inspector property value `{value}` must be a signed integer")),
        ScenePropertyValue::Unsigned(_) => value
            .trim()
            .parse::<u64>()
            .map(ScenePropertyValue::Unsigned)
            .map_err(|_| format!("Inspector property value `{value}` must be an unsigned integer")),
        ScenePropertyValue::Scalar(_) => value
            .trim()
            .parse::<f32>()
            .map(ScenePropertyValue::Scalar)
            .map_err(|_| format!("Inspector property value `{value}` must be a number")),
        ScenePropertyValue::String(_) => Ok(ScenePropertyValue::String(value.to_string())),
        ScenePropertyValue::Enum(_) => Ok(ScenePropertyValue::Enum(value.to_string())),
        ScenePropertyValue::Resource(_) => Ok(ScenePropertyValue::Resource(value.to_string())),
        ScenePropertyValue::Vec2(_)
        | ScenePropertyValue::Vec3(_)
        | ScenePropertyValue::Vec4(_)
        | ScenePropertyValue::Quaternion(_)
        | ScenePropertyValue::Entity(_)
        | ScenePropertyValue::AnimationParameter(_) => Err(
            "Inspector component drawer only supports scalar, bool, string, enum, and resource fields"
                .to_string(),
        ),
    }
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(format!("Inspector property value `{value}` must be a bool")),
    }
}
