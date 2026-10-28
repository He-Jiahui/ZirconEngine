//! Read-only inspection of existing physics configuration; authoring stays with its owners.
use zircon_runtime_interface::reflect::{ReflectError, ReflectedValue};

use super::super::{ColliderComponent, ColliderShape, JointComponent, JointKind};

pub(in crate::scene::components::scene) fn read_shape(
    component: &ColliderComponent,
) -> Result<ReflectedValue, ReflectError> {
    // Display the discriminant without copying potentially unbounded mesh/compound payloads.
    Ok(ReflectedValue::Enum(
        match &component.shape {
            ColliderShape::Box { .. } => "Box",
            ColliderShape::Sphere { .. } => "Sphere",
            ColliderShape::Capsule { .. } => "Capsule",
            ColliderShape::Cylinder { .. } => "Cylinder",
            ColliderShape::ConvexHull { .. } => "ConvexHull",
            ColliderShape::TriangleMesh { .. } => "TriangleMesh",
            ColliderShape::HeightField { .. } => "HeightField",
            ColliderShape::Compound { .. } => "Compound",
        }
        .to_string(),
    ))
}

pub(in crate::scene::components::scene) fn read_material(
    component: &ColliderComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(component.material.map_or(ReflectedValue::Null, |material| {
        ReflectedValue::Resource(material.id().to_string())
    }))
}

pub(in crate::scene::components::scene) fn read_joint_type(
    component: &JointComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(ReflectedValue::Enum(
        match component.joint_type {
            JointKind::Fixed => "Fixed",
            JointKind::Distance => "Distance",
            JointKind::Hinge => "Hinge",
            JointKind::Slider => "Slider",
            JointKind::ConeTwist => "ConeTwist",
            JointKind::Generic6Dof => "Generic6Dof",
        }
        .to_string(),
    ))
}

pub(in crate::scene::components::scene) fn read_limits(
    component: &JointComponent,
) -> Result<ReflectedValue, ReflectError> {
    Ok(component
        .limits
        .map_or(ReflectedValue::Null, ReflectedValue::Vec2))
}
