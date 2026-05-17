use zircon_runtime::scene::NodeId;
use zircon_runtime_interface::math::Vec3;
use zircon_runtime_interface::reflect::{ReflectObjectAddress, ReflectReadRequest, ReflectedValue};
use zircon_runtime_interface::resource::{MaterialMarker, ModelMarker, ResourceHandle};

use crate::core::editing::command::{EditorCommand, NodeEditState};
use crate::core::editing::intent::EditorIntent;

use super::editor_state::EditorState;
use super::no_project_open::no_project_open;
use super::parse_parent_field::parse_parent_field;

const NAME_COMPONENT_TYPE_PATH: &str = "zircon_runtime::scene::components::Name";
const HIERARCHY_COMPONENT_TYPE_PATH: &str = "zircon_runtime::scene::components::Hierarchy";
const LOCAL_TRANSFORM_COMPONENT_TYPE_PATH: &str =
    "zircon_runtime::scene::components::LocalTransform";

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
                    .try_with_world(|scene| scene.find_node(node_id).map(|_| node_id))
                    .flatten()
            });
        let Some(node_id) = selected else {
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
        let mut reflected_updates =
            self.prepare_reflected_node_updates(parent, Vec3::new(x, y, z))?;
        reflected_updates.extend(self.prepare_reflected_component_updates(node_id)?);
        let selected = self.viewport_controller.selected_node();
        let mut commands = Vec::new();

        for update in reflected_updates {
            let result = self
                .world
                .try_with_world_mut(|scene| {
                    EditorCommand::set_reflected_scene_field(
                        scene,
                        selected,
                        node_id,
                        update.component_type_path,
                        update.field_name,
                        update.value,
                    )
                })
                .ok_or_else(no_project_open)?;
            match result {
                Ok(Some(command)) => commands.push(command),
                Ok(None) => {}
                Err(error) => {
                    self.rollback_inspector_commands(&commands)?;
                    self.sync_selection_state();
                    return Err(error);
                }
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

    fn rollback_inspector_commands(&mut self, commands: &[EditorCommand]) -> Result<(), String> {
        self.world
            .try_with_world_mut(|scene| {
                for command in commands.iter().rev() {
                    command.undo(scene)?;
                }
                Ok::<(), String>(())
            })
            .ok_or_else(no_project_open)??;
        Ok(())
    }

    fn prepare_reflected_node_updates(
        &self,
        parent: Option<NodeId>,
        translation: Vec3,
    ) -> Result<Vec<ReflectedInspectorUpdate>, String> {
        let name = self.name_field.trim().to_string();
        if name.is_empty() {
            return Err("node name cannot be empty".to_string());
        }
        Ok(vec![
            ReflectedInspectorUpdate {
                component_type_path: NAME_COMPONENT_TYPE_PATH.to_string(),
                field_name: "value".to_string(),
                value: ReflectedValue::String(name),
            },
            ReflectedInspectorUpdate {
                component_type_path: HIERARCHY_COMPONENT_TYPE_PATH.to_string(),
                field_name: "parent".to_string(),
                value: ReflectedValue::Entity(parent),
            },
            ReflectedInspectorUpdate {
                component_type_path: LOCAL_TRANSFORM_COMPONENT_TYPE_PATH.to_string(),
                field_name: "translation".to_string(),
                value: ReflectedValue::Vec3(translation.to_array()),
            },
        ])
    }

    fn prepare_reflected_component_updates(
        &self,
        node_id: NodeId,
    ) -> Result<Vec<ReflectedInspectorUpdate>, String> {
        let updates = self.inspector_dynamic_fields.clone();
        self.world
            .try_with_world(|scene| {
                updates
                    .into_iter()
                    .map(|(field_id, value)| {
                        let (component_type_path, field_name) =
                            split_reflected_field_id(&field_id)?;
                        let current = scene
                            .reflect_read(ReflectReadRequest::new(
                                ReflectObjectAddress::component(node_id, &component_type_path)
                                    .map_err(|error| error.to_string())?,
                                field_name.clone(),
                            ))
                            .map_err(|error| error.to_string())?
                            .field
                            .value;
                        let value = reflected_value_from_text(&value, &current)?;
                        Ok(ReflectedInspectorUpdate {
                            component_type_path,
                            field_name,
                            value,
                        })
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

struct ReflectedInspectorUpdate {
    component_type_path: String,
    field_name: String,
    value: ReflectedValue,
}

fn split_reflected_field_id(field_id: &str) -> Result<(String, String), String> {
    let (component_type_path, field_name) = field_id
        .rsplit_once('.')
        .ok_or_else(|| format!("unsupported inspector field {field_id}"))?;
    if component_type_path.trim().is_empty() || field_name.trim().is_empty() {
        return Err(format!("unsupported inspector field {field_id}"));
    }
    Ok((component_type_path.to_string(), field_name.to_string()))
}

fn reflected_value_from_text(
    value: &str,
    current: &ReflectedValue,
) -> Result<ReflectedValue, String> {
    match current {
        ReflectedValue::Bool(_) => parse_bool(value).map(ReflectedValue::Bool),
        ReflectedValue::Integer(_) => value
            .trim()
            .parse::<i64>()
            .map(ReflectedValue::Integer)
            .map_err(|_| format!("Inspector property value `{value}` must be a signed integer")),
        ReflectedValue::Unsigned(_) => value
            .trim()
            .parse::<u64>()
            .map(ReflectedValue::Unsigned)
            .map_err(|_| format!("Inspector property value `{value}` must be an unsigned integer")),
        ReflectedValue::Scalar(_) => value
            .trim()
            .parse::<f32>()
            .map(ReflectedValue::Scalar)
            .map_err(|_| format!("Inspector property value `{value}` must be a number")),
        ReflectedValue::String(_) => Ok(ReflectedValue::String(value.to_string())),
        ReflectedValue::Enum(_) => Ok(ReflectedValue::Enum(value.to_string())),
        ReflectedValue::Resource(_) => Ok(ReflectedValue::Resource(value.to_string())),
        ReflectedValue::Vec2(_) => {
            parse_f32_array::<2>(value, "Vec2").map(ReflectedValue::Vec2)
        }
        ReflectedValue::Vec3(_) => {
            parse_f32_array::<3>(value, "Vec3").map(ReflectedValue::Vec3)
        }
        ReflectedValue::Vec4(_) => {
            parse_f32_array::<4>(value, "Vec4").map(ReflectedValue::Vec4)
        }
        ReflectedValue::Quaternion(_) => parse_f32_array::<4>(value, "Quaternion")
            .map(ReflectedValue::Quaternion),
        ReflectedValue::Entity(_) => parse_entity_value(value).map(ReflectedValue::Entity),
        ReflectedValue::Null
        | ReflectedValue::List(_)
        | ReflectedValue::Map(_)
        | ReflectedValue::Json(_) => Err(
            "Inspector component drawer only supports scalar, bool, string, enum, resource, vector, quaternion, and entity fields"
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

fn parse_f32_array<const N: usize>(value: &str, type_name: &str) -> Result<[f32; N], String> {
    let trimmed = value.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .or_else(|| {
            trimmed
                .strip_prefix('(')
                .and_then(|value| value.strip_suffix(')'))
        })
        .unwrap_or(trimmed);
    let components = inner
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .filter(|component| !component.trim().is_empty())
        .collect::<Vec<_>>();

    if components.len() != N {
        return Err(format!(
            "Inspector property value `{value}` must be a {type_name} with {N} finite numbers"
        ));
    }

    let mut parsed = [0.0_f32; N];
    for (slot, component) in parsed.iter_mut().zip(components) {
        let component = component.parse::<f32>().map_err(|_| {
            format!(
                "Inspector property value `{value}` must be a {type_name} with {N} finite numbers"
            )
        })?;
        if !component.is_finite() {
            return Err(format!(
                "Inspector property value `{value}` must be a {type_name} with {N} finite numbers"
            ));
        }
        *slot = component;
    }

    Ok(parsed)
}

fn parse_entity_value(value: &str) -> Result<Option<NodeId>, String> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("none")
        || trimmed.eq_ignore_ascii_case("null")
    {
        return Ok(None);
    }
    trimmed
        .parse::<NodeId>()
        .map(Some)
        .map_err(|_| format!("Inspector property value `{value}` must be an entity id or none"))
}
