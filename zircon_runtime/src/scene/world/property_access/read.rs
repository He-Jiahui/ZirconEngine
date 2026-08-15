use crate::core::framework::scene::{ComponentPropertyPath, ScenePropertyValue};
use crate::scene::components::{
    AmbientLight, AnimationGraphPlayerComponent, AnimationPlayerComponent,
    AnimationSequencePlayerComponent, AnimationSkeletonComponent,
    AnimationStateMachinePlayerComponent, CameraComponent, DirectionalLight, LocalTransform,
    MeshRenderer, Mobility, Name, PointLight, RectLight, SpotLight,
};
use crate::scene::{EntityId, SceneError, SceneResult};

use super::super::World;
use super::value_conversion::normalized_identifier_matches;

macro_rules! direct_property_field {
    ($segments:expr, { $($field:expr => $value:expr),+ $(,)? }) => {{
        match $segments {
            [target] => {
                $(
                    if normalized_identifier_matches(target, $field) {
                        Some($value)
                    } else
                )+
                {
                    None
                }
            }
            _ => None,
        }
    }};
}

impl World {
    pub fn property(
        &self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<ScenePropertyValue> {
        self.property_impl(entity, property_path)
    }

    fn property_impl(
        &self,
        entity: EntityId,
        property_path: &ComponentPropertyPath,
    ) -> SceneResult<ScenePropertyValue> {
        if !self.contains_entity(entity) {
            return Err(SceneError::PropertyUnavailable {
                entity,
                property_path: property_path.to_string(),
            });
        }

        if let Some(value) = self.static_property_value(
            entity,
            property_path.component(),
            property_path.property_segments(),
        ) {
            return Ok(value);
        }

        if let Some(value) = self.dynamic_component_property(entity, property_path) {
            return Ok(value);
        }

        Err(SceneError::PropertyUnavailable {
            entity,
            property_path: property_path.to_string(),
        })
    }

    // Inspector enumeration is intentionally not a fallback here. A targeted property
    // read dispatches to one component and one field, materializing only that field.
    fn static_property_value(
        &self,
        entity: EntityId,
        component: &str,
        segments: &[String],
    ) -> Option<ScenePropertyValue> {
        let value = if normalized_identifier_matches(component, "Name") {
            let name = self.get::<Name>(entity)?;
            property_segments_match(segments, &["value"])
                .then(|| ScenePropertyValue::String(name.0.clone()))
        } else if normalized_identifier_matches(component, "Hierarchy") {
            property_segments_match(segments, &["parent"])
                .then(|| ScenePropertyValue::Entity(self.parent_of(entity)))
        } else if normalized_identifier_matches(component, "Transform") {
            let local = self.get::<LocalTransform>(entity)?;
            direct_property_field!(segments, {
                "translation" => ScenePropertyValue::Vec3(local.transform.translation.to_array()),
                "rotation" => ScenePropertyValue::Quaternion(local.transform.rotation.to_array()),
                "scale" => ScenePropertyValue::Vec3(local.transform.scale.to_array()),
            })
        } else if normalized_identifier_matches(component, "Active") {
            property_segments_match(segments, &["enabled"])
                .then(|| self.active_self(entity).map(ScenePropertyValue::Bool))
                .flatten()
        } else if normalized_identifier_matches(component, "RenderLayer") {
            property_segments_match(segments, &["mask"])
                .then(|| {
                    self.render_layer_mask(entity)
                        .map(|mask| ScenePropertyValue::Unsigned(mask as u64))
                })
                .flatten()
        } else if normalized_identifier_matches(component, "Mobility") {
            property_segments_match(segments, &["kind"])
                .then(|| {
                    self.mobility(entity).map(|mobility| {
                        ScenePropertyValue::Enum(match mobility {
                            Mobility::Dynamic => "dynamic".to_string(),
                            Mobility::Static => "static".to_string(),
                        })
                    })
                })
                .flatten()
        } else if normalized_identifier_matches(component, "Camera") {
            let camera = self.get::<CameraComponent>(entity)?;
            direct_property_field!(segments, {
                "fov_y_radians" => ScenePropertyValue::Scalar(camera.fov_y_radians),
                "z_near" => ScenePropertyValue::Scalar(camera.z_near),
                "z_far" => ScenePropertyValue::Scalar(camera.z_far),
            })
        } else if normalized_identifier_matches(component, "MeshRenderer") {
            self.mesh_renderer_property_value(entity, segments)
        } else if normalized_identifier_matches(component, "AmbientLight") {
            let light = self.get::<AmbientLight>(entity)?;
            direct_property_field!(segments, {
                "color" => ScenePropertyValue::Vec3(light.color.to_array()),
                "intensity" => ScenePropertyValue::Scalar(light.intensity),
                "affects_lightmapped_meshes" => ScenePropertyValue::Bool(light.affects_lightmapped_meshes),
            })
        } else if normalized_identifier_matches(component, "DirectionalLight") {
            let light = self.get::<DirectionalLight>(entity)?;
            direct_property_field!(segments, {
                "direction" => ScenePropertyValue::Vec3(light.direction.to_array()),
                "color" => ScenePropertyValue::Vec3(light.color.to_array()),
                "intensity" => ScenePropertyValue::Scalar(light.intensity),
            })
        } else if normalized_identifier_matches(component, "PointLight") {
            let light = self.get::<PointLight>(entity)?;
            direct_property_field!(segments, {
                "color" => ScenePropertyValue::Vec3(light.color.to_array()),
                "intensity" => ScenePropertyValue::Scalar(light.intensity),
                "range" => ScenePropertyValue::Scalar(light.range),
            })
        } else if normalized_identifier_matches(component, "RectLight") {
            let light = self.get::<RectLight>(entity)?;
            direct_property_field!(segments, {
                "color" => ScenePropertyValue::Vec3(light.color.to_array()),
                "intensity" => ScenePropertyValue::Scalar(light.intensity),
                "range" => ScenePropertyValue::Scalar(light.range),
                "size" => ScenePropertyValue::Vec2(light.size.to_array()),
            })
        } else if normalized_identifier_matches(component, "SpotLight") {
            let light = self.get::<SpotLight>(entity)?;
            direct_property_field!(segments, {
                "direction" => ScenePropertyValue::Vec3(light.direction.to_array()),
                "color" => ScenePropertyValue::Vec3(light.color.to_array()),
                "intensity" => ScenePropertyValue::Scalar(light.intensity),
                "range" => ScenePropertyValue::Scalar(light.range),
                "inner_angle_radians" => ScenePropertyValue::Scalar(light.inner_angle_radians),
                "outer_angle_radians" => ScenePropertyValue::Scalar(light.outer_angle_radians),
            })
        } else if let Some(value) = self.physics_property_value(entity, component, segments) {
            Some(value)
        } else if normalized_identifier_matches(component, "AnimationSkeleton") {
            let skeleton = self.get::<AnimationSkeletonComponent>(entity)?;
            property_segments_match(segments, &["skeleton"])
                .then(|| ScenePropertyValue::Resource(skeleton.skeleton.id().to_string()))
        } else if normalized_identifier_matches(component, "AnimationPlayer") {
            let player = self.get::<AnimationPlayerComponent>(entity)?;
            self.animation_player_property_value(
                segments,
                "clip",
                player.clip.id().to_string(),
                player.playback_speed,
                player.time_seconds,
                Some(player.weight),
                player.looping,
                player.playing,
            )
        } else if normalized_identifier_matches(component, "AnimationSequencePlayer") {
            let player = self.get::<AnimationSequencePlayerComponent>(entity)?;
            self.animation_player_property_value(
                segments,
                "sequence",
                player.sequence.id().to_string(),
                player.playback_speed,
                player.time_seconds,
                None,
                player.looping,
                player.playing,
            )
        } else if normalized_identifier_matches(component, "AnimationGraphPlayer") {
            let player = self.get::<AnimationGraphPlayerComponent>(entity)?;
            self.animation_graph_property_value(
                segments,
                "graph",
                player.graph.id().to_string(),
                player.playing,
                &player.parameters,
            )
        } else if normalized_identifier_matches(component, "AnimationStateMachinePlayer") {
            let player = self.get::<AnimationStateMachinePlayerComponent>(entity)?;
            self.animation_state_machine_property_value(segments, player)
        } else {
            None
        };

        if value.is_some() {
            self.record_scene_property_entry_visit();
        }
        value
    }

    fn mesh_renderer_property_value(
        &self,
        entity: EntityId,
        segments: &[String],
    ) -> Option<ScenePropertyValue> {
        let mesh = self.get::<MeshRenderer>(entity)?;
        direct_property_field!(segments, {
            "model" => ScenePropertyValue::Resource(mesh.model.id().to_string()),
            "material" => ScenePropertyValue::Resource(mesh.material.id().to_string()),
            "render_queue" => ScenePropertyValue::Integer(mesh.render_queue.into()),
            "material_queue" => ScenePropertyValue::Integer(mesh.material_queue.into()),
            "order_in_layer" => ScenePropertyValue::Integer(mesh.order_in_layer.into()),
            "depth_bias" => ScenePropertyValue::Scalar(mesh.depth_bias),
            "primitive_binding_count" => ScenePropertyValue::Unsigned(mesh.primitives.len() as u64),
            "lod_level_count" => ScenePropertyValue::Unsigned(mesh.lods.len() as u64),
            "morph_weight_count" => ScenePropertyValue::Unsigned(mesh.morph_weights.len() as u64),
            "tint" => ScenePropertyValue::Vec4(mesh.tint.to_array()),
        })
        .or_else(|| {
            property_segments_match(segments, &["mesh"])
                .then(|| {
                    mesh.mesh
                        .map(|handle| ScenePropertyValue::Resource(handle.id().to_string()))
                })
                .flatten()
        })
        .or_else(|| {
            let [field, index] = segments else {
                return None;
            };
            if !normalized_identifier_matches(field, "morph_weights") {
                return None;
            }
            let index = index.parse::<usize>().ok()?;
            mesh.morph_weights
                .get(index)
                .copied()
                .map(ScenePropertyValue::Scalar)
        })
    }

    fn animation_player_property_value(
        &self,
        segments: &[String],
        resource_field: &str,
        resource: String,
        playback_speed: crate::core::math::Real,
        time_seconds: crate::core::math::Real,
        weight: Option<crate::core::math::Real>,
        looping: bool,
        playing: bool,
    ) -> Option<ScenePropertyValue> {
        direct_property_field!(segments, {
            resource_field => ScenePropertyValue::Resource(resource),
            "playback_speed" => ScenePropertyValue::Scalar(playback_speed),
            "time_seconds" => ScenePropertyValue::Scalar(time_seconds),
            "looping" => ScenePropertyValue::Bool(looping),
            "playing" => ScenePropertyValue::Bool(playing),
        })
        .or_else(|| {
            property_segments_match(segments, &["weight"])
                .then(|| weight.map(ScenePropertyValue::Scalar))
                .flatten()
        })
    }

    fn animation_graph_property_value(
        &self,
        segments: &[String],
        resource_field: &str,
        resource: String,
        playing: bool,
        parameters: &std::collections::BTreeMap<
            String,
            crate::core::framework::animation::AnimationParameterValue,
        >,
    ) -> Option<ScenePropertyValue> {
        direct_property_field!(segments, {
            resource_field => ScenePropertyValue::Resource(resource),
            "playing" => ScenePropertyValue::Bool(playing),
        })
        .or_else(|| animation_parameter_property_value(segments, parameters))
    }

    fn animation_state_machine_property_value(
        &self,
        segments: &[String],
        player: &AnimationStateMachinePlayerComponent,
    ) -> Option<ScenePropertyValue> {
        direct_property_field!(segments, {
            "state_machine" => ScenePropertyValue::Resource(player.state_machine.id().to_string()),
            "playing" => ScenePropertyValue::Bool(player.playing),
            "active_state" => ScenePropertyValue::String(player.active_state.clone().unwrap_or_default()),
        })
        .or_else(|| animation_parameter_property_value(segments, &player.parameters))
    }
}

fn property_segments_match(segments: &[String], expected: &[&str]) -> bool {
    segments.len() == expected.len()
        && segments
            .iter()
            .zip(expected)
            .all(|(actual, expected)| normalized_identifier_matches(actual, expected))
}

fn animation_parameter_property_value(
    segments: &[String],
    parameters: &std::collections::BTreeMap<
        String,
        crate::core::framework::animation::AnimationParameterValue,
    >,
) -> Option<ScenePropertyValue> {
    let [group, target] = segments else {
        return None;
    };
    if !normalized_identifier_matches(group, "parameters") {
        return None;
    }
    parameters
        .iter()
        .find(|(key, _)| normalized_identifier_matches(key, target))
        .map(|(_, value)| ScenePropertyValue::AnimationParameter(value.clone()))
}
