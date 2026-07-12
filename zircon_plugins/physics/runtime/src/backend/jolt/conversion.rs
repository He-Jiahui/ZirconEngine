use std::collections::HashMap;
use std::ffi::CStr;
use std::ptr;

use joltc_sys::{
    JPC_BoxShapeSettings, JPC_BoxShapeSettings_Create, JPC_CapsuleShapeSettings,
    JPC_CapsuleShapeSettings_Create, JPC_ConvexHullShapeSettings,
    JPC_ConvexHullShapeSettings_Create, JPC_CylinderShapeSettings,
    JPC_CylinderShapeSettings_Create, JPC_MotionType, JPC_Quat, JPC_RVec3, JPC_Shape,
    JPC_Shape_Release, JPC_SphereShapeSettings, JPC_SphereShapeSettings_Create,
    JPC_StaticCompoundShapeSettings, JPC_StaticCompoundShapeSettings_Create, JPC_String,
    JPC_String_c_str, JPC_String_delete, JPC_SubShapeSettings, JPC_Vec3, JPC_MOTION_TYPE_DYNAMIC,
    JPC_MOTION_TYPE_KINEMATIC, JPC_MOTION_TYPE_STATIC,
};
use zircon_runtime::core::framework::physics::{
    PhysicsBodyType, PhysicsColliderShape, PhysicsMeshAsset,
};
use zircon_runtime::core::math::{Quat, Real, Vec3};
use zircon_runtime::core::resource::AssetReference;

use crate::backend::{PhysicsBackendError, PhysicsBackendObjectKind};

use super::mesh_shape::create_mesh_shape;

pub(super) fn vec3(values: [Real; 3]) -> JPC_Vec3 {
    JPC_Vec3 {
        x: values[0],
        y: values[1],
        z: values[2],
        _w: values[2],
    }
}

pub(super) fn rvec3(value: Vec3) -> JPC_RVec3 {
    JPC_RVec3 {
        x: value.x,
        y: value.y,
        z: value.z,
        _w: value.z,
    }
}

pub(super) fn quat(value: Quat) -> JPC_Quat {
    JPC_Quat {
        x: value.x,
        y: value.y,
        z: value.z,
        w: value.w,
    }
}

pub(super) fn zircon_vec3(value: JPC_Vec3) -> [Real; 3] {
    [value.x, value.y, value.z]
}

pub(super) fn zircon_translation(value: JPC_RVec3) -> Vec3 {
    Vec3::new(value.x, value.y, value.z)
}

pub(super) fn zircon_quat(value: JPC_Quat) -> Quat {
    Quat::from_xyzw(value.x, value.y, value.z, value.w)
}

pub(super) fn motion_type(body_type: PhysicsBodyType) -> JPC_MotionType {
    match body_type {
        PhysicsBodyType::Static => JPC_MOTION_TYPE_STATIC,
        PhysicsBodyType::Kinematic => JPC_MOTION_TYPE_KINEMATIC,
        PhysicsBodyType::Dynamic => JPC_MOTION_TYPE_DYNAMIC,
    }
}

pub(super) unsafe fn create_shape(
    shape: &PhysicsColliderShape,
    mesh_assets: &HashMap<AssetReference, PhysicsMeshAsset>,
    density: Option<Real>,
) -> Result<*mut JPC_Shape, PhysicsBackendError> {
    let mut native_shape = ptr::null_mut();
    let mut native_error = ptr::null_mut();
    let created = match shape {
        PhysicsColliderShape::Box { half_extents } => JPC_BoxShapeSettings_Create(
            &JPC_BoxShapeSettings {
                HalfExtent: vec3(*half_extents),
                Density: density.unwrap_or_else(|| JPC_BoxShapeSettings::default().Density),
                ..JPC_BoxShapeSettings::default()
            },
            &mut native_shape,
            &mut native_error,
        ),
        PhysicsColliderShape::Sphere { radius } => JPC_SphereShapeSettings_Create(
            &JPC_SphereShapeSettings {
                Radius: *radius,
                Density: density.unwrap_or_else(|| JPC_SphereShapeSettings::default().Density),
                ..JPC_SphereShapeSettings::default()
            },
            &mut native_shape,
            &mut native_error,
        ),
        PhysicsColliderShape::Capsule {
            radius,
            half_height,
        } => JPC_CapsuleShapeSettings_Create(
            &JPC_CapsuleShapeSettings {
                Radius: *radius,
                HalfHeightOfCylinder: *half_height,
                Density: density.unwrap_or_else(|| JPC_CapsuleShapeSettings::default().Density),
                ..JPC_CapsuleShapeSettings::default()
            },
            &mut native_shape,
            &mut native_error,
        ),
        PhysicsColliderShape::Cylinder {
            radius,
            half_height,
        } => JPC_CylinderShapeSettings_Create(
            &JPC_CylinderShapeSettings {
                Radius: *radius,
                HalfHeight: *half_height,
                Density: density.unwrap_or_else(|| JPC_CylinderShapeSettings::default().Density),
                ..JPC_CylinderShapeSettings::default()
            },
            &mut native_shape,
            &mut native_error,
        ),
        PhysicsColliderShape::ConvexHull { points } => {
            let native_points = points.iter().copied().map(vec3).collect::<Vec<_>>();
            JPC_ConvexHullShapeSettings_Create(
                &JPC_ConvexHullShapeSettings {
                    Points: native_points.as_ptr(),
                    PointsLen: native_points.len(),
                    Density: density
                        .unwrap_or_else(|| JPC_ConvexHullShapeSettings::default().Density),
                    ..JPC_ConvexHullShapeSettings::default()
                },
                &mut native_shape,
                &mut native_error,
            )
        }
        PhysicsColliderShape::Compound { children } => {
            return create_compound_shape(children, mesh_assets, density);
        }
        PhysicsColliderShape::TriangleMesh { mesh } => {
            let asset = resolve_mesh_asset(mesh_assets, mesh, "triangle mesh")?;
            if !matches!(asset, PhysicsMeshAsset::TriangleMesh { .. }) {
                return Err(mesh_asset_kind_mismatch(mesh, "triangle mesh"));
            }
            return create_mesh_shape(asset);
        }
        PhysicsColliderShape::HeightField {
            resolution,
            heights,
        } => {
            let asset = resolve_mesh_asset(mesh_assets, heights, "height field")?;
            match asset {
                PhysicsMeshAsset::HeightField {
                    resolution: asset_resolution,
                    ..
                } if asset_resolution == resolution => return create_mesh_shape(asset),
                PhysicsMeshAsset::HeightField { .. } => {
                    return Err(PhysicsBackendError::InvalidDescriptor {
                        kind: PhysicsBackendObjectKind::Shape,
                        detail: format!(
                            "height-field collider resolution does not match registered asset {heights}"
                        ),
                    });
                }
                PhysicsMeshAsset::TriangleMesh { .. } => {
                    return Err(mesh_asset_kind_mismatch(heights, "height field"));
                }
            }
        }
    };
    if created && !native_shape.is_null() {
        return Ok(native_shape);
    }

    let detail = take_native_error(native_error);
    Err(PhysicsBackendError::InvalidDescriptor {
        kind: PhysicsBackendObjectKind::Shape,
        detail,
    })
}

