use crate::core::framework::animation::AnimationParameterValue;
use crate::core::framework::scene::{
    ComponentPropertyPath, ScenePropertyEntry, ScenePropertyValue,
};
use crate::scene::components::{ColliderShape, JointKind, Mobility, RigidBodyType};
use crate::scene::EntityId;

use super::super::World;
use super::value_conversion::combine_rule_label;

impl World {
    pub(super) fn property_entries(&self, entity: EntityId) -> Vec<ScenePropertyEntry> {
        if !self.contains_entity(entity) {
            return Vec::new();
        }

        let mut entries = Vec::new();
        let mut push = |path: &str, value: ScenePropertyValue, animatable: bool| {
            entries.push(ScenePropertyEntry::new(
                ComponentPropertyPath::parse(path).expect("valid property path"),
                value,
                animatable,
            ));
        };

        if let Some(name) = self.names.get(&entity) {
            push(
                "Name.value",
                ScenePropertyValue::String(name.0.clone()),
                false,
            );
        }
        if self.contains_entity(entity) {
            push(
                "Hierarchy.parent",
                ScenePropertyValue::Entity(self.parent_of(entity)),
                false,
            );
        }
        if let Some(local) = self.local_transforms.get(&entity).copied() {
            push(
                "Transform.translation",
                ScenePropertyValue::Vec3(local.transform.translation.to_array()),
                true,
            );
            push(
                "Transform.rotation",
                ScenePropertyValue::Quaternion(local.transform.rotation.to_array()),
                true,
            );
            push(
                "Transform.scale",
                ScenePropertyValue::Vec3(local.transform.scale.to_array()),
                true,
            );
        }
        if let Some(active) = self.active_self(entity) {
            push("Active.enabled", ScenePropertyValue::Bool(active), false);
        }
        if let Some(mask) = self.render_layer_mask(entity) {
            push(
                "RenderLayer.mask",
                ScenePropertyValue::Unsigned(mask as u64),
                false,
            );
        }
        if let Some(mobility) = self.mobility(entity) {
            push(
                "Mobility.kind",
                ScenePropertyValue::Enum(match mobility {
                    Mobility::Dynamic => "dynamic".to_string(),
                    Mobility::Static => "static".to_string(),
                }),
                false,
            );
        }
        if let Some(camera) = self.cameras.get(&entity) {
            push(
                "Camera.fov_y_radians",
                ScenePropertyValue::Scalar(camera.fov_y_radians),
                true,
            );
            push(
                "Camera.z_near",
                ScenePropertyValue::Scalar(camera.z_near),
                true,
            );
            push(
                "Camera.z_far",
                ScenePropertyValue::Scalar(camera.z_far),
                true,
            );
        }
        if let Some(mesh) = self.mesh_renderers.get(&entity) {
            push(
                "MeshRenderer.model",
                ScenePropertyValue::Resource(mesh.model.id().to_string()),
                false,
            );
            push(
                "MeshRenderer.material",
                ScenePropertyValue::Resource(mesh.material.id().to_string()),
                false,
            );
            push(
                "MeshRenderer.tint",
                ScenePropertyValue::Vec4(mesh.tint.to_array()),
                true,
            );
        }
        if let Some(light) = self.directional_lights.get(&entity) {
            push(
                "DirectionalLight.direction",
                ScenePropertyValue::Vec3(light.direction.to_array()),
                true,
            );
            push(
                "DirectionalLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push(
                "DirectionalLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
        }
        if let Some(light) = self.point_lights.get(&entity) {
            push(
                "PointLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push(
                "PointLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push(
                "PointLight.range",
                ScenePropertyValue::Scalar(light.range),
                true,
            );
        }
        if let Some(light) = self.spot_lights.get(&entity) {
            push(
                "SpotLight.direction",
                ScenePropertyValue::Vec3(light.direction.to_array()),
                true,
            );
            push(
                "SpotLight.color",
                ScenePropertyValue::Vec3(light.color.to_array()),
                true,
            );
            push(
                "SpotLight.intensity",
                ScenePropertyValue::Scalar(light.intensity),
                true,
            );
            push(
                "SpotLight.range",
                ScenePropertyValue::Scalar(light.range),
                true,
            );
            push(
                "SpotLight.inner_angle_radians",
                ScenePropertyValue::Scalar(light.inner_angle_radians),
                true,
            );
            push(
                "SpotLight.outer_angle_radians",
                ScenePropertyValue::Scalar(light.outer_angle_radians),
                true,
            );
        }
        if let Some(rigid_body) = self.rigid_bodies.get(&entity) {
            push(
                "RigidBody.kind",
                ScenePropertyValue::Enum(match rigid_body.body_type {
                    RigidBodyType::Static => "static".to_string(),
                    RigidBodyType::Dynamic => "dynamic".to_string(),
                    RigidBodyType::Kinematic => "kinematic".to_string(),
                }),
                false,
            );
            push(
                "RigidBody.mass",
                ScenePropertyValue::Scalar(rigid_body.mass),
                true,
            );
            push(
                "RigidBody.linear_damping",
                ScenePropertyValue::Scalar(rigid_body.linear_damping),
                true,
            );
            push(
                "RigidBody.angular_damping",
                ScenePropertyValue::Scalar(rigid_body.angular_damping),
                true,
            );
            push(
                "RigidBody.gravity_scale",
                ScenePropertyValue::Scalar(rigid_body.gravity_scale),
                true,
            );
            push(
                "RigidBody.can_sleep",
                ScenePropertyValue::Bool(rigid_body.can_sleep),
                false,
            );
            for (axis_name, axis_index) in [("x", 0usize), ("y", 1usize), ("z", 2usize)] {
                push(
                    &format!("RigidBody.lock_translation.{axis_name}"),
                    ScenePropertyValue::Bool(rigid_body.lock_translation[axis_index]),
                    false,
                );
                push(
                    &format!("RigidBody.lock_rotation.{axis_name}"),
                    ScenePropertyValue::Bool(rigid_body.lock_rotation[axis_index]),
                    false,
                );
            }
        }
        if let Some(collider) = self.colliders.get(&entity) {
            push(
                "Collider.sensor",
                ScenePropertyValue::Bool(collider.sensor),
                false,
            );
            push(
                "Collider.layer",
                ScenePropertyValue::Unsigned(collider.layer as u64),
                false,
            );
            push(
                "Collider.collision_group",
                ScenePropertyValue::Unsigned(collider.collision_group as u64),
                false,
            );
            push(
                "Collider.collision_mask",
                ScenePropertyValue::Unsigned(collider.collision_mask as u64),
                false,
            );
            if let Some(material) = collider.material {
                push(
                    "Collider.material",
                    ScenePropertyValue::Resource(material.id().to_string()),
                    false,
                );
            }
            push(
                "Collider.local_transform.translation",
                ScenePropertyValue::Vec3(collider.local_transform.translation.to_array()),
                true,
            );
            push(
                "Collider.local_transform.rotation",
                ScenePropertyValue::Quaternion(collider.local_transform.rotation.to_array()),
                true,
            );
            push(
                "Collider.local_transform.scale",
                ScenePropertyValue::Vec3(collider.local_transform.scale.to_array()),
                true,
            );
            if let Some(material_override) = collider.material_override.as_ref() {
                push(
                    "Collider.material_override.static_friction",
                    ScenePropertyValue::Scalar(material_override.static_friction),
                    true,
                );
                push(
                    "Collider.material_override.dynamic_friction",
                    ScenePropertyValue::Scalar(material_override.dynamic_friction),
                    true,
                );
                push(
                    "Collider.material_override.restitution",
                    ScenePropertyValue::Scalar(material_override.restitution),
                    true,
                );
                push(
                    "Collider.material_override.friction_combine",
                    ScenePropertyValue::Enum(
                        combine_rule_label(material_override.friction_combine).to_string(),
                    ),
                    false,
                );
                push(
                    "Collider.material_override.restitution_combine",
                    ScenePropertyValue::Enum(
                        combine_rule_label(material_override.restitution_combine).to_string(),
                    ),
                    false,
                );
            }
            match &collider.shape {
                ColliderShape::Box { half_extents } => {
                    push(
                        "Collider.shape.kind",
                        ScenePropertyValue::Enum("box".to_string()),
                        false,
                    );
                    push(
                        "Collider.shape.half_extents",
                        ScenePropertyValue::Vec3(half_extents.to_array()),
                        true,
                    );
                }
                ColliderShape::Sphere { radius } => {
                    push(
                        "Collider.shape.kind",
                        ScenePropertyValue::Enum("sphere".to_string()),
                        false,
                    );
                    push(
                        "Collider.shape.radius",
                        ScenePropertyValue::Scalar(*radius),
                        true,
                    );
                }
                ColliderShape::Capsule {
                    radius,
                    half_height,
                } => {
                    push(
                        "Collider.shape.kind",
                        ScenePropertyValue::Enum("capsule".to_string()),
                        false,
                    );
                    push(
                        "Collider.shape.radius",
                        ScenePropertyValue::Scalar(*radius),
                        true,
                    );
                    push(
                        "Collider.shape.half_height",
                        ScenePropertyValue::Scalar(*half_height),
                        true,
                    );
                }
            }
        }
        if let Some(joint) = self.joints.get(&entity) {
            push(
                "Joint.kind",
                ScenePropertyValue::Enum(match joint.joint_type {
                    JointKind::Fixed => "fixed".to_string(),
                    JointKind::Distance => "distance".to_string(),
                    JointKind::Hinge => "hinge".to_string(),
                }),
                false,
            );
            push(
                "Joint.connected_entity",
                ScenePropertyValue::Entity(joint.connected_entity),
                false,
            );
            push(
                "Joint.anchor",
                ScenePropertyValue::Vec3(joint.anchor.to_array()),
                true,
            );
            push(
                "Joint.axis",
                ScenePropertyValue::Vec3(joint.axis.to_array()),
                true,
            );
            if let Some(limits) = joint.limits {
                push(
                    "Joint.limits.min",
                    ScenePropertyValue::Scalar(limits[0]),
                    true,
                );
                push(
                    "Joint.limits.max",
                    ScenePropertyValue::Scalar(limits[1]),
                    true,
                );
            }
            push(
                "Joint.collide_connected",
                ScenePropertyValue::Bool(joint.collide_connected),
                false,
            );
        }
        if let Some(skeleton) = self.animation_skeletons.get(&entity) {
            push(
                "AnimationSkeleton.skeleton",
                ScenePropertyValue::Resource(skeleton.skeleton.id().to_string()),
                false,
            );
        }
        if let Some(player) = self.animation_players.get(&entity) {
            push(
                "AnimationPlayer.clip",
                ScenePropertyValue::Resource(player.clip.id().to_string()),
                false,
            );
            push(
                "AnimationPlayer.playback_speed",
                ScenePropertyValue::Scalar(player.playback_speed),
                true,
            );
            push(
                "AnimationPlayer.time_seconds",
                ScenePropertyValue::Scalar(player.time_seconds),
                true,
            );
            push(
                "AnimationPlayer.weight",
                ScenePropertyValue::Scalar(player.weight),
                true,
            );
            push(
                "AnimationPlayer.looping",
                ScenePropertyValue::Bool(player.looping),
                false,
            );
            push(
                "AnimationPlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
        }
        if let Some(player) = self.animation_sequence_players.get(&entity) {
            push(
                "AnimationSequencePlayer.sequence",
                ScenePropertyValue::Resource(player.sequence.id().to_string()),
                false,
            );
            push(
                "AnimationSequencePlayer.playback_speed",
                ScenePropertyValue::Scalar(player.playback_speed),
                true,
            );
            push(
                "AnimationSequencePlayer.time_seconds",
                ScenePropertyValue::Scalar(player.time_seconds),
                true,
            );
            push(
                "AnimationSequencePlayer.looping",
                ScenePropertyValue::Bool(player.looping),
                false,
            );
            push(
                "AnimationSequencePlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
        }
        if let Some(player) = self.animation_graph_players.get(&entity) {
            push(
                "AnimationGraphPlayer.graph",
                ScenePropertyValue::Resource(player.graph.id().to_string()),
                false,
            );
            push(
                "AnimationGraphPlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
            for (key, value) in &player.parameters {
                push(
                    &format!("AnimationGraphPlayer.parameters.{key}"),
                    ScenePropertyValue::AnimationParameter(value.clone()),
                    animation_parameter_is_animatable(value),
                );
            }
        }
        if let Some(player) = self.animation_state_machine_players.get(&entity) {
            push(
                "AnimationStateMachinePlayer.state_machine",
                ScenePropertyValue::Resource(player.state_machine.id().to_string()),
                false,
            );
            push(
                "AnimationStateMachinePlayer.playing",
                ScenePropertyValue::Bool(player.playing),
                false,
            );
            push(
                "AnimationStateMachinePlayer.active_state",
                ScenePropertyValue::String(player.active_state.clone().unwrap_or_default()),
                false,
            );
            for (key, value) in &player.parameters {
                push(
                    &format!("AnimationStateMachinePlayer.parameters.{key}"),
                    ScenePropertyValue::AnimationParameter(value.clone()),
                    animation_parameter_is_animatable(value),
                );
            }
        }

        entries
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
