use serde::{Deserialize, Serialize};

use crate::core::math::precision::{Mat4, Quat, Vec3};
use crate::core::math::transform::transform_to_mat4;

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
