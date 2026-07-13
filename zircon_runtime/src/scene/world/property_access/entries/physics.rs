use crate::core::framework::scene::physics::{
    PhysicsCcdMode, PhysicsMassProperties, PhysicsSleepPolicy,
};
use crate::core::framework::scene::ScenePropertyValue;
use crate::scene::components::{JointKind, RigidBodyType};
use crate::scene::EntityId;

#[path = "collider_shape.rs"]
mod collider_shape;

use super::super::super::World;
use super::super::value_conversion::combine_rule_label;
use collider_shape::{
    collider_shape_property_entry_capacity, visit_collider_shape_property_entries,
};

impl World {
    pub(super) fn visit_physics_property_entries<F>(
        &self,
        entity: EntityId,
        visitor: &mut F,
    ) -> bool
    where
        F: FnMut(&str, ScenePropertyValue, bool) -> bool,
    {
        macro_rules! push_entry {
            ($path:expr, $value:expr, $animatable:expr $(,)?) => {
                if !visitor($path, $value, $animatable) {
                    return false;
                }
            };
        }

        if let Some(rigid_body) = self.rigid_bodies.get(&entity) {
            push_entry!(
                "RigidBody.kind",
                ScenePropertyValue::Enum(match rigid_body.body_type {
                    RigidBodyType::Static => "static".to_string(),
                    RigidBodyType::Dynamic => "dynamic".to_string(),
                    RigidBodyType::Kinematic => "kinematic".to_string(),
                }),
                false,
            );
            push_entry!(
                "RigidBody.mass",
                ScenePropertyValue::Scalar(rigid_body.mass),
                true,
            );
            push_entry!(
                "RigidBody.mass_properties.mode",
                ScenePropertyValue::Enum(match rigid_body.mass_properties {
                    PhysicsMassProperties::Explicit { .. } => "explicit".to_string(),
                    PhysicsMassProperties::AutoFromShape { .. } => {
                        "auto_from_shape".to_string()
                    }
                }),
                false,
            );
            push_entry!(
                "RigidBody.mass_properties.density",
                ScenePropertyValue::Scalar(match rigid_body.mass_properties {
                    PhysicsMassProperties::AutoFromShape { density } => density,
                    PhysicsMassProperties::Explicit { .. } => 1.0,
                }),
                true,
            );
            push_entry!(
                "RigidBody.linear_velocity",
                ScenePropertyValue::Vec3(rigid_body.linear_velocity.to_array()),
                true,
            );
            push_entry!(
                "RigidBody.angular_velocity",
                ScenePropertyValue::Vec3(rigid_body.angular_velocity.to_array()),
                true,
            );
            push_entry!(
                "RigidBody.linear_damping",
                ScenePropertyValue::Scalar(rigid_body.linear_damping),
                true,
            );
            push_entry!(
                "RigidBody.angular_damping",
                ScenePropertyValue::Scalar(rigid_body.angular_damping),
                true,
            );
            push_entry!(
                "RigidBody.gravity_scale",
                ScenePropertyValue::Scalar(rigid_body.gravity_scale),
                true,
            );
            push_entry!(
                "RigidBody.ccd_mode",
                ScenePropertyValue::Enum(match rigid_body.ccd_mode {
                    PhysicsCcdMode::Disabled => "disabled".to_string(),
                    PhysicsCcdMode::LinearCast => "linear_cast".to_string(),
                }),
                false,
            );
            push_entry!(
                "RigidBody.sleep_policy",
                ScenePropertyValue::Enum(match rigid_body.sleep_policy {
                    PhysicsSleepPolicy::Allow => "allow".to_string(),
                    PhysicsSleepPolicy::Never => "never".to_string(),
                }),
                false,
            );
            for (axis_name, axis_index) in [("x", 0usize), ("y", 1usize), ("z", 2usize)] {
                push_entry!(
                    &format!("RigidBody.lock_translation.{axis_name}"),
                    ScenePropertyValue::Bool(rigid_body.lock_translation[axis_index]),
                    false,
                );
                push_entry!(
                    &format!("RigidBody.lock_rotation.{axis_name}"),
                    ScenePropertyValue::Bool(rigid_body.lock_rotation[axis_index]),
                    false,
                );
            }
        }
        if let Some(collider) = self.colliders.get(&entity) {
            push_entry!(
                "Collider.sensor",
                ScenePropertyValue::Bool(collider.sensor),
                false,
            );
            push_entry!(
                "Collider.layer",
                ScenePropertyValue::Unsigned(collider.layer as u64),
                false,
            );
            push_entry!(
                "Collider.collision_group",
                ScenePropertyValue::Unsigned(collider.collision_group as u64),
                false,
            );
            push_entry!(
                "Collider.collision_mask",
                ScenePropertyValue::Unsigned(collider.collision_mask as u64),
                false,
            );
            if let Some(material) = collider.material {
                push_entry!(
                    "Collider.material",
                    ScenePropertyValue::Resource(material.id().to_string()),
                    false,
                );
            }
            push_entry!(
                "Collider.local_transform.translation",
                ScenePropertyValue::Vec3(collider.local_transform.translation.to_array()),
                true,
            );
            push_entry!(
                "Collider.local_transform.rotation",
                ScenePropertyValue::Quaternion(collider.local_transform.rotation.to_array()),
                true,
            );
            push_entry!(
                "Collider.local_transform.scale",
                ScenePropertyValue::Vec3(collider.local_transform.scale.to_array()),
                true,
            );
            if let Some(material_override) = collider.material_override.as_ref() {
                push_entry!(
                    "Collider.material_override.static_friction",
                    ScenePropertyValue::Scalar(material_override.static_friction),
                    true,
                );
                push_entry!(
                    "Collider.material_override.dynamic_friction",
                    ScenePropertyValue::Scalar(material_override.dynamic_friction),
                    true,
                );
                push_entry!(
                    "Collider.material_override.restitution",
                    ScenePropertyValue::Scalar(material_override.restitution),
                    true,
                );
                push_entry!(
                    "Collider.material_override.friction_combine",
                    ScenePropertyValue::Enum(
                        combine_rule_label(material_override.friction_combine).to_string(),
                    ),
                    false,
                );
                push_entry!(
                    "Collider.material_override.restitution_combine",
                    ScenePropertyValue::Enum(
                        combine_rule_label(material_override.restitution_combine).to_string(),
                    ),
                    false,
                );
            }
            if !visit_collider_shape_property_entries(&collider.shape, "Collider.shape", visitor) {
                return false;
            }
        }
        if let Some(joint) = self.joints.get(&entity) {
            push_entry!(
                "Joint.kind",
                ScenePropertyValue::Enum(match joint.joint_type {
                    JointKind::Fixed => "fixed".to_string(),
                    JointKind::Distance => "distance".to_string(),
                    JointKind::Hinge => "hinge".to_string(),
                    JointKind::Slider => "slider".to_string(),
                    JointKind::ConeTwist => "cone_twist".to_string(),
                    JointKind::Generic6Dof => "generic_6dof".to_string(),
                }),
                false,
            );
            push_entry!(
                "Joint.connected_entity",
                ScenePropertyValue::Entity(joint.connected_entity),
                false,
            );
            push_entry!(
                "Joint.anchor",
                ScenePropertyValue::Vec3(joint.anchor.to_array()),
                true,
            );
            push_entry!(
                "Joint.axis",
                ScenePropertyValue::Vec3(joint.axis.to_array()),
                true,
            );
            if let Some(limits) = joint.limits {
                push_entry!(
                    "Joint.limits.min",
                    ScenePropertyValue::Scalar(limits[0]),
                    true,
                );
                push_entry!(
                    "Joint.limits.max",
                    ScenePropertyValue::Scalar(limits[1]),
                    true,
                );
            }
            push_entry!(
                "Joint.collide_connected",
                ScenePropertyValue::Bool(joint.collide_connected),
                false,
            );
        }

        true
    }

    pub(super) fn physics_property_entry_capacity_hint(&self, entity: EntityId) -> usize {
        let mut capacity = 0;
        if self.rigid_bodies.contains_key(&entity) {
            capacity += 17;
        }
        if let Some(collider) = self.colliders.get(&entity) {
            capacity += 7;
            if collider.material.is_some() {
                capacity += 1;
            }
            if collider.material_override.is_some() {
                capacity += 5;
            }
            capacity += collider_shape_property_entry_capacity(&collider.shape);
        }
        if let Some(joint) = self.joints.get(&entity) {
            capacity += 5;
            if joint.limits.is_some() {
                capacity += 2;
            }
        }

        capacity
    }
}
