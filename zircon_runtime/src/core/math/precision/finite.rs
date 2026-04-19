use super::aliases::{Mat4, Quat, Real, Vec2, Vec3, Vec4};

pub fn is_finite_scalar(value: Real) -> bool {
    value.is_finite()
}

pub fn is_finite_vec2(value: Vec2) -> bool {
    value.is_finite()
}

pub fn is_finite_vec3(value: Vec3) -> bool {
    value.is_finite()
}

pub fn is_finite_vec4(value: Vec4) -> bool {
    value.is_finite()
}

pub fn is_finite_quat(value: Quat) -> bool {
    value.is_finite()
}

pub fn is_finite_mat4(value: Mat4) -> bool {
    value.is_finite()
}
