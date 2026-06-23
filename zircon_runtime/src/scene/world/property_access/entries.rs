use std::fmt::Write as _;

use serde_json::Value;

mod physics;

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::scene::{
    ComponentPropertyPath, ScenePropertyEntry, ScenePropertyValue,
};
use crate::scene::components::Mobility;
use crate::scene::EntityId;

use super::super::World;
use super::value_conversion::normalized_identifier_matches;

const MESH_RENDERER_MORPH_WEIGHT_PATH_PREFIX: &str = "MeshRenderer.morph_weights.";

impl World {
    pub(super) fn property_entries(&self, entity: EntityId) -> Vec<ScenePropertyEntry> {
        if !self.contains_entity(entity) {
            return Vec::new();
        }

        let mut entries = Vec::with_capacity(self.property_entry_capacity_hint(entity));
        self.visit_property_entries(entity, true, |path, value, animatable| {
            entries.push(ScenePropertyEntry::new(
                ComponentPropertyPath::parse(path).expect("valid property path"),
                value,
                animatable,
            ));
            true
        });

        entries
    }

    pub(super) fn property_entry_value(
        &self,
        entity: EntityId,
        target_component: &str,
        target_segments: &[String],
    ) -> Option<ScenePropertyValue> {
        let mut found = None;
        self.visit_property_entries(entity, false, |path, value, _animatable| {
            if property_path_literal_matches_normalized(path, target_component, target_segments) {
                found = Some(value);
                false
            } else {
                true
            }
        });
        found
    }

