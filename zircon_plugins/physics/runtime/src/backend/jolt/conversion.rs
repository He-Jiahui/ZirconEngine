use std::ffi::CStr;
use std::ptr;

use joltc_sys::{
    JPC_BoxShapeSettings, JPC_BoxShapeSettings_Create, JPC_CapsuleShapeSettings,
    JPC_CapsuleShapeSettings_Create, JPC_MotionType, JPC_Quat, JPC_RVec3, JPC_Shape,
    JPC_SphereShapeSettings, JPC_SphereShapeSettings_Create, JPC_String, JPC_String_c_str,
    JPC_String_delete, JPC_Vec3, JPC_MOTION_TYPE_DYNAMIC, JPC_MOTION_TYPE_KINEMATIC,
    JPC_MOTION_TYPE_STATIC,
};
use zircon_runtime::core::framework::physics::{PhysicsBodyType, PhysicsColliderShape};
use zircon_runtime::core::math::{Quat, Real, Vec3};

use crate::backend::{PhysicsBackendError, PhysicsBackendObjectKind};

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
) -> Result<*mut JPC_Shape, PhysicsBackendError> {
    let mut native_shape = ptr::null_mut();
    let mut native_error = ptr::null_mut();
    let created = match shape {
        PhysicsColliderShape::Box { half_extents } => JPC_BoxShapeSettings_Create(
            &JPC_BoxShapeSettings {
                HalfExtent: vec3(*half_extents),
                ..JPC_BoxShapeSettings::default()
            },
            &mut native_shape,
            &mut native_error,
        ),
        PhysicsColliderShape::Sphere { radius } => JPC_SphereShapeSettings_Create(
            &JPC_SphereShapeSettings {
                Radius: *radius,
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
                ..JPC_CapsuleShapeSettings::default()
            },
            &mut native_shape,
            &mut native_error,
        ),
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
