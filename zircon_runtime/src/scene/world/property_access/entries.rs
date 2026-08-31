use serde_json::Value;

mod animation;
mod camera;
mod lighting;
mod mesh;
mod physics;

use crate::core::framework::scene::{
    ComponentPropertyPath, ScenePropertyEntry, ScenePropertyValue,
};
use crate::scene::EntityId;
use crate::scene::components::{LocalTransform, Mobility, Name};

use super::super::World;

impl World {
    pub(super) fn property_entries(&self, entity: EntityId) -> Vec<ScenePropertyEntry> {
        if !self.contains_entity(entity) {
            return Vec::new();
        }

        let mut entries = Vec::with_capacity(self.property_entry_capacity_hint(entity));
        self.visit_property_entries(entity, true, |path, value, animatable| {
            entries.push(ScenePropertyEntry::new(
                ComponentPropertyPath::parse(path).expect("valid property path"),
                value(),
                animatable,
            ));
            true
        });

        entries
    }

    fn visit_property_entries<F>(&self, entity: EntityId, include_dynamic: bool, mut visitor: F)
    where
        F: FnMut(&str, &mut dyn FnMut() -> ScenePropertyValue, bool) -> bool,
    {
        if !self.contains_entity(entity) {
            return;
        }

        // A targeted read compares the path before invoking this factory, so it
        // never materializes values for unrelated inspector fields.
        macro_rules! push_entry {
            ($path:expr, $value:expr, $animatable:expr $(,)?) => {
                let mut build_value = || $value;
                if !visitor($path, &mut build_value, $animatable) {
                    return;
                }
            };
        }

        if let Some(name) = self.get::<Name>(entity) {
            push_entry!(
                "Name.value",
                ScenePropertyValue::String(name.0.clone()),
                false,
            );
        }
        push_entry!(
            "Hierarchy.parent",
            ScenePropertyValue::Entity(self.parent_of(entity)),
            false,
        );
        if let Some(local) = self.get::<LocalTransform>(entity).copied() {
            push_entry!(
                "Transform.translation",
                ScenePropertyValue::Vec3(local.transform.translation.to_array()),
                true,
            );
            push_entry!(
                "Transform.rotation",
                ScenePropertyValue::Quaternion(local.transform.rotation.to_array()),
                true,
            );
            push_entry!(
                "Transform.scale",
                ScenePropertyValue::Vec3(local.transform.scale.to_array()),
                true,
            );
        }
        if let Some(active) = self.active_self(entity) {
            push_entry!("Active.enabled", ScenePropertyValue::Bool(active), false);
        }
        if let Some(mask) = self.render_layer_mask(entity) {
            push_entry!(
                "RenderLayer.mask",
                ScenePropertyValue::Unsigned(mask as u64),
                false,
            );
        }
        if let Some(mobility) = self.mobility(entity) {
            push_entry!(
                "Mobility.kind",
                ScenePropertyValue::Enum(match mobility {
                    Mobility::Dynamic => "dynamic".to_string(),
                    Mobility::Static => "static".to_string(),
                }),
                false,
            );
        }
        if !self.visit_camera_property_entries(entity, &mut visitor) {
            return;
        }
        if !self.visit_mesh_property_entries(entity, &mut visitor) {
            return;
        }
        if !self.visit_lighting_property_entries(entity, &mut visitor) {
            return;
        }
        if !self.visit_physics_property_entries(entity, &mut visitor) {
            return;
        }
        if !self.visit_animation_property_entries(entity, &mut visitor) {
            return;
        }
        if include_dynamic {
            let Some(dynamic_components) = self.dynamic_components.get(&entity) else {
                return;
            };
            for (component_id, component_value) in dynamic_components {
                let Some(properties) = component_value.as_object() else {
                    continue;
                };
                for (property, value) in properties {
                    let Some(scene_value) = dynamic_scene_value_from_json(value) else {
                        continue;
                    };
                    push_entry!(
                        &format!("{component_id}.{property}"),
                        scene_value.clone(),
                        true,
                    );
                }
            }
        }
    }

    fn property_entry_capacity_hint(&self, entity: EntityId) -> usize {
        // The counts mirror the property groups pushed by `property_entries`.
        let mut capacity = 1;

        if self.contains_component::<Name>(entity) {
            capacity += 1;
        }
        if self.contains_component::<LocalTransform>(entity) {
            capacity += 3;
        }
        if self.active_self(entity).is_some() {
            capacity += 1;
        }
        if self.render_layer_mask(entity).is_some() {
            capacity += 1;
        }
        if self.mobility(entity).is_some() {
            capacity += 1;
        }
        capacity += self.camera_property_entry_capacity_hint(entity);
        capacity += self.mesh_property_entry_capacity_hint(entity);
        capacity += self.lighting_property_entry_capacity_hint(entity);
        capacity += self.physics_property_entry_capacity_hint(entity);
        capacity += self.animation_property_entry_capacity_hint(entity);
        if let Some(dynamic_components) = self.dynamic_components.get(&entity) {
            for (_component_id, component_value) in dynamic_components {
                let Some(properties) = component_value.as_object() else {
                    continue;
                };
                for value in properties.values() {
                    if dynamic_scene_value_is_projectable(value) {
                        capacity += 1;
                    }
                }
            }
        }

        capacity
    }
}

fn dynamic_scene_value_from_json(value: &Value) -> Option<ScenePropertyValue> {
    match value {
        Value::Bool(value) => Some(ScenePropertyValue::Bool(*value)),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                return Some(ScenePropertyValue::Integer(value));
            }
            if let Some(value) = value.as_u64() {
                return Some(ScenePropertyValue::Unsigned(value));
            }
            if let Some(value) = value.as_f64() {
                return Some(ScenePropertyValue::Scalar(value as _));
            }
            None
        }
        Value::String(value) => Some(ScenePropertyValue::String(value.clone())),
        Value::Null | Value::Array(_) | Value::Object(_) => None,
    }
}

fn dynamic_scene_value_is_projectable(value: &Value) -> bool {
    match value {
        Value::Bool(_) | Value::String(_) => true,
        Value::Number(value) => {
            value.as_i64().is_some() || value.as_u64().is_some() || value.as_f64().is_some()
        }
        Value::Null | Value::Array(_) | Value::Object(_) => false,
    }
}
