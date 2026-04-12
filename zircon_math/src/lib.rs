//! Shared math types for the first vertical slice.

pub use glam::{EulerRot, Mat4, Quat, UVec2, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
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

pub fn perspective(fov_y_radians: f32, aspect_ratio: f32, z_near: f32, z_far: f32) -> Mat4 {
    Mat4::perspective_rh(
        fov_y_radians,
        aspect_ratio.max(0.001),
        z_near.max(0.001),
        z_far,
    )
}

pub fn view_matrix(transform: Transform) -> Mat4 {
    transform.matrix().inverse()
}

pub fn clamp_viewport_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_matrix_contains_translation() {
        let transform = Transform::from_translation(Vec3::new(3.0, 2.0, 1.0));
        let matrix = transform.matrix();

        assert_eq!(matrix.w_axis.truncate(), Vec3::new(3.0, 2.0, 1.0));
    }

    #[test]
    fn look_at_faces_target() {
        let transform = Transform::looking_at(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y);

        assert!((transform.forward() - Vec3::new(0.0, 0.0, -1.0)).length() < 0.001);
    }
}
