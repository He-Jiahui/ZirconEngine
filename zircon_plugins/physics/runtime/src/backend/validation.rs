//! 提供 builtin 与 Jolt 共用的描述符粗校验；provider 仍须检查自身资源注册、几何支持和原生接口约束。

use zircon_runtime::core::framework::{
    physics::PhysicsColliderShape,
    scene::physics::{PhysicsMassProperties, PhysicsMaterialMetadata},
};

use super::BodyDesc;

// body 与 collider 必须属于同一实体；该门同时检查已同步数值，但自动质量只检查输入密度而未检查派生结果。
pub(super) fn body_desc_is_valid(desc: &BodyDesc) -> bool {
    let body = &desc.body;
    body.entity == desc.collider.entity
        && body.mass_properties.is_valid()
        && (matches!(
            body.mass_properties,
            PhysicsMassProperties::AutoFromShape { .. }
        ) || (body.mass.is_finite() && body.mass > 0.0))
        && body.linear_velocity.iter().all(|value| value.is_finite())
        && body.angular_velocity.iter().all(|value| value.is_finite())
        && body.linear_damping.is_finite()
        && body.angular_damping.is_finite()
        && body.gravity_scale.is_finite()
        && body.transform.translation.is_finite()
        && body.transform.rotation.is_finite()
        && body.transform.scale.is_finite()
}

// 该函数只做共享结构和有限值检查；网格数据、provider 支持范围及派生几何仍由各 backend 决定。
pub(super) fn shape_is_valid(shape: &PhysicsColliderShape) -> bool {
    match shape {
        PhysicsColliderShape::Box { half_extents } => half_extents
            .iter()
            .all(|extent| extent.is_finite() && *extent >= 0.0),
        PhysicsColliderShape::Sphere { radius } => radius.is_finite() && *radius > 0.0,
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => radius.is_finite() && *radius > 0.0 && half_height.is_finite() && *half_height >= 0.0,
        PhysicsColliderShape::Cylinder {
            radius,
            half_height,
        } => radius.is_finite() && *radius > 0.0 && half_height.is_finite() && *half_height > 0.0,
        PhysicsColliderShape::ConvexHull { points } => {
            points.len() >= 4
                && points
                    .iter()
                    .flatten()
                    .all(|coordinate| coordinate.is_finite())
        }
        PhysicsColliderShape::TriangleMesh { .. } => true,
        PhysicsColliderShape::HeightField { resolution, .. } => {
            resolution[0] >= 2 && resolution[1] >= 2
        }
        PhysicsColliderShape::Compound { children } => {
            !children.is_empty()
                && children.iter().all(|(transform, child)| {
                    transform.translation.is_finite()
                        && transform.rotation.is_finite()
                        && transform.scale.is_finite()
                        && transform.scale.to_array() == [1.0; 3]
                        && shape_is_valid(child)
                })
        }
    }
}

// TODO: [CR-PHYSICS-BACKEND-0003] 共享层只要求材质值有限；Jolt 另拒绝负摩擦并将恢复系数限于 [0,1]，Builtin 接受相同有限输入。
// 需要由材质合同确定共同范围或明确 provider 差异，不能仅凭 native 习惯统一数值。证据：builtin/runtime.rs::create_shape、jolt/runtime.rs::create_shape；关联 PH-P1-019 / PHY4-P1-010。
pub(super) fn material_is_valid(material: &PhysicsMaterialMetadata) -> bool {
    material.static_friction.is_finite()
        && material.dynamic_friction.is_finite()
        && material.restitution.is_finite()
}
