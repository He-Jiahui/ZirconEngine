//! Shared math contracts used by runtime and editor-facing DTOs.

pub use glam::{EulerRot, UVec2};

pub type Real = f32;
pub type Vec2 = glam::Vec2;
pub type Vec3 = glam::Vec3;
pub type Vec4 = glam::Vec4;
pub type Quat = glam::Quat;
pub type Mat4 = glam::Mat4;

pub type RenderScalar = f32;
pub type RenderVec2 = glam::Vec2;
pub type RenderVec3 = glam::Vec3;
pub type RenderVec4 = glam::Vec4;
pub type RenderMat4 = glam::Mat4;

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

pub fn to_render_scalar(value: Real) -> Option<RenderScalar> {
    is_finite_scalar(value).then_some(value as RenderScalar)
}

pub fn to_render_vec2(value: Vec2) -> Option<RenderVec2> {
    Some(RenderVec2::new(
        to_render_scalar(value.x)?,
        to_render_scalar(value.y)?,
    ))
}

pub fn to_render_vec3(value: Vec3) -> Option<RenderVec3> {
    Some(RenderVec3::new(
        to_render_scalar(value.x)?,
        to_render_scalar(value.y)?,
        to_render_scalar(value.z)?,
    ))
}

pub fn to_render_vec4(value: Vec4) -> Option<RenderVec4> {
    Some(RenderVec4::new(
        to_render_scalar(value.x)?,
        to_render_scalar(value.y)?,
        to_render_scalar(value.z)?,
        to_render_scalar(value.w)?,
    ))
}

pub fn to_render_mat4(value: Mat4) -> Option<RenderMat4> {
    let cols = value.to_cols_array();
    Some(RenderMat4::from_cols_array(&[
        to_render_scalar(cols[0])?,
        to_render_scalar(cols[1])?,
        to_render_scalar(cols[2])?,
        to_render_scalar(cols[3])?,
        to_render_scalar(cols[4])?,
        to_render_scalar(cols[5])?,
        to_render_scalar(cols[6])?,
        to_render_scalar(cols[7])?,
        to_render_scalar(cols[8])?,
        to_render_scalar(cols[9])?,
        to_render_scalar(cols[10])?,
        to_render_scalar(cols[11])?,
        to_render_scalar(cols[12])?,
        to_render_scalar(cols[13])?,
        to_render_scalar(cols[14])?,
        to_render_scalar(cols[15])?,
    ]))
}

pub fn compose_trs(translation: Vec3, rotation: Quat, scale: Vec3) -> Mat4 {
    Mat4::from_scale_rotation_translation(scale, rotation, translation)
}

pub fn transform_to_mat4(transform: Transform) -> Mat4 {
    compose_trs(transform.translation, transform.rotation, transform.scale)
}

pub fn affine_inverse(matrix: Mat4) -> Mat4 {
    matrix.inverse()
}

pub fn perspective(fov_y_radians: Real, aspect_ratio: Real, z_near: Real, z_far: Real) -> Mat4 {
    Mat4::perspective_rh(
        fov_y_radians,
        aspect_ratio.max(0.001),
        z_near.max(0.001),
        z_far,
    )
}

pub fn view_matrix(transform: Transform) -> Mat4 {
    affine_inverse(transform_to_mat4(transform))
}

pub fn clamp_viewport_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const fn identity() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Self::identity()
        }
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn matrix(self) -> Mat4 {
        transform_to_mat4(self)
    }

    pub fn forward(self) -> Vec3 {
        (self.rotation * -Vec3::Z).normalize_or_zero()
    }

    pub fn right(self) -> Vec3 {
        (self.rotation * Vec3::X).normalize_or_zero()
    }

    pub fn up(self) -> Vec3 {
        (self.rotation * Vec3::Y).normalize_or_zero()
    }

    pub fn looking_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let forward = (target - eye).normalize_or_zero();
        let right = forward.cross(up).normalize_or_zero();
        let corrected_up = right.cross(forward).normalize_or_zero();
        let basis = Mat4::from_cols(
            right.extend(0.0),
            corrected_up.extend(0.0),
            (-forward).extend(0.0),
            eye.extend(1.0),
        );

        Self {
            translation: eye,
            rotation: Quat::from_mat4(&basis),
            scale: Vec3::ONE,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}