unsafe fn create_compound_shape(
    children: &[(
        zircon_runtime::core::math::Transform,
        Box<PhysicsColliderShape>,
    )],
    mesh_assets: &HashMap<AssetReference, PhysicsMeshAsset>,
    density: Option<Real>,
) -> Result<*mut JPC_Shape, PhysicsBackendError> {
    let mut owned_shapes = Vec::with_capacity(children.len());
    let mut sub_shapes = Vec::with_capacity(children.len());
    for (transform, child) in children {
        let child_shape = match create_shape(child, mesh_assets, density) {
            Ok(shape) => shape,
            Err(error) => {
                release_shapes(owned_shapes);
                return Err(error);
            }
        };
        owned_shapes.push(child_shape);
        sub_shapes.push(JPC_SubShapeSettings {
            Shape: child_shape,
            Position: vec3(transform.translation.to_array()),
            Rotation: quat(transform.rotation),
            ..JPC_SubShapeSettings::default()
        });
    }
    let mut native_shape = ptr::null_mut();
    let mut native_error = ptr::null_mut();
    let created = JPC_StaticCompoundShapeSettings_Create(
        &JPC_StaticCompoundShapeSettings {
            SubShapes: sub_shapes.as_ptr(),
            SubShapesLen: sub_shapes.len(),
            ..JPC_StaticCompoundShapeSettings::default()
        },
        &mut native_shape,
        &mut native_error,
    );
    release_shapes(owned_shapes);
    if created && !native_shape.is_null() {
        Ok(native_shape)
    } else {
        Err(PhysicsBackendError::InvalidDescriptor {
            kind: PhysicsBackendObjectKind::Shape,
            detail: take_native_error(native_error),
        })
    }
}

fn resolve_mesh_asset<'a>(
    mesh_assets: &'a HashMap<AssetReference, PhysicsMeshAsset>,
    reference: &AssetReference,
    expected_kind: &str,
) -> Result<&'a PhysicsMeshAsset, PhysicsBackendError> {
    mesh_assets
        .get(reference)
        .ok_or_else(|| PhysicsBackendError::InvalidDescriptor {
            kind: PhysicsBackendObjectKind::Shape,
            detail: format!(
                "{expected_kind} collider references unregistered physics mesh asset {reference}"
            ),
        })
}

fn mesh_asset_kind_mismatch(
    reference: &AssetReference,
    expected_kind: &str,
) -> PhysicsBackendError {
    PhysicsBackendError::InvalidDescriptor {
        kind: PhysicsBackendObjectKind::Shape,
        detail: format!(
            "physics mesh asset {reference} does not contain the expected {expected_kind} payload"
        ),
    }
}

unsafe fn release_shapes(shapes: Vec<*mut JPC_Shape>) {
    for shape in shapes {
        JPC_Shape_Release(shape);
    }
}

unsafe fn take_native_error(error: *mut JPC_String) -> String {
    if error.is_null() {
        return "JoltC rejected shape creation without an error message".to_string();
    }
    let message = JPC_String_c_str(error);
    let detail = if message.is_null() {
        "JoltC rejected shape creation with an empty error".to_string()
    } else {
        CStr::from_ptr(message).to_string_lossy().into_owned()
    };
    JPC_String_delete(error);
    detail
}
