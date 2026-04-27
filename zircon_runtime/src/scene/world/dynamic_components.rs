use serde_json::{Map, Value};

use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::EntityId;

use super::World;

impl World {
    pub fn set_dynamic_component(
        &mut self,
        entity: EntityId,
        component_id: impl Into<String>,
        value: Value,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot attach dynamic component to missing entity {entity}"
            ));
        }
        let component_id = component_id.into();
        let components = self.dynamic_components.entry(entity).or_default();
        if components.get(&component_id) == Some(&value) {
            return Ok(false);
        }
        components.insert(component_id, value);
        Ok(true)
    }

    pub fn dynamic_component(&self, entity: EntityId, component_id: &str) -> Option<&Value> {
        self.dynamic_components
            .get(&entity)
            .and_then(|components| components.get(component_id))
    }

    pub fn remove_dynamic_component(
        &mut self,
        entity: EntityId,
        component_id: &str,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!(
                "cannot remove dynamic component from missing entity {entity}"
            ));
        }
        let Some(components) = self.dynamic_components.get_mut(&entity) else {
            return Ok(false);
        };
        let removed = components.remove(component_id).is_some();
        if components.is_empty() {
            self.dynamic_components.remove(&entity);
        }
        Ok(removed)
    }

    pub fn dynamic_component_count_for_plugin(&self, plugin_id: &str) -> usize {
        self.dynamic_component_refs_for_plugin(plugin_id).count()
    }

    pub fn ensure_plugin_components_can_unload(&self, plugin_id: &str) -> Result<(), String> {
        let active_components = self
            .dynamic_component_refs_for_plugin(plugin_id)
            .map(|(entity, component_id)| format!("{component_id} on entity {entity}"))
            .collect::<Vec<_>>();
        if active_components.is_empty() {
            return Ok(());
        }
        Err(format!(
            "plugin `{plugin_id}` cannot unload while dynamic components are active: {}",
            active_components.join(", ")
        ))
    }

    pub(crate) fn dynamic_component_property(
        &self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
    ) -> Option<ScenePropertyValue> {
        let (component_id, property) = split_dynamic_property_path(property_path)?;
        let value = self.dynamic_component(entity, &component_id)?;
        json_property(value, &property).and_then(scene_value_from_json)
    }

    pub(crate) fn set_dynamic_component_property(
        &mut self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
        value: ScenePropertyValue,
    ) -> Result<bool, String> {
        if !self.contains_entity(entity) {
            return Err(format!("cannot update missing entity {entity}"));
        }
        let (component_id, property) = split_dynamic_property_path(property_path)
            .ok_or_else(|| format!("unknown property `{property_path}`"))?;
        let Some(json_value) = json_from_scene_value(value) else {
            return Err(format!(
                "property `{property_path}` cannot be written to a dynamic component"
            ));
        };
        let components = self.dynamic_components.entry(entity).or_default();
        let component = components
            .entry(component_id.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        let Some(object) = component.as_object_mut() else {
            return Err(format!(
                "dynamic component `{component_id}` is not an object"
            ));
        };
        if object.get(&property) == Some(&json_value) {
            return Ok(false);
        }
        object.insert(property, json_value);
        Ok(true)
    }

    fn dynamic_component_refs_for_plugin<'a>(
        &'a self,
        plugin_id: &'a str,
    ) -> impl Iterator<Item = (EntityId, &'a str)> + 'a {
        let prefix = format!("{plugin_id}.");
        self.dynamic_components
            .iter()
            .flat_map(move |(entity, components)| {
                let prefix = prefix.clone();
                components
                    .keys()
                    .filter(move |component_id| component_id.starts_with(&prefix))
                    .map(move |component_id| (*entity, component_id.as_str()))
            })
    }
}

fn split_dynamic_property_path(property_path: &ComponentPropertyPath) -> Option<(String, String)> {
    let (component_id, property) = property_path.as_str().rsplit_once('.')?;
    Some((component_id.to_string(), property.to_string()))
}

fn json_property<'a>(value: &'a Value, property: &str) -> Option<&'a Value> {
    value.as_object()?.get(property)
}

fn scene_value_from_json(value: &Value) -> Option<ScenePropertyValue> {
    match value {
        Value::Bool(value) => Some(ScenePropertyValue::Bool(*value)),
        Value::Number(value) => value
            .as_i64()
            .map(ScenePropertyValue::Integer)
            .or_else(|| value.as_u64().map(ScenePropertyValue::Unsigned))
            .or_else(|| {
                value
                    .as_f64()
                    .map(|value| ScenePropertyValue::Scalar(value as _))
            }),
        Value::String(value) => Some(ScenePropertyValue::String(value.clone())),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
    }
}

fn json_from_scene_value(value: ScenePropertyValue) -> Option<Value> {
    match value {
        ScenePropertyValue::Bool(value) => Some(Value::Bool(value)),
        ScenePropertyValue::Integer(value) => Some(Value::Number(value.into())),
        ScenePropertyValue::Unsigned(value) => Some(Value::Number(value.into())),
        ScenePropertyValue::Scalar(value) => value
            .to_string()
            .parse::<serde_json::Number>()
            .ok()
            .map(Value::Number),
        ScenePropertyValue::String(value) | ScenePropertyValue::Enum(value) => {
            Some(Value::String(value))
        }
        ScenePropertyValue::Vec2(_)
        | ScenePropertyValue::Vec3(_)
        | ScenePropertyValue::Vec4(_)
        | ScenePropertyValue::Quaternion(_)
        | ScenePropertyValue::Entity(_)
        | ScenePropertyValue::Resource(_)
        | ScenePropertyValue::AnimationParameter(_) => None,
    }
}
