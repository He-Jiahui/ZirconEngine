use crate::core::math::{Real, Vec3};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VisibilityBounds {
    pub center: Vec3,
    pub radius: Real,
}
