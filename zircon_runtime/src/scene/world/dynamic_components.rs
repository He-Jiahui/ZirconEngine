use std::fmt::Write as _;

use serde_json::{Map, Number, Value};

use crate::core::framework::scene::ComponentTypeDescriptor;
use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::{reflect::RuntimeTypeRegistration, EntityId};
use zircon_runtime_interface::reflect::ReflectError;

use super::error::{SceneError, SceneResult};
use super::World;

#[derive(Clone, Debug, PartialEq)]
pub struct DynamicComponentInstance {
    pub component_id: String,
    pub value: Value,
    pub descriptor: Option<ComponentTypeDescriptor>,
}

impl World {
    pub fn register_component_type(
        &mut self,
        descriptor: ComponentTypeDescriptor,
    ) -> SceneResult<()> {
        let registration =
            crate::scene::reflect::registration_from_component_descriptor(&descriptor)?;
        if self.type_registry.contains(&descriptor.type_id) {
            return Err(ReflectError::DuplicateTypePath {
                type_path: descriptor.type_id.clone(),
            }
            .into());
        }
        let component =
            crate::scene::reflect::reflect_component_for_dynamic_descriptor(&descriptor);
        self.component_types.register(descriptor)?;
        self.type_registry.register(RuntimeTypeRegistration {
            registration,
            component: Some(component),
            resource: None,
        })?;
        Ok(())
    }

    pub fn component_type_descriptor(&self, type_id: &str) -> Option<&ComponentTypeDescriptor> {
        self.component_types.descriptor(type_id)
    }

    pub fn component_type_descriptors(&self) -> Vec<&ComponentTypeDescriptor> {
        let descriptors = self.component_types.descriptors();
        let mut result = Vec::with_capacity(descriptors.size_hint().0);
        for descriptor in descriptors {
            result.push(descriptor);
        }
        result
    }