    fn visit_property_entries<F>(&self, entity: EntityId, include_dynamic: bool, mut visitor: F)
    where
        F: FnMut(&str, ScenePropertyValue, bool) -> bool,
    {
        if !self.contains_entity(entity) {
            return;
        }

        macro_rules! push_entry {
            ($path:expr, $value:expr, $animatable:expr $(,)?) => {
                if !visitor($path, $value, $animatable) {
                    return;
                }
            };
        }

        if let Some(name) = self.names.get(&entity) {
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
        if let Some(local) = self.local_transforms.get(&entity).copied() {
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
        if let Some(camera) = self.cameras.get(&entity) {
            push_entry!(
                "Camera.fov_y_radians",
                ScenePropertyValue::Scalar(camera.fov_y_radians),
                true,
            );
            push_entry!(
                "Camera.z_near",
                ScenePropertyValue::Scalar(camera.z_near),
                true,
            );
            push_entry!(
                "Camera.z_far",
                ScenePropertyValue::Scalar(camera.z_far),
                true,
            );
        }
        if let Some(mesh) = self.mesh_renderers.get(&entity) {
            push_entry!(
                "MeshRenderer.model",
                ScenePropertyValue::Resource(mesh.model.id().to_string()),
                false,
            );
            if let Some(mesh_handle) = mesh.mesh {
                push_entry!(
                    "MeshRenderer.mesh",
                    ScenePropertyValue::Resource(mesh_handle.id().to_string()),
                    false,
                );
            }
            push_entry!(
                "MeshRenderer.material",
                ScenePropertyValue::Resource(mesh.material.id().to_string()),
                false,
            );
            push_entry!(
                "MeshRenderer.render_queue",
                ScenePropertyValue::Integer(mesh.render_queue.into()),
                true,
            );
            push_entry!(
                "MeshRenderer.material_queue",
                ScenePropertyValue::Integer(mesh.material_queue.into()),
                true,
            );
            push_entry!(
                "MeshRenderer.order_in_layer",
                ScenePropertyValue::Integer(mesh.order_in_layer.into()),
                true,
            );
            push_entry!(
                "MeshRenderer.depth_bias",
                ScenePropertyValue::Scalar(mesh.depth_bias),
                true,
            );
            push_entry!(
                "MeshRenderer.primitive_binding_count",
                ScenePropertyValue::Unsigned(mesh.primitives.len() as u64),
                false,
            );
            push_entry!(
                "MeshRenderer.lod_level_count",
                ScenePropertyValue::Unsigned(mesh.lods.len() as u64),
                false,
            );
            push_entry!(
                "MeshRenderer.morph_weight_count",
                ScenePropertyValue::Unsigned(mesh.morph_weights.len() as u64),
                false,
            );
            let mut morph_weight_index = 0;
            while morph_weight_index < mesh.morph_weights.len() {
                let path = mesh_renderer_morph_weight_path(morph_weight_index);
                let weight = mesh.morph_weights[morph_weight_index];
                push_entry!(&path, ScenePropertyValue::Scalar(weight), true,);
                morph_weight_index += 1;
            }
            push_entry!(
                "MeshRenderer.tint",
                ScenePropertyValue::Vec4(mesh.tint.to_array()),
                true,
            );
        }
        if let Some(light) = self.ambient_lights.get(&entity) {
            push_entry!(
                "AmbientLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "AmbientLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push_entry!(
                "AmbientLight.affects_lightmapped_meshes",
                ScenePropertyValue::Bool(light.affects_lightmapped_meshes),
                false,
            );
        }
        if let Some(light) = self.directional_lights.get(&entity) {
            push_entry!(
                "DirectionalLight.direction",
                ScenePropertyValue::Vec3(light.direction.to_array()),
                true,
            );
            push_entry!(
                "DirectionalLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "DirectionalLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
        }
        if let Some(light) = self.point_lights.get(&entity) {
            push_entry!(
                "PointLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "PointLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push_entry!(
                "PointLight.range",
                ScenePropertyValue::Scalar(light.range),
                true,
            );
        }
        if let Some(light) = self.rect_lights.get(&entity) {
            push_entry!(
                "RectLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "RectLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push_entry!(
                "RectLight.range",
                ScenePropertyValue::Scalar(light.range),
                true,
            );
            push_entry!(
                "RectLight.size",
                ScenePropertyValue::Vec2(light.size.to_array()),
                true,
            );
        }
        if let Some(light) = self.spot_lights.get(&entity) {
            push_entry!(
                "SpotLight.direction",
                ScenePropertyValue::Vec3(light.direction.to_array()),
                true,
            );
            push_entry!(
                "SpotLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push_entry!(
                "SpotLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push_entry!(
                "SpotLight.range",
                ScenePropertyValue::Scalar(light.range),
                true,
            );
            push_entry!(
                "SpotLight.inner_angle_radians",
                ScenePropertyValue::Scalar(light.inner_angle_radians),
                true,
            );
            push_entry!(
                "SpotLight.outer_angle_radians",
                ScenePropertyValue::Scalar(light.outer_angle_radians),
                true,
            );
        }
        if !self.visit_physics_property_entries(entity, &mut visitor) {
            return;
        }
        if let Some(skeleton) = self.animation_skeletons.get(&entity) {
            push_entry!(
                "AnimationSkeleton.skeleton",
                ScenePropertyValue::Resource(skeleton.skeleton.id().to_string()),
                false,
            );
        }
        if let Some(player) = self.animation_players.get(&entity) {
            push_entry!(
                "AnimationPlayer.clip",
                ScenePropertyValue::Resource(player.clip.id().to_string()),
                false,
            );
            push_entry!(
                "AnimationPlayer.playback_speed",
                ScenePropertyValue::Scalar(player.playback_speed),
                true,
            );
            push_entry!(
                "AnimationPlayer.time_seconds",
                ScenePropertyValue::Scalar(player.time_seconds),
                true,
            );
            push_entry!(
                "AnimationPlayer.weight",
                ScenePropertyValue::Scalar(player.weight),
                true,
            );
            push_entry!(
                "AnimationPlayer.looping",
                ScenePropertyValue::Bool(player.looping),
                false,
            );
            push_entry!(
                "AnimationPlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
        }
        if let Some(player) = self.animation_sequence_players.get(&entity) {
            push_entry!(
                "AnimationSequencePlayer.sequence",
                ScenePropertyValue::Resource(player.sequence.id().to_string()),
                false,
            );
            push_entry!(
                "AnimationSequencePlayer.playback_speed",
                ScenePropertyValue::Scalar(player.playback_speed),
                true,
            );
            push_entry!(
                "AnimationSequencePlayer.time_seconds",
                ScenePropertyValue::Scalar(player.time_seconds),
                true,
            );
            push_entry!(
                "AnimationSequencePlayer.looping",
                ScenePropertyValue::Bool(player.looping),
                false,
            );
            push_entry!(
                "AnimationSequencePlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
        }
        if let Some(player) = self.animation_graph_players.get(&entity) {
            push_entry!(
                "AnimationGraphPlayer.graph",
                ScenePropertyValue::Resource(player.graph.id().to_string()),
                false,
            );
            push_entry!(
                "AnimationGraphPlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
            for (key, value) in &player.parameters {
                push_entry!(
                    &format!("AnimationGraphPlayer.parameters.{key}"),
                    ScenePropertyValue::AnimationParameter(value.clone()),
                    animation_parameter_is_animatable(value),
                );
            }
        }
        if let Some(player) = self.animation_state_machine_players.get(&entity) {
            push_entry!(
                "AnimationStateMachinePlayer.state_machine",
                ScenePropertyValue::Resource(player.state_machine.id().to_string()),
                false,
            );
            push_entry!(
                "AnimationStateMachinePlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
            push_entry!(
                "AnimationStateMachinePlayer.active_state",
                ScenePropertyValue::String(match &player.active_state {
                    Some(active_state) => active_state.clone(),
                    None => String::new(),
                }),
                false,
            );
            for (key, value) in &player.parameters {
                push_entry!(
                    &format!("AnimationStateMachinePlayer.parameters.{key}"),
                    ScenePropertyValue::AnimationParameter(value.clone()),
                    animation_parameter_is_animatable(value),
                );
            }
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
                    push_entry!(&format!("{component_id}.{property}"), scene_value, true);
                }
            }
        }
    }

    fn property_entry_capacity_hint(&self, entity: EntityId) -> usize {
        // The counts mirror the property groups pushed by `property_entries`.
        let mut capacity = 1;

        if self.names.contains_key(&entity) {
            capacity += 1;
        }
        if self.local_transforms.contains_key(&entity) {
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
        if self.cameras.contains_key(&entity) {
            capacity += 3;
        }
        if let Some(mesh) = self.mesh_renderers.get(&entity) {
            capacity += 10 + mesh.morph_weights.len();
            if mesh.mesh.is_some() {
                capacity += 1;
            }
        }
        if self.ambient_lights.contains_key(&entity) {
            capacity += 3;
        }
        if self.directional_lights.contains_key(&entity) {
            capacity += 3;
        }
        if self.point_lights.contains_key(&entity) {
            capacity += 3;
        }
        if self.rect_lights.contains_key(&entity) {
            capacity += 4;
        }
        if self.spot_lights.contains_key(&entity) {
            capacity += 6;
        }
        capacity += self.physics_property_entry_capacity_hint(entity);
        if self.animation_skeletons.contains_key(&entity) {
            capacity += 1;
        }
        if self.animation_players.contains_key(&entity) {
            capacity += 6;
        }
        if self.animation_sequence_players.contains_key(&entity) {
            capacity += 5;
        }
        if let Some(player) = self.animation_graph_players.get(&entity) {
            capacity += 2 + player.parameters.len();
        }
        if let Some(player) = self.animation_state_machine_players.get(&entity) {
            capacity += 3 + player.parameters.len();
        }
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

fn mesh_renderer_morph_weight_path(index: usize) -> String {
    let prefix_len = MESH_RENDERER_MORPH_WEIGHT_PATH_PREFIX.len();
    let mut path = String::with_capacity(prefix_len + decimal_digit_count(index));
    path.push_str(MESH_RENDERER_MORPH_WEIGHT_PATH_PREFIX);
    write!(&mut path, "{index}").expect("writing to a String cannot fail");
    path
}

fn decimal_digit_count(mut value: usize) -> usize {
    let mut digits = 1;
    while value >= 10 {
        value /= 10;
        digits += 1;
    }
    digits
}

fn property_path_literal_matches_normalized(
    path: &str,
    target_component: &str,
    target_segments: &[String],
) -> bool {
    let Some((component, segments)) = path.split_once('.') else {
        return false;
    };

    normalized_identifier_matches(component, target_component)
        && property_literal_segments_match_normalized(segments, target_segments)
}

fn property_literal_segments_match_normalized(segments: &str, target_segments: &[String]) -> bool {
    let mut target_index = 0;
    for segment in segments.split('.') {
        if target_index >= target_segments.len() {
            return false;
        }
        if !normalized_identifier_matches(segment, &target_segments[target_index]) {
            return false;
        }
        target_index += 1;
    }

    target_index == target_segments.len()
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

fn animation_parameter_is_animatable(value: &AnimationParameterValue) -> bool {
    matches!(
        value,
        AnimationParameterValue::Integer(_)
            | AnimationParameterValue::Scalar(_)
            | AnimationParameterValue::Vec2(_)
            | AnimationParameterValue::Vec3(_)
            | AnimationParameterValue::Vec4(_)
    )
}