    pub fn set_dynamic_component(
        &mut self,
        entity: EntityId,
        component_id: impl Into<String>,
        value: Value,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "attach dynamic component to",
                entity,
            ));
        }
        let component_id = component_id.into();
        self.validate_dynamic_component_type(&component_id)?;
        let components = self.dynamic_components.entry(entity).or_default();
        if components.get(&component_id) == Some(&value) {
            return Ok(false);
        }
        components.insert(component_id.clone(), value);
        self.insert_dynamic_component_presence(entity, &component_id)?;
        Ok(true)
    }

    pub fn dynamic_component(&self, entity: EntityId, component_id: &str) -> Option<&Value> {
        let components = self.dynamic_components.get(&entity)?;
        components.get(component_id)
    }

    pub fn dynamic_components_for_entity(&self, entity: EntityId) -> Vec<DynamicComponentInstance> {
        let Some(components) = self.dynamic_components.get(&entity) else {
            return Vec::new();
        };
        let mut instances = Vec::with_capacity(components.len());
        for (component_id, value) in components {
            instances.push(DynamicComponentInstance {
                component_id: component_id.clone(),
                value: value.clone(),
                descriptor: self.component_types.descriptor(component_id).cloned(),
            });
        }
        instances.sort_by(|left, right| left.component_id.cmp(&right.component_id));
        instances
    }

    pub fn remove_dynamic_component(
        &mut self,
        entity: EntityId,
        component_id: &str,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity(
                "remove dynamic component from",
                entity,
            ));
        }
        let Some(components) = self.dynamic_components.get_mut(&entity) else {
            return Ok(false);
        };
        let removed = components.remove(component_id).is_some();
        if components.is_empty() {
            self.dynamic_components.remove(&entity);
        }
        if removed {
            self.remove_dynamic_component_presence(entity, component_id)?;
        }
        Ok(removed)
    }

    pub fn dynamic_component_count_for_plugin(&self, plugin_id: &str) -> usize {
        let mut count = 0_usize;
        for components in self.dynamic_components.values() {
            for component_id in components.keys() {
                if dynamic_component_belongs_to_plugin(component_id, plugin_id) {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn ensure_plugin_components_can_unload(&self, plugin_id: &str) -> SceneResult<()> {
        let mut active_components = String::new();
        let mut has_active_components = false;
        for (entity, components) in &self.dynamic_components {
            for component_id in components.keys() {
                if !dynamic_component_belongs_to_plugin(component_id, plugin_id) {
                    continue;
                }
                if has_active_components {
                    active_components.push_str(", ");
                }
                has_active_components = true;
                let _ = write!(&mut active_components, "{component_id} on entity {entity}");
            }
        }
        if !has_active_components {
            return Ok(());
        }
        Err(SceneError::PluginComponentsActive {
            plugin_id: plugin_id.to_string(),
            active_components,
        })
    }

    pub(crate) fn dynamic_component_property(
        &self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
    ) -> Option<ScenePropertyValue> {
        let (component_id, property) = split_dynamic_property_path(property_path)?;
        let value = self.dynamic_component(entity, component_id)?;
        let value = json_property(value, property)?;
        scene_value_from_json(value)
    }

    pub(crate) fn set_dynamic_component_property(
        &mut self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
        value: ScenePropertyValue,
    ) -> SceneResult<bool> {
        if !self.contains_entity(entity) {
            return Err(SceneError::missing_entity("update", entity));
        }
        let Some((component_id, property)) = split_dynamic_property_path(property_path) else {
            return Err(SceneError::UnknownDynamicComponentProperty {
                property_path: property_path.to_string(),
            });
        };
        self.validate_dynamic_component_type(component_id)?;
        self.validate_dynamic_component_property_write(component_id, property)?;
        let Some(json_value) = json_from_scene_value(value) else {
            return Err(SceneError::DynamicComponentPropertyUnsupportedValue {
                property_path: property_path.to_string(),
            });
        };
        let components = self.dynamic_components.entry(entity).or_default();
        let component = components
            .entry(component_id.to_string())
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(object) = component.as_object_mut() else {
            return Err(SceneError::DynamicComponentNotObject {
                component_id: component_id.to_string(),
            });
        };
        if object.get(property) == Some(&json_value) {
            return Ok(false);
        }
        object.insert(property.to_string(), json_value);
        self.insert_dynamic_component_presence(entity, component_id)?;
        Ok(true)
    }

    fn validate_dynamic_component_type(&self, component_id: &str) -> SceneResult<()> {
        if self.component_types.is_empty() || self.component_types.contains(component_id) {
            return Ok(());
        }
        Err(SceneError::UnregisteredDynamicComponentType {
            component_id: component_id.to_string(),
        })
    }

    fn validate_dynamic_component_property_write(
        &self,
        component_id: &str,
        property: &str,
    ) -> SceneResult<()> {
        let Some(descriptor) = self.component_types.descriptor(component_id) else {
            return Ok(());
        };
        if descriptor.properties.is_empty() {
            return Ok(());
        }
        let mut property_descriptor = None;
        for descriptor in &descriptor.properties {
            if descriptor.name == property {
                property_descriptor = Some(descriptor);
                break;
            }
        }
        let Some(property_descriptor) = property_descriptor else {
            return Err(SceneError::UndeclaredDynamicComponentProperty {
                component_id: component_id.to_string(),
                property: property.to_string(),
            });
        };
        if !property_descriptor.editable {
            return Err(SceneError::NonEditableDynamicComponentProperty {
                component_id: component_id.to_string(),
                property: property.to_string(),
            });
        }
        Ok(())
    }
}

fn split_dynamic_property_path(property_path: &ComponentPropertyPath) -> Option<(&str, &str)> {
    property_path.as_str().rsplit_once('.')
}

fn dynamic_component_belongs_to_plugin(component_id: &str, plugin_id: &str) -> bool {
    let Some(suffix) = component_id.strip_prefix(plugin_id) else {
        return false;
    };
    suffix.starts_with('.')
}

fn json_property<'a>(value: &'a Value, property: &str) -> Option<&'a Value> {
    value.as_object()?.get(property)
}

fn scene_value_from_json(value: &Value) -> Option<ScenePropertyValue> {
    match value {
        Value::Bool(value) => Some(ScenePropertyValue::Bool(*value)),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                return Some(ScenePropertyValue::Integer(value));
            }
            if let Some(value) = value.as_u64() {
                return Some(ScenePropertyValue::Unsigned(value));
            }
            match value.as_f64() {
                Some(value) => Some(ScenePropertyValue::Scalar(value as _)),
                None => None,
            }
        }
        Value::String(value) => Some(ScenePropertyValue::String(value.clone())),
        Value::Array(values) => scene_vector_from_json(values),
        Value::Object(object) => scene_object_from_json(object),
        Value::Null => None,
    }
}

fn json_from_scene_value(value: ScenePropertyValue) -> Option<Value> {
    match value {
        ScenePropertyValue::Bool(value) => Some(Value::Bool(value)),
        ScenePropertyValue::Integer(value) => Some(Value::Number(value.into())),
        ScenePropertyValue::Unsigned(value) => Some(Value::Number(value.into())),
        ScenePropertyValue::Scalar(value) => match finite_json_number(value) {
            Some(number) => Some(Value::Number(number)),
            None => None,
        },
        ScenePropertyValue::String(value) | ScenePropertyValue::Enum(value) => {
            Some(Value::String(value))
        }
        ScenePropertyValue::Vec2(value) => vector_to_json(value),
        ScenePropertyValue::Vec3(value) => vector_to_json(value),
        ScenePropertyValue::Vec4(value) => vector_to_json(value),
        ScenePropertyValue::Entity(value) => {
            let entity = match value {
                Some(entity) => Value::Number(Number::from(entity)),
                None => Value::Null,
            };
            Some(single_property_object("entity", entity))
        }
        ScenePropertyValue::Resource(value) => {
            Some(single_property_object("resource", Value::String(value)))
        }
        ScenePropertyValue::Quaternion(_) | ScenePropertyValue::AnimationParameter(_) => None,
    }
}

fn scene_vector_from_json(values: &[Value]) -> Option<ScenePropertyValue> {
    match values {
        [x, y] => Some(ScenePropertyValue::Vec2([
            json_number_as_f32(x)?,
            json_number_as_f32(y)?,
        ])),
        [x, y, z] => Some(ScenePropertyValue::Vec3([
            json_number_as_f32(x)?,
            json_number_as_f32(y)?,
            json_number_as_f32(z)?,
        ])),
        [x, y, z, w] => Some(ScenePropertyValue::Vec4([
            json_number_as_f32(x)?,
            json_number_as_f32(y)?,
            json_number_as_f32(z)?,
            json_number_as_f32(w)?,
        ])),
        _ => None,
    }
}

fn json_number_as_f32(value: &Value) -> Option<f32> {
    match value.as_f64() {
        Some(value) => Some(value as _),
        None => None,
    }
}

fn scene_object_from_json(object: &Map<String, Value>) -> Option<ScenePropertyValue> {
    if let Some(value) = object.get("resource") {
        if let Some(value) = value.as_str() {
            return Some(ScenePropertyValue::Resource(value.to_string()));
        }
    }
    if let Some(value) = object.get("entity") {
        return match value {
            Value::Null => Some(ScenePropertyValue::Entity(None)),
            Value::Number(number) => match number.as_u64() {
                Some(entity) => Some(ScenePropertyValue::Entity(Some(entity))),
                None => None,
            },
            _ => None,
        };
    }
    None
}

fn single_property_object(key: &str, value: Value) -> Value {
    let mut object = Map::with_capacity(1);
    object.insert(key.to_string(), value);
    Value::Object(object)
}

fn vector_to_json<const N: usize>(values: [f32; N]) -> Option<Value> {
    let mut array = Vec::with_capacity(N);
    for value in values {
        array.push(Value::Number(finite_json_number(value)?));
    }
    Some(Value::Array(array))
}

fn finite_json_number(value: f32) -> Option<Number> {
    if !value.is_finite() {
        return None;
    }
    let text = value.to_string();
    let Ok(value) = text.parse::<f64>() else {
        return None;
    };
    Number::from_f64(value)
}
